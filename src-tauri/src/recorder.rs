use crate::ffmpeg_util::CaptureHandle;
use crate::project::{Region, Settings};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct CountdownControl {
    pub cancel: Arc<AtomicBool>,
    pub skip: Arc<AtomicBool>,
}

#[derive(Clone)]
pub enum Phase {
    Idle,
    Countdown(CountdownControl),
    Recording,
    Paused,
}

#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub parts_len: usize,
    pub sys_parts_len: usize,
    pub recorded_ms: u64,
}

pub struct Session {
    pub parts: Vec<PathBuf>,
    pub sys_parts: Vec<PathBuf>,
    pub cur: Option<CaptureHandle>,
    pub cur_sys: Option<crate::sysaudio::SysSeg>,
    pub recorded_ms: u64,
    /// Timestamp when current segment started (None = paused)
    pub resumed_at: Option<Instant>,
    /// Final output file path (segments are named X.pNN.mp4 alongside it)
    pub out_base: PathBuf,
    pub file_name: String,
    pub target_file_name: Option<String>,
    pub capture_rect: Region,
    pub settings: Settings,
    pub has_sys: bool,
    pub checkpoints: Vec<Checkpoint>,
    pub markers: Vec<f64>,
    pub last_rollback_at: Option<Instant>,
}

impl Session {
    pub fn recorded_total_ms(&self) -> u64 {
        self.recorded_ms
            + self
                .resumed_at
                .map(|t| t.elapsed().as_millis() as u64)
                .unwrap_or(0)
    }
}

pub struct Recorder {
    pub phase: Phase,
    pub session: Option<Session>,
}

impl Recorder {
    pub fn new() -> Self {
        Self { phase: Phase::Idle, session: None }
    }
}
