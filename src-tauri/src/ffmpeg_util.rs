use crate::project::Settings;
use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Serialize, Clone)]
pub struct FfmpegInfo {
    pub found: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = cmd;
}

fn candidates(configured: Option<&str>) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(c) = configured.map(|s| s.trim().trim_matches('"').to_string()) {
        if !c.is_empty() {
            v.push(c.clone());
            // Auto-append ffmpeg.exe if user specified a directory (e.g. D:\ffmpeg or D:\ffmpeg\bin)
            let p = std::path::PathBuf::from(&c);
            if p.is_dir() {
                v.push(p.join("ffmpeg.exe").to_string_lossy().into_owned());
                v.push(p.join("bin").join("ffmpeg.exe").to_string_lossy().into_owned());
            }
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            v.push(d.join("ffmpeg.exe").to_string_lossy().into_owned());
        }
    }
    v.push("ffmpeg".into());
    v
}

fn probe(exe: &str) -> Option<FfmpegInfo> {
    let mut cmd = Command::new(&exe);
    cmd.arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(&mut cmd);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let first = s.lines().next()?;
    if !first.contains("ffmpeg version") {
        return None;
    }
    let version = first.split_whitespace().nth(2).map(|x| x.to_string());
    Some(FfmpegInfo { found: true, version, path: Some(exe.to_string()) })
}

pub fn detect(configured: Option<&str>) -> FfmpegInfo {
    for c in candidates(configured) {
        if let Some(info) = probe(&c) {
            return info;
        }
    }
    FfmpegInfo { found: false, version: None, path: None }
}

/// Enumerate DirectShow audio (microphone) devices.
pub fn list_mics(configured: Option<&str>) -> Vec<String> {
    for c in candidates(configured) {
        if probe(&c).is_none() {
            continue;
        }
        let mut cmd = Command::new(&c);
        cmd.args(["-hide_banner", "-list_devices", "true", "-f", "dshow", "-i", "dummy"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        hide_console(&mut cmd);
        if let Ok(out) = cmd.output() {
            let s = String::from_utf8_lossy(&out.stderr);
            let mut found = Vec::new();
            for line in s.lines() {
                if !line.contains("(audio)") || line.contains("Alternative name") {
                    continue;
                }
                if let Some(a) = line.find('"') {
                    if let Some(b) = line[a + 1..].find('"') {
                        let name = line[a + 1..a + 1 + b].to_string();
                        if !name.is_empty() && !found.contains(&name) {
                            found.push(name);
                        }
                    }
                }
            }
            return found;
        }
    }
    Vec::new()
}

/// A running ffmpeg capture process handle.
pub struct CaptureHandle {
    pub child: Child,
    pub stdin: Option<ChildStdin>,
    /// stderr tail buffer (used to parse duration and error lines)
    pub tail: Arc<Mutex<String>>,
    pub out: std::path::PathBuf,
    pub file_name: String,
    pub stop_signal: Option<Arc<AtomicBool>>,
    pub worker_thread: Option<JoinHandle<()>>,
    pub mic_child: Option<Child>,
    pub mic_stdin: Option<ChildStdin>,
    pub temp_video_path: Option<std::path::PathBuf>,
    pub temp_mic_path: Option<std::path::PathBuf>,
    pub ffmpeg_exe: String,
}

/// Spawn gdigrab desktop capture -> x264/aac -> mp4.
/// rect is physical pixel absolute coordinates (primary monitor origin), matching gdigrab offsets;
/// Pass None to capture virtual desktop (not recommended due to black bars on multi-monitor).
pub fn spawn_capture(settings: &Settings, out: &Path, rect: Option<&crate::project::Region>) -> Result<CaptureHandle, String> {
    let ff = detect(settings.ffmpeg_path.as_deref());
    if !ff.found {
        return Err("ffmpeg not found".into());
    }
    let exe = ff.path.unwrap_or_else(|| "ffmpeg".into());
    let fps = settings.fps.clamp(5, 60).to_string();
    let crf = settings.crf.clamp(14, 34).to_string();

    let mut args: Vec<String> = vec![
        "-hide_banner".into(),
        "-y".into(),
        "-f".into(),
        "gdigrab".into(),
        "-framerate".into(),
        fps,
    ];
    if !settings.capture_cursor {
        args.push("-draw_mouse".into());
        args.push("0".into());
    }
    if let Some(r) = rect {
        if r.w > 0 && r.h > 0 {
            args.push("-offset_x".into());
            args.push(r.x.to_string());
            args.push("-offset_y".into());
            args.push(r.y.to_string());
            args.push("-video_size".into());
            args.push(format!("{}x{}", r.w, r.h));
        }
    }
    args.push("-i".into());
    args.push("desktop".into());

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
        // Ensure even dimensions required by yuv420p
        "-vf".into(),
        "scale=trunc(iw/2)*2:trunc(ih/2)*2".into(),
    ]);
    if with_audio {
        args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "128k".into()]);
    }
    args.extend(["-movflags".into(), "+faststart".into(), out.to_string_lossy().into_owned()]);

    let mut cmd = Command::new(&exe);
    cmd.args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn ffmpeg: {e}"))?;
    let stdin = child.stdin.take();
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
    let file_name = out.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(CaptureHandle {
        child,
        stdin,
        tail,
        out: out.to_path_buf(),
        file_name,
        stop_signal: None,
        worker_thread: None,
        mic_child: None,
        mic_stdin: None,
        temp_video_path: None,
        temp_mic_path: None,
        ffmpeg_exe: exe,
    })
}

