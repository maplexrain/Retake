// Dedicated Window Capture Engine:
// Directly captures application window render contents using Win32 PrintWindow (PW_RENDERFULLCONTENT).
// This guarantees immunity against window displacements (movement) and window z-order occlusions (layer stacking).
// Captured raw BGRA video frames are piped directly to FFmpeg stdin.

use crate::ffmpeg_util::{detect, hide_console, CaptureHandle};
use crate::project::Settings;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
mod win32 {
    use core::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type HDC = *mut c_void;
    pub type HBITMAP = *mut c_void;
    pub type HGDIOBJ = *mut c_void;

    #[repr(C)]
    #[derive(Default, Copy, Clone, Debug)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    pub struct BITMAPINFOHEADER {
        pub bi_size: u32,
        pub bi_width: i32,
        pub bi_height: i32,
        pub bi_planes: u16,
        pub bi_bit_count: u16,
        pub bi_compression: u32,
        pub bi_size_image: u32,
        pub bi_x_pels_per_meter: i32,
        pub bi_y_pels_per_meter: i32,
        pub bi_clr_used: u32,
        pub bi_clr_important: u32,
    }

    #[repr(C)]
    pub struct BITMAPINFO {
        pub bmi_header: BITMAPINFOHEADER,
        pub bmi_colors: [u32; 1],
    }

    pub const DIB_RGB_COLORS: u32 = 0;
    pub const BI_RGB: u32 = 0;
    pub const PW_RENDERFULLCONTENT: u32 = 2;
    pub const DWMWA_EXTENDED_FRAME_BOUNDS: u32 = 9;
    pub const HALFTONE: i32 = 4;
    pub const SRCCOPY: u32 = 0x00CC_0020;

    #[link(name = "user32")]
    extern "system" {
        pub fn IsWindow(hwnd: HWND) -> i32;
        pub fn IsIconic(hwnd: HWND) -> i32;
        pub fn GetWindowRect(hwnd: HWND, lpRect: *mut RECT) -> i32;
        pub fn PrintWindow(hwnd: HWND, hdcBlt: HDC, nFlags: u32) -> i32;
    }

    #[link(name = "dwmapi")]
    extern "system" {
        pub fn DwmGetWindowAttribute(hwnd: HWND, dwAttribute: u32, pvAttribute: *mut c_void, cbAttribute: u32) -> i32;
    }

    #[link(name = "gdi32")]
    extern "system" {
        pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
        pub fn DeleteDC(hdc: HDC) -> i32;
        pub fn CreateDIBSection(
            hdc: HDC,
            pbmi: *const BITMAPINFO,
            usage: u32,
            ppvBits: *mut *mut c_void,
            hSection: *mut c_void,
            offset: u32,
        ) -> HBITMAP;
        pub fn DeleteObject(ho: HGDIOBJ) -> i32;
        pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
        pub fn SetStretchBltMode(hdc: HDC, mode: i32) -> i32;
        pub fn StretchBlt(
            hdcDest: HDC,
            xDest: i32,
            yDest: i32,
            wDest: i32,
            hDest: i32,
            hdcSrc: HDC,
            xSrc: i32,
            ySrc: i32,
            wSrc: i32,
            hSrc: i32,
            rop: u32,
        ) -> i32;
    }
}

#[cfg(windows)]
pub fn get_window_client_or_frame_size(hwnd: isize) -> Option<(i32, i32)> {
    unsafe {
        let h = hwnd as usize as win32::HWND;
        if win32::IsWindow(h) == 0 {
            return None;
        }
        let mut r = win32::RECT::default();
        let hr = win32::DwmGetWindowAttribute(
            h,
            win32::DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut r as *mut _ as *mut core::ffi::c_void,
            std::mem::size_of::<win32::RECT>() as u32,
        );
        if hr != 0 {
            if win32::GetWindowRect(h, &mut r) == 0 {
                return None;
            }
        }
        let w = (r.right - r.left).max(64);
        let h = (r.bottom - r.top).max(64);
        // Ensure even dimensions required by h264/yuv420p
        Some(((w + 1) / 2 * 2, (h + 1) / 2 * 2))
    }
}

#[cfg(windows)]
struct DIBBuffer {
    hdc: win32::HDC,
    hbmp: win32::HBITMAP,
    old_bmp: win32::HGDIOBJ,
    bits_ptr: *mut u8,
    width: i32,
    height: i32,
}

