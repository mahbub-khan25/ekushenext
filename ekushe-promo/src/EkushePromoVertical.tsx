import React from 'react';
import { AbsoluteFill, Audio, Sequence, staticFile } from 'remotion';
import './styles/fonts.css';

import { Scene1HeritageVertical } from './scenes/vertical/Scene1HeritageVertical';
import { Scene2WordsAndPainVertical } from './scenes/vertical/Scene2WordsAndPainVertical';
import { Scene3IntroducingEkusheVertical } from './scenes/vertical/Scene3IntroducingEkusheVertical';
import { Scene4ModernFeaturesVertical } from './scenes/vertical/Scene4ModernFeaturesVertical';
import { Scene5OutroVertical } from './scenes/vertical/Scene5OutroVertical';

export const EkushePromoVertical: React.FC = () => {
  const fontStyles = `
    @font-face {
      font-family: 'Kalpurush';
      src: url('${staticFile('fonts/Kalpurush.ttf')}') format('truetype');
      font-weight: normal;
    }
    @font-face {
      font-family: 'HindSiliguri';
      src: url('${staticFile('fonts/HindSiliguri-Bold.ttf')}') format('truetype');
      font-weight: bold;
    }
    @font-face {
      font-family: 'HindSiliguri';
      src: url('${staticFile('fonts/HindSiliguri-Regular.ttf')}') format('truetype');
      font-weight: normal;
    }
  `;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: '#020617',
        fontFamily: "'Kalpurush', 'HindSiliguri', sans-serif",
      }}
    >
      <style>{fontStyles}</style>

      {/* 45s High-Fidelity Synthesized Soundtrack */}
      <Audio src={staticFile('ekushe-music.wav')} volume={0.88} />

      {/* Scene 1: Heritage & International Mother Language Day (0s - 8s) */}
      <Sequence from={0} durationInFrames={480}>
        <Scene1HeritageVertical />
      </Sequence>

      {/* Scene 2: Beauty of Words & Tribute to Pioneers (8s - 17s) */}
      <Sequence from={480} durationInFrames={540}>
        <Scene2WordsAndPainVertical />
      </Sequence>

      {/* Scene 3: The Rebirth — Ekushe Next & Light Mode Studio Live Typing (17s - 27s) */}
      <Sequence from={1020} durationInFrames={600}>
        <Scene3IntroducingEkusheVertical />
      </Sequence>

      {/* Scene 4: Daily Modern Features & Light Mode Studio Showcase (27s - 37s) */}
      <Sequence from={1620} durationInFrames={600}>
        <Scene4ModernFeaturesVertical />
      </Sequence>

      {/* Scene 5: Outro, Grand Celebration & Call to Action (37s - 45s) */}
      <Sequence from={2220} durationInFrames={480}>
        <Scene5OutroVertical />
      </Sequence>
    </AbsoluteFill>
  );
};
