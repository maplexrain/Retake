// System audio capture: WASAPI loopback (render endpoint loopback) -> append to WAV (float32).
// Standard ffmpeg builds lack a native wasapi input device, so we capture it directly and mux upon completion.

#[cfg(windows)]
pub struct SysSeg {
    pub stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub handle: Option<std::thread::JoinHandle<()>>,
    pub path: std::path::PathBuf,
}

/// Start system audio loopback capture, writing to out (WAV). Fails with Err (does not block video recording).
#[cfg(windows)]
pub fn start_sys_capture(out: std::path::PathBuf) -> Result<SysSeg, String> {
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let out2 = out.clone();
    let handle = std::thread::spawn(move || {
        if let Err(e) = capture_loop(out2, stop2) {
            eprintln!("[retake] System audio capture error: {e}");
        }
    });
    Ok(SysSeg { stop, handle: Some(handle), path: out })
}

/// Stop capture and wait for worker thread to finish (writing WAV header), returns file path.
#[cfg(windows)]
pub fn stop_sys_capture(seg: SysSeg) -> Result<std::path::PathBuf, String> {
    use std::sync::atomic::Ordering;
    seg.stop.store(true, Ordering::Relaxed);
    if let Some(h) = seg.handle {
        let _ = h.join();
    }
    Ok(seg.path)
}

#[cfg(not(windows))]
pub struct SysSeg;

#[cfg(not(windows))]
pub fn start_sys_capture(_out: std::path::PathBuf) -> Result<SysSeg, String> {
    Err("System audio capture is only supported on Windows".into())
}

#[cfg(not(windows))]
pub fn stop_sys_capture(seg: SysSeg) -> Result<std::path::PathBuf, String> {
    Ok(std::path::PathBuf::new())
}

#[cfg(not(windows))]
pub fn get_audio_levels(_check_mic: bool, _check_sys: bool) -> (f32, f32) {
    (0.0, 0.0)
}

#[cfg(windows)]
pub fn get_audio_levels(check_mic: bool, check_sys: bool) -> (f32, f32) {
    use windows::Win32::Media::Audio::{
        eCapture, eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
    };
    use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    let enumerator: Result<IMMDeviceEnumerator, _> =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) };
    let Ok(enumerator) = enumerator else {
        return (0.0, 0.0);
    };

    let mut mic_level = 0.0f32;
    if check_mic {
        if let Ok(device) = unsafe { enumerator.GetDefaultAudioEndpoint(eCapture, eMultimedia) } {
            if let Ok(meter) = unsafe { device.Activate::<IAudioMeterInformation>(CLSCTX_ALL, None) } {
                if let Ok(peak) = unsafe { meter.GetPeakValue() } {
                    mic_level = peak;
                }
            }
        }
    }

    let mut sys_level = 0.0f32;
    if check_sys {
        if let Ok(device) = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) } {
            if let Ok(meter) = unsafe { device.Activate::<IAudioMeterInformation>(CLSCTX_ALL, None) } {
                if let Ok(peak) = unsafe { meter.GetPeakValue() } {
                    sys_level = peak;
                }
            }
        }
    }

    (mic_level, sys_level)
}

// ---------------- Windows Implementation ----------------

