import { writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const __dirname = dirname(fileURLToPath(import.meta.url));
const outDir = join(__dirname, "..", "public", "sprites", "default");

const sheets = {
  idle: {
    frames: 4,
    colors: [
      [255, 182, 193],
      [255, 160, 180],
      [255, 140, 170],
      [255, 120, 160],
    ],
  },
  walk: {
    frames: 6,
    colors: [
      [135, 206, 250],
      [120, 198, 248],
      [100, 190, 245],
      [80, 182, 242],
      [60, 174, 239],
      [40, 166, 236],
    ],
  },
  sleep: {
    frames: 2,
    colors: [
      [186, 170, 255],
      [160, 145, 240],
    ],
  },
  talk: {
    frames: 4,
    colors: [
      [255, 218, 121],
      [255, 200, 100],
      [255, 180, 80],
      [255, 160, 60],
    ],
  },
};

function crc32(buffer) {
  let crc = 0xffffffff;
  for (let i = 0; i < buffer.length; i++) {
    crc ^= buffer[i];
    for (let j = 0; j < 8; j++) {
      crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1;
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const typeBuf = Buffer.from(type, "ascii");
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])));
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

function framePixels([r, g, b], frameIndex) {
  const pixels = Buffer.alloc(64 * 64 * 4, 0);
  const bob = frameIndex % 2 === 0 ? 0 : 2;
  for (let y = 0; y < 64; y++) {
    for (let x = 0; x < 64; x++) {
      const dx = x - 32;
      const dy = y - (38 + bob);
      const inBody = dx * dx + dy * dy < 20 * 20;
      const eyeL = (x - 24) * (x - 24) + (y - (28 + bob)) * (y - (28 + bob)) < 16;
      const eyeR = (x - 40) * (x - 40) + (y - (28 + bob)) * (y - (28 + bob)) < 16;
      const idx = (y * 64 + x) * 4;
      if (eyeL || eyeR) {
        pixels[idx] = 30;
        pixels[idx + 1] = 30;
        pixels[idx + 2] = 40;
        pixels[idx + 3] = 255;
      } else if (inBody) {
        pixels[idx] = r;
        pixels[idx + 1] = g;
        pixels[idx + 2] = b;
        pixels[idx + 3] = 255;
      }
    }
  }
  return pixels;
}

function createSheet(name, { frames, colors }) {
  const width = 64 * frames;
  const height = 64;
  const scanlines = Buffer.alloc(height * (1 + width * 4));
  let offset = 0;

  for (let y = 0; y < height; y++) {
    scanlines[offset++] = 0;
    for (let frame = 0; frame < frames; frame++) {
      const frameBuf = framePixels(colors[frame], frame);
      frameBuf.copy(scanlines, offset, y * 64 * 4, (y + 1) * 64 * 4);
      offset += 64 * 4;
    }
  }

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;

  const png = Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", ihdr),
    chunk("IDAT", zlib.deflateSync(scanlines)),
    chunk("IEND", Buffer.alloc(0)),
  ]);

  writeFileSync(join(outDir, `${name}.png`), png);
  console.log(`Created ${name}.png (${width}x${height})`);
}

mkdirSync(outDir, { recursive: true });
for (const [name, config] of Object.entries(sheets)) {
  createSheet(name, config);
}
