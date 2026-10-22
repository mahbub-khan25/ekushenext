import React from 'react';
import { Composition } from 'remotion';
import { Promo } from './v2/Promo';
import { TOTAL } from './v2/theme';
import { Teaser1, TEASER1_FRAMES } from './teasers/Teaser1';
import { Teaser2, TEASER2_FRAMES } from './teasers/Teaser2';
import { Teaser3, TEASER3_FRAMES } from './teasers/Teaser3';
import { Teaser4, TEASER4_FRAMES } from './teasers/Teaser4';
import { Teaser5, TEASER5_FRAMES } from './teasers/Teaser5';

/** 60 s, 60 fps, 2K: 2560×1440 (16:9, YouTube) and 1440×2560 (9:16, Reels / Shorts / TikTok). */
export const Root: React.FC = () => (
  <>
    <Composition id="EkusheNext-16x9" component={Promo} durationInFrames={TOTAL} fps={60} width={2560} height={1440} />
    <Composition id="EkusheNext-9x16" component={Promo} durationInFrames={TOTAL} fps={60} width={1440} height={2560} />
    {/* Teasers: 15 s each, same 2K sizes */}
    <Composition id="Teaser1-9x16" component={Teaser1} durationInFrames={TEASER1_FRAMES} fps={60} width={1440} height={2560} />
    <Composition id="Teaser1-16x9" component={Teaser1} durationInFrames={TEASER1_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="Teaser2-9x16" component={Teaser2} durationInFrames={TEASER2_FRAMES} fps={60} width={1440} height={2560} />
    <Composition id="Teaser2-16x9" component={Teaser2} durationInFrames={TEASER2_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="Teaser3-9x16" component={Teaser3} durationInFrames={TEASER3_FRAMES} fps={60} width={1440} height={2560} />
    <Composition id="Teaser3-16x9" component={Teaser3} durationInFrames={TEASER3_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="Teaser4-9x16" component={Teaser4} durationInFrames={TEASER4_FRAMES} fps={60} width={1440} height={2560} />
    <Composition id="Teaser4-16x9" component={Teaser4} durationInFrames={TEASER4_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="Teaser5-9x16" component={Teaser5} durationInFrames={TEASER5_FRAMES} fps={60} width={1440} height={2560} />
    <Composition id="Teaser5-16x9" component={Teaser5} durationInFrames={TEASER5_FRAMES} fps={60} width={2560} height={1440} />
  </>
);
