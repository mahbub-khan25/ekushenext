import React from 'react';
import { AbsoluteFill, Audio, Sequence, staticFile } from 'remotion';
import { Fonts } from './Fonts';
import { SceneDemo, SceneFeatures, SceneMartyrs, SceneOpen, SceneOutro, ScenePride, SceneProblem, SceneReveal, SceneSpeed } from './scenes';
import { C, T } from './theme';

const S: React.FC<{ at: readonly [number, number]; children: React.ReactNode }> = ({ at, children }) => (
  <Sequence from={at[0]} durationInFrames={at[1] - at[0]}>
    {children}
  </Sequence>
);

/** The whole 60-second promo; the same component lays itself out for 16:9 and 9:16. */
export const Promo: React.FC = () => (
  <AbsoluteFill style={{ background: C.night }}>
    <Fonts />
    <S at={T.open}><SceneOpen /></S>
    <S at={T.martyrs}><SceneMartyrs /></S>
    <S at={T.pride}><ScenePride /></S>
    <S at={T.problem}><SceneProblem /></S>
    <S at={T.reveal}><SceneReveal /></S>
    <S at={T.demo}><SceneDemo /></S>
    <S at={T.features}><SceneFeatures /></S>
    <S at={T.speed}><SceneSpeed /></S>
    <S at={T.outro}><SceneOutro /></S>
    <Audio src={staticFile('music/ekushe-score.wav')} />
  </AbsoluteFill>
);
