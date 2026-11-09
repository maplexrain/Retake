import { useEffect, useState } from 'react'
import { emit, listen } from '@tauri-apps/api/event'

export type Lang = 'en' | 'zh'

const STORAGE_KEY = 'retake_lang'

export const translations = {
  en: {
    // Brand & Status
    'brand.badge': 'Screen Recorder',
    'status.recording': 'REC {time}',
    'status.paused': '⏸ Paused · {time}',
    'status.counting': 'Countdown {sec}s',
    'status.ready': 'Ready',
    'titlebar.openOutput': 'Open output folder',
    'titlebar.minimize': 'Minimize',
    'titlebar.maximize': 'Maximize / Restore',
    'titlebar.close': 'Close',
    'titlebar.switchLang': 'Switch Language (EN / 中文)',

    // Boot & Banner
    'boot.loading': 'Preparing Retake recording environment…',
    'boot.initFailed': 'Initialization failed: {err}',
    'banner.noFfmpeg': '⚠️ FFmpeg was not detected on this system. Recording engine is unavailable.',
    'banner.recheck': 'Recheck',

    // Scope
    'scope.title': 'Capture Scope',
    'scope.customRegion': 'Custom Region',
    'scope.selectMonitor': 'Select Display…',
    'scope.primary': ' [Primary]',
    'scope.crop': 'Crop',
    'scope.cropHint': 'Drag a transparent overlay to crop recording area',
    'scope.clear': 'Clear',
    'scope.regionLabel': '⛶ Region {w}×{h}',

    // Audio
    'audio.title': 'Audio Capture',
    'audio.refresh': 'Refresh',
    'audio.refreshHint': 'Refresh microphone device list',
    'audio.selectMic': 'Select Microphone…',
    'audio.noMic': 'No device detected',
    'audio.toggleMicOn': 'Enable microphone',
    'audio.toggleMicOff': 'Disable microphone',
    'audio.system': 'System Audio',
    'audio.toggleSysOn': 'Enable system audio',
    'audio.toggleSysOff': 'Disable system audio',
    'audio.hearing': 'Active',
    'audio.silent': 'Silent',
    'audio.hearingHint': 'Audio active (level: {pct}%)',
    'audio.silentHint': 'Ambient silence',
    'audio.micFound': 'Found {count} microphone(s)',
    'audio.noMicFound': 'No microphone devices found',

    // Hero & Controls
    'hero.start': 'Record',
    'hero.startHint': 'Start Recording (Alt+1)',
    'hero.stopHint': 'Finish & Save Recording (Alt+1)',
    'hero.pauseHint': 'Pause Recording (Alt+2)',
    'hero.resumeHint': 'Resume Recording (Alt+2)',
    'hero.pause': 'Pause',
    'hero.resume': 'Resume',
    'hero.cancel': 'Cancel',
    'hero.cancelHint': 'Discard & Cancel Recording (Alt+3)',
    'hero.counting': 'Starting Soon',
    'hero.countingTitle': 'Counting down...',
    'hero.pausedLabel': 'Paused · {time}',
    'hero.errNoFfmpeg': 'FFmpeg is not ready',
    'hero.errNoScope': 'Please select a display or crop an area first',

    // Settings Drawer
    'settings.title': 'Advanced Settings & Output',
    'settings.fps': 'Frame Rate',
    'settings.countdown': 'Countdown',
    'settings.noCountdown': 'No Countdown',
    'settings.sec': '{sec}s',
    'settings.quality': 'Encoding Quality',
    'settings.crfUltra': 'Ultra (CRF 18)',
    'settings.crfHigh': 'High (CRF 23)',
    'settings.crfCompact': 'Compact (CRF 28)',
    'settings.captureCursor': 'Capture Mouse Cursor',
    'settings.excludeSelf': 'Exclude Main Window from Capture',
    'settings.outDir': 'Recording Output Folder',
    'settings.chooseDir': 'Change output directory',
    'settings.openDir': 'Open in File Explorer',
    'settings.ffmpegPath': 'FFmpeg Path',
    'settings.ffmpegPlaceholder': 'Leave blank to auto-detect from system PATH',
    'settings.recheck': 'Check',
    'settings.recheckHint': 'Recheck FFmpeg',
    'settings.ready': 'Ready ({ver})',
    'settings.notFound': 'Not Found',

    // Footer & Shortcuts
    'foot.estimate': 'Size estimate: ~{mb} MB / min',
    'foot.estimateEmpty': 'Select recording scope to estimate file size',
    'sc.record': 'Alt+1 Record/Stop',
    'sc.pause': 'Alt+2 Pause/Resume',
    'sc.checkpoint': 'Alt+F Checkpoint',
    'sc.rollback': 'Alt+G Rollback',
    'sc.cancel': 'Alt+3 Cancel',

    // Toasts
    'toast.saved': 'Recording saved: {file}',
    'toast.cancelled': 'Recording cancelled',
    'toast.saveFailed': 'Save failed: {err}',
    'toast.pathInvalid': 'Path is unavailable',
    'toast.openFailed': 'Failed to open: {err}',
    'toast.ffmpegReady': 'ffmpeg {ver} is ready',
    'toast.ffmpegNotFound': 'ffmpeg not found',

    // Floating HUD
    'hud.recording': 'Recording',
    'hud.paused': 'Paused',
    'hud.recHint': 'Recording in progress',
    'hud.pauseHint': 'Recording paused',
    'hud.checkpointCount': 'Total marked checkpoints',
    'hud.recTime': 'Effective recorded duration',
    'hud.checkpointBtn': 'Mark',
    'hud.checkpointHint': 'Mark checkpoint (Shortcut Alt+F)',
    'hud.checkpointTitle': 'Checkpoint #{num} ({time})',
    'hud.rollbackBtn': 'Rollback',
    'hud.rollbackHint': 'Rollback to previous checkpoint & discard bad take (Shortcut Alt+G)',
    'hud.resumeHint': 'Resume recording (Alt+2)',
    'hud.pauseActionHint': 'Pause recording (Alt+2)',
    'hud.stopBtn': '⏹ Finish',
    'hud.stopHint': 'Finish & save recording (Alt+1)',
    'hud.cancelHint': 'Discard & cancel recording (Alt+3)',
    'hud.toastCheckpoint': '✓ Marked checkpoint #{num} ({time})',
    'hud.toastRollback': '⤺ Rolled back to {target} · continuing',
    'hud.beginning': 'beginning 00:00',

    // Countdown Overlay
    'countdown.title': 'Recording Starting Soon',
    'countdown.fullscreen': '🖥 Full Screen Capture',
    'countdown.region': '⛶ Region {w}×{h}',
    'countdown.micOk': '🎤 Microphone Ready',
    'countdown.sysOk': '🔊 System Audio On',
    'countdown.noCursor': '🚫 Cursor Hidden',
    'countdown.cancel': 'Cancel Recording',
    'countdown.cancelHint': 'Cancel recording (ESC)',

    // Selector
    'selector.hint': 'Drag to crop recording area · Esc to cancel',
  },
  zh: {
    // Brand & Status
    'brand.badge': '录屏',
    'status.recording': 'REC {time}',
    'status.paused': '⏸ 已暂停 · {time}',
    'status.counting': '倒计时 {sec}s',
    'status.ready': '就绪',
    'titlebar.openOutput': '打开输出文件夹',
    'titlebar.minimize': '最小化',
    'titlebar.maximize': '最大化/还原',
    'titlebar.close': '退出',
    'titlebar.switchLang': '切换语言 (EN / 中文)',

    // Boot & Banner
    'boot.loading': '正在准备 Retake 录屏环境…',
    'boot.initFailed': '初始化失败: {err}',
    'banner.noFfmpeg': '⚠️ 系统未检测到 ffmpeg，录制引擎不可用。',
    'banner.recheck': '重新检测',

    // Scope
    'scope.title': '录制范围',
    'scope.customRegion': '自定义选区',
    'scope.selectMonitor': '选择显示器…',
    'scope.primary': ' [主]',
    'scope.crop': '框选',
    'scope.cropHint': '全屏透明浮层框选录制范围',
    'scope.clear': '清除',
    'scope.regionLabel': '⛶ 选区 {w}×{h}',

    // Audio
    'audio.title': '音频收声',
    'audio.refresh': '刷新',
    'audio.refreshHint': '刷新麦克风设备列表',
    'audio.selectMic': '选择麦克风…',
    'audio.noMic': '未检测到设备',
    'audio.toggleMicOn': '开启麦克风',
    'audio.toggleMicOff': '关闭麦克风',
    'audio.system': '系统声音',
    'audio.toggleSysOn': '开启系统声音',
    'audio.toggleSysOff': '关闭系统声音',
    'audio.hearing': '收声中',
    'audio.silent': '静音',
    'audio.hearingHint': '收声正常（电平：{pct}%）',
    'audio.silentHint': '当前环境静音中',
    'audio.micFound': '发现 {count} 个麦克风',
    'audio.noMicFound': '未发现麦克风设备',

    // Hero & Controls
    'hero.start': '开始录制',
    'hero.startHint': '开始录制（Alt+1）',
    'hero.stopHint': '完成并保存录屏（Alt+1）',
    'hero.pauseHint': '暂停录制（Alt+2）',
    'hero.resumeHint': '恢复录制（Alt+2）',
    'hero.pause': '暂停',
    'hero.resume': '继续',
    'hero.cancel': '取消',
    'hero.cancelHint': '作废取消当前录制（Alt+3）',
    'hero.counting': '即将开始',
    'hero.countingTitle': '倒计时中...',
    'hero.pausedLabel': '已暂停 · {time}',
    'hero.errNoFfmpeg': 'ffmpeg 未就绪',
    'hero.errNoScope': '请先选择显示器或框选区域',

    // Settings Drawer
    'settings.title': '高级录制参数与路径',
    'settings.fps': '录屏帧率',
    'settings.countdown': '开始倒计时',
    'settings.noCountdown': '无倒计时',
    'settings.sec': '{sec}秒',
    'settings.quality': '编码画质',
    'settings.crfUltra': '极清 (CRF 18)',
    'settings.crfHigh': '高清 (CRF 23)',
    'settings.crfCompact': '紧凑 (CRF 28)',
    'settings.captureCursor': '捕获鼠标指针',
    'settings.excludeSelf': '主窗口不入镜',
    'settings.outDir': '录屏保存路径',
    'settings.chooseDir': '更改保存目录',
    'settings.openDir': '在文件管理器中打开',
    'settings.ffmpegPath': 'ffmpeg 路径',
    'settings.ffmpegPlaceholder': '留空自动使用系统 PATH 中的 ffmpeg',
    'settings.recheck': '检测',
    'settings.recheckHint': '重新检测 ffmpeg',
    'settings.ready': '已就绪 ({ver})',
    'settings.notFound': '未找到',

    // Footer & Shortcuts
    'foot.estimate': '画质估算：约 {mb} MB / 分钟',
    'foot.estimateEmpty': '选择录制范围后显示体积估算',
    'sc.record': 'Alt+1 录制/完成',
    'sc.pause': 'Alt+2 暂停/恢复',
    'sc.checkpoint': 'Alt+F 标记回溯',
    'sc.rollback': 'Alt+G 回溯',
    'sc.cancel': 'Alt+3 取消',

    // Toasts
    'toast.saved': '录制已保存：{file}',
    'toast.cancelled': '录制已取消',
    'toast.saveFailed': '保存失败: {err}',
    'toast.pathInvalid': '路径不可用',
    'toast.openFailed': '打开失败: {err}',
    'toast.ffmpegReady': 'ffmpeg {ver} 已就绪',
    'toast.ffmpegNotFound': '未找到 ffmpeg',

    // Floating HUD
    'hud.recording': '录制中',
    'hud.paused': '已暂停',
    'hud.recHint': '正在录制',
    'hud.pauseHint': '录制已暂停',
    'hud.checkpointCount': '已标记的回溯点总数',
    'hud.recTime': '当前已录制有效时长',
    'hud.checkpointBtn': '标记',
    'hud.checkpointHint': '标记回溯点（快捷键 Alt+F）',
    'hud.checkpointTitle': '回溯点 #{num} ({time})',
    'hud.rollbackBtn': '回溯',
    'hud.rollbackHint': '回溯到上一个回溯点，丢弃之后片段（快捷键 Alt+G）',
    'hud.resumeHint': '继续录制（Alt+2）',
    'hud.pauseActionHint': '暂停录制（Alt+2）',
    'hud.stopBtn': '⏹ 完成',
    'hud.stopHint': '完成并保存录屏（Alt+1）',
    'hud.cancelHint': '作废取消录屏（Alt+3）',
    'hud.toastCheckpoint': '✓ 已标记回溯点 #{num} ({time})',
    'hud.toastRollback': '⤺ 已回溯至 {target} · 继续录制',
    'hud.beginning': '开头 00:00',

    // Countdown Overlay
    'countdown.title': '即将开始录屏',
    'countdown.fullscreen': '🖥 全屏捕获',
    'countdown.region': '⛶ 选区 {w}×{h}',
    'countdown.micOk': '🎤 麦克风已就绪',
    'countdown.sysOk': '🔊 系统声已开启',
    'countdown.noCursor': '🚫 不录鼠标',
    'countdown.cancel': '取消录屏',
    'countdown.cancelHint': '取消录屏（ESC）',

    // Selector
    'selector.hint': '拖拽框选录制区域 · Esc 取消',
  },
} as const

