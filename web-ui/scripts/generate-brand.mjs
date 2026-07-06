// Rasterizes the brand SVGs into the PNG sizes used by the app and README.
// Run from web-ui/: npm run brand
import sharp from "sharp";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const brand = path.join(root, "assets", "brand");
const publicDir = path.join(root, "web-ui", "public");
await mkdir(publicDir, { recursive: true });

// The masked eye holes rasterize as transparency, so these PNGs work on any background.
const jobs = [
  { src: "logo.svg", out: path.join(publicDir, "favicon-32.png"), size: 32 },
  { src: "logo.svg", out: path.join(publicDir, "apple-touch-icon.png"), size: 180 },
  { src: "logo.svg", out: path.join(brand, "logo-256.png"), size: 256 },
  { src: "logo.svg", out: path.join(brand, "logo-512.png"), size: 512 },
];

for (const { src, out, size } of jobs) {
  // logo.svg has a 32-unit viewBox; scale density so the raster isn't upsampled.
  await sharp(path.join(brand, src), { density: Math.ceil((72 * size) / 32) })
    .resize(size, size)
    .png()
    .toFile(out);
  console.log(`✓ ${path.relative(root, out)} (${size}x${size})`);
}