#[cfg(windows)]
fn capture_loop(out: std::path::PathBuf, stop: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<(), String> {
    use std::sync::atomic::Ordering;

    use windows::Win32::Media::Audio::{
        eMultimedia, eRender, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, IAudioCaptureClient,
        IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|e| format!("COM initialization failed: {e}"))?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| format!("Failed to create device enumerator: {e}"))?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .map_err(|e| format!("Failed to get default output device: {e}"))?;
        let client: IAudioClient = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| format!("Failed to activate audio client: {e}"))?;
        let mixfmt = client.GetMixFormat().map_err(|e| format!("Failed to get mix format: {e}"))?;
        let fmt = &*mixfmt;
        let channels = fmt.nChannels as u32;
        let rate = fmt.nSamplesPerSec;
        let bytes_per_sample = 4u32; // Float32 mix format
        let block_align = channels * bytes_per_sample;

        client
            .Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, 20_000_000, 0, fmt, None)
            .map_err(|e| format!("Failed to initialize loopback capture: {e}"))?;
        let capture: IAudioCaptureClient = client.GetService().map_err(|e| format!("Failed to get capture client: {e}"))?;
        client.Start().map_err(|e| format!("Failed to start capture: {e}"))?;

        let mut file = std::fs::File::create(&out).map_err(|e| format!("Failed to create system audio file: {e}"))?;
        write_wav_header(&mut file, channels, rate)?;

        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            // Drain all available readable packets
            loop {
                let frames = capture.GetNextPacketSize().map_err(|e| format!("GetNextPacketSize: {e}"))?;
                if frames == 0 {
                    break;
                }
                let mut data: *mut u8 = std::ptr::null_mut();
                let mut frames_read = 0u32;
                let mut flags = 0u32;
                capture
                    .GetBuffer(&mut data, &mut frames_read, &mut flags, None, None)
                    .map_err(|e| format!("GetBuffer: {e}"))?;
                let silent = flags & windows::Win32::Media::Audio::AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
                if frames_read > 0 && !silent && !data.is_null() {
                    let bytes =
                        std::slice::from_raw_parts(data as *const u8, frames_read as usize * block_align as usize);
                    use std::io::Write;
                    let _ = file.write_all(bytes);
                }
                let _ = capture.ReleaseBuffer(frames_read);
            }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }

        let _ = client.Stop();
        finalize_wav_header(&mut file, block_align);
        Ok(())
    }
}

/// 48-byte header: RIFF + WAVE + fmt(IEEE float32) + fact + data placeholder.
#[cfg(windows)]
fn write_wav_header(file: &mut std::fs::File, channels: u32, rate: u32) -> Result<(), String> {
    use std::io::Write;
    let block_align = channels * 4u32;
    let byte_rate = rate * block_align;
    let mut h = Vec::with_capacity(56);
    h.extend_from_slice(b"RIFF");
    h.extend_from_slice(&0u32.to_le_bytes()); // 4..8 patched on finalize
    h.extend_from_slice(b"WAVE");
    h.extend_from_slice(b"fmt ");
    h.extend_from_slice(&16u32.to_le_bytes());
    h.extend_from_slice(&3u16.to_le_bytes()); // WAVE_FORMAT_IEEE_FLOAT
    h.extend_from_slice(&(channels as u16).to_le_bytes());
    h.extend_from_slice(&rate.to_le_bytes());
    h.extend_from_slice(&byte_rate.to_le_bytes());
    h.extend_from_slice(&(block_align as u16).to_le_bytes());
    h.extend_from_slice(&32u16.to_le_bytes()); // bits per sample
    h.extend_from_slice(b"fact");
    h.extend_from_slice(&4u32.to_le_bytes());
    h.extend_from_slice(&0u32.to_le_bytes()); // frame count, patched on finalize
    h.extend_from_slice(b"data");
    h.extend_from_slice(&0u32.to_le_bytes()); // 52..56 patched on finalize
    file.write_all(&h).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn finalize_wav_header(file: &mut std::fs::File, block_align: u32) {
    use std::io::{Seek, SeekFrom, Write};
    if let Ok(len) = file.seek(SeekFrom::End(0)) {
        let data_size = (len - 56) as u32;
        let _ = file.seek(SeekFrom::Start(4));
        let _ = file.write_all(&((len - 8) as u32).to_le_bytes());
        let _ = file.seek(SeekFrom::Start(44));
        let _ = file.write_all(&(data_size / block_align.max(1)).to_le_bytes());
        let _ = file.seek(SeekFrom::Start(52));
        let _ = file.write_all(&data_size.to_le_bytes());
        let _ = file.flush();
    }
}
