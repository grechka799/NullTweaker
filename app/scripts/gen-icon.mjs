import zlib from "node:zlib";
import fs from "node:fs";

const S = 1024;
const SCALE = S / 64;

const px = new Uint8Array(S * S * 4);

function put(x, y, r, g, b, a = 255) {
  if (x < 0 || y < 0 || x >= S || y >= S) return;
  const i = (y * S + x) * 4;
  const na = a / 255;
  const oa = px[i + 3] / 255;
  const outA = na + oa * (1 - na);
  if (outA === 0) return;
  px[i] = Math.round((r * na + px[i] * oa * (1 - na)) / outA);
  px[i + 1] = Math.round((g * na + px[i + 1] * oa * (1 - na)) / outA);
  px[i + 2] = Math.round((b * na + px[i + 2] * oa * (1 - na)) / outA);
  px[i + 3] = Math.round(outA * 255);
}

const R = 14 * SCALE;
for (let y = 0; y < S; y++) {
  for (let x = 0; x < S; x++) {
    const dx = Math.max(R - x, 0, x - (S - 1 - R));
    const dy = Math.max(R - y, 0, y - (S - 1 - R));
    const d = Math.hypot(dx, dy);
    if (d <= R - 0.5) put(x, y, 0x11, 0x11, 0x11);
  }
}

const T = 6 * SCALE;
function bar(x0, y0, x1, y1) {
  for (let y = Math.floor(y0 - T); y <= Math.ceil(y1 + T); y++) {
    for (let x = Math.floor(x0 - T); x <= Math.ceil(x1 + T); x++) {
      if (x >= x0 && x <= x1 && y >= y0 && y <= y1) {
        put(x, y, 255, 255, 255, 255);
        continue;
      }
      const dx = Math.max(x0 - x, 0, x - x1);
      const dy = Math.max(y0 - y, 0, y - y1);
      const d = Math.hypot(dx, dy);
      if (d <= T - 0.5) {
        const cov = Math.min(1, Math.max(0, T - 0.5 - (d - (T - 0.5))));
        put(x, y, 255, 255, 255, Math.round(255 * cov));
      }
    }
  }
}

bar(18 * SCALE, 17 * SCALE, 46 * SCALE, 23 * SCALE);
bar(29 * SCALE, 20 * SCALE, 35 * SCALE, 44 * SCALE);

function crc32(buf) {
  let c, table = [];
  for (let n = 0; n < 256; n++) {
    c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  let crc = 0xffffffff;
  for (const b of buf) crc = table[(crc ^ b) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "latin1"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(S, 0);
ihdr.writeUInt32BE(S, 4);
ihdr[8] = 8;
ihdr[9] = 6;

const raw = Buffer.alloc(S * (S * 4 + 1));
for (let y = 0; y < S; y++) {
  raw[y * (S * 4 + 1)] = 0;
  Buffer.from(px.buffer, y * S * 4, S * 4).copy(raw, y * (S * 4 + 1) + 1);
}

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

fs.writeFileSync(process.argv[2] ?? "app-icon.png", png);
console.log("icon written");
