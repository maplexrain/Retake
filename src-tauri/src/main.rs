#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ffmpeg_util;
mod project;
mod recorder;
mod sysaudio;
mod win;
mod window_capture;

use ffmpeg_util::FfmpegInfo;
use project::{Project, Recording, Region};
use recorder::{Phase, Recorder, Session};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScopePreviewInfo {
    pub rect: Region,
    pub monitor: MonitorInfo,
}

pub struct AppState {
    pub project: Mutex<Project>,
    pub recorder: Mutex<Recorder>,
    /// Monitor index currently covered by region selection overlay
    pub region_monitor: Mutex<Option<usize>>,
    pub active_target: Mutex<Option<String>>,
    pub scope_preview_info: Mutex<Option<ScopePreviewInfo>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub scale: f64,
    pub is_primary: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PhaseInfo {
    pub phase: String,
    pub recorded_ms: Option<u64>,
    pub resumed_at: Option<u64>,
    pub markers: Vec<f64>,
    pub target_file_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub project: Project,
    pub ffmpeg: FfmpegInfo,
    pub mics: Vec<String>,
    pub monitors: Vec<MonitorInfo>,
    pub output_dir: String,
    pub phase: PhaseInfo,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn phase_info(rec: &Recorder, active_target: Option<String>, manifest_mode: bool) -> PhaseInfo {
    let markers = rec.session.as_ref().map(|s| s.markers.clone()).unwrap_or_default();
    let target_file_name = if manifest_mode {
        rec.session
            .as_ref()
            .and_then(|s| s.target_file_name.clone())
            .or(active_target)
    } else {
        None
    };
    match (&rec.phase, &rec.session) {
        (Phase::Recording, Some(s)) => PhaseInfo {
            phase: "recording".into(),
            recorded_ms: Some(s.recorded_total_ms()),
            resumed_at: Some(now_ms()),
            markers,
            target_file_name,
        },
        (Phase::Paused, Some(s)) => PhaseInfo {
            phase: "paused".into(),
            recorded_ms: Some(s.recorded_ms),
            resumed_at: None,
            markers,
            target_file_name,
        },
        (Phase::Countdown(_), _) => PhaseInfo {
            phase: "countdown".into(),
            recorded_ms: None,
            resumed_at: None,
            markers: vec![],
            target_file_name,
        },
        _ => PhaseInfo {
            phase: "idle".into(),
            recorded_ms: None,
            resumed_at: None,
            markers: vec![],
            target_file_name,
        },
    }
}

fn emit_phase(app: &AppHandle) {
    let state = app.state::<AppState>();
    let rec = state.recorder.lock().unwrap();
    let active_target = state.active_target.lock().unwrap().clone();
    let manifest_mode = state.project.lock().unwrap().manifest_mode;
    let _ = app.emit("rec://phase", phase_info(&rec, active_target, manifest_mode));
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn hide_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

fn list_monitor_infos(app: &AppHandle) -> Vec<MonitorInfo> {
    let monitors = app.available_monitors().unwrap_or_default();
    monitors
        .iter()
        .enumerate()
        .map(|(i, m)| MonitorInfo {
            index: i,
            name: m.name().cloned().unwrap_or_else(|| format!("Display {}", i + 1)),
            x: m.position().x,
            y: m.position().y,
            w: m.size().width as i32,
            h: m.size().height as i32,
            scale: m.scale_factor(),
            is_primary: m.position().x == 0 && m.position().y == 0,
        })
        .collect()
}

/// Read capture rectangle for specified monitor index (physical absolute coords); returns None if index is invalid.
fn monitor_rect(app: &AppHandle, index: usize) -> Option<Region> {
    let monitors = app.available_monitors().ok()?;
    let m = monitors.get(index)?;
    Some(Region {
        x: m.position().x,
        y: m.position().y,
        w: m.size().width as i32,
        h: m.size().height as i32,
    })
}

pub fn output_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let configured = { app.state::<AppState>().project.lock().unwrap().output_dir.clone() };
    if let Some(c) = configured {
        let p = PathBuf::from(c);
        if !p.as_os_str().is_empty() {
            std::fs::create_dir_all(&p).map_err(|e| format!("Failed to create output directory: {e}"))?;
            return Ok(p);
        }
    }
    let dir = app
        .path()
        .document_dir()
        .map(|d| d.join("Retake"))
        .unwrap_or_else(|_| std::env::temp_dir().join("Retake"));
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create output directory: {e}"))?;
    Ok(dir)
}

fn apply_app_exclusion(app: &AppHandle, record_app_windows: bool) {
    let exclude = !record_app_windows;
    #[cfg(windows)]
    {
        for label in ["main", "hud", "countdown", "selector", "scope_preview"] {
            if let Some(w) = app.get_webview_window(label) {
                win::set_window_capture_exclusion(&w, exclude);
            }
        }
    }
    #[cfg(not(windows))]
    let _ = (app, record_app_windows);
}

// ---------------- IPC Commands ----------------

#[tauri::command]
async fn bootstrap(app: AppHandle) -> Result<Bootstrap, String> {
    let state = app.state::<AppState>();
    let project = state.project.lock().unwrap().clone();
    let ff = ffmpeg_util::detect(project.settings.ffmpeg_path.as_deref());
    let mics = ffmpeg_util::list_mics(project.settings.ffmpeg_path.as_deref());
    let monitors = list_monitor_infos(&app);
    let output_dir = output_dir(&app)?.to_string_lossy().into_owned();
    let active_target = state.active_target.lock().unwrap().clone();
    let manifest_mode = project.manifest_mode;
    let phase = phase_info(&state.recorder.lock().unwrap(), active_target, manifest_mode);
    println!("[retake] bootstrap ffmpeg found={} path={:?}", ff.found, ff.path);
    Ok(Bootstrap {
        project,
        ffmpeg: ff,
        mics,
        monitors,
        output_dir,
        phase,
    })
}

#[tauri::command]
fn save_project(app: AppHandle, mut project: Project) -> Result<(), String> {
    let state = app.state::<AppState>();
    let record_app = project.settings.record_app_windows;
    project.settings.exclude_self = !record_app;
    {
        let mut p = state.project.lock().unwrap();
        *p = project;
    }
    apply_app_exclusion(&app, record_app);
    let p = state.project.lock().unwrap().clone();
    project::save(&app, &p)?;
    emit_project_changed(&app, &p);
    Ok(())
}

fn emit_project_changed(app: &AppHandle, p: &Project) {
    let _ = app.emit("project://changed", p);
}

#[tauri::command]
async fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    Ok(list_monitor_infos(&app))
}

// ---------------- Recording Session ----------------

/// Spawn a new recording segment (video + optional system audio WAV)
fn spawn_part(ses: &mut Session) -> Result<(), String> {
    let part = ses.out_base.with_extension(format!("p{:02}.mp4", ses.parts.len()));
    let handle = if ses.settings.capture_mode == "window" {
        if let Some(hwnd) = ses.settings.window_hwnd {
            window_capture::spawn_window_capture(&ses.settings, &part, hwnd)?
        } else {
            return Err("未指定要录制的目标应用窗口".into());
        }
    } else {
        ffmpeg_util::spawn_capture(&ses.settings, &part, Some(&ses.capture_rect))?
    };
    ses.cur = Some(handle);
    ses.resumed_at = Some(Instant::now());
    if ses.has_sys {
        let sys_path = ses.out_base.with_extension(format!("p{:02}.sys.wav", ses.parts.len()));
        match sysaudio::start_sys_capture(sys_path) {
            Ok(seg) => ses.cur_sys = Some(seg),
            Err(e) => eprintln!("[retake] Failed to start system audio capture: {e}"),
        }
    }
    Ok(())
}

/// Finish current segment (retain files for final concatenation)
fn finish_part(ses: &mut Session, timeout_ms: u64) -> Result<(), String> {
    if let Some(mut h) = ses.cur.take() {
        ffmpeg_util::finish(&mut h, timeout_ms);
        let size = std::fs::metadata(&h.out).map(|m| m.len()).unwrap_or(0);
        if !h.out.exists() || size < 1024 {
            let tail = h.tail.lock().unwrap().clone();
            let errline = tail
                .lines()
                .rev()
                .find(|l| l.contains("Error") || l.contains("error") || l.contains("Could not") || l.contains("No such"))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Recording output incomplete (file size {size} bytes)"));
            return Err(format!("Recording failed: {errline}"));
        }
        ses.parts.push(h.out);
    }
    if let Some(sc) = ses.cur_sys.take() {
        match sysaudio::stop_sys_capture(sc) {
            Ok(p) => ses.sys_parts.push(p),
            Err(e) => eprintln!("[retake] Failed to stop system audio capture: {e}"),
        }
    }
    if let Some(t) = ses.resumed_at.take() {
        ses.recorded_ms += t.elapsed().as_millis() as u64;
    }
    Ok(())
}

/// Segment concatenation + system audio muxing -> final video file
fn finalize_recording(ses: &mut Session) -> Result<Recording, String> {
    finish_part(ses, 8000)?;
    let ff = ffmpeg_util::detect(ses.settings.ffmpeg_path.as_deref());
    if !ff.found {
        return Err("ffmpeg not found, unable to save".into());
    }
    let exe = ff.path.unwrap_or_else(|| "ffmpeg".into());
    let out = ses.out_base.clone();

    // 1) Lossless video segment concatenation
    if ses.parts.len() == 1 {
        if out.exists() {
            let _ = std::fs::remove_file(&out);
        }
        std::fs::rename(&ses.parts[0], &out).map_err(|e| format!("Failed to save: {e}"))?;
        ses.parts.clear();
    } else if !ses.parts.is_empty() {
        let list_path = ses.out_base.with_extension("parts.txt");
        let mut list = String::from("ffconcat version 1.0\n");
        for p in &ses.parts {
            list.push_str(&format!("file '{}'\n", p.to_string_lossy().replace('\\', "/")));
        }
        std::fs::write(&list_path, &list).map_err(|e| e.to_string())?;
        let args = vec![
            "-hide_banner".into(),
            "-y".into(),
            "-f".into(),
            "concat".into(),
            "-safe".into(),
            "0".into(),
            "-i".into(),
            list_path.to_string_lossy().into_owned(),
            "-c".into(),
            "copy".into(),
            "-movflags".into(),
            "+faststart".into(),
            out.to_string_lossy().into_owned(),
        ];
        let ok = ffmpeg_util::run_exiting(&exe, &args);
        let _ = std::fs::remove_file(&list_path);
        if !ok {
            return Err("Segment merge failed; original segments are retained in the output directory".into());
        }
        for p in &ses.parts {
            let _ = std::fs::remove_file(p);
        }
        ses.parts.clear();
    } else {
        return Err("No segments available to save".into());
    }

    // 2) System audio muxing (WAV segments -> concat -> mix with video)
    let sys_all: Vec<PathBuf> = ses.sys_parts.iter().filter(|p| p.exists()).cloned().collect();
    if !sys_all.is_empty() {
        let has_mic = ses.settings.mic.as_deref().map(|m| !m.trim().is_empty()).unwrap_or(false);
        let sys_join = ses.out_base.with_extension("sys.wav");
        let joined = if sys_all.len() == 1 {
            std::fs::copy(&sys_all[0], &sys_join).is_ok()
        } else {
            let list_path = ses.out_base.with_extension("syslist.txt");
            let mut list = String::from("ffconcat version 1.0\n");
            for p in &sys_all {
                list.push_str(&format!("file '{}'\n", p.to_string_lossy().replace('\\', "/")));
            }
            let _ = std::fs::write(&list_path, &list);
            let args = vec![
                "-hide_banner".into(),
                "-y".into(),
                "-f".into(),
                "concat".into(),
                "-safe".into(),
                "0".into(),
                "-i".into(),
                list_path.to_string_lossy().into_owned(),
                "-c:a".into(),
                "pcm_f32le".into(),
                sys_join.to_string_lossy().into_owned(),
            ];
            let ok = ffmpeg_util::run_exiting(&exe, &args);
            let _ = std::fs::remove_file(&list_path);
            ok
        };
        let mixed = ses.out_base.with_extension("sysmix.mp4");
        let mix_ok = if !joined {
            false
        } else if has_mic {
            let args = vec![
                "-hide_banner".into(),
                "-y".into(),
                "-i".into(),
                out.to_string_lossy().into_owned(),
                "-i".into(),
                sys_join.to_string_lossy().into_owned(),
                "-filter_complex".into(),
                "[0:a][1:a]amix=inputs=2:duration=first:normalize=0[a]".into(),
                "-map".into(),
                "0:v".into(),
                "-map".into(),
                "[a]".into(),
                "-c:v".into(),
                "copy".into(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "160k".into(),
                "-movflags".into(),
                "+faststart".into(),
                mixed.to_string_lossy().into_owned(),
            ];
            ffmpeg_util::run_exiting(&exe, &args)
        } else {
            let args = vec![
                "-hide_banner".into(),
                "-y".into(),
                "-i".into(),
                out.to_string_lossy().into_owned(),
                "-i".into(),
                sys_join.to_string_lossy().into_owned(),
                "-map".into(),
                "0:v".into(),
                "-map".into(),
                "1:a".into(),
                "-c:v".into(),
                "copy".into(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "160k".into(),
                "-shortest".into(),
                "-movflags".into(),
                "+faststart".into(),
                mixed.to_string_lossy().into_owned(),
            ];
            ffmpeg_util::run_exiting(&exe, &args)
        };
        if mix_ok && mixed.exists() {
            if out.exists() {
                let _ = std::fs::remove_file(&out);
            }
            std::fs::rename(&mixed, &out).map_err(|e| format!("Failed to replace output after audio muxing: {e}"))?;
            for p in &ses.sys_parts {
                let _ = std::fs::remove_file(p);
            }
            let _ = std::fs::remove_file(&sys_join);
        } else {
            eprintln!("[retake] System audio muxing failed; retaining video without system audio and sys.wav");
        }
        ses.sys_parts.clear();
    }

    let size = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
    Ok(Recording {
        file: out.to_string_lossy().into_owned(),
        file_name: ses.file_name.clone(),
        size,
        duration: ses.recorded_ms as f64 / 1000.0,
        created_at: now_ms(),
    })
}

pub fn record_start_impl(app: &AppHandle) -> Result<(), String> {
    println!("[retake] record_start");
    let state = app.state::<AppState>();
    {
        let rec = state.recorder.lock().unwrap();
        match rec.phase {
            Phase::Recording => return Err("Recording is already in progress".into()),
            Phase::Paused => return Err("Recording is paused; please resume or finish first".into()),
            Phase::Countdown(_) => return Err("Countdown is in progress".into()),
            Phase::Idle => {}
        }
    }
    let project = { state.project.lock().unwrap().clone() };
    let ff = ffmpeg_util::detect(project.settings.ffmpeg_path.as_deref());
    if !ff.found {
        let cfg = project.settings.ffmpeg_path.as_deref().unwrap_or("(not set)");
        return Err(format!("ffmpeg not found (current configured path: {cfg}). Please specify in settings, e.g. D:\\ffmpeg\\bin\\ffmpeg.exe"));
    }
    // Capture scope: window mode, or region, or monitor
    let capture_rect = if project.settings.capture_mode == "window" {
        if let Some(hwnd) = project.settings.window_hwnd {
            let (w, h) = window_capture::get_window_client_or_frame_size(hwnd)
                .ok_or("所选目标应用窗口已关闭或不可用，请重新选择窗口")?;
            let bounds = win::get_window_bounds(hwnd).unwrap_or((0, 0, w, h));
            Region { x: bounds.0, y: bounds.1, w, h }
        } else {
            return Err("请先选择要录制的目标应用窗口".into());
        }
    } else {
        match (&project.settings.region, project.settings.monitor) {
            (Some(r), _) => r.clone(),
            (None, Some(idx)) => {
                monitor_rect(app, idx).ok_or("Selected monitor does not exist, please re-select in Recording Scope")?
            }
            (None, None) => {
                return Err("Please select a recording scope: choose a monitor, window or select a region".into());
            }
        }
    };
    let countdown = project.settings.countdown_sec.clamp(0, 10);
    // Hide main window when recording starts
    hide_main(app);
    if countdown == 0 {
        start_recording_now(app, capture_rect)?;
        return Ok(());
    }
    let _ = show_countdown_window(app, project.settings.monitor);
    let flag = Arc::new(AtomicBool::new(false));
    {
        state.recorder.lock().unwrap().phase = Phase::Countdown(flag.clone());
    }
    let app2 = app.clone();
    std::thread::spawn(move || {
        for left in (1..=countdown).rev() {
            if flag.load(Ordering::Relaxed) {
                close_countdown_window(&app2);
                return;
            }
            let _ = app2.emit("rec://countdown", json!({ "left": left }));
            std::thread::sleep(Duration::from_secs(1));
        }
        if flag.load(Ordering::Relaxed) {
            close_countdown_window(&app2);
            return;
        }
        close_countdown_window(&app2);
        if let Err(e) = start_recording_now(&app2, capture_rect) {
            close_hud_window(&app2);
            {
                app2.state::<AppState>().recorder.lock().unwrap().phase = Phase::Idle;
            }
            show_main(&app2);
            emit_phase(&app2);
            let _ = app2.emit("rec://error", json!({ "message": e }));
        }
    });
    emit_phase(app);
    Ok(())
}

fn start_recording_now(app: &AppHandle, capture_rect: Region) -> Result<(), String> {
    let state = app.state::<AppState>();
    let p = state.project.lock().unwrap().clone();
    let settings = p.settings.clone();
    let is_manifest_mode = p.manifest_mode && p.manifest_dir.as_deref().map(|d| !d.trim().is_empty()).unwrap_or(false);
    let dir = if is_manifest_mode {
        let md = PathBuf::from(p.manifest_dir.as_ref().unwrap().trim());
        if !md.exists() {
            let _ = std::fs::create_dir_all(&md);
        }
        md
    } else {
        output_dir(app)?
    };
    let (target_file_name, fname, out_base) = {
        let active = if is_manifest_mode {
            state.active_target.lock().unwrap().clone()
        } else {
            None
        };
        if let Some(mut t) = active {
            t = t.trim().to_string();
            if !t.is_empty() {
                if !t.to_lowercase().ends_with(".mp4") {
                    t.push_str(".mp4");
                }
                let sanitized: String = t
                    .chars()
                    .map(|c| if "<>:\"/\\|?*".contains(c) { '_' } else { c })
                    .collect();
                let out = dir.join(&sanitized);
                (Some(sanitized.clone()), sanitized, out)
            } else {
                let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let f = format!("rec_{stamp}.mp4");
                let out = dir.join(&f);
                (None, f, out)
            }
        } else {
            let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let f = format!("rec_{stamp}.mp4");
            let out = dir.join(&f);
            (None, f, out)
        }
    };
    let has_sys = settings.system_audio;
    let mut ses = Session {
        parts: Vec::new(),
        sys_parts: Vec::new(),
        cur: None,
        cur_sys: None,
        recorded_ms: 0,
        resumed_at: None,
        out_base,
        file_name: fname,
        target_file_name,
        capture_rect: capture_rect.clone(),
        settings: settings.clone(),
        has_sys,
        checkpoints: Vec::new(),
        markers: Vec::new(),
        last_rollback_at: None,
    };
    spawn_part(&mut ses)?;
    {
        let mut rec = state.recorder.lock().unwrap();
        rec.session = Some(ses);
        rec.phase = Phase::Recording;
    }
    let monitor_idx = settings.monitor;
    let _ = show_hud_window(app, &capture_rect, monitor_idx);
    emit_phase(app);
    Ok(())
}

#[tauri::command]
async fn record_start(app: AppHandle, target_filename: Option<String>) -> Result<(), String> {
    let is_manifest_mode = {
        app.state::<AppState>().project.lock().unwrap().manifest_mode
    };
    if is_manifest_mode {
        if let Some(t) = target_filename {
            *app.state::<AppState>().active_target.lock().unwrap() = if t.trim().is_empty() {
                None
            } else {
                Some(t.trim().to_string())
            };
        }
    } else {
        *app.state::<AppState>().active_target.lock().unwrap() = None;
    }
    record_start_impl(&app)
}

#[tauri::command]
async fn record_stop(app: AppHandle) -> Result<Recording, String> {
    match tauri::async_runtime::spawn_blocking(move || stop_recording(&app)).await {
        Ok(r) => r,
        Err(e) => Err(e.to_string()),
    }
}

pub fn stop_recording(app: &AppHandle) -> Result<Recording, String> {
    close_hud_window(app);
    let state = app.state::<AppState>();
    let taken = {
        let mut rec = state.recorder.lock().unwrap();
        match rec.phase.clone() {
            Phase::Countdown(flag) => {
                flag.store(true, Ordering::Relaxed);
                rec.phase = Phase::Idle;
                show_main(app);
                emit_phase(app);
                return Err("Countdown cancelled".into());
            }
            Phase::Recording | Phase::Paused => {
                rec.phase = Phase::Idle;
                rec.session.take()
            }
            Phase::Idle => None,
        }
    };
    let Some(mut ses) = taken else {
        show_main(app);
        emit_phase(app);
        return Err("No active recording in progress".into());
    };
    let recording = match finalize_recording(&mut ses) {
        Ok(t) => t,
        Err(e) => {
            show_main(app);
            emit_phase(app);
            return Err(e);
        }
    };
    show_main(app);
    emit_phase(app);
    let _ = app.emit("rec://saved", json!(&recording));
    Ok(recording)
}

#[tauri::command]
async fn record_cancel(app: AppHandle) -> Result<(), String> {
    match tauri::async_runtime::spawn_blocking(move || cancel_recording(&app, true)).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Cancel current recording. show_after: whether to re-show main window
pub fn cancel_recording(app: &AppHandle, show_after: bool) {
    close_hud_window(app);
    close_countdown_window(app);
    let state = app.state::<AppState>();
    let had = {
        let mut rec = state.recorder.lock().unwrap();
        match rec.phase.clone() {
            Phase::Countdown(flag) => {
                flag.store(true, Ordering::Relaxed);
                rec.phase = Phase::Idle;
                false
            }
            Phase::Recording | Phase::Paused => {
                rec.phase = Phase::Idle;
                rec.session.take().map(|mut ses| {
                    if let Some(h) = ses.cur.take() {
                        ffmpeg_util::kill_now(h.child);
                        let _ = std::fs::remove_file(&h.out);
                    }
                    if let Some(sc) = ses.cur_sys.take() {
                        let _ = sysaudio::stop_sys_capture(sc);
                    }
                    for p in &ses.parts {
                        let _ = std::fs::remove_file(p);
                    }
                    for p in &ses.sys_parts {
                        let _ = std::fs::remove_file(p);
                    }
                    true
                }).unwrap_or(false)
            }
            Phase::Idle => false,
        }
    };
    if show_after {
        show_main(app);
    }
    emit_phase(app);
    if had {
        let _ = app.emit("rec://cancelled", json!({}));
    }
}

#[tauri::command]
async fn record_pause_resume(app: AppHandle) -> Result<(), String> {
    match tauri::async_runtime::spawn_blocking(move || pause_resume_impl(&app)).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn pause_resume_impl(app: &AppHandle) -> Result<(), String> {
    println!("[retake] pause_resume");
    let state = app.state::<AppState>();
    let mut rec = state.recorder.lock().unwrap();
    match rec.phase.clone() {
        Phase::Recording => {
            let mut ses = rec.session.take().ok_or("Invalid recorder state")?;
            if let Err(e) = finish_part(&mut ses, 8000) {
                rec.session = Some(ses);
                return Err(e);
            }
            rec.session = Some(ses);
            rec.phase = Phase::Paused;
            drop(rec);
            emit_phase(app);
            Ok(())
        }
        Phase::Paused => {
            let mut ses = rec.session.take().ok_or("Invalid recorder state")?;
            match spawn_part(&mut ses) {
                Ok(()) => {
                    rec.session = Some(ses);
                    rec.phase = Phase::Recording;
                    drop(rec);
                    emit_phase(app);
                    Ok(())
                }
                Err(e) => {
                    rec.session = Some(ses);
                    Err(e)
                }
            }
        }
        _ => Err("No active recording to pause or resume".into()),
    }
}

fn checkpoint_impl(app: &AppHandle) -> Result<(), String> {
    println!("[retake] checkpoint_impl");
    let state = app.state::<AppState>();
    let mut rec = state.recorder.lock().unwrap();
    let is_recording = matches!(rec.phase, Phase::Recording);
    let is_paused = matches!(rec.phase, Phase::Paused);

    if !is_recording && !is_paused {
        return Err("No active recording in progress".into());
    }

    let mut ses = rec.session.take().ok_or("Recording session does not exist")?;

    if is_recording {
        // Finish current part and store segment
        if let Err(e) = finish_part(&mut ses, 8000) {
            rec.session = Some(ses);
            return Err(e);
        }
    }

    // recorded_ms now includes all finished parts
    let cp_ms = ses.recorded_ms;
    let parts_len = ses.parts.len();
    let sys_parts_len = ses.sys_parts.len();

    ses.checkpoints.push(recorder::Checkpoint {
        parts_len,
        sys_parts_len,
        recorded_ms: cp_ms,
    });
    ses.markers.push(cp_ms as f64 / 1000.0);
    ses.last_rollback_at = None;

    if is_recording {
        // Immediately spawn next part to continue recording without blocking
        if let Err(e) = spawn_part(&mut ses) {
            rec.session = Some(ses);
            return Err(e);
        }
    }

    let markers = ses.markers.clone();
    rec.session = Some(ses);
    drop(rec);

    emit_phase(app);
    let _ = app.emit("rec://checkpoint", json!({
        "ms": cp_ms,
        "markers": markers,
        "count": markers.len(),
    }));

    Ok(())
}

fn rollback_impl(app: &AppHandle) -> Result<(), String> {
    println!("[retake] rollback_impl");
    let state = app.state::<AppState>();
    let mut rec = state.recorder.lock().unwrap();

    let is_recording = matches!(rec.phase, Phase::Recording);
    let is_paused = matches!(rec.phase, Phase::Paused);

    if !is_recording && !is_paused {
        return Err("No active recording in progress".into());
    }

    let mut ses = rec.session.take().ok_or("Recording session does not exist")?;

    // 1. Discard currently active uncompleted segment
    if let Some(h) = ses.cur.take() {
        ffmpeg_util::kill_now(h.child);
        let _ = std::fs::remove_file(&h.out);
    }
    if let Some(sc) = ses.cur_sys.take() {
        let _ = sysaudio::stop_sys_capture(sc);
    }
    ses.resumed_at = None;

    // 2. Check if Alt+G pressed rapidly within 1.5s to pop earlier checkpoints
    let is_rapid_repeat = ses
        .last_rollback_at
        .map(|t| t.elapsed() < Duration::from_millis(1500))
        .unwrap_or(false);

    let (target_ms, msg) = if ses.checkpoints.is_empty() {
        // No checkpoints exist, roll back to beginning (00:00)
        for p in ses.parts.drain(..) {
            let _ = std::fs::remove_file(p);
        }
        for p in ses.sys_parts.drain(..) {
            let _ = std::fs::remove_file(p);
        }
        ses.markers.clear();
        ses.recorded_ms = 0;
        ses.last_rollback_at = Some(Instant::now());
        (0u64, "Rolled back to beginning (00:00)")
    } else {
        if is_rapid_repeat {
            // Repeated rollback: pop latest checkpoint and roll back further
            ses.checkpoints.pop();
            ses.markers.pop();
        }

        if ses.checkpoints.is_empty() {
            for p in ses.parts.drain(..) {
                let _ = std::fs::remove_file(p);
            }
            for p in ses.sys_parts.drain(..) {
                let _ = std::fs::remove_file(p);
            }
            ses.markers.clear();
            ses.recorded_ms = 0;
            ses.last_rollback_at = Some(Instant::now());
            (0u64, "Rolled back to beginning (00:00)")
        } else {
            let cp = ses.checkpoints.last().unwrap().clone();
            // Remove all saved segments after target checkpoint
            while ses.parts.len() > cp.parts_len {
                if let Some(p) = ses.parts.pop() {
                    let _ = std::fs::remove_file(p);
                }
            }
            while ses.sys_parts.len() > cp.sys_parts_len {
                if let Some(p) = ses.sys_parts.pop() {
                    let _ = std::fs::remove_file(p);
                }
            }
            ses.recorded_ms = cp.recorded_ms;
            ses.last_rollback_at = Some(Instant::now());
            (cp.recorded_ms, "Rolled back to checkpoint")
        }
    };

    // 3. Continue recording
    spawn_part(&mut ses)?;
    rec.phase = Phase::Recording;

    let markers = ses.markers.clone();
    rec.session = Some(ses);
    drop(rec);

    emit_phase(app);
    let _ = app.emit("rec://rollback", json!({
        "ms": target_ms,
        "markers": markers,
        "count": markers.len(),
        "message": msg,
    }));

    Ok(())
}

#[tauri::command]
async fn record_checkpoint(app: AppHandle) -> Result<(), String> {
    match tauri::async_runtime::spawn_blocking(move || checkpoint_impl(&app)).await {
        Ok(r) => r,
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn record_rollback(app: AppHandle) -> Result<(), String> {
    match tauri::async_runtime::spawn_blocking(move || rollback_impl(&app)).await {
        Ok(r) => r,
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn get_hud_state(app: AppHandle) -> Result<PhaseInfo, String> {
    let state = app.state::<AppState>();
    let rec = state.recorder.lock().unwrap();
    let active_target = state.active_target.lock().unwrap().clone();
    let manifest_mode = state.project.lock().unwrap().manifest_mode;
    Ok(phase_info(&rec, active_target, manifest_mode))
}

// ---------------- Manifest Commands ----------------

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestDirValidation {
    pub valid: bool,
    pub has_manifest: bool,
    pub file_count: usize,
    pub error: Option<String>,
}

#[tauri::command]
async fn validate_manifest_dir(dir: String) -> Result<ManifestDirValidation, String> {
    let p = std::path::Path::new(&dir);
    if !p.exists() {
        return Ok(ManifestDirValidation {
            valid: false,
            has_manifest: false,
            file_count: 0,
            error: Some("Path does not exist".into()),
        });
    }
    if !p.is_dir() {
        return Ok(ManifestDirValidation {
            valid: false,
            has_manifest: false,
            file_count: 0,
            error: Some("Selected path is not a directory".into()),
        });
    }
    let entries = match std::fs::read_dir(p) {
        Ok(e) => e,
        Err(err) => {
            return Ok(ManifestDirValidation {
                valid: false,
                has_manifest: false,
                file_count: 0,
                error: Some(format!("Cannot read directory: {err}")),
            });
        }
    };
    let mut count = 0;
    let mut has_manifest = false;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.eq_ignore_ascii_case(project::MANIFEST_FILENAME) {
            has_manifest = true;
        }
        count += 1;
    }
    if has_manifest {
        Ok(ManifestDirValidation {
            valid: true,
            has_manifest: true,
            file_count: count,
            error: None,
        })
    } else if count == 0 {
        Ok(ManifestDirValidation {
            valid: true,
            has_manifest: false,
            file_count: 0,
            error: None,
        })
    } else {
        Ok(ManifestDirValidation {
            valid: false,
            has_manifest: false,
            file_count: count,
            error: Some("The directory must either be empty or contain a retake_manifest.json file".into()),
        })
    }
}

fn resolve_manifest_dir(app: &AppHandle, dir: Option<String>) -> Result<std::path::PathBuf, String> {
    if let Some(d) = dir {
        if !d.trim().is_empty() {
            return Ok(std::path::PathBuf::from(d.trim()));
        }
    }
    let p = app.state::<AppState>().project.lock().unwrap().clone();
    if let Some(md) = p.manifest_dir {
        if !md.trim().is_empty() {
            return Ok(std::path::PathBuf::from(md.trim()));
        }
    }
    output_dir(app)
}

#[tauri::command]
async fn load_manifest(app: AppHandle, dir: Option<String>) -> Result<Option<project::RecordingManifest>, String> {
    let target_dir = resolve_manifest_dir(&app, dir)?;
    Ok(project::load_manifest_from_dir(&target_dir))
}

#[tauri::command]
async fn save_manifest(app: AppHandle, manifest: project::RecordingManifest, dir: Option<String>) -> Result<(), String> {
    let target_dir = resolve_manifest_dir(&app, dir)?;
    if !target_dir.exists() {
        let _ = std::fs::create_dir_all(&target_dir);
    }
    project::save_manifest_to_dir(&target_dir, &manifest)
}

#[tauri::command]
async fn set_active_manifest_target(app: AppHandle, target_filename: Option<String>) -> Result<(), String> {
    let cleaned = target_filename.and_then(|t| {
        let s = t.trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    });
    *app.state::<AppState>().active_target.lock().unwrap() = cleaned;
    emit_phase(&app);
    Ok(())
}

#[tauri::command]
async fn open_manifest_file(app: AppHandle, dir: Option<String>) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let target_dir = resolve_manifest_dir(&app, dir)?;
    if !target_dir.exists() {
        let _ = std::fs::create_dir_all(&target_dir);
    }
    let file_path = target_dir.join(project::MANIFEST_FILENAME);
    if !file_path.exists() {
        let default_manifest = project::RecordingManifest {
            version: 1,
            title: "Recording Manifest".into(),
            active_item_id: None,
            items: vec![],
        };
        let _ = project::save_manifest_to_dir(&target_dir, &default_manifest);
    }
    app.opener()
        .open_path(file_path.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}

// ---------------- Overlays and Floating Windows ----------------

fn show_countdown_window(app: &AppHandle, monitor_index: Option<usize>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("countdown") {
        let _ = w.close();
    }
    let m = match monitor_index {
        Some(idx) => {
            let ms = app.available_monitors().map_err(|e| e.to_string())?;
            ms.get(idx).cloned().or_else(|| app.primary_monitor().ok().flatten())
        }
        None => app.primary_monitor().map_err(|e| e.to_string())?,
    }.ok_or("Monitor not found")?;
    let scale = m.scale_factor();
    let pos = m.position();
    let size = m.size();
    let w = WebviewWindowBuilder::new(app, "countdown", WebviewUrl::App("index.html#/countdown".into()))
        .title("Retake Countdown")
        .position(pos.x as f64 / scale, pos.y as f64 / scale)
        .inner_size(size.width as f64 / scale, size.height as f64 / scale)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;
    let record_app = app.state::<AppState>().project.lock().unwrap().settings.record_app_windows;
    win::set_window_capture_exclusion(&w, !record_app);
    let _ = w.set_focus();
    Ok(())
}

fn close_countdown_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("countdown") {
        let _ = w.close();
    }
}

fn find_monitor_for_rect(app: &AppHandle, rect: &Region) -> Option<usize> {
    let monitors = app.available_monitors().ok()?;
    let cx = rect.x + rect.w / 2;
    let cy = rect.y + 10;
    for (i, m) in monitors.iter().enumerate() {
        let pos = m.position();
        let size = m.size();
        if cx >= pos.x && cx < (pos.x + size.width as i32) &&
           cy >= pos.y && cy < (pos.y + size.height as i32) {
            return Some(i);
        }
    }
    None
}

fn show_hud_window(app: &AppHandle, capture_rect: &Region, monitor_index: Option<usize>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("hud") {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    let target_idx = monitor_index.or_else(|| find_monitor_for_rect(app, capture_rect));
    let m = match target_idx {
        Some(idx) => {
            let ms = app.available_monitors().map_err(|e| e.to_string())?;
            ms.get(idx).cloned().or_else(|| app.primary_monitor().ok().flatten())
        }
        None => app.primary_monitor().map_err(|e| e.to_string())?,
    }.ok_or("Monitor not found")?;

    let scale = m.scale_factor();
    let pos = m.position();
    let size = m.size();
    let hud_w = 460.0;
    let hud_h = 76.0;
    let hud_x = (pos.x as f64 / scale) + ((size.width as f64 / scale) - hud_w) / 2.0;
    let hud_y = (pos.y as f64 / scale) + 16.0;

    let w = WebviewWindowBuilder::new(app, "hud", WebviewUrl::App("index.html#/hud".into()))
        .title("Retake HUD")
        .position(hud_x, hud_y)
        .inner_size(hud_w, hud_h)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;
    let record_app = app.state::<AppState>().project.lock().unwrap().settings.record_app_windows;
    win::set_window_capture_exclusion(&w, !record_app);
    Ok(())
}

fn close_hud_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("hud") {
        let _ = w.close();
    }
}

#[tauri::command]
async fn list_app_windows() -> Result<Vec<win::AppWindowInfo>, String> {
    Ok(win::list_windows())
}

#[tauri::command]
async fn get_app_window_bounds(hwnd: isize) -> Result<Option<Region>, String> {
    Ok(win::get_window_bounds(hwnd).map(|(x, y, w, h)| Region { x, y, w, h }))
}

#[tauri::command]
async fn show_scope_preview(app: AppHandle, rect: Region) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("scope_preview") {
        let _ = w.close();
    }
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let cx = rect.x + rect.w / 2;
    let cy = rect.y + rect.h / 2;
    let (monitor_idx, target_monitor) = monitors
        .iter()
        .enumerate()
        .find(|(_, m)| {
            let px = m.position().x;
            let py = m.position().y;
            let pw = m.size().width as i32;
            let ph = m.size().height as i32;
            cx >= px && cx < px + pw && cy >= py && cy < py + ph
        })
        .or_else(|| monitors.iter().enumerate().next())
        .ok_or("No monitor found for scope preview")?;

    let scale = target_monitor.scale_factor();
    let pos = target_monitor.position();
    let size = target_monitor.size();

    let mon_info = MonitorInfo {
        index: monitor_idx,
        name: target_monitor.name().cloned().unwrap_or_else(|| format!("Display {}", monitor_idx + 1)),
        x: pos.x,
        y: pos.y,
        w: size.width as i32,
        h: size.height as i32,
        scale,
        is_primary: pos.x == 0 && pos.y == 0,
    };

    let preview_info = ScopePreviewInfo {
        rect: rect.clone(),
        monitor: mon_info,
    };

    *app.state::<AppState>().scope_preview_info.lock().unwrap() = Some(preview_info);

    let w = WebviewWindowBuilder::new(&app, "scope_preview", WebviewUrl::App("index.html#/scope-preview".into()))
        .title("Retake Scope Preview")
        .position(pos.x as f64 / scale, pos.y as f64 / scale)
        .inner_size(size.width as f64 / scale, size.height as f64 / scale)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;

    let record_app = app.state::<AppState>().project.lock().unwrap().settings.record_app_windows;
    win::set_window_capture_exclusion(&w, !record_app);
    Ok(())
}

#[tauri::command]
async fn close_scope_preview(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("scope_preview") {
        let _ = w.close();
    }
    *app.state::<AppState>().scope_preview_info.lock().unwrap() = None;
    let _ = app.emit("scope-preview://closed", ());
    Ok(())
}

#[tauri::command]
async fn get_scope_preview_info(app: AppHandle) -> Result<Option<ScopePreviewInfo>, String> {
    Ok(app.state::<AppState>().scope_preview_info.lock().unwrap().clone())
}

#[tauri::command]
async fn start_region_select(app: AppHandle, monitor_index: Option<usize>) -> Result<(), String> {
    println!("[retake] start_region_select monitor={:?}", monitor_index);
    let m = match monitor_index {
        Some(idx) => {
            let ms = app.available_monitors().map_err(|e| e.to_string())?;
            ms.get(idx).cloned().ok_or("Selected monitor does not exist")?
        }
        None => app.primary_monitor().map_err(|e| e.to_string())?.ok_or("Monitor not found")?,
    };
    *app.state::<AppState>().region_monitor.lock().unwrap() = monitor_index;
    if let Some(w) = app.get_webview_window("selector") {
        let _ = w.close();
    }
    let scale = m.scale_factor();
    let pos = m.position();
    let size = m.size();
    let w = WebviewWindowBuilder::new(&app, "selector", WebviewUrl::App("index.html#/selector".into()))
        .title("Select Recording Area")
        .position(pos.x as f64 / scale, pos.y as f64 / scale)
        .inner_size(size.width as f64 / scale, size.height as f64 / scale)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;
    let record_app = app.state::<AppState>().project.lock().unwrap().settings.record_app_windows;
    win::set_window_capture_exclusion(&w, !record_app);
    Ok(())
}

#[derive(Deserialize)]
pub struct RegionPick {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[tauri::command]
async fn finish_region_select(app: AppHandle, pick: Option<RegionPick>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("selector") {
        let _ = w.close();
    }
    if let Some(pk) = pick {
        if pk.w < 8.0 || pk.h < 8.0 {
            return Ok(());
        }
        let monitor_index = *app.state::<AppState>().region_monitor.lock().unwrap();
        let m = match monitor_index {
            Some(idx) => app
                .available_monitors()
                .map_err(|e| e.to_string())?
                .get(idx)
                .cloned()
                .ok_or("Selected monitor does not exist")?,
            None => app.primary_monitor().map_err(|e| e.to_string())?.ok_or("Monitor not found")?,
        };
        let scale = m.scale_factor();
        let pos = m.position();
        let region = Region {
            x: pos.x + (pk.x * scale) as i32,
            y: pos.y + (pk.y * scale) as i32,
            w: (pk.w * scale) as i32,
            h: (pk.h * scale) as i32,
        };
        {
            let state = app.state::<AppState>();
            let mut p = state.project.lock().unwrap();
            p.settings.region = Some(region.clone());
            let _ = project::save(&app, &p);
        }
        let _ = app.emit_to("main", "region://updated", json!({ "x": region.x, "y": region.y, "w": region.w, "h": region.h }));
    }
    Ok(())
}

// ---------------- FFmpeg Utilities ----------------

#[tauri::command]
async fn check_ffmpeg(path: Option<String>) -> Result<FfmpegInfo, String> {
    match tauri::async_runtime::spawn_blocking(move || ffmpeg_util::detect(path.as_deref())).await {
        Ok(f) => Ok(f),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn list_mics(path: Option<String>) -> Result<Vec<String>, String> {
    match tauri::async_runtime::spawn_blocking(move || ffmpeg_util::list_mics(path.as_deref())).await {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_string()),
    }
}

// ---------------- Global Shortcuts ----------------
// Alt+1 Start/Stop · Alt+2 Pause/Resume · Alt+3 Cancel · Alt+F Checkpoint · Alt+G Rollback

fn shortcut_of(s: &str) -> Shortcut {
    Shortcut::from_str(s).expect("Invalid shortcut")
}

fn handle_shortcut(app: &AppHandle, sc: &Shortcut) {
    if *sc == shortcut_of("alt+1") {
        let ph = { app.state::<AppState>().recorder.lock().unwrap().phase.clone() };
        match ph {
            Phase::Idle => {
                let _ = record_start_impl(app);
            }
            Phase::Countdown(_) => {
                let a = app.clone();
                std::thread::spawn(move || cancel_recording(&a, true));
            }
            Phase::Recording | Phase::Paused => {
                let a = app.clone();
                std::thread::spawn(move || {
                    let _ = stop_recording(&a);
                });
            }
        }
    } else if *sc == shortcut_of("alt+2") {
        let a = app.clone();
        std::thread::spawn(move || {
            let _ = pause_resume_impl(&a);
        });
    } else if *sc == shortcut_of("alt+3") {
        let a = app.clone();
        std::thread::spawn(move || {
            cancel_recording(&a, true);
        });
    } else if *sc == shortcut_of("alt+f") {
        let a = app.clone();
        std::thread::spawn(move || {
            let _ = checkpoint_impl(&a);
        });
    } else if *sc == shortcut_of("alt+g") {
        let a = app.clone();
        std::thread::spawn(move || {
            let _ = rollback_impl(&a);
        });
    }
}

fn shutdown_kill(app: &AppHandle) {
    close_hud_window(app);
    close_countdown_window(app);
    let state = app.state::<AppState>();
    let mut rec = state.recorder.lock().unwrap();
    if let Phase::Countdown(f) = rec.phase.clone() {
        f.store(true, Ordering::Relaxed);
        rec.phase = Phase::Idle;
    }
    if let Some(mut ses) = rec.session.take() {
        rec.phase = Phase::Idle;
        if let Some(mut h) = ses.cur.take() {
            let _ = h.child.kill();
            let _ = h.child.wait();
            let _ = std::fs::remove_file(&h.out);
        }
        if let Some(sc) = ses.cur_sys.take() {
            let _ = sysaudio::stop_sys_capture(sc);
        }
        for p in &ses.parts {
            let _ = std::fs::remove_file(p);
        }
        for p in &ses.sys_parts {
            let _ = std::fs::remove_file(p);
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, sc, event| {
                    if event.state == ShortcutState::Pressed {
                        handle_shortcut(app, sc);
                    }
                })
                .build(),
        )
        .manage(AppState {
            project: Mutex::new(Project::default()),
            recorder: Mutex::new(Recorder::new()),
            region_monitor: Mutex::new(None),
            active_target: Mutex::new(None),
            scope_preview_info: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            save_project,
            list_monitors,
            record_start,
            record_stop,
            record_cancel,
            record_pause_resume,
            record_checkpoint,
            record_rollback,
            get_hud_state,
            start_region_select,
            finish_region_select,
            check_ffmpeg,
            list_mics,
            load_manifest,
            save_manifest,
            set_active_manifest_target,
            open_manifest_file,
            validate_manifest_dir,
            list_app_windows,
            get_app_window_bounds,
            show_scope_preview,
            close_scope_preview,
            get_scope_preview_info,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let p = project::load(&handle);
            *handle.state::<AppState>().project.lock().unwrap() = p.clone();
            apply_app_exclusion(&handle, p.settings.record_app_windows);
            let gs = handle.global_shortcut();
            for sc in ["alt+1", "alt+2", "alt+3", "alt+f", "alt+g"] {
                gs.register(sc).map_err(|e| format!("Failed to register shortcut {sc}: {e}"))?;
            }

            // Real-time audio peak level polling (lightweight WASAPI peak query for frontend visualization)
            let meter_handle = handle.clone();
            std::thread::spawn(move || {
                loop {
                    let (mic_on, sys_on) = {
                        match meter_handle.try_state::<AppState>() {
                            Some(state) => match state.project.try_lock() {
                                Ok(p) => (p.settings.mic.is_some(), p.settings.system_audio),
                                Err(_) => (false, false),
                            },
                            None => break,
                        }
                    };
                    if mic_on || sys_on {
                        let (m, s) = sysaudio::get_audio_levels(mic_on, sys_on);
                        let _ = meter_handle.emit("audio://levels", serde_json::json!({ "mic": m, "sys": s }));
                        std::thread::sleep(Duration::from_millis(50));
                    } else {
                        std::thread::sleep(Duration::from_millis(150));
                    }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            use tauri::RunEvent;
            match event {
                RunEvent::ExitRequested { .. } | RunEvent::Exit => shutdown_kill(app),
                RunEvent::WindowEvent { label, event: tauri::WindowEvent::CloseRequested { .. }, .. } if label == "main" => {
                    app.exit(0);
                }
                _ => {}
            }
        });
}
