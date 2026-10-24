import React from 'react';
import { Composition } from 'remotion';
import { Promo } from './v2/Promo';
import { TOTAL } from './v2/theme';
import { Teaser1, TEASER1_FRAMES } from './teasers/Teaser1';
import { Teaser2, TEASER2_FRAMES } from './teasers/Teaser2';
import { Teaser3, TEASER3_FRAMES } from './teasers/Teaser3';
import { Teaser4, TEASER4_FRAMES } from './teasers/Teaser4';
import { Teaser5, TEASER5_FRAMES } from './teasers/Teaser5';
import { Anthem, ANTHEM_FRAMES } from './teasers/Anthem';
import { Reveal, REVEAL_FRAMES } from './teasers/Reveal';
import { FinalTeaser, FINAL_TEASER_FRAMES } from './teasers/FinalTeaser';
import { Everywhere, EVERYWHERE_FRAMES } from './teasers/Everywhere';
import { FullPromo, FULL_PROMO_FRAMES } from './teasers/FullPromo';

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
    {/* Real example: the national anthem typed live in a note app */}
    <Composition id="Anthem-16x9" component={Anthem} durationInFrames={ANTHEM_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="Anthem-9x16" component={Anthem} durationInFrames={ANTHEM_FRAMES} fps={60} width={1440} height={2560} />
    {/* Launch-date reveal: 30 October 2026, then every top feature */}
    <Composition id="Reveal-16x9" component={Reveal} durationInFrames={REVEAL_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="Reveal-9x16" component={Reveal} durationInFrames={REVEAL_FRAMES} fps={60} width={1440} height={2560} />
    {/* The last teaser: teasers 3, 4 and 5 combined, one ending */}
    <Composition id="FinalTeaser-16x9" component={FinalTeaser} durationInFrames={FINAL_TEASER_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="FinalTeaser-9x16" component={FinalTeaser} durationInFrames={FINAL_TEASER_FRAMES} fps={60} width={1440} height={2560} />
    {/* One keyboard, everywhere: Windows, macOS, Linux and Android, one engine */}
    <Composition id="Everywhere-16x9" component={Everywhere} durationInFrames={EVERYWHERE_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="Everywhere-9x16" component={Everywhere} durationInFrames={EVERYWHERE_FRAMES} fps={60} width={1440} height={2560} />
    {/* The full promo: every feature and every device; Windows and Linux on 30 October, the rest later */}
    <Composition id="FullPromo-16x9" component={FullPromo} durationInFrames={FULL_PROMO_FRAMES} fps={60} width={2560} height={1440} />
    <Composition id="FullPromo-9x16" component={FullPromo} durationInFrames={FULL_PROMO_FRAMES} fps={60} width={1440} height={2560} />
  </>
);
