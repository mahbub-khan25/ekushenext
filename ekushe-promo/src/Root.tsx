import React from 'react';
import { Composition } from 'remotion';
import { Promo } from './v2/Promo';
import { TOTAL } from './v2/theme';
import { Teaser1, TEASER1_FRAMES } from './teasers/Teaser1';

/** 60 s, 60 fps, 2K: 2560×1440 (16:9, YouTube) and 1440×2560 (9:16, Reels / Shorts / TikTok). */
export const Root: React.FC = () => (
  <>
    <Composition id="EkusheNext-16x9" component={Promo} durationInFrames={TOTAL} fps={60} width={2560} height={1440} />
    <Composition id="EkusheNext-9x16" component={Promo} durationInFrames={TOTAL} fps={60} width={1440} height={2560} />
    {/* Teasers: 15 s each, same 2K sizes */}
    <Composition id="Teaser1-9x16" component={Teaser1} durationInFrames={TEASER1_FRAMES} fps={60} width={1440} height={2560} />
    <Composition id="Teaser1-16x9" component={Teaser1} durationInFrames={TEASER1_FRAMES} fps={60} width={2560} height={1440} />
  </>
);
