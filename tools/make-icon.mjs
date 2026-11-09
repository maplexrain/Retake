// 生成 Tauri 所需图标（纯 Node 实现，无第三方依赖）：
//   src-tauri/icons/icon.png (512) / 128x128.png / 32x32.png / icon.ico (256, PNG-in-ICO)
import { deflateSync } from 'node:zlib'
import { writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const outDir = join(root, 'src-tauri', 'icons')
mkdirSync(outDir, { recursive: true })

// ---------- PNG ----------
const CRC_TABLE = (() => {
  const t = new Int32Array(256)
  for (let n = 0; n < 256; n++) {
    let c = n
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
    t[n] = c
  }
  return t
})()
function crc32(buf) {
  let c = -1
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8)
  return (c ^ -1) >>> 0
}
function chunk(type, data) {
  const len = Buffer.alloc(4)
  len.writeUInt32BE(data.length)
  const t = Buffer.from(type, 'ascii')
  const crc = Buffer.alloc(4)
  crc.writeUInt32BE(crc32(Buffer.concat([t, data])))
  return Buffer.concat([len, t, data, crc])
}

const clamp01 = (v) => Math.max(0, Math.min(1, v))
// 线性插值两个 [r,g,b,a]
function over(dst, src, a) {
  const out = [0, 0, 0, 0]
  for (let i = 0; i < 3; i++) out[i] = src[i] * a + dst[i] * (1 - a)
  out[3] = src[3] * a + dst[3] * (1 - a)
  return out
}

function render(size) {
  const px = new Float32Array(size * size * 4)
  const cx = size / 2, cy = size / 2
  const rr = size * 0.2      // 圆角半径
  const hw = size / 2, hh = size / 2
  const rRing = size * 0.345, halfRing = size * 0.032
  const rDot = size * 0.265
  const BG = [0.13, 0.15, 0.21], RING = [0.96, 0.97, 0.99], DOT = [0.9, 0.28, 0.3]
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const fx = x + 0.5, fy = y + 0.5
      // 圆角矩形 SDF
      const qx = Math.abs(fx - cx) - (hw - rr)
      const qy = Math.abs(fy - cy) - (hh - rr)
      const dRect = Math.hypot(Math.max(qx, 0), Math.max(qy, 0)) + Math.min(Math.max(qx, qy), 0) - rr
      let a = clamp01(0.5 - dRect)
      let col = [BG[0], BG[1], BG[2], 1]
      const d0 = Math.hypot(fx - cx, fy - cy)
      const aRing = clamp01(0.5 + halfRing - Math.abs(d0 - rRing)) * a
      col = over(col, RING, aRing)
      const aDot = clamp01(0.5 - (d0 - rDot)) * a
      col = over(col, DOT, aDot)
      const i = (y * size + x) * 4
      px[i] = col[0]; px[i + 1] = col[1]; px[i + 2] = col[2]; px[i + 3] = col[3]
    }
  }
  // 编码
  const raw = Buffer.alloc(size * (size * 4 + 1))
  for (let y = 0; y < size; y++) {
    const row = y * (size * 4 + 1)
    raw[row] = 0
    for (let x = 0; x < size; x++) {
      const i = (y * size + x) * 4
      raw[row + 1 + x * 4] = Math.round(clamp01(px[i]) * 255)
      raw[row + 2 + x * 4] = Math.round(clamp01(px[i + 1]) * 255)
      raw[row + 3 + x * 4] = Math.round(clamp01(px[i + 2]) * 255)
      raw[row + 4 + x * 4] = Math.round(clamp01(px[i + 3]) * 255)
    }
  }
  const ihdr = Buffer.alloc(13)
  ihdr.writeUInt32BE(size, 0)
  ihdr.writeUInt32BE(size, 4)
  ihdr[8] = 8; ihdr[9] = 6
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', deflateSync(raw)),
    chunk('IEND', Buffer.alloc(0)),
  ])
}

// ---------- ICO（内嵌 PNG） ----------
function ico(pngBytes, size) {
  const head = Buffer.alloc(6)
  head.writeUInt16LE(0, 0); head.writeUInt16LE(1, 2); head.writeUInt16LE(1, 4)
  const e = Buffer.alloc(16)
  e[0] = size >= 256 ? 0 : size
  e[1] = size >= 256 ? 0 : size
  e[2] = 0; e[3] = 0
  e.writeUInt16LE(1, 4)   // planes
  e.writeUInt16LE(32, 6)  // bpp
  e.writeUInt32LE(pngBytes.length, 8)
  e.writeUInt32LE(22, 12) // 数据偏移
  return Buffer.concat([head, e, pngBytes])
}

writeFileSync(join(outDir, 'icon.png'), render(512))
writeFileSync(join(outDir, '128x128.png'), render(128))
writeFileSync(join(outDir, '32x32.png'), render(32))
writeFileSync(join(outDir, 'icon.ico'), ico(render(256), 256))
console.log('icons written to', outDir)
