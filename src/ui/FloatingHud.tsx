import { useEffect, useState, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { api, type PhaseView } from '../api'
import { fmtDur } from '../fmt'
import { useI18n } from '../i18n'

interface ToastState {
  message: string
  type: 'cp' | 'rb' | 'info'
}

export default function FloatingHud() {
  const { t } = useI18n()
  const [phase, setPhase] = useState<'recording' | 'paused' | 'idle'>('recording')
  const [recordedMs, setRecordedMs] = useState(0)
  const [resumedAt, setResumedAt] = useState<number | null>(Date.now())
  const [elapsedSec, setElapsedSec] = useState(0)
  const [markers, setMarkers] = useState<number[]>([])
  const [toast, setToast] = useState<ToastState | null>(null)
  const toastTimerRef = useRef<number | null>(null)

  const showToast = (message: string, type: 'cp' | 'rb' | 'info') => {
    if (toastTimerRef.current) {
      window.clearTimeout(toastTimerRef.current)
    }
    setToast({ message, type })
    toastTimerRef.current = window.setTimeout(() => {
      setToast(null)
      toastTimerRef.current = null
    }, 2000)
  }

  // Initialize and listen for events
  useEffect(() => {
    document.body.classList.add('fill-window')
    document.body.classList.add('transparent')

    // Retrieve initial HUD state
    api.getHudState()
      .then(p => {
        if (p.phase === 'recording' || p.phase === 'paused') {
          setPhase(p.phase)
          setRecordedMs(p.recordedMs || 0)
          setResumedAt(p.resumedAt || null)
          if (p.markers) setMarkers(p.markers)
        }
      })
      .catch(() => {})

    const unlistens: Array<() => void> = []

    listen<PhaseView>('rec://phase', e => {
      const p = e.payload
      if (p.phase === 'recording' || p.phase === 'paused' || p.phase === 'idle') {
        setPhase(p.phase)
        setRecordedMs(p.recordedMs || 0)
        setResumedAt(p.resumedAt || null)
        if (p.markers !== undefined) {
          setMarkers(p.markers)
        }
      }
    }).then(u => unlistens.push(u))

    listen<{ ms: number; markers: number[]; count: number }>('rec://checkpoint', e => {
      const sec = e.payload.ms / 1000
      setMarkers(e.payload.markers)
      showToast(t('hud.toastCheckpoint', { num: e.payload.count, time: fmtDur(sec) }), 'cp')
    }).then(u => unlistens.push(u))

    listen<{ ms: number; markers: number[]; count: number; message: string }>('rec://rollback', e => {
      const sec = e.payload.ms / 1000
      setMarkers(e.payload.markers)
      const targetText = e.payload.ms === 0 ? t('hud.beginning') : fmtDur(sec)
      showToast(t('hud.toastRollback', { target: targetText }), 'rb')
    }).then(u => unlistens.push(u))

    // Keyboard listener (when HUD is focused)
    const onKey = (e: KeyboardEvent) => {
      if (e.altKey && (e.key === 'f' || e.key === 'F')) {
        e.preventDefault()
        api.recordCheckpoint().catch(() => {})
      } else if (e.altKey && (e.key === 'g' || e.key === 'G')) {
        e.preventDefault()
        api.recordRollback().catch(() => {})
      } else if (e.altKey && e.key === '2') {
        e.preventDefault()
        api.recordPauseResume().catch(() => {})
      } else if (e.altKey && e.key === '1') {
        e.preventDefault()
        api.recordStop().catch(() => {})
      } else if (e.altKey && e.key === '3') {
        e.preventDefault()
        api.recordCancel().catch(() => {})
      }
    }
    window.addEventListener('keydown', onKey)

    return () => {
      unlistens.forEach(fn => fn())
      window.removeEventListener('keydown', onKey)
      if (toastTimerRef.current) window.clearTimeout(toastTimerRef.current)
      document.body.classList.remove('fill-window')
      document.body.classList.remove('transparent')
    }
  }, [t])

  // Millisecond precision time update
  useEffect(() => {
    const updateTime = () => {
      if (phase === 'recording' && resumedAt) {
        const cur = (recordedMs + (Date.now() - resumedAt)) / 1000
        setElapsedSec(Math.max(0, cur))
      } else {
        setElapsedSec(Math.max(0, recordedMs / 1000))
      }
    }
    updateTime()
    const interval = setInterval(updateTime, 40)
    return () => clearInterval(interval)
  }, [phase, recordedMs, resumedAt])

  // Progress bar scale: min 30s, auto-stretches with current duration
  const allPoints = [elapsedSec, ...(markers.length ? markers : [0])]
  const maxSec = Math.max(30, Math.max(...allPoints) * 1.15)
  const progressPct = Math.min(100, Math.max(0, (elapsedSec / maxSec) * 100))

  const handleCheckpoint = () => {
    api.recordCheckpoint().catch(() => {})
  }

  const handleRollback = () => {
    api.recordRollback().catch(() => {})
  }

  const handleTogglePause = () => {
    api.recordPauseResume().catch(() => {})
  }

  const handleStop = () => {
    api.recordStop().catch(() => {})
  }

  const handleCancel = () => {
    api.recordCancel().catch(() => {})
  }

  const isPaused = phase === 'paused'

  return (
    <div className="hud-shell">
      <div className={`hud-container ${isPaused ? 'paused' : 'recording'}`} data-tauri-drag-region>
        {/* Top row: status pill + checkpoint progress bar + checkpoint count */}
        <div className="hud-top-row" data-tauri-drag-region>
          <div className={`hud-badge ${isPaused ? 'paused' : 'rec'}`} title={isPaused ? t('hud.pauseHint') : t('hud.recHint')}>
            <span className="hud-dot" />
            <span className="hud-badge-text">{isPaused ? t('hud.paused') : t('hud.recording')}</span>
          </div>

          <div className="hud-progress-wrap" data-tauri-drag-region>
            <div className="hud-progress-track">
              {/* Progress bar filled */}
              <div
                className={`hud-progress-fill ${isPaused ? 'paused' : ''}`}
                style={{ width: `${progressPct}%` }}
              >
                <div className="hud-progress-glow" />
              </div>

              {/* Checkpoint pins */}
              {markers.map((sec, idx) => {
                const pct = Math.min(100, Math.max(0, (sec / maxSec) * 100))
                return (
                  <div
                    key={idx}
                    className="hud-marker-pin"
                    style={{ left: `${pct}%` }}
                    title={t('hud.checkpointTitle', { num: idx + 1, time: fmtDur(sec) })}
                  >
                    <div className="hud-marker-stem" />
                    <div className="hud-marker-pip">⚑</div>
                  </div>
                )
              })}
            </div>
          </div>

          <div className="hud-marker-count" title={t('hud.checkpointCount')}>
            <span>⚑</span>
            <span>{markers.length}</span>
          </div>
        </div>

        {/* Bottom row: recorded time + quick actions / toast notification */}
        <div className="hud-bottom-row" data-tauri-drag-region>
          <div className="hud-time-display" title={t('hud.recTime')}>
            <span className="hud-time-num">{fmtDur(elapsedSec)}</span>
          </div>

          {toast ? (
            <div className={`hud-toast-banner ${toast.type}`}>
              <span>{toast.message}</span>
            </div>
          ) : (
            <div className="hud-actions">
              <button
                className="hud-btn hud-btn-checkpoint"
                onClick={handleCheckpoint}
                title={t('hud.checkpointHint')}
              >
                <span className="hud-btn-icon">⚑</span>
                <span>{t('hud.checkpointBtn')}</span>
                <span className="hud-btn-kbd">Alt+F</span>
              </button>

              <button
                className="hud-btn hud-btn-rollback"
                onClick={handleRollback}
                title={t('hud.rollbackHint')}
              >
                <span className="hud-btn-icon">⤺</span>
                <span>{t('hud.rollbackBtn')}</span>
                <span className="hud-btn-kbd">Alt+G</span>
              </button>

              <button
                className="hud-btn hud-btn-icononly"
                onClick={handleTogglePause}
                title={isPaused ? t('hud.resumeHint') : t('hud.pauseActionHint')}
              >
                <span>{isPaused ? '▶' : '⏸'}</span>
              </button>

              <button
                className="hud-btn hud-btn-stop"
                onClick={handleStop}
                title={t('hud.stopHint')}
              >
                <span>{t('hud.stopBtn')}</span>
              </button>

              <button
                className="hud-btn hud-btn-cancel"
                onClick={handleCancel}
                title={t('hud.cancelHint')}
              >
                <span>✕</span>
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