export type I18nKey = keyof typeof translations.en

export function getLanguage(): Lang {
  try {
    const val = localStorage.getItem(STORAGE_KEY)
    if (val === 'zh' || val === 'en') return val
  } catch {
    // ignore
  }
  return 'en' // English is default
}

export function setLanguage(lang: Lang): void {
  try {
    localStorage.setItem(STORAGE_KEY, lang)
  } catch {
    // ignore
  }
  emit('i18n://changed', { lang }).catch(() => {})
}

export function formatText(key: I18nKey, lang: Lang, params?: Record<string, string | number>): string {
  const dict = translations[lang] || translations.en
  let msg: string = dict[key] ?? translations.en[key] ?? key
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      msg = msg.replaceAll(`{${k}}`, String(v))
    }
  }
  return msg
}

export function useI18n() {
  const [lang, setLangState] = useState<Lang>(getLanguage)

  useEffect(() => {
    const onStorage = (e: StorageEvent) => {
      if (e.key === STORAGE_KEY && (e.newValue === 'en' || e.newValue === 'zh')) {
        setLangState(e.newValue as Lang)
      }
    }
    window.addEventListener('storage', onStorage)

    let unlisten: (() => void) | null = null
    listen<{ lang: Lang }>('i18n://changed', e => {
      if (e.payload.lang === 'en' || e.payload.lang === 'zh') {
        setLangState(e.payload.lang)
      }
    }).then(u => {
      unlisten = u
    }).catch(() => {})

    return () => {
      window.removeEventListener('storage', onStorage)
      if (unlisten) unlisten()
    }
  }, [])

  const setLang = (next: Lang) => {
    setLangState(next)
    setLanguage(next)
  }

  const t = (key: I18nKey, params?: Record<string, string | number>) => formatText(key, lang, params)

  return { lang, setLang, t }
}
