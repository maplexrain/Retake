import { useEffect, useState } from 'react'
import { api, ScopePreviewInfo } from '../api'
import { useI18n } from '../i18n'

export default function ScopePreview() {
  const { t } = useI18n()
  const [info, setInfo] = useState<ScopePreviewInfo | null>(null)

  useEffect(() => {
    document.body.classList.add('transparent')
    document.body.classList.add('fill-window')

    api.getScopePreviewInfo().then(res => {
      if (res) setInfo(res)
    })

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        api.closeScopePreview()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => {
      document.body.classList.remove('transparent')
      document.body.classList.remove('fill-window')
      window.removeEventListener('keydown', onKey)
    }
  }, [])

  if (!info) return null

  // Calculate coordinates relative to this monitor in logical CSS pixels
  const left = (info.rect.x - info.monitor.x) / info.monitor.scale
  const top = (info.rect.y - info.monitor.y) / info.monitor.scale
  const width = info.rect.w / info.monitor.scale
  const height = info.rect.h / info.monitor.scale

  const handleClose = () => {
    api.closeScopePreview()
  }

  return (
    <div className="scope-preview-overlay" onClick={handleClose}>
      <div
        className="scope-preview-box"
        style={{ left, top, width, height }}
        onClick={e => e.stopPropagation()}
      >
        {/* Animated glowing border corners */}
        <div className="scope-preview-corners">
          <div className="corner-tl" />
          <div className="corner-tr" />
          <div className="corner-bl" />
          <div className="corner-br" />
        </div>

        {/* Top Floating Badge with info & close button */}
        <div className="scope-preview-pill">
          <div className="pill-dot" />
          <span className="pill-title">{t('scope.previewTitle')}</span>
          <span className="pill-res">{info.rect.w} × {info.rect.h}</span>
          <span className="pill-coord">(X: {info.rect.x}, Y: {info.rect.y})</span>
          <button className="pill-close-btn" onClick={handleClose} title="Close (Esc)">
            ✕ {t('scope.closePreview')}
          </button>
        </div>
      </div>
    </div>
  )
}
