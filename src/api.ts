import { invoke } from '@tauri-apps/api/core'

export interface Region {
  x: number
  y: number
  w: number
  h: number
}

export interface Settings {
  fps: number
  crf: number
  mic: string | null
  ffmpegPath: string | null
  region: Region | null
  monitor: number | null
  countdownSec: number
  captureCursor: boolean
  systemAudio: boolean
  excludeSelf: boolean
}

export interface Project {
  outputDir: string | null
  settings: Settings
}

export interface Recording {
  file: string
  fileName: string
  size: number
  duration: number
  createdAt: number
}

export interface FfmpegInfo {
  found: boolean
  version: string | null
  path: string | null
}

export interface MonitorInfo {
  index: number
  name: string
  x: number
  y: number
  w: number
  h: number
  scale: number
  isPrimary: boolean
}

export interface PhaseView {
  phase: 'idle' | 'countdown' | 'recording' | 'paused'
  recordedMs?: number | null
  resumedAt?: number | null
  markers?: number[]
  targetFileName?: string | null
}

export interface ManifestItem {
  id: string
  title: string
  targetFileName: string
  status: 'pending' | 'recorded'
  duration?: number | null
  recordedAt?: number | null
  notes?: string
}

export interface RecordingManifest {
  version: number
  title: string
  activeItemId?: string | null
  items: ManifestItem[]
}

export interface Bootstrap {
  project: Project
  ffmpeg: FfmpegInfo
  mics: string[]
  monitors: MonitorInfo[]
  outputDir: string
  phase: PhaseView
}

export const api = {
  bootstrap: () => invoke<Bootstrap>('bootstrap'),
  saveProject: (project: Project) => invoke<void>('save_project', { project }),
  recordStart: (targetFilename?: string | null) =>
    invoke<void>('record_start', { targetFilename: targetFilename ?? null }),
  recordStop: () => invoke<Recording>('record_stop'),
  recordCancel: () => invoke<void>('record_cancel'),
  recordPauseResume: () => invoke<void>('record_pause_resume'),
  recordCheckpoint: () => invoke<void>('record_checkpoint'),
  recordRollback: () => invoke<void>('record_rollback'),
  getHudState: () => invoke<PhaseView>('get_hud_state'),
  loadManifest: (dir?: string | null) =>
    invoke<RecordingManifest | null>('load_manifest', { dir: dir ?? null }),
  saveManifest: (manifest: RecordingManifest, dir?: string | null) =>
    invoke<void>('save_manifest', { manifest, dir: dir ?? null }),
  setActiveManifestTarget: (targetFilename?: string | null) =>
    invoke<void>('set_active_manifest_target', { targetFilename: targetFilename ?? null }),
  openManifestFile: (dir?: string | null) =>
    invoke<void>('open_manifest_file', { dir: dir ?? null }),
  startRegionSelect: (monitorIndex: number | null) =>
    invoke<void>('start_region_select', { monitorIndex }),
  finishRegionSelect: (pick: { x: number; y: number; w: number; h: number } | null) =>
    invoke<void>('finish_region_select', { pick }),
  checkFfmpeg: (path: string | null) => invoke<FfmpegInfo>('check_ffmpeg', { path }),
  listMics: (path: string | null) => invoke<string[]>('list_mics', { path }),
  listMonitors: () => invoke<MonitorInfo[]>('list_monitors'),
}
