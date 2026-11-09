import { useEffect, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { open as pickFolderDialog } from '@tauri-apps/plugin-dialog'
import { openPath } from '@tauri-apps/plugin-opener'
import { api } from '../api'
import type { Bootstrap, MonitorInfo, PhaseView, Project, Recording, Region, Settings } from '../api'
import { clamp, fmtDur } from '../fmt'
import { useI18n, type I18nKey } from '../i18n'
import logoSvg from '../assets/logo.svg'
import {
  ChevronDownIcon, ChevronRightIcon, CropIcon,
  FolderIcon, GearIcon, LockIcon, MaxIcon, MicIcon, MinusIcon, MouseIcon,
  PauseIcon, PlayIcon, RefreshIcon, SpeakerIcon, StopIcon, XIcon,
} from './icons'

type Toast = { id: number; text: string; kind: 'ok' | 'err' | 'info' }

export default function Main() {
  const { lang, setLang, t } = useI18n()
  const [boot, setBoot] = useState<Bootstrap | null>(null)
  const [project, setProject] = useState<Project | null>(null)
  const [phase, setPhase] = useState<PhaseView>({ phase: 'idle' })
  const [countdown, setCountdown] = useState(0)
  const [elapsed, setElapsed] = useState(0)
  const [toasts, setToasts] = useState<Toast[]>([])
  const [mics, setMics] = useState<string[]>([])
  const [monitors, setMonitors] = useState<MonitorInfo[]>([])
  const [busy, setBusy] = useState(false)
  const [showMore, setShowMore] = useState(false)
  const [micLevel, setMicLevel] = useState(0)
  const [sysLevel, setSysLevel] = useState(0)
  const toastSeq = useRef(0)
  const skipNextSave = useRef(false)

  const toast = (text: string, kind: Toast['kind'] = 'info') => {
    const id = ++toastSeq.current
    setToasts(t => [...t, { id, text, kind }])
    setTimeout(() => setToasts(t => t.filter(x => x.id !== id)), 4600)
  }

  // ---- Bootstrap & Initialization ----
  useEffect(() => {
    api.bootstrap()
      .then(b => {
        setBoot(b)
        let p = b.project
        if (p.settings.monitor == null && !p.settings.region && b.monitors.length > 0) {
          const prim = b.monitors.find(m => m.isPrimary) ?? b.monitors[0]
          p = { ...p, settings: { ...p.settings, monitor: prim.index } }
        }
        setProject(p)
        setPhase(b.phase)
        setMics(b.mics)
        setMonitors(b.monitors)
      })
      .catch(e => toast(t('boot.initFailed', { err: String(e) }), 'err'))
  }, [])

  // ---- Event Subscriptions ----
  useEffect(() => {
    const un: Array<() => void> = []
    const add = <T,>(ev: string, fn: (p: T) => void) => {
      listen<T>(ev, e => fn(e.payload)).then(u => un.push(u))
    }
    add<PhaseView>('rec://phase', p => {
      setPhase(p)
      setCountdown(0)
      if (p.phase !== 'recording') setElapsed(0)
    })
    add<{ left: number }>('rec://countdown', p => setCountdown(p.left))
    add<{ message: string }>('rec://error', p => toast(p.message, 'err'))
    add<{ mic: number; sys: number }>('audio://levels', p => {
      setMicLevel(p.mic)
      setSysLevel(p.sys)
    })
    add<Recording>('rec://saved', r => toast(t('toast.saved', { file: r.fileName }), 'ok'))
    add<unknown>('rec://cancelled', () => toast(t('toast.cancelled'), 'info'))
    add<Project>('project://changed', p => {
      skipNextSave.current = true
      setProject(p)
    })
    add<Region>('region://updated', r =>
      setProject(pr => (pr ? { ...pr, settings: { ...pr.settings, region: r } } : pr)),
    )
    return () => un.forEach(f => f())
  }, [t])

  // ---- Auto-Save Project Settings ----
  const firstSave = useRef(true)
  useEffect(() => {
    if (!project) return
    if (firstSave.current) {
      firstSave.current = false
      return
    }
    if (skipNextSave.current) {
      skipNextSave.current = false
      return
    }
    const timer = setTimeout(() => {
      api.saveProject(project).catch(e => toast(t('toast.saveFailed', { err: String(e) }), 'err'))
    }, 500)
    return () => clearTimeout(timer)
  }, [project, t])

  // ---- Recording Timer ----
  useEffect(() => {
    if (phase.phase !== 'recording') return
    const base = phase.recordedMs ?? 0
    const resumedAt = phase.resumedAt ?? Date.now()
    const iv = setInterval(() => setElapsed((base + (Date.now() - resumedAt)) / 1000), 200)
    return () => clearInterval(iv)
  }, [phase.phase, phase.recordedMs, phase.resumedAt])

  // ---- Auto-Fit Window Height to Compact Content ----
  useEffect(() => {
    const win = getCurrentWindow()
    let last = -1
    const update = () => {
      const titlebar = document.querySelector('.titlebar') as HTMLElement | null
      const banner = document.querySelector('.banner') as HTMLElement | null
      const deck = document.querySelector('.deck') as HTMLElement | null
      let h = 230
      if (titlebar && deck) {
        h = titlebar.offsetHeight + (banner ? banner.offsetHeight : 0) + deck.offsetHeight
      } else {
        const appEl = document.querySelector('.app') as HTMLElement | null
        h = appEl ? appEl.scrollHeight : 230
      }
      h = Math.max(200, Math.ceil(h))
      if (Math.abs(h - last) <= 1) return
      last = h
      void win.setSize(new LogicalSize(window.innerWidth, h))
    }
    update()
    window.addEventListener('resize', update)
    const iv = setInterval(update, 200)
    return () => {
      window.removeEventListener('resize', update)
      clearInterval(iv)
    }
  }, [boot, showMore, phase.phase])

  // ---- Derived Values ----
  const ffmpegOk = boot?.ffmpeg.found ?? false
  const recording = phase.phase === 'recording'
  const paused = phase.phase === 'paused'
  const counting = phase.phase === 'countdown'
  const s = project?.settings
  const monitor = s?.monitor != null ? monitors.find(m => m.index === s.monitor) ?? null : null
  const effRect = s?.region ?? (monitor ? { x: monitor.x, y: monitor.y, w: monitor.w, h: monitor.h } : null)
  const scopeReady = !!effRect
  const canStart = ffmpegOk && scopeReady && !recording && !counting && !paused && !busy

  const estimateMB = (() => {
    if (!effRect || !s) return null
    const crf = clamp(s.crf, 14, 32)
    const bpp = 0.09 * (23 / crf)
    return Math.max(1, Math.round((effRect.w * effRect.h * s.fps * bpp) / 8 * 60 / (1024 * 1024)))
  })()

  // ---- User Actions ----
  const patchSettings = (fields: Partial<Settings>) =>
    setProject(p => (p ? { ...p, settings: { ...p.settings, ...fields } } : p))

  const start = () => api.recordStart().catch(e => toast(String(e), 'err'))
  const stop = async () => {
    setBusy(true)
    try {
      const res = await api.recordStop()
      toast(t('toast.saved', { file: res.fileName }), 'ok')
    } catch (e) {
      toast(String(e), 'err')
    } finally {
      setBusy(false)
    }
  }
  const cancel = () => api.recordCancel().catch(e => toast(String(e), 'err'))
  const pauseResume = () => api.recordPauseResume().catch(e => toast(String(e), 'err'))

  const chooseOutput = async () => {
    const dir = await pickFolderDialog({ directory: true, title: t('settings.chooseDir') })
    if (typeof dir === 'string' && dir) {
      setProject(p => (p ? { ...p, outputDir: dir } : p))
      setBoot(b => (b ? { ...b, outputDir: dir } : b))
    }
  }

  const openAny = (p: string | null | undefined) => {
    if (!p || !p.trim()) {
      toast(t('toast.pathInvalid'), 'err')
      return
    }
    openPath(p).catch(e => toast(t('toast.openFailed', { err: String(e) }), 'err'))
  }

  const recheck = async () => {
    const ff = await api.checkFfmpeg(project?.settings.ffmpegPath ?? null)
    setBoot(b => (b ? { ...b, ffmpeg: ff } : b))
    toast(ff.found ? t('toast.ffmpegReady', { ver: ff.version ?? '' }) : t('toast.ffmpegNotFound'), ff.found ? 'ok' : 'err')
  }

  const refreshMics = async () => {
    const m = await api.listMics(project?.settings.ffmpegPath ?? null)
    setMics(m)
    toast(m.length ? t('audio.micFound', { count: m.length }) : t('audio.noMicFound'), 'info')
  }

  const startRegion = () => api.startRegionSelect(s?.monitor ?? null).catch(e => toast(String(e), 'err'))
  const clearRegion = () => patchSettings({ region: null })
  const pickMonitor = (idx: number) => patchSettings({ monitor: idx, region: null })

  if (!project || !boot || !s) {
    return <div className="boot-loading">{t('boot.loading')}</div>
  }

  return (
    <div className={`app ${recording ? 'recording-glow' : ''}`}>
      {/* Sleek Custom Titlebar */}
      <header className="titlebar">
        <div className="brand" data-tauri-drag-region>
          <img src={logoSvg} alt="Retake" className="brand-logo-img" data-tauri-drag-region />
          <span className="brand-name" data-tauri-drag-region>Retake</span>
          <span className="brand-badge" data-tauri-drag-region>{t('brand.badge')}</span>
        </div>

        {/* Dynamic Recording Status Badge */}
        <div className="titlebar-status" data-tauri-drag-region>
          {recording ? (
            <div className="status-pill recording">
              <span className="rec-beacon" />
              <span>{t('status.recording', { time: fmtDur(elapsed) })}</span>
            </div>
          ) : paused ? (
            <div className="status-pill paused">
              <span>{t('status.paused', { time: fmtDur(elapsed) })}</span>
            </div>
          ) : counting ? (
            <div className="status-pill counting">
              <span>{t('status.counting', { sec: countdown })}</span>
            </div>
          ) : (
            <div className="status-pill ready">
              <span className="ready-dot" />
              <span>{t('status.ready')}</span>
              {effRect && <span className="res-tag">{effRect.w}×{effRect.h}</span>}
            </div>
          )}
        </div>

        {/* Titlebar Actions */}
        <div className="titlebar-actions">
          <button
            className="lang-pill"
            title={t('titlebar.switchLang')}
            onClick={() => setLang(lang === 'en' ? 'zh' : 'en')}
          >
            {lang === 'en' ? 'EN' : '中'}
          </button>
          <button
            className="icon-btn"
            title={t('titlebar.openOutput')}
            onClick={() => openAny(boot.outputDir)}
          >
            <FolderIcon size={15} />
          </button>
          <span className="win-sep" />
          <button className="win-btn" title={t('titlebar.minimize')} onClick={() => getCurrentWindow().minimize()}>
            <MinusIcon size={14} />
          </button>
          <button className="win-btn" title={t('titlebar.maximize')} onClick={() => getCurrentWindow().toggleMaximize()}>
            <MaxIcon size={13} />
          </button>
          <button className="win-btn close" title={t('titlebar.close')} onClick={() => getCurrentWindow().close()}>
            <XIcon size={14} />
          </button>
        </div>
      </header>

      {/* FFmpeg Missing Banner */}
      {!ffmpegOk && (
        <div className="banner">
          <div className="banner-text">{t('banner.noFfmpeg')}</div>
          <button className="btn small banner-btn" onClick={recheck}>{t('banner.recheck')}</button>
        </div>
      )}

      {/* Main Studio Deck */}
      <main className="deck">
        <div className="deck-grid">
          {/* Section 1: Capture Scope */}
          <section className="deck-col scope-col">
            <div className="col-head">
              <span className="col-title">{t('scope.title')}</span>
              {s.region && <span className="col-tag">{t('scope.customRegion')}</span>}
            </div>

            <div className="col-body">
              {/* Display / Region Selection Dropdown */}
              <select
                className="flat-select"
                value={s.region ? '__region__' : s.monitor ?? ''}
                onChange={e => {
                  const v = e.target.value
                  if (v === '__region__') startRegion()
                  else if (v === '') patchSettings({ monitor: null })
                  else pickMonitor(parseInt(v))
                }}
              >
                <option value="" disabled>{t('scope.selectMonitor')}</option>
                {monitors.map(m => (
                  <option key={m.index} value={m.index}>
                    {m.name} ({m.w}×{m.h}){m.isPrimary ? t('scope.primary') : ''}
                  </option>
                ))}
                {s.region && <option value="__region__">{t('scope.regionLabel', { w: s.region.w, h: s.region.h })}</option>}
              </select>

              {/* Resolution & Crop Actions */}
              <div className="scope-meta-row">
                <span className="res-plain-text">
                  <LockIcon size={11} className="res-plain-icon" />
                  <span>{effRect ? `${effRect.w} × ${effRect.h}` : '—'}</span>
                </span>

                <div className="scope-btn-group">
                  <button
                    className="ghost-btn"
                    onClick={startRegion}
                    disabled={counting || recording}
                    title={t('scope.cropHint')}
                  >
                    <CropIcon size={12} />
                    <span>{t('scope.crop')}</span>
                  </button>
                  {s.region && (
                    <button
                      className="ghost-btn danger"
                      onClick={clearRegion}
                      disabled={recording}
                    >
                      {t('scope.clear')}
                    </button>
                  )}
                </div>
              </div>
            </div>
          </section>

          {/* Section 2: Audio Capture */}
          <section className="deck-col audio-col">
            <div className="col-head">
              <span className="col-title">{t('audio.title')}</span>
              <button
                className="ghost-link"
                onClick={refreshMics}
                disabled={recording}
                title={t('audio.refreshHint')}
              >
                <RefreshIcon size={11} />
                <span>{t('audio.refresh')}</span>
              </button>
            </div>

            <div className="col-body audio-rows">
              {/* Microphone Row */}
              <div className={`audio-row ${s.mic ? 'active' : ''}`}>
                <MicIcon size={15} className={`audio-row-ico ${s.mic ? 'on' : ''}`} />
                <select
                  className="flat-select audio-select"
                  value={s.mic ?? ''}
                  disabled={!s.mic}
                  onChange={e => patchSettings({ mic: e.target.value || null })}
                >
                  <option value="">{mics.length ? t('audio.selectMic') : t('audio.noMic')}</option>
                  {mics.map(m => <option key={m} value={m}>{m}</option>)}
                </select>
                <AudioVisualizer level={micLevel} active={!!s.mic} t={t} />
                <Switch
                  checked={!!s.mic}
                  disabled={recording || counting}
                  onChange={v => patchSettings({ mic: v ? (mics[0] ?? '') : null })}
                  title={s.mic ? t('audio.toggleMicOff') : t('audio.toggleMicOn')}
                />
              </div>

              {/* System Audio Row */}
              <div className={`audio-row ${s.systemAudio ? 'active' : ''}`}>
                <SpeakerIcon size={15} className={`audio-row-ico ${s.systemAudio ? 'on' : ''}`} />
                <div className="audio-row-name">
                  <span>{t('audio.system')}</span>
                  <span className="audio-wasapi-tip">WASAPI</span>
                </div>
                <AudioVisualizer level={sysLevel} active={!!s.systemAudio} t={t} />
                <Switch
                  checked={!!s.systemAudio}
                  disabled={recording || counting}
                  onChange={v => patchSettings({ systemAudio: v })}
                  title={s.systemAudio ? t('audio.toggleSysOff') : t('audio.toggleSysOn')}
                />
              </div>
            </div>
          </section>

          {/* Section 3: Recording Control Center */}
          <section className="deck-col hero-col">
            <div className="hero-center">
              {counting ? (
                <div className="hero-btn counting" title={t('hero.countingTitle')}>
                  <span className="hero-count-val">{countdown}</span>
                </div>
              ) : recording ? (
                <button
                  className="hero-btn recording"
                  onClick={stop}
                  disabled={busy}
                  title={t('hero.stopHint')}
                >
                  <StopIcon size={26} />
                </button>
              ) : paused ? (
                <button
                  className="hero-btn paused"
                  onClick={pauseResume}
                  disabled={busy}
                  title={t('hero.resumeHint')}
                >
                  <PlayIcon size={26} />
                </button>
              ) : (
                <button
                  className="hero-btn idle"
                  disabled={!canStart}
                  onClick={start}
                  title={
                    !ffmpegOk
                      ? t('hero.errNoFfmpeg')
                      : !scopeReady
                        ? t('hero.errNoScope')
                        : t('hero.startHint')
                  }
                >
                  <span className="hero-core" />
                </button>
              )}

              {/* Core Action Label */}
              <div className="hero-label">
                {recording ? (
                  <span className="live-dur">{fmtDur(elapsed)}</span>
                ) : paused ? (
                  <span className="paused-dur">{t('hero.pausedLabel', { time: fmtDur(elapsed) })}</span>
                ) : counting ? (
                  <span className="counting-label">{t('hero.counting')}</span>
                ) : (
                  <span className="idle-label">{t('hero.start')}</span>
                )}
              </div>

              {/* Sub Actions during recording or paused */}
              {(recording || paused || counting) && (
                <div className="hero-sub-bar">
                  {recording && (
                    <button
                      className="ghost-pill"
                      onClick={pauseResume}
                      title={t('hero.pauseHint')}
                    >
                      <PauseIcon size={11} />
                      <span>{t('hero.pause')}</span>
                    </button>
                  )}
                  {paused && (
                    <button
                      className="ghost-pill resume"
                      onClick={pauseResume}
                      title={t('hero.resumeHint')}
                    >
                      <PlayIcon size={11} />
                      <span>{t('hero.resume')}</span>
                    </button>
                  )}
                  <button
                    className="ghost-pill cancel"
                    onClick={cancel}
                    title={t('hero.cancelHint')}
                  >
                    <XIcon size={11} />
                    <span>{t('hero.cancel')}</span>
                  </button>
                </div>
              )}
            </div>
          </section>
        </div>

        {/* Collapsible Settings Drawer */}
        <div className="drawer-section">
          <button
            className="drawer-toggle-bar"
            onClick={() => setShowMore(v => !v)}
          >
            <div className="toggle-bar-left">
              <GearIcon size={12} className="gear-ico" />
              <span>{t('settings.title')}</span>
            </div>
            <div className="toggle-bar-right">
              <span className="toggle-hint">{s.fps} FPS · CRF {s.crf} · {s.countdownSec}s {t('settings.countdown')}</span>
              {showMore ? <ChevronDownIcon size={12} /> : <ChevronRightIcon size={12} />}
            </div>
          </button>

          {showMore && (
            <div className="drawer-content">
              <div className="settings-flow">
                {/* Frame Rate */}
                <div className="setting-row">
                  <label className="setting-label">{t('settings.fps')}</label>
                  <div className="segmented-track">
                    {[24, 30, 60].map(fps => (
                      <button
                        key={fps}
                        className={`segmented-item ${s.fps === fps ? 'active' : ''}`}
                        onClick={() => patchSettings({ fps })}
                      >
                        {fps} FPS
                      </button>
                    ))}
                  </div>
                </div>

                {/* Countdown */}
                <div className="setting-row">
                  <label className="setting-label">{t('settings.countdown')}</label>
                  <div className="segmented-track">
                    {[0, 3, 5, 10].map(sec => (
                      <button
                        key={sec}
                        className={`segmented-item ${s.countdownSec === sec ? 'active' : ''}`}
                        onClick={() => patchSettings({ countdownSec: sec })}
                      >
                        {sec === 0 ? t('settings.noCountdown') : t('settings.sec', { sec })}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Encoding Quality */}
                <div className="setting-row">
                  <label className="setting-label">{t('settings.quality')}</label>
                  <div className="segmented-track">
                    {[
                      { val: 18, label: t('settings.crfUltra') },
                      { val: 23, label: t('settings.crfHigh') },
                      { val: 28, label: t('settings.crfCompact') },
                    ].map(item => (
                      <button
                        key={item.val}
                        className={`segmented-item ${s.crf === item.val ? 'active' : ''}`}
                        onClick={() => patchSettings({ crf: item.val })}
                      >
                        {item.label}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Toggle Switches */}
                <div className="setting-row toggle-options-row">
                  <div className="inline-switch-item">
                    <span className="switch-text">{t('settings.captureCursor')}</span>
                    <Switch
                      checked={s.captureCursor}
                      disabled={recording || counting}
                      onChange={v => patchSettings({ captureCursor: v })}
                    />
                  </div>

                  <div className="inline-switch-item">
                    <span className="switch-text">{t('settings.excludeSelf')}</span>
                    <Switch
                      checked={s.excludeSelf}
                      disabled={recording || counting}
                      onChange={v => patchSettings({ excludeSelf: v })}
                    />
                  </div>
                </div>

                {/* Output Directory */}
                <div className="setting-row path-row">
                  <label className="setting-label">{t('settings.outDir')}</label>
                  <div className="flat-input-group">
                    <input className="flat-input" readOnly value={boot.outputDir} />
                    <button className="ghost-icon-btn" title={t('settings.chooseDir')} onClick={chooseOutput}>
                      <FolderIcon size={13} />
                    </button>
                    <button className="ghost-icon-btn" title={t('settings.openDir')} onClick={() => openAny(boot.outputDir)}>
                      <ChevronRightIcon size={13} />
                    </button>
                  </div>
                </div>

                {/* FFmpeg Path */}
                <div className="setting-row path-row">
                  <div className="setting-label-with-status">
                    <label className="setting-label">{t('settings.ffmpegPath')}</label>
                    <span className={`status-tag ${ffmpegOk ? 'ok' : 'err'}`}>
                      {ffmpegOk ? t('settings.ready', { ver: boot.ffmpeg.version ?? 'OK' }) : t('settings.notFound')}
                    </span>
                  </div>
                  <div className="flat-input-group">
                    <input
                      className="flat-input"
                      value={s.ffmpegPath ?? ''}
                      placeholder={t('settings.ffmpegPlaceholder')}
                      onChange={e => patchSettings({ ffmpegPath: e.target.value || null })}
                    />
                    <button className="ghost-btn" onClick={recheck} title={t('settings.recheckHint')}>
                      <RefreshIcon size={12} />
                      <span>{t('settings.recheck')}</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer Info & Shortcuts */}
        <footer className="deck-foot">
          <div className="foot-estimate">
            <span className="dot-hint" />
            <span>{estimateMB != null ? t('foot.estimate', { mb: estimateMB }) : t('foot.estimateEmpty')}</span>
          </div>
          <div className="foot-shortcuts">
            <span className="shortcut-tag">{t('sc.record')}</span>
            <span className="shortcut-tag">{t('sc.pause')}</span>
            <span className="shortcut-tag">{t('sc.checkpoint')}</span>
            <span className="shortcut-tag">{t('sc.rollback')}</span>
            <span className="shortcut-tag">{t('sc.cancel')}</span>
          </div>
        </footer>
      </main>

      {/* Floating Toast Notification */}
      <div className="toasts">
        {toasts.map(toastItem => (
          <div key={toastItem.id} className={`toast-card ${toastItem.kind}`}>
            <span className="toast-dot" />
            <span className="toast-msg">{toastItem.text}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function AudioVisualizer({
  level,
  active,
  t,
}: {
  level: number
  active: boolean
  t: (key: I18nKey, params?: Record<string, string | number>) => string
}) {
  if (!active) return null

  // 4-band audio spectrum visualizer (WASAPI peak level)
  const norm = Math.max(0, Math.min(1, level))
  const isHearing = norm > 0.015
  const b1 = isHearing ? Math.min(100, Math.max(15, Math.round(norm * 110))) : 15
  const b2 = isHearing ? Math.min(100, Math.max(20, Math.round(norm * 140))) : 15
  const b3 = isHearing ? Math.min(100, Math.max(25, Math.round(norm * 165))) : 15
  const b4 = isHearing ? Math.min(100, Math.max(18, Math.round(norm * 120))) : 15

  const getClass = (h: number) => {
    if (!isHearing) return ''
    if (h >= 85) return 'active peak'
    if (h >= 60) return 'active hot'
    return 'active'
  }

  return (
    <div
      className="real-visualizer"
      title={isHearing ? t('audio.hearingHint', { pct: Math.round(norm * 100) }) : t('audio.silentHint')}
    >
      <span className="viz-bars">
        <i className={`viz-bar ${getClass(b1)}`} style={{ height: `${b1}%` }} />
        <i className={`viz-bar ${getClass(b2)}`} style={{ height: `${b2}%` }} />
        <i className={`viz-bar ${getClass(b3)}`} style={{ height: `${b3}%` }} />
        <i className={`viz-bar ${getClass(b4)}`} style={{ height: `${b4}%` }} />
      </span>
      <span className={`viz-tag ${isHearing ? 'hearing' : ''}`}>
        {isHearing ? t('audio.hearing') : t('audio.silent')}
      </span>
    </div>
  )
}

function Switch({
  checked,
  disabled,
  onChange,
  title,
}: {
  checked: boolean
  disabled?: boolean
  onChange: (v: boolean) => void
  title?: string
}) {
  return (
    <button
      className={`toggle-sw ${checked ? 'on' : ''}`}
      role="switch"
      aria-checked={checked}
      title={title}
      disabled={disabled}
      onClick={() => onChange(!checked)}
    >
      <span className="toggle-thumb" />
    </button>
  )
}