/// Gracefully stop ffmpeg. If worker_thread is running, signal it to stop and wait. Returns parsed duration in seconds.
pub fn finish(h: &mut CaptureHandle, timeout_ms: u64) -> f64 {
    // 1. If dedicated mic child is running, gracefully stop it by sending 'q'
    if let Some(mut m_si) = h.mic_stdin.take() {
        let _ = m_si.write_all(b"q");
        let _ = m_si.flush();
    }
    if let Some(mut m_child) = h.mic_child.take() {
        let _ = m_child.wait();
    }

    // 2. Stop window capture worker thread
    if let Some(stop) = h.stop_signal.as_ref() {
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    // For standard gdigrab, send 'q' to stdin
    if let Some(mut si) = h.stdin.take() {
        let _ = si.write_all(b"q");
        let _ = si.flush();
    }
    if let Some(worker) = h.worker_thread.take() {
        let _ = worker.join();
    }

    // 3. Wait for main video ffmpeg to finish
    let start = std::time::Instant::now();
    loop {
        match h.child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed().as_millis() as u64 > timeout_ms {
                    let _ = h.child.kill();
                    let _ = h.child.wait();
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(40));
            }
            Err(_) => break,
        }
    }

    // 4. If separate video and mic were recorded, mux them via zero-reencode stream copy
    if let (Some(temp_v), Some(temp_m)) = (h.temp_video_path.take(), h.temp_mic_path.take()) {
        if temp_v.exists() && temp_m.exists() {
            let mut mux_cmd = Command::new(&h.ffmpeg_exe);
            mux_cmd.args([
                "-hide_banner",
                "-y",
                "-i",
                temp_v.to_str().unwrap_or_default(),
                "-i",
                temp_m.to_str().unwrap_or_default(),
                "-c:v",
                "copy",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                "-movflags",
                "+faststart",
                h.out.to_str().unwrap_or_default(),
            ]);
            hide_console(&mut mux_cmd);
            let _ = mux_cmd.status();
            let _ = std::fs::remove_file(&temp_v);
            let _ = std::fs::remove_file(&temp_m);
        } else if temp_v.exists() {
            let _ = std::fs::rename(&temp_v, &h.out);
        }
    }

    let t = h.tail.lock().unwrap().clone();
    parse_last_time(&t).unwrap_or(0.0)
}

pub fn kill_now(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Run a foreground ffmpeg subcommand (e.g. concat/mux), returns whether it succeeded.
pub fn run_exiting(exe: &str, args: &[String]) -> bool {
    let mut cmd = Command::new(&exe);
    cmd.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    match cmd.output() {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

fn parse_last_time(s: &str) -> Option<f64> {
    let mut last: Option<f64> = None;
    let mut rest = s;
    while let Some(i) = rest.find("time=") {
        let after = &rest[i + 5..];
        let end = after
            .find(|c: char| !(c.is_ascii_digit() || c == ':' || c == '.'))
            .unwrap_or(after.len());
        let tok = &after[..end];
        let parts: Vec<&str> = tok.split(':').collect();
        if parts.len() == 3 {
            if let (Ok(h), Ok(m), Ok(sec)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                last = Some(h * 3600.0 + m * 60.0 + sec);
            }
        }
        rest = &rest[i + 5..];
    }
    last
}
