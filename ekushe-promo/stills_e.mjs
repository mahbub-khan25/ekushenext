import { bundle } from '@remotion/bundler';
import { renderStill, selectComposition } from '@remotion/renderer';
import path from 'node:path';
const frames = process.argv.slice(3).map(Number);
const ids = process.argv[2].split(',');
const serveUrl = await bundle({ entryPoint: path.resolve('src/index.ts') });
for (const id of ids) {
  const composition = await selectComposition({ serveUrl, id });
  for (const frame of frames) {
    const output = `out/stills/${id}-${String(frame).padStart(4, '0')}.jpg`;
    await renderStill({ composition, serveUrl, output, frame, imageFormat: 'jpeg', jpegQuality: 80, scale: 0.4 });
    console.log(output);
  }
}