#[cfg(windows)]
impl DIBBuffer {
    fn new(width: i32, height: i32) -> Result<Self, String> {
        unsafe {
            let hdc = win32::CreateCompatibleDC(core::ptr::null_mut());
            if hdc.is_null() {
                return Err("Failed to create compatible DC".into());
            }

            let bmi = win32::BITMAPINFO {
                bmi_header: win32::BITMAPINFOHEADER {
                    bi_size: std::mem::size_of::<win32::BITMAPINFOHEADER>() as u32,
                    bi_width: width,
                    bi_height: -height, // Negative indicates Top-down DIB, matching FFmpeg rawvideo scanlines
                    bi_planes: 1,
                    bi_bit_count: 32,
                    bi_compression: win32::BI_RGB,
                    bi_size_image: (width * height * 4) as u32,
                    bi_x_pels_per_meter: 0,
                    bi_y_pels_per_meter: 0,
                    bi_clr_used: 0,
                    bi_clr_important: 0,
                },
                bmi_colors: [0],
            };

            let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();
            let hbmp = win32::CreateDIBSection(
                hdc,
                &bmi,
                win32::DIB_RGB_COLORS,
                &mut bits,
                core::ptr::null_mut(),
                0,
            );

            if hbmp.is_null() || bits.is_null() {
                win32::DeleteDC(hdc);
                return Err("Failed to create DIB section".into());
            }

            let old_bmp = win32::SelectObject(hdc, hbmp as win32::HGDIOBJ);
            win32::SetStretchBltMode(hdc, win32::HALFTONE);

            Ok(Self {
                hdc,
                hbmp,
                old_bmp,
                bits_ptr: bits as *mut u8,
                width,
                height,
            })
        }
    }

    fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.bits_ptr, (self.width * self.height * 4) as usize)
        }
    }
}

#[cfg(windows)]
impl Drop for DIBBuffer {
    fn drop(&mut self) {
        unsafe {
            if !self.old_bmp.is_null() {
                win32::SelectObject(self.hdc, self.old_bmp);
            }
            if !self.hbmp.is_null() {
                win32::DeleteObject(self.hbmp as win32::HGDIOBJ);
            }
            if !self.hdc.is_null() {
                win32::DeleteDC(self.hdc);
            }
        }
    }
}

