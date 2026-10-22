// Render check frames for both formats: node stills.mjs [frames...]
import { bundle } from '@remotion/bundler';
import { renderStill, selectComposition } from '@remotion/renderer';
import path from 'node:path';
const frames = process.argv.slice(2).map(Number);
const serveUrl = await bundle({ entryPoint: path.resolve('src/index.ts') });
for (const id of (process.env.IDS||'Anthem-16x9,Anthem-9x16').split(',')) {
  const composition = await selectComposition({ serveUrl, id });
  for (const frame of frames) {
    const output = `out/stills/${id}-${String(frame).padStart(4, '0')}.jpg`;
    await renderStill({ composition, serveUrl, output, frame, imageFormat: 'jpeg', jpegQuality: 80, scale: 0.5 });
    console.log(output);
  }
}
