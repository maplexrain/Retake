use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recording {
    pub file: String,
    pub file_name: String,
    pub size: u64,
    pub duration: f64,
    pub created_at: u64,
}

/// Recording region: physical absolute coordinates (origin at primary monitor top-left, matching gdigrab offsets).
#[derive(Serialize, Deserialize, Clone)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub fps: u32,
    pub crf: u32,
    pub mic: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub region: Option<Region>,
    /// Selected monitor index (effective when region is None; recording blocked if both are None)
    pub monitor: Option<usize>,
    /// Countdown duration in seconds: 0-10 (0 = start immediately)
    pub countdown_sec: u32,
    /// Whether to capture mouse cursor
    pub capture_cursor: bool,
    /// Whether to record system audio (WASAPI loopback capture on default render device)
    pub system_audio: bool,
    /// Capture mode: "display" | "region" | "window"
    pub capture_mode: String,
    /// Selected window hwnd when capture_mode is "window"
    pub window_hwnd: Option<isize>,
    /// Selected window title when capture_mode is "window"
    pub window_title: Option<String>,
    /// Whether application windows (main deck, HUD, countdown) are allowed to be recorded.
    /// Default: false (Retake UI is excluded from recording/screenshots).
    /// When true: allows all UI to be recorded/screenshotted.
    pub record_app_windows: bool,
    /// Whether main window is excluded from screen capture (default: true)
    pub exclude_self: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: 30,
            crf: 23,
            mic: None,
            ffmpeg_path: None,
            region: None,
            monitor: None,
            countdown_sec: 3,
            capture_cursor: true,
            system_audio: false,
            capture_mode: "display".into(),
            window_hwnd: None,
            window_title: None,
            record_app_windows: false,
            exclude_self: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Project {
    pub output_dir: Option<String>,
    pub settings: Settings,
    pub manifest_mode: bool,
    pub manifest_dir: Option<String>,
}

pub const MANIFEST_FILENAME: &str = "retake_manifest.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestItem {
    pub id: String,
    pub title: String,
    pub target_file_name: String,
    /// "pending" | "recorded"
    pub status: String,
    pub duration: Option<f64>,
    pub recorded_at: Option<u64>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecordingManifest {
    pub version: u32,
    pub title: String,
    pub active_item_id: Option<String>,
    pub items: Vec<ManifestItem>,
}

pub fn load_manifest_from_dir(dir: &std::path::Path) -> Option<RecordingManifest> {
    let file = dir.join(MANIFEST_FILENAME);
    let s = std::fs::read_to_string(file).ok()?;
    serde_json::from_str(&s).ok()
}

pub fn save_manifest_to_dir(dir: &std::path::Path, manifest: &RecordingManifest) -> Result<(), String> {
    let file = dir.join(MANIFEST_FILENAME);
    let s = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    std::fs::write(file, s).map_err(|e| e.to_string())
}

pub fn load(app: &tauri::AppHandle) -> Project {
    let dir = app.path().app_data_dir().unwrap_or_else(|_| std::env::temp_dir());
    let file = dir.join("project.json");
    match std::fs::read_to_string(&file) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Project::default(),
    }
}

pub fn save(app: &tauri::AppHandle, p: &Project) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = dir.join("project.json");
    let s = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
    std::fs::write(file, s).map_err(|e| e.to_string())
}
