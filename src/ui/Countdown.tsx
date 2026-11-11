import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { api } from '../api'
import type { Bootstrap } from '../api'
import { useI18n } from '../i18n'
import { PlayIcon, XIcon } from './icons'

export default function Countdown() {
  const { t } = useI18n()
  const [seconds, setSeconds] = useState(3)
  const [animKey, setAnimKey] = useState(0)
  const [boot, setBoot] = useState<Bootstrap | null>(null)

  useEffect(() => {
    document.documentElement.classList.add('transparent')
    document.body.classList.add('fill-window')
    document.body.classList.add('transparent')
    const rootEl = document.getElementById('root')
    if (rootEl) rootEl.classList.add('transparent')

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
      if (e.key === 'Escape' || (e.altKey && e.key === '3')) {
        api.recordCancel().catch(() => {})
      } else if (e.key === 'Enter' || e.key === ' ' || (e.altKey && e.key === '1')) {
        api.recordSkipCountdown().catch(() => {})
      }
    }
    window.addEventListener('keydown', onKey)

    return () => {
      un.forEach(f => f())
      window.removeEventListener('keydown', onKey)
      document.documentElement.classList.remove('transparent')
      document.body.classList.remove('fill-window')
      document.body.classList.remove('transparent')
      if (rootEl) rootEl.classList.remove('transparent')
    }
  }, [])

  const cancel = () => {
    api.recordCancel().catch(() => {})
  }

  const startNow = () => {
    api.recordSkipCountdown().catch(() => {})
  }

  const s = boot?.project.settings
  const hasRegion = !!s?.region
  const hasMic = !!s?.mic
  const hasSys = !!s?.systemAudio

  return (
    <div className="cd-overlay">
      <div className="cd-center">
        {/* Core countdown number - magnified, borderless, no background, no circular frame */}
        <div className="cd-num-display" key={animKey}>
          <span className="cd-num">{seconds}</span>
        </div>

        {/* Title and dots */}
        <div className="cd-title">
          <span>{t('countdown.title')}</span>
          <span className="cd-dots"><i /><i /><i /></span>
        </div>

        {/* Status badges */}
        <div className="cd-badges">
          {boot?.phase?.targetFileName && (
            <span className="cd-badge target" title={boot.phase.targetFileName}>
              🎯 {boot.phase.targetFileName}
            </span>
          )}
          <span className="cd-badge">
            {hasRegion ? t('countdown.region', { w: s.region!.w, h: s.region!.h }) : t('countdown.fullscreen')}
          </span>
          {hasMic && <span className="cd-badge ok">{t('countdown.micOk')}</span>}
          {hasSys && <span className="cd-badge ok">{t('countdown.sysOk')}</span>}
          {s?.captureCursor === false && <span className="cd-badge">{t('countdown.noCursor')}</span>}
        </div>

        {/* Actions row: Start Now & Cancel */}
        <div className="cd-actions">
          <button className="cd-start-btn" onClick={startNow} title={t('countdown.startNowHint')}>
            <PlayIcon size={14} />
            <span>{t('countdown.startNow')}</span>
            <span className="cd-kbd">Enter</span>
          </button>
          <button className="cd-cancel-btn" onClick={cancel} title={t('countdown.cancelHint')}>
            <XIcon size={13} />
            <span>{t('countdown.cancel')}</span>
            <span className="cd-kbd">ESC</span>
          </button>
        </div>
      </div>
    </div>
  )
}
