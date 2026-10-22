import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { Burst, END_LEN, EndCard, io, pad3, Streaks, Sub, Sweep } from './common';

/** Teaser 3 of 5: "যেভাবে বলেন, সেভাবে লিখুন" (type the way you speak). 22 s at 60 fps.
 * Roman words morph into Bangla, the headline slams in, then the first real look at the candidate
 * popup (recorded from the live demo on ekushenext.com), a 3D ring of features, and the ending. */
export const T3 = { words: 0, head: 200, popup: 330, ring: 700, tag: 1000, end: 1140 } as const;
export const TEASER3_FRAMES = T3.end + END_LEN;
export const WORD_AT = [10, 70, 130];                  // each Roman word starts typing
export const MORPH_AT = WORD_AT.map((w) => w + 38);    // …and turns into Bangla
export const HERO_AT = T3.popup + 30, HERO_EVERY = 7;  // the recorded keystrokes (41 keys)
const HERO_FRAMES = 42;
const HERO = { wide: { dir: 'hero', w: 3720, h: 1525 }, tall: { dir: 'heroV', w: 1896, h: 1347 } };

const WORDS: [string, string][] = [['amar', 'আমার'], ['bhasha', 'ভাষা'], ['bangla', 'বাংলা']];

/** One word: Roman letters drop in one by one, then the word melts into its Bangla form. */
const MorphWord: React.FC<{ roman: string; bangla: string; at: number; morph: number }> = ({ roman, bangla, at, morph }) => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const m = ease(f, morph, morph + 18);
  const size = (V ? 190 : 170) * u;
  return (
    <div style={{ position: 'relative', height: size * 1.25, minWidth: size * 2.6, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <div style={{ position: 'absolute', display: 'flex', opacity: 1 - m, filter: `blur(${m * 16 * u}px)`, transform: `scale(${1 + m * 0.3})` }}>
        {roman.split('').map((ch, i) => {
          const t = spring({ frame: f - at - i * 4, fps: 60, config: { damping: 12, stiffness: 170 } });
          return <span key={i} style={{ display: 'inline-block', fontFamily: FONT_LATIN, fontWeight: 700, fontSize: size * 0.82, color: C.greenBright, opacity: t, transform: `translateY(${(1 - t) * -90 * u}px) rotateX(${(1 - t) * 90}deg)` }}>{ch}</span>;
        })}
      </div>
      <div style={{ position: 'absolute', fontFamily: FONT_BN, fontWeight: 700, fontSize: size, color: C.white, opacity: m, transform: `scale(${0.6 + m * 0.4})`, filter: `blur(${(1 - m) * 14 * u}px)`, textShadow: `0 0 ${60 * u * m}px rgba(224,48,63,0.6)` }}>{bangla}</div>
    </div>
  );
};

const Words: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', opacity: interpolate(f, [185, 205], [1, 0], clamp), transform: `scale(${1 + ease(f, 180, 205) * 0.2})` }}>
      <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 10 : 50) * u }}>
        {WORDS.map(([r, b], i) => <MorphWord key={r} roman={r} bangla={b} at={WORD_AT[i]} morph={MORPH_AT[i]} />)}
      </div>
    </AbsoluteFill>
  );
};

/** The headline, letter by letter with a slam, and a burst. */
const Headline: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const lines = V ? ['যেভাবে বলেন,', 'সেভাবে লিখুন'] : ['যেভাবে বলেন, সেভাবে লিখুন'];
  let k = 0;
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [112, 130], [1, 0], clamp), transform: `scale(${1 + ease(f, 100, 130) * 0.15})` }}>
      {lines.map((line) => (
        <div key={line} style={{ display: 'flex', flexWrap: 'nowrap', gap: '0.25em', fontSize: (V ? 170 : 150) * u }}>
          {line.split(' ').map((w, i) => {
            const t = spring({ frame: f - 4 - (k++) * 7, fps: 60, config: { damping: 11, stiffness: 190 } });
            return <span key={i} style={{ display: 'inline-block', fontFamily: FONT_BN, fontWeight: 700, lineHeight: 1.2, color: C.white, opacity: Math.min(1, t * 1.4), transform: `translateY(${(1 - t) * 90 * u}px) scale(${1.7 - t * 0.7}) rotate(${(1 - t) * (i % 2 ? 8 : -8)}deg)`, filter: `blur(${(1 - Math.min(1, t)) * 10 * u}px)`, textShadow: `0 0 ${50 * u}px rgba(47,209,142,0.35)` }}>{w}</span>;
          })}
        </div>
      ))}
      <Sub o={ease(f, 40, 60)} size={40} style={{ marginTop: 24 * u }}>Type it the way you say it</Sub>
    </AbsoluteFill>
  );
};

