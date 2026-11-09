export const fmtDur = (s: number) => {
  const t = Math.max(0, Math.round(s || 0))
  return `${String(Math.floor(t / 60)).padStart(2, '0')}:${String(t % 60).padStart(2, '0')}`
}

export const fmtSize = (b: number) =>
  b > 1024 * 1024 ? `${(b / 1024 / 1024).toFixed(1)} MB` : `${Math.max(1, Math.round(b / 1024))} KB`

export const clamp = (v: number, a: number, b: number) => Math.min(b, Math.max(a, v))