/// Spawn dedicated window capture -> x264/aac -> mp4.
/// Captures target hwnd directly without screen area clipping.
#[cfg(windows)]
pub fn spawn_window_capture(
    settings: &Settings,
    out: &Path,
    hwnd: isize,
) -> Result<CaptureHandle, String> {
    let ff = detect(settings.ffmpeg_path.as_deref());
    if !ff.found {
        return Err("ffmpeg not found".into());
    }
    let exe = ff.path.unwrap_or_else(|| "ffmpeg".into());

    let (base_w, base_h) = get_window_client_or_frame_size(hwnd)
        .ok_or_else(|| "目标应用窗口不存在或已关闭".to_string())?;

    let fps = settings.fps.clamp(5, 60);
    let crf = settings.crf.clamp(14, 34).to_string();

    let mut args: Vec<String> = vec![
        "-hide_banner".into(),
        "-y".into(),
        "-f".into(),
        "rawvideo".into(),
        "-pix_fmt".into(),
        "bgra".into(),
        "-s".into(),
        format!("{}x{}", base_w, base_h),
        "-r".into(),
        fps.to_string(),
        "-i".into(),
        "-".into(), // Piped raw video frames from worker thread
    ];

    let mic = settings.mic.as_deref().map(str::trim).filter(|m| !m.is_empty());
    let with_audio = mic.is_some();
    if let Some(m) = mic {
        args.push("-f".into());
        args.push("dshow".into());
        args.push("-i".into());
        args.push(format!("audio={m}"));
    }

    args.extend([
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        "veryfast".into(),
        "-crf".into(),
        crf,
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-vf".into(),
        "scale=trunc(iw/2)*2:trunc(ih/2)*2".into(),
    ]);
    if with_audio {
        args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "128k".into()]);
    }
    args.extend(["-movflags".into(), "+faststart".into(), out.to_string_lossy().into_owned()]);

    let mut cmd = Command::new(exe);
    cmd.args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    hide_console(&mut cmd);

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn ffmpeg: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("Failed to open ffmpeg stdin")?;
    let stderr = child.stderr.take();

    let tail = Arc::new(Mutex::new(String::new()));
    if let Some(err) = stderr {
        let tail2 = tail.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(err).lines().map_while(Result::ok) {
                let mut t = tail2.lock().unwrap();
                t.push_str(&line);
                t.push('\n');
                if t.len() > 16000 {
                    let cut = t.len() - 8000;
                    let idx = t[cut..].find('\n').map(|i| cut + i + 1).unwrap_or(cut);
                    t.drain(..idx);
                }
            }
        });
    }

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_for_thread = stop_signal.clone();

    // Spawn dedicated window frame capture thread
    let worker_thread = std::thread::spawn(move || {
        let target_hwnd = hwnd as usize as win32::HWND;
        let dib = match DIBBuffer::new(base_w, base_h) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[window_capture] DIBBuffer creation failed: {e}");
                return;
            }
        };

        let frame_size = (base_w * base_h * 4) as usize;
        let mut last_valid_frame = vec![0u8; frame_size];
        let mut has_valid_frame = false;

        let frame_dur = std::time::Duration::from_nanos((1_000_000_000.0 / fps as f64) as u64);
        let mut next_tick = std::time::Instant::now();

        while !stop_for_thread.load(Ordering::Relaxed) {
            let now = std::time::Instant::now();
            if now < next_tick {
                std::thread::sleep(next_tick - now);
            }
            next_tick += frame_dur;
            if next_tick < now {
                next_tick = now + frame_dur;
            }

            let is_alive = unsafe { win32::IsWindow(target_hwnd) != 0 };
            let is_minimized = unsafe { win32::IsIconic(target_hwnd) != 0 };

            let mut captured_ok = false;
            if is_alive && !is_minimized {
                let mut r = win32::RECT::default();
                let hr = unsafe {
                    win32::DwmGetWindowAttribute(
                        target_hwnd,
                        win32::DWMWA_EXTENDED_FRAME_BOUNDS,
                        &mut r as *mut _ as *mut core::ffi::c_void,
                        std::mem::size_of::<win32::RECT>() as u32,
                    )
                };
                if hr != 0 {
                    unsafe {
                        win32::GetWindowRect(target_hwnd, &mut r);
                    }
                }
                let curr_w = (r.right - r.left).max(1);
                let curr_h = (r.bottom - r.top).max(1);

                if curr_w == base_w && curr_h == base_h {
                    // Match dimensions: render directly to destination DIB
                    let ok = unsafe { win32::PrintWindow(target_hwnd, dib.hdc, win32::PW_RENDERFULLCONTENT) != 0 };
                    if ok || unsafe { win32::PrintWindow(target_hwnd, dib.hdc, 0) != 0 } {
                        captured_ok = true;
                    }
                } else {
                    // Window size resized: dynamically render into temp DIB and scale smoothly to base canvas
                    if let Ok(temp_dib) = DIBBuffer::new(curr_w, curr_h) {
                        let ok = unsafe { win32::PrintWindow(target_hwnd, temp_dib.hdc, win32::PW_RENDERFULLCONTENT) != 0 };
                        if ok || unsafe { win32::PrintWindow(target_hwnd, temp_dib.hdc, 0) != 0 } {
                            unsafe {
                                win32::StretchBlt(
                                    dib.hdc,
                                    0,
                                    0,
                                    base_w,
                                    base_h,
                                    temp_dib.hdc,
                                    0,
                                    0,
                                    curr_w,
                                    curr_h,
                                    win32::SRCCOPY,
                                );
                            }
                            captured_ok = true;
                        }
                    }
                }
            }

            if captured_ok {
                let slice = dib.as_slice();
                last_valid_frame.copy_from_slice(slice);
                has_valid_frame = true;
                if stdin.write_all(slice).is_err() {
                    break;
                }
            } else if has_valid_frame {
                // Window minimized or temporarily unrendered: feed freeze frame to maintain A/V sync
                if stdin.write_all(&last_valid_frame).is_err() {
                    break;
                }
            } else {
                // Initial frame fallback
                if stdin.write_all(&last_valid_frame).is_err() {
                    break;
                }
            }
        }

        let _ = stdin.flush();
    });

    let file_name = out.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(CaptureHandle {
        child,
        stdin: None,
        tail,
        out: out.to_path_buf(),
        file_name,
        stop_signal: Some(stop_signal),
        worker_thread: Some(worker_thread),
    })
}

#[cfg(not(windows))]
pub fn spawn_window_capture(
    _settings: &Settings,
    _out: &Path,
    _hwnd: isize,
) -> Result<CaptureHandle, String> {
    Err("Dedicated window capture is only supported on Windows".into())
}
