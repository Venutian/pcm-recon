// Generates PCM Recon icons with a cycling mark: wheel, mountain, and road.
import { writeFileSync, mkdirSync } from "fs";
import zlib from "zlib";

mkdirSync("src-tauri/icons", { recursive: true });

function crc32(buf) {
  let crc = 0xffffffff;
  for (const b of buf) {
    crc ^= b;
    for (let i = 0; i < 8; i++) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
  }
  return (~crc) >>> 0;
}

function chunk(type, data) {
  const t = Buffer.from(type, "ascii");
  const len = Buffer.allocUnsafe(4);
  len.writeUInt32BE(data.length);
  const crcBuf = Buffer.allocUnsafe(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([t, data])));
  return Buffer.concat([len, t, data, crcBuf]);
}

function clamp(value, min = 0, max = 1) {
  return Math.max(min, Math.min(max, value));
}

function mix(a, b, t) {
  return Math.round(a + (b - a) * clamp(t));
}

function blend(base, next, t) {
  return {
    r: mix(base.r, next.r, t),
    g: mix(base.g, next.g, t),
    b: mix(base.b, next.b, t),
  };
}

function distToSegment(px, py, ax, ay, bx, by) {
  const abx = bx - ax;
  const aby = by - ay;
  const apx = px - ax;
  const apy = py - ay;
  const denom = abx * abx + aby * aby || 1;
  const t = clamp((apx * abx + apy * aby) / denom);
  const qx = ax + abx * t;
  const qy = ay + aby * t;
  return Math.hypot(px - qx, py - qy);
}

function makePng(size) {
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.allocUnsafe(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;

  const raw = Buffer.alloc(size * (1 + size * 3));
  const cx = size / 2;
  const cy = size / 2;
  const badgeR = size * 0.42;

  for (let y = 0; y < size; y++) {
    const row = y * (1 + size * 3);
    raw[row] = 0;
    for (let x = 0; x < size; x++) {
      const dx = x - cx;
      const dy = y - cy;
      const dist = Math.hypot(dx, dy);

      let color = { r: 0x08, g: 0x0d, b: 0x1a };

      if (dist <= badgeR) {
        const top = { r: 0x18, g: 0x24, b: 0x3a };
        const bottom = { r: 0x0d, g: 0x15, b: 0x25 };
        color = blend(top, bottom, y / size);
      }

      if (dist > badgeR * 0.88 && dist <= badgeR) {
        const ringT = (dist - badgeR * 0.88) / (badgeR * 0.12);
        color = blend(color, { r: 0x52, g: 0xb4, b: 0xf4 }, ringT);
      }

      if (dist <= badgeR * 0.84) {
        const horizon = cy + size * 0.03;
        const p1 = { x: size * 0.22, y: horizon + size * 0.05 };
        const p2 = { x: size * 0.41, y: horizon - size * 0.2 };
        const p3 = { x: size * 0.56, y: horizon - size * 0.07 };
        const p4 = { x: size * 0.73, y: horizon + size * 0.03 };
        const ridgeWidth = size * 0.03;
        const ridge = Math.min(
          distToSegment(x, y, p1.x, p1.y, p2.x, p2.y),
          distToSegment(x, y, p2.x, p2.y, p3.x, p3.y),
          distToSegment(x, y, p3.x, p3.y, p4.x, p4.y),
        );
        if (ridge <= ridgeWidth && y <= horizon + size * 0.08) {
          const tint = 1 - ridge / ridgeWidth;
          color = blend(color, { r: 0xe7, g: 0xf2, b: 0xff }, tint * 0.95);
        }

        const roadBottom = cy + size * 0.29;
        const roadTop = cy - size * 0.06;
        const roadLeftBottom = cx - size * 0.14;
        const roadRightBottom = cx + size * 0.14;
        const roadLeftTop = cx - size * 0.03;
        const roadRightTop = cx + size * 0.03;
        if (y >= roadTop && y <= roadBottom) {
          const t = (y - roadTop) / (roadBottom - roadTop);
          const left = roadLeftTop + (roadLeftBottom - roadLeftTop) * t;
          const right = roadRightTop + (roadRightBottom - roadRightTop) * t;
          if (x >= left && x <= right) {
            color = blend(color, { r: 0x1f, g: 0x2e, b: 0x46 }, 0.92);
            const center = (left + right) / 2;
            const stripeWidth = Math.max(1.3, size * 0.012);
            if (Math.abs(x - center) <= stripeWidth && y > roadTop + size * 0.04) {
              const dash = Math.floor((y - roadTop) / Math.max(3, size * 0.07)) % 2 === 0;
              if (dash) color = { r: 0xf0, g: 0xc0, b: 0x30 };
            }
          }
        }

        const wheelR = size * 0.09;
        const wheelCx = cx;
        const wheelCy = cy + size * 0.1;
        const wheelDist = Math.hypot(x - wheelCx, y - wheelCy);
        const rim = Math.abs(wheelDist - wheelR);
        const spoke = Math.min(Math.abs(x - wheelCx), Math.abs(y - wheelCy));
        if (rim <= size * 0.016) {
          color = { r: 0xe8, g: 0xb8, b: 0x00 };
        } else if (wheelDist < wheelR - size * 0.018 && spoke <= size * 0.012) {
          color = { r: 0x94, g: 0xc9, b: 0xff };
        }

        if (wheelDist <= size * 0.018) {
          color = { r: 0xf4, g: 0xf8, b: 0xff };
        }
      }

      raw[row + 1 + x * 3] = color.r;
      raw[row + 1 + x * 3 + 1] = color.g;
      raw[row + 1 + x * 3 + 2] = color.b;
    }
  }

  const idat = zlib.deflateSync(raw, { level: 9 });
  return Buffer.concat([sig, chunk("IHDR", ihdr), chunk("IDAT", idat), chunk("IEND", Buffer.alloc(0))]);
}

function makeIco(sizes) {
  const pngs = sizes.map((s) => makePng(s));
  const header = Buffer.allocUnsafe(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(pngs.length, 4);
  let offset = 6 + pngs.length * 16;
  const dirs = pngs.map((png, i) => {
    const d = Buffer.allocUnsafe(16);
    const s = sizes[i];
    d[0] = s >= 256 ? 0 : s;
    d[1] = s >= 256 ? 0 : s;
    d[2] = 0;
    d[3] = 0;
    d.writeUInt16LE(1, 4);
    d.writeUInt16LE(32, 6);
    d.writeUInt32LE(png.length, 8);
    d.writeUInt32LE(offset, 12);
    offset += png.length;
    return d;
  });
  return Buffer.concat([header, ...dirs, ...pngs]);
}

writeFileSync("src-tauri/icons/32x32.png", makePng(32));
writeFileSync("src-tauri/icons/128x128.png", makePng(128));
writeFileSync("src-tauri/icons/128x128@2x.png", makePng(256));
writeFileSync("src-tauri/icons/icon.ico", makeIco([16, 32, 48, 256]));
writeFileSync("src-tauri/icons/icon.icns", makePng(512));
console.log("Icons written.");