/** The first real look at the popup: the live demo from ekushenext.com in a floating glass screen
 * that swoops in, with a callout on the suggestions. */
const PopupLook: React.FC = () => {
  const f = useCurrentFrame(); // starts at T3.popup
  const { u, V, width } = useLayout();
  const k = Math.max(0, Math.min(HERO_FRAMES - 1, Math.floor((f - (HERO_AT - T3.popup)) / HERO_EVERY) + 1));
  const swoop = spring({ frame: f, fps: 60, config: { damping: 18, stiffness: 70 } });
  const rec = V ? HERO.tall : HERO.wide;
  const W = V ? width * 0.92 : 2200 * u;
  const H = W * (rec.h / rec.w);
  const out = ease(f, 352, 370);
  const call = spring({ frame: f - 70, fps: 60, config: { damping: 14 } });
  const drift = Math.sin(f / 60) * 1.5;
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out, perspective: 2600 * u }}>
      <div style={{ position: 'absolute', top: (V ? 330 : 120) * u, left: 0, right: 0, textAlign: 'center', opacity: io(f, 10, 360, 20) }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 78 : 70) * u, color: C.white }}>প্রথমবার দেখুন</div>
        <Sub style={{ marginTop: 6 * u }}>A first look</Sub>
      </div>
      <div style={{ width: W, height: H, marginTop: (V ? 140 : 120) * u, borderRadius: 44 * u, overflow: 'hidden', transform: `translateY(${(1 - swoop) * 500 * u}px) rotateX(${(1 - swoop) * 40 + drift}deg) rotateY(${(1 - swoop) * -18 + drift}deg) scale(${(0.7 + swoop * 0.3) * (1 + out * 0.2)})`, boxShadow: `0 0 0 ${2 * u}px rgba(255,255,255,0.16), 0 ${60 * u}px ${140 * u}px -${30 * u}px rgba(0,0,0,0.9), 0 0 ${160 * u}px -${20 * u}px rgba(224,48,63,0.35)` }}>
        <Img src={staticFile(`teaser3/${rec.dir}/${pad3(f < HERO_AT - T3.popup ? 0 : k)}.jpg`)} style={{ width: W, height: H, display: 'block' }} />
        <Sweep at={40} dur={60} strength={0.12} />
      </div>
      <div style={{ marginTop: 50 * u, display: 'flex', alignItems: 'center', gap: 18 * u, padding: `${16 * u}px ${30 * u}px`, borderRadius: 999, background: 'rgba(47,209,142,0.14)', border: `${2 * u}px solid rgba(47,209,142,0.5)`, opacity: call * io(f, 70, 350, 16), transform: `translateY(${(1 - call) * 40 * u}px)` }}>
        <span style={{ width: 18 * u, height: 18 * u, borderRadius: 99, background: C.greenBright, boxShadow: `0 0 ${20 * u}px ${C.greenBright}` }} />
        <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 50 : 46) * u, color: C.white }}>সঠিক শব্দটি সবার আগে</span>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: 30 * u, color: 'rgba(234,243,238,0.7)' }}>The right word, first</span>
      </div>
    </AbsoluteFill>
  );
};

