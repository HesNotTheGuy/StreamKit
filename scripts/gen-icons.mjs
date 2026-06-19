// Generates placeholder StreamKit icons using Canvas API via node
// Run: node scripts/gen-icons.mjs
import { createCanvas } from 'canvas';
import { writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const iconDir = join(__dirname, '..', 'src-tauri', 'icons');
mkdirSync(iconDir, { recursive: true });

function drawIcon(size) {
  const c = createCanvas(size, size);
  const ctx = c.getContext('2d');
  const r = size / 2;

  // Background circle
  const grad = ctx.createRadialGradient(r, r * 0.9, r * 0.1, r, r, r);
  grad.addColorStop(0, '#7c6af7');
  grad.addColorStop(1, '#4a3fd4');
  ctx.fillStyle = grad;
  ctx.beginPath();
  ctx.arc(r, r, r, 0, Math.PI * 2);
  ctx.fill();

  // "SK" text
  ctx.fillStyle = '#ffffff';
  ctx.font = `bold ${size * 0.38}px Arial`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText('SK', r, r);

  return c.toBuffer('image/png');
}

const sizes = [32, 128, 256];
for (const s of sizes) {
  const buf = drawIcon(s);
  const name = s === 256 ? '128x128@2x' : `${s}x${s}`;
  writeFileSync(join(iconDir, `${name}.png`), buf);
  console.log(`Created ${name}.png`);
}

// icon.png (256px — used by tray)
writeFileSync(join(iconDir, 'icon.png'), drawIcon(256));
console.log('Created icon.png');

console.log('Done! (icon.ico must be created separately or with tauri icon command)');
