import { useEffect, useRef, useState } from 'react'
import type { MouseEvent as RMouseEvent } from 'react'
import { api } from '../api'
import { useI18n } from '../i18n'

type Rect = { x: number; y: number; w: number; h: number }

export default function Selector() {
  const { t } = useI18n()
  const [rect, setRect] = useState<Rect | null>(null)
  const startRef = useRef<{ x: number; y: number } | null>(null)
  const rectRef = useRef<Rect | null>(null)
  rectRef.current = rect

  useEffect(() => {
    document.body.classList.add('transparent')
    document.body.classList.add('fill-window')
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') api.finishRegionSelect(null)
    }
    window.addEventListener('keydown', onKey)
    return () => {
      document.body.classList.remove('transparent')
      document.body.classList.remove('fill-window')
      window.removeEventListener('keydown', onKey)
    }
  }, [])

  const down = (e: RMouseEvent<HTMLDivElement>) => {
    if (e.button !== 0) return
    startRef.current = { x: e.clientX, y: e.clientY }
    setRect({ x: e.clientX, y: e.clientY, w: 0, h: 0 })
  }
  const move = (e: RMouseEvent<HTMLDivElement>) => {
    const s = startRef.current
    if (!s) return
    setRect({
      x: Math.min(s.x, e.clientX),
      y: Math.min(s.y, e.clientY),
      w: Math.abs(e.clientX - s.x),
      h: Math.abs(e.clientY - s.y),
    })
  }
  const up = () => {
    const r = rectRef.current
    startRef.current = null
    if (r && r.w > 12 && r.h > 12) {
      api.finishRegionSelect({ x: r.x, y: r.y, w: r.w, h: r.h })
    } else {
      setRect(null)
    }
  }

  return (
    <div className="sel-root" onMouseDown={down} onMouseMove={move} onMouseUp={up}>
      <div className="sel-hint">{t('selector.hint')}</div>
      {rect && rect.w > 2 && (
        <div className="sel-rect" style={{ left: rect.x, top: rect.y, width: rect.w, height: rect.h }}>
          <span className="sel-size">{Math.round(rect.w)} × {Math.round(rect.h)}</span>
        </div>
      )}
    </div>
  )
}