/** A 3D ring of what's inside, turning so each one comes to the front in turn. */
const FEATURES: [string, string][] = [
  ['যুক্তবর্ণ মুখস্থ নয়', 'No conjunct shortcuts'],
  ['শুদ্ধ বানান আগে', 'Standard spelling first'],
  ['পরের শব্দ আগেই', 'Next-word suggestions'],
  ['Backspace-এ ফেরত', 'One key to undo'],
  ['ইন্টারনেট ছাড়াই', 'Works offline'],
  ['সম্পূর্ণ ফ্রি', 'Completely free'],
];
const Ring: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const n = FEATURES.length;
  const turn = (f / 300) * Math.PI * 2 * 0.95; // almost one full turn
  const R = (V ? 430 : 780) * u;
  const enter = spring({ frame: f, fps: 60, config: { damping: 16 } });
  const out = ease(f, 284, 300);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', opacity: Math.min(enter, 1 - out), perspective: 3000 * u }}>
      <div style={{ position: 'absolute', top: (V ? 520 : 170) * u, textAlign: 'center', width: '100%' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 80 : 72) * u, color: C.white }}>ভেতরে আরও অনেক কিছু</div>
        <Sub style={{ marginTop: 6 * u }}>And much more inside</Sub>
      </div>
      <div style={{ position: 'relative', width: 10, height: 10, transformStyle: 'preserve-3d', transform: `translateY(${(V ? 120 : 110) * u}px) rotateX(-10deg)` }}>
        {FEATURES.map(([bn, en], i) => {
          const a = (i / n) * Math.PI * 2 - turn;
          const z = Math.cos(a), x = Math.sin(a) * R;
          const front = Math.max(0, z);
          return (
            <div key={bn} style={{ position: 'absolute', left: 0, top: 0, transform: `translate(-50%,-50%) translate3d(${x}px, 0px, ${z * R * 0.6}px) scale(${(V ? 0.55 : 0.75) + front * (V ? 0.6 : 0.45)})`, zIndex: Math.round(z * 100) + 100, opacity: z < 0 ? 0.08 : 0.25 + front * 0.75, filter: `blur(${(1 - front) * 6 * u}px)` }}>
              <div style={{ padding: `${28 * u}px ${44 * u}px`, borderRadius: 34 * u, whiteSpace: 'nowrap', background: front > 0.92 ? 'rgba(47,209,142,0.22)' : 'rgba(255,255,255,0.08)', border: `${2 * u}px solid ${front > 0.92 ? 'rgba(47,209,142,0.75)' : 'rgba(255,255,255,0.2)'}`, boxShadow: front > 0.92 ? `0 0 ${80 * u}px rgba(47,209,142,0.45)` : 'none', backdropFilter: `blur(${16 * u}px)`, textAlign: 'center' }}>
                <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 66 : 60) * u, color: C.white }}>{bn}</div>
                <div style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: 28 * u, color: 'rgba(234,243,238,0.7)', marginTop: 4 * u }}>{en}</div>
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};

/** The line before the ending. */
const Tagline: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const t = spring({ frame: f - 6, fps: 60, config: { damping: 10, stiffness: 160 } });
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [120, 140], [1, 0], clamp) }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 150 : 140) * u, color: C.white, transform: `scale(${1.8 - t * 0.8})`, opacity: Math.min(1, t * 1.5), textAlign: 'center', lineHeight: 1.15, textShadow: `0 0 ${70 * u}px rgba(224,48,63,0.55)` }}>বাংলা লেখার<br />নতুন মান</div>
      <Sub o={ease(f, 30, 50)} size={40} style={{ marginTop: 26 * u }}>A new standard for writing Bangla</Sub>
    </AbsoluteFill>
  );
};

export const Teaser3: React.FC = () => {
  const f = useCurrentFrame();
  const sun = interpolate(f, [0, 60, T3.popup, T3.ring, T3.tag, T3.end], [0.3, 0.7, 0.55, 0.6, 1, 1], clamp);
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Backdrop sun={sun} field={interpolate(f, [0, 300, TEASER3_FRAMES], [0.5, 0.8, 1], clamp)} warmth={interpolate(f, [T3.tag, TEASER3_FRAMES], [0, 0.4], clamp)} />
      <Petals count={28} opacity={0.7} />
      <Streaks from={T3.head - 16} to={T3.head + 20} count={34} seed="a" />
      <Streaks from={T3.popup - 16} to={T3.popup + 24} count={34} seed="b" />
      <Streaks from={T3.tag - 16} to={T3.tag + 24} count={40} color="rgba(255,120,130,0.8)" seed="c" />
      <Sequence from={T3.words} durationInFrames={205}><Words /></Sequence>
      <Sequence from={T3.head} durationInFrames={130}><Headline /></Sequence>
      <Sequence from={T3.popup} durationInFrames={370}><PopupLook /></Sequence>
      <Sequence from={T3.ring} durationInFrames={300}><Ring /></Sequence>
      <Sequence from={T3.tag} durationInFrames={140}><Tagline /></Sequence>
      <Burst at={T3.head + 30} y={0.45} seed="h" count={40} />
      <Burst at={T3.tag + 10} y={0.45} seed="t" count={40} />
      <Sequence from={T3.end}><EndCard /></Sequence>
      <Audio src={staticFile('music/teaser3.wav')} />
    </AbsoluteFill>
  );
};
