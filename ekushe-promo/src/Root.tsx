import React from 'react';
import { Composition } from 'remotion';
import { EkushePromo } from './EkushePromo';
import { EkushePromoVertical } from './EkushePromoVertical';

export const Root: React.FC = () => {
  return (
    <>
      {/* 16:9 Landscape YouTube / Desktop Promo — 60 FPS Ultra-Smooth High Quality */}
      <Composition
        id="EkushePromo"
        component={EkushePromo}
        durationInFrames={2700} // 45 seconds at 60 fps
        fps={60}
        width={1920}
        height={1080}
      />

      {/* 9:16 Vertical Mobile / YouTube Shorts / Instagram Reels / TikTok Promo — 60 FPS */}
      <Composition
        id="EkushePromoVertical"
        component={EkushePromoVertical}
        durationInFrames={2700} // 45 seconds at 60 fps
        fps={60}
        width={1080}
        height={1920}
      />
    </>
  );
};
