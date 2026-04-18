// Generates PCM Recon icons — dark background with a stylised "R" mark
import { writeFileSync, mkdirSync } from 'fs';
import zlib from 'zlib';

mkdirSync('src-tauri/icons', { recursive: true });

function crc32(buf) {
  let crc = 0xFFFFFFFF;
  for (const b of buf) {
    crc ^= b;
    for (let i = 0; i < 8; i++) crc = (crc >>> 1) ^ (crc & 1 ? 0xEDB88320 : 0);
  }
  return (~crc) >>> 0;
}

function chunk(type, data) {
  const t = Buffer.from(type, 'ascii');
  const len = Buffer.allocUnsafe(4); len.writeUInt32BE(data.length);
  const crcBuf = Buffer.allocUnsafe(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([t, data])));
  return Buffer.concat([len, t, data, crcBuf]);
}

function makePng(size) {
  const sig = Buffer.from([137,80,78,71,13,10,26,10]);
  const ihdr = Buffer.allocUnsafe(13);
  ihdr.writeUInt32BE(size, 0); ihdr.writeUInt32BE(size, 4);
  ihdr[8]=8; ihdr[9]=2; ihdr[10]=0; ihdr[11]=0; ihdr[12]=0;

  // Draw icon: dark navy bg + accent circle + white "R" letter
  const raw = Buffer.alloc(size * (1 + size * 3));
  const cx = size / 2, cy = size / 2, r = size * 0.42;
  const ir = size * 0.30; // inner radius for letter area

  for (let y = 0; y < size; y++) {
    const row = y * (1 + size * 3);
    raw[row] = 0;
    for (let x = 0; x < size; x++) {
      const dx = x - cx, dy = y - cy;
      const dist = Math.sqrt(dx*dx + dy*dy);
      const nx = dx / r, ny = dy / r; // normalised

      let rr = 0x08, gg = 0x0d, bb = 0x1a; // bg dark navy

      if (dist <= r) {
        // Accent circle fill — gradient-ish solid
        rr = 0x0f; gg = 0x18; bb = 0x29;
      }

      // Circle border ring
      if (dist > r * 0.88 && dist <= r) {
        const t = (dist - r*0.88) / (r * 0.12);
        rr = Math.round(0x4d * t + rr * (1-t));
        gg = Math.round(0x88 * t + gg * (1-t));
        bb = Math.round(0xf5 * t + bb * (1-t));
      }

      // Draw "R" glyph inside circle using simple pixel rules
      if (dist < r * 0.82) {
        const gx = (x / size - 0.5) * 2;  // -1..1
        const gy = (y / size - 0.5) * 2;

        // Letter "R" — left vertical stroke
        const stemLeft = -0.28, stemRight = -0.10;
        const topY = -0.55, botY = 0.55;
        const bowlRight = 0.28, bowlMidY = -0.02;

        const inStem = gx >= stemLeft && gx <= stemRight && gy >= topY && gy <= botY;

        // Bowl: top half right side — half-circle-ish
        const bowlCX = stemRight, bowlCY = (topY + bowlMidY) / 2;
        const bowlR = (bowlMidY - topY) / 2;
        const inBowlArc = Math.sqrt((gx-bowlCX)**2 + (gy-bowlCY)**2) < bowlR + 0.13
                       && Math.sqrt((gx-bowlCX)**2 + (gy-bowlCY)**2) > bowlR - 0.03
                       && gx > stemRight - 0.02 && gy < bowlMidY + 0.05;

        // Bowl fill top connector
        const inBowlFill = gy >= topY && gy <= topY + 0.12 && gx >= stemLeft && gx <= bowlRight + 0.05;
        const inBowlMid  = gy >= bowlMidY - 0.07 && gy <= bowlMidY + 0.07 && gx >= stemLeft && gx <= stemRight + 0.18;

        // Leg: diagonal from bowl-right going down-right
        const legDX = gx - (stemRight + 0.04);
        const legDY = gy - (bowlMidY + 0.04);
        const legAngle = legDY - legDX * 1.1;
        const inLeg = legAngle >= -0.06 && legAngle <= 0.06 && gy > bowlMidY && gy < botY && gx > stemLeft - 0.02;

        if (inStem || inBowlArc || inBowlFill || inBowlMid || inLeg) {
          rr = 0xff; gg = 0xff; bb = 0xff;
        }
      }

      raw[row + 1 + x*3]   = rr;
      raw[row + 1 + x*3+1] = gg;
      raw[row + 1 + x*3+2] = bb;
    }
  }

  const idat = zlib.deflateSync(raw, { level: 9 });
  return Buffer.concat([sig, chunk('IHDR', ihdr), chunk('IDAT', idat), chunk('IEND', Buffer.alloc(0))]);
}

function crc32Num(buf) { return crc32(buf); }

function makeIco(sizes) {
  const pngs = sizes.map(s => makePng(s));
  const header = Buffer.allocUnsafe(6);
  header.writeUInt16LE(0,0); header.writeUInt16LE(1,2); header.writeUInt16LE(pngs.length,4);
  let offset = 6 + pngs.length * 16;
  const dirs = pngs.map((png, i) => {
    const d = Buffer.allocUnsafe(16);
    const s = sizes[i];
    d[0]=s>=256?0:s; d[1]=s>=256?0:s; d[2]=0; d[3]=0;
    d.writeUInt16LE(1,4); d.writeUInt16LE(32,6);
    d.writeUInt32LE(png.length,8); d.writeUInt32LE(offset,12);
    offset += png.length;
    return d;
  });
  return Buffer.concat([header,...dirs,...pngs]);
}

writeFileSync('src-tauri/icons/32x32.png',      makePng(32));
writeFileSync('src-tauri/icons/128x128.png',     makePng(128));
writeFileSync('src-tauri/icons/128x128@2x.png',  makePng(256));
writeFileSync('src-tauri/icons/icon.ico',        makeIco([16,32,48,256]));
writeFileSync('src-tauri/icons/icon.icns',       makePng(512));
console.log('Icons written.');
