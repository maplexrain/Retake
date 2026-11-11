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
    'scope.modeDisplay': 'Display',
    'scope.modeWindows': 'App Windows',
    'scope.modeRegion': 'Crop',
    'scope.selectWindows': 'Select Target Window',
    'scope.selectedWindowsCount': '{count} windows selected',
    'scope.targetWindowSelected': '{name}',
    'scope.lockedWindow': 'Locked',
    'scope.noWindowsSelected': 'No target window selected',
    'scope.searchWindows': 'Search window or process name...',
    'scope.selectAll': 'Select All',
    'scope.clearAll': 'Clear',
    'scope.refreshWindows': 'Refresh List',
    'scope.noWindowsFound': 'No visible application windows found',
    'scope.windowBounds': '{w} × {h} at ({x}, {y})',
    'scope.unionBounds': 'Window Bounds: {w} × {h}',
    'scope.windowsDrawerTitle': 'Dedicated Window Capture (Immune to displacement & overlap)',
    'scope.windowsFoundCount': '{count} windows detected',
    'scope.previewScope': 'Preview Scope',
    'scope.previewingScope': 'Previewing (Click to close)',
    'scope.previewTitle': 'Recording Bounds Preview',
    'scope.closePreview': 'Close Preview',
    'scope.previewHint': 'Display real-time highlight boundary frame on screen',
    'scope.landscape': 'Landscape',
    'scope.portrait': 'Portrait',

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
    'settings.recordAppWindows': 'Allow App Windows in Capture',
    'settings.recordAppWindowsHint': 'When enabled, allows Retake UI (main deck, HUD, countdown) to be captured. Default: Off (invisible in recordings).',
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
    'countdown.startNow': 'Start Now',
    'countdown.startNowHint': 'Start recording immediately without waiting (Enter)',
    'countdown.cancel': 'Cancel Recording',
    'countdown.cancelHint': 'Cancel recording (ESC)',

    // Selector
    'selector.hint': 'Drag to crop recording area · Esc to cancel',

    // Manifest Mode & Drawer
    'manifest.modeTitle': 'Manifest Recording Mode',
    'manifest.modeEnabled': 'Manifest Mode Active',
    'manifest.modeDisabled': 'Normal Recording Mode',
    'manifest.modeHint': 'Record pre-planned clips and iterate with auto-overwrite',
    'manifest.selectDir': 'Choose Manifest Folder…',
    'manifest.changeDir': 'Change Directory',
    'manifest.dirRequired': 'Please select a manifest directory to enable manifest mode',
    'manifest.dirInvalid': 'Directory invalid: Must be either empty or contain retake_manifest.json',
    'manifest.dirEmptyCreated': 'Empty directory selected. Initialized new retake_manifest.json.',
    'manifest.dirLoaded': 'Loaded manifest ({count} clips)',
    'manifest.activeTarget': 'Target Clip',
    'manifest.noActiveTarget': 'No target selected (timestamp naming)',
    'manifest.clearTarget': 'Clear Target',
    'manifest.willOverwrite': '⚠️ Re-recording without switching will overwrite this clip',
    'manifest.createDefault': 'Create Manifest',
    'manifest.openJson': 'Open JSON',
    'manifest.reload': 'Reload',
    'manifest.addItem': 'Add Clip',
    'manifest.empty': 'No clips in manifest. Click "Add Clip" to get started.',
    'manifest.clipName': 'Clip Name / File',
    'manifest.clipNamePlaceholder': 'Enter clip name...',
    'manifest.clipTitle': 'Clip title / description...',
    'manifest.clipFile': 'target_name.mp4',
    'manifest.delete': 'Delete clip',
    'manifest.recordThis': 'Record this clip',
    'manifest.active': 'Active Recording Target',
    'manifest.statusRecorded': 'Recorded',
    'manifest.statusPending': 'Pending',
    'manifest.play': 'Open video file',
    'manifest.targetBadge': 'TARGET: {name}',
    'manifest.switchOnNotice': '✓ Switched to Manifest Mode',
    'manifest.switchOffNotice': 'Switched to Normal Recording Mode',
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
    'scope.modeDisplay': '显示器',
    'scope.modeWindows': '应用窗口',
    'scope.modeRegion': '框选区域',
    'scope.selectWindows': '选择目标应用窗口',
    'scope.selectedWindowsCount': '已选 {count} 个窗口',
    'scope.targetWindowSelected': '{name}',
    'scope.lockedWindow': '已锁定',
    'scope.noWindowsSelected': '未选择目标应用窗口',
    'scope.searchWindows': '搜索窗口标题或进程名...',
    'scope.selectAll': '全选',
    'scope.clearAll': '清空',
    'scope.refreshWindows': '刷新列表',
    'scope.noWindowsFound': '未找到可见应用窗口',
    'scope.windowBounds': '{w} × {h} (坐标: {x}, {y})',
    'scope.unionBounds': '窗口录制分辨率: {w} × {h}',
    'scope.windowsDrawerTitle': '专属应用窗口录制（即使位移或被遮挡也能完整录制该应用）',
    'scope.windowsFoundCount': '检测到 {count} 个可用应用窗口',
    'scope.previewScope': '预览范围',
    'scope.previewingScope': '预览中 (点击关闭)',
    'scope.previewTitle': '录制范围预览',
    'scope.closePreview': '关闭预览',
    'scope.previewHint': '在屏幕上高亮显示当前录制区域的外框',
    'scope.landscape': '横屏',
    'scope.portrait': '竖屏',

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
    'settings.recordAppWindows': '应用窗口可被录制',
    'settings.recordAppWindowsHint': '开启后允许所有 Retake UI（主面板、HUD、倒计时）被录屏与截图；默认关闭（界面在录屏中自动隐形）。',
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
    'countdown.startNow': '立即开始',
    'countdown.startNowHint': '跳过倒计时立即开始录屏（Enter）',
    'countdown.cancel': '取消录屏',
    'countdown.cancelHint': '取消录屏（ESC）',

    // Selector
    'selector.hint': '拖拽框选录制区域 · Esc 取消',

    // Manifest Mode & Drawer
    'manifest.modeTitle': '按清单录制模式',
    'manifest.modeEnabled': '已开启清单模式',
    'manifest.modeDisabled': '常规录制模式',
    'manifest.modeHint': '按规划的片段逐项录制，未切换自动覆盖重录',
    'manifest.selectDir': '选择清单保存目录…',
    'manifest.changeDir': '更改目录',
    'manifest.dirRequired': '开启清单模式前必须选择一个目录',
    'manifest.dirInvalid': '目录不符合要求：必须为空目录或已包含 retake_manifest.json 文件',
    'manifest.dirEmptyCreated': '已选择空目录，已自动新建 retake_manifest.json。',
    'manifest.dirLoaded': '已加载目录清单 ({count} 个片段)',
    'manifest.activeTarget': '当前目标片段',
    'manifest.noActiveTarget': '未选择目标（默认时间戳命名）',
    'manifest.clearTarget': '取消目标',
    'manifest.willOverwrite': '⚠️ 若不切换直接重新录制，将覆盖同名片段',
    'manifest.createDefault': '新建清单',
    'manifest.openJson': '打开 JSON',
    'manifest.reload': '刷新',
    'manifest.addItem': '添加片段',
    'manifest.empty': '清单中暂无片段，点击“添加片段”开始规划。',
    'manifest.clipName': '片段名称',
    'manifest.clipNamePlaceholder': '输入片段名称…',
    'manifest.clipTitle': '片段名称/说明…',
    'manifest.clipFile': '目标文件名.mp4',
    'manifest.delete': '删除此项',
    'manifest.recordThis': '设为录制目标',
    'manifest.active': '当前录制目标',
    'manifest.statusRecorded': '已录制',
    'manifest.statusPending': '待录制',
    'manifest.play': '打开视频',
    'manifest.targetBadge': '目标: {name}',
    'manifest.switchOnNotice': '✓ 已开启按清单录制模式',
    'manifest.switchOffNotice': '已切换至常规录制模式',
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
