import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { api } from '../api'
import type { Bootstrap } from '../api'
import { useI18n } from '../i18n'

export default function Countdown() {
  const { t } = useI18n()
  const [seconds, setSeconds] = useState(3)
  const [animKey, setAnimKey] = useState(0)
  const [boot, setBoot] = useState<Bootstrap | null>(null)

  useEffect(() => {
    document.body.classList.add('fill-window')
    document.body.classList.add('transparent')

    api.bootstrap()
      .then(b => {
        setBoot(b)
        setSeconds(b.project.settings.countdownSec || 3)
      })
      .catch(() => {})

    const un: Array<() => void> = []
    listen<{ left: number }>('rec://countdown', e => {
      setSeconds(e.payload.left)
      setAnimKey(k => k + 1)
    }).then(u => un.push(u))

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' || (e.altKey && e.key === '1') || (e.altKey && e.key === '3')) {
        api.recordCancel().catch(() => {})
      }
    }
    window.addEventListener('keydown', onKey)

    return () => {
      un.forEach(f => f())
      window.removeEventListener('keydown', onKey)
      document.body.classList.remove('fill-window')
      document.body.classList.remove('transparent')
    }
  }, [])

  const cancel = () => {
    api.recordCancel().catch(() => {})
  }

  const s = boot?.project.settings
  const hasRegion = !!s?.region
  const hasMic = !!s?.mic
  const hasSys = !!s?.systemAudio

  return (
    <div className="cd-overlay">
      <div className="cd-center">
        {/* Countdown ring & core number */}
        <div className="cd-ring-wrap" key={animKey}>
          <svg className="cd-svg" viewBox="0 0 200 200">
            <defs>
              <linearGradient id="cdGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stopColor="#ff4d4f" />
                <stop offset="50%" stopColor="#ff7875" />
                <stop offset="100%" stopColor="#ffa39e" />
              </linearGradient>
            </defs>
            <circle className="cd-circle-bg" cx="100" cy="100" r="88" />
            <circle className="cd-circle-fg" cx="100" cy="100" r="88" />
          </svg>
          <div className="cd-num">{seconds}</div>
        </div>

        {/* Title and dots */}
        <div className="cd-title">
          <span>{t('countdown.title')}</span>
          <span className="cd-dots"><i /><i /><i /></span>
        </div>

        {/* Status badges */}
        <div className="cd-badges">
          <span className="cd-badge">
            {hasRegion ? t('countdown.region', { w: s.region!.w, h: s.region!.h }) : t('countdown.fullscreen')}
          </span>
          {hasMic && <span className="cd-badge ok">{t('countdown.micOk')}</span>}
          {hasSys && <span className="cd-badge ok">{t('countdown.sysOk')}</span>}
          {s?.captureCursor === false && <span className="cd-badge">{t('countdown.noCursor')}</span>}
        </div>

        {/* Cancel button */}
        <button className="cd-cancel-btn" onClick={cancel} title={t('countdown.cancelHint')}>
          <span className="cd-cancel-x">✕</span>
          <span>{t('countdown.cancel')}</span>
          <span className="cd-kbd">ESC</span>
        </button>
      </div>
    </div>
  )
}
