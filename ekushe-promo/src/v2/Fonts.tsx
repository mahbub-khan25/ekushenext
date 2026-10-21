import React, { useEffect, useState } from 'react';
import { continueRender, delayRender, staticFile } from 'remotion';

const faces = [
  { family: 'Hind Siliguri', file: 'fonts/HindSiliguri-Regular.ttf', weight: '400' },
  { family: 'Hind Siliguri', file: 'fonts/HindSiliguri-SemiBold.ttf', weight: '600' },
  { family: 'Hind Siliguri', file: 'fonts/HindSiliguri-Bold.ttf', weight: '700' },
  { family: 'Kalpurush', file: 'fonts/Kalpurush.ttf', weight: '400' },
  { family: 'Inter Tight', file: 'fonts/InterTight.woff2', weight: '300 900' },
];

/** Loads the bundled fonts before the first frame is captured. */
export const Fonts: React.FC = () => {
  const [handle] = useState(() => delayRender('fonts'));
  useEffect(() => {
    Promise.all(
      faces.map((f) => {
        const face = new FontFace(f.family, `url(${staticFile(f.file)})`, { weight: f.weight });
        return face.load().then((loaded) => document.fonts.add(loaded)).catch(() => undefined);
      }),
    ).then(() => continueRender(handle));
  }, [handle]);
  return null;
};
