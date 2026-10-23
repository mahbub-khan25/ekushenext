import React from 'react';
import { AbsoluteFill, Audio, interpolate, Sequence, staticFile, useCurrentFrame } from 'remotion';
import { clamp } from '../v2/parts';
import { T3, Teaser3 } from './Teaser3';
import { T4, Teaser4 } from './Teaser4';
import { Teaser5, TEASER5_FRAMES } from './Teaser5';

/** The last teaser: teasers 3, 4 and 5 back to back (each without its own ending), then one ending.
 * About 59 s at 60 fps. The music (music/final_teaser.py) is the three scores joined at the same marks. */
export const J4 = T3.end;            // teaser 4 starts
export const J5 = T3.end + T4.end;   // teaser 5 starts
export const FINAL_TEASER_FRAMES = J5 + TEASER5_FRAMES;
const XF = 12; // a short flash at each join

const Join: React.FC<{ at: number }> = ({ at }) => {
  const f = useCurrentFrame();
  const o = interpolate(f, [at - XF, at, at + XF], [0, 1, 0], clamp);
  if (o <= 0) return null;
  return <AbsoluteFill style={{ background: 'radial-gradient(circle at 50% 45%, rgba(255,255,255,0.9), rgba(47,209,142,0.5) 40%, rgba(3,18,12,1) 80%)', opacity: o * 0.85, pointerEvents: 'none' }} />;
};

export const FinalTeaser: React.FC = () => (
  <AbsoluteFill>
    <Sequence from={0} durationInFrames={T3.end}><Teaser3 audio={false} /></Sequence>
    <Sequence from={J4} durationInFrames={T4.end}><Teaser4 audio={false} /></Sequence>
    <Sequence from={J5} durationInFrames={TEASER5_FRAMES}><Teaser5 audio={false} /></Sequence>
    <Join at={J4} />
    <Join at={J5} />
    <Audio src={staticFile('music/final_teaser.wav')} />
  </AbsoluteFill>
);
