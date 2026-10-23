import React from 'react';
import { AbsoluteFill, Audio, interpolate, random, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease } from '../v2/parts';
import { bn, C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { Burst, END_LEN, EndCard, Streaks, Sub } from './common';
import WORDS from './t4words.json';

/** Teaser 4 of 5: "চোখের পলকের চেয়েও দ্রুত" (faster than a blink). About 20.7 s at 60 fps.
 * A blink in extreme slow motion; while the eye is closed, 24 real words are typed and placed;
 * the eye opens, the numbers race, a tagline, the ending. Every word and suggestion shown is real
 * output of the engine (t4words.json). */
export const T4 = { blink: 0, rapid: 240, open: 600, race: 700, tag: 960, end: 1060 } as const;
export const TEASER4_FRAMES = T4.end + END_LEN;
export const WORD_EVERY = 14;                 // a new word every 14 frames during the rapid fire
const W = WORDS as { roman: string; cands: string[] }[];

/** The eye: almond outline, a red-sun iris ringed in green, lids that close and open. */
const Eye: React.FC<{ close: number; size: number }> = ({ close, size }) => {
  const f = useCurrentFrame();
  const h = size * 0.5;
  const lid = h * 0.52 * close; // how far each lid has travelled
  return (
    <svg width={size} height={h * 1.2} viewBox={`0 ${-h * 0.1} ${size} ${h * 1.2}`} style={{ overflow: 'visible' }}>
      <defs>
        <clipPath id="eyeShape"><path d={`M0 ${h / 2} Q ${size / 2} ${-h * 0.45} ${size} ${h / 2} Q ${size / 2} ${h * 1.45} 0 ${h / 2} Z`} /></clipPath>
        <radialGradient id="iris"><stop offset="0" stopColor="#1a0306" /><stop offset="0.28" stopColor="#1a0306" /><stop offset="0.32" stopColor={C.red} /><stop offset="0.78" stopColor="#B81E2C" /><stop offset="0.86" stopColor={C.greenBright} /><stop offset="1" stopColor="#0B6B4A" /></radialGradient>
      </defs>
      <g clipPath="url(#eyeShape)">
        <rect x={0} y={-h} width={size} height={h * 3} fill="#EEF4F1" />
        <circle cx={size / 2 + Math.sin(f / 50) * size * 0.03} cy={h / 2} r={h * 0.46} fill="url(#iris)" />
        <circle cx={size / 2 - h * 0.12} cy={h / 2 - h * 0.14} r={h * 0.07} fill="#fff" opacity={0.9} />
        <rect x={0} y={-h} width={size} height={h + lid + h * 0.02} fill="#0B2A1F" />
        <rect x={0} y={h - lid - h * 0.02} width={size} height={h * 2} fill="#0B2A1F" />
      </g>
      <path d={`M0 ${h / 2} Q ${size / 2} ${-h * 0.45 + lid * 1.9} ${size} ${h / 2}`} fill="none" stroke="#EEF4F1" strokeWidth={size * 0.012} strokeLinecap="round" />
      <path d={`M0 ${h / 2} Q ${size / 2} ${h * 1.45 - lid * 1.9} ${size} ${h / 2}`} fill="none" stroke="#EEF4F1" strokeWidth={size * 0.012} strokeLinecap="round" opacity={0.6} />
    </svg>
  );
};

/** 0–4 s: one blink, slowed right down, with a clock. */
const SlowBlink: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  // open for a moment, then the lids fall slowly, speeding up at the end like a real blink in slow motion
  const close = Math.pow(interpolate(f, [50, 232], [0, 1], clamp), 1.8);
  const ms = Math.round(interpolate(f, [50, 232], [0, 150], clamp));
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 40 * u }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 90 : 80) * u, color: C.white, opacity: ease(f, 0, 30) }}>চোখের একটি পলক</div>
      <div style={{ transform: `scale(${1 + f * 0.0009})` }}><Eye close={close} size={(V ? 1100 : 1000) * u} /></div>
      <div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: (V ? 80 : 72) * u, color: C.mustard, fontVariantNumeric: 'tabular-nums', opacity: ease(f, 30, 50) }}>{bn(ms)} <span style={{ fontFamily: FONT_BN, fontSize: '0.6em' }}>মিলিসেকেন্ড</span></div>
      <Sub o={ease(f, 60, 90)}>One blink takes 100–400 milliseconds</Sub>
    </AbsoluteFill>
  );
};

/** 4–10 s: slowed down so you can see it: the 24 words (194 keystrokes) Ekushe processes in about
 * 0.19 s, which is within one blink. */
const Rapid: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width, height } = useLayout();
  const live = Math.min(W.length - 1, Math.floor(f / WORD_EVERY));
  const lf = f - live * WORD_EVERY;
  const cur = W[live];
  const typed = cur.roman.slice(0, Math.min(cur.roman.length, Math.floor(lf / 1.2) + 1));
  const placed = W.slice(0, live);
  const out = ease(f, 345, 360);
  return (
    <AbsoluteFill style={{ opacity: 1 - out * 0.0 }}>
      {/* the placed words drift into a cloud */}
      {placed.map((w, i) => {
        const age = f - (i + 1) * WORD_EVERY;
        const t = ease(age, 0, 30);
        const a = random(`ra${i}`) * Math.PI * 2;
        const r = (0.28 + random(`rr${i}`) * 0.22) * Math.min(width, height) * (V ? 1.15 : 1.4);
        const x = width / 2 + Math.cos(a) * r * t * (V ? 0.7 : 1.15);
        const y = height / 2 + Math.sin(a) * r * t * (V ? 1.25 : 0.62);
        return (
          <div key={w.roman} style={{ position: 'absolute', left: x, top: y, transform: `translate(-50%,-50%) scale(${1.3 - t * 0.6})`, fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 76 : 66) * u, color: C.white, opacity: 0.25 + (1 - t) * 0.75, filter: `blur(${t * 1.5 * u}px)`, whiteSpace: 'nowrap' }}>{w.cands[0]}</div>
        );
      })}
      {/* the word being typed, with its real suggestions */}
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column' }}>
        <div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: (V ? 70 : 62) * u, color: C.greenBright }}>{typed}</div>
        <div style={{ display: 'flex', gap: 14 * u, marginTop: 18 * u, padding: 14 * u, borderRadius: 22 * u, background: 'rgba(26,40,34,0.85)', border: `${2 * u}px solid rgba(255,255,255,0.14)`, boxShadow: `0 ${20 * u}px ${60 * u}px rgba(0,0,0,0.6)` }}>
          {cur.cands.slice(0, V ? 3 : 4).map((c, i) => (
            <div key={c + i} style={{ display: 'flex', alignItems: 'baseline', gap: 10 * u, padding: `${8 * u}px ${20 * u}px`, borderRadius: 14 * u, background: i === 0 ? C.greenBright : 'rgba(255,255,255,0.08)', color: i === 0 ? '#03170F' : C.white, fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 58 : 52) * u }}>
              <span style={{ fontFamily: FONT_LATIN, fontSize: 26 * u, opacity: 0.6 }}>{bn(i + 1)}</span>{c}
            </div>
          ))}
        </div>
      </AbsoluteFill>
      <div style={{ position: 'absolute', top: (V ? 260 : 110) * u, left: 0, right: 0, textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 84 : 72) * u, color: C.white }}>এক পলকের সময়ে…</div>
        <Sub style={{ marginTop: 6 * u }}>In the time of a single blink…</Sub>
      </div>
      <div style={{ position: 'absolute', bottom: (V ? 300 : 110) * u, left: 0, right: 0, display: 'flex', justifyContent: 'center', gap: 24 * u, alignItems: 'baseline' }}>
        <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 120 : 100) * u, color: C.mustard, fontVariantNumeric: 'tabular-nums' }}>{bn(live)}</span>
        <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 52 : 46) * u, color: C.white }}>টি শব্দ</span>
      </div>
    </AbsoluteFill>
  );
};

/** 10–11.7 s: the eye snaps open. */
const Open: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const open = spring({ frame: f - 4, fps: 60, config: { damping: 12, stiffness: 260 } });
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 30 * u, opacity: interpolate(f, [80, 100], [1, 0], clamp) }}>
      <Eye close={1 - open} size={(V ? 1100 : 1000) * u} />
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 96 : 84) * u, color: C.white, opacity: ease(f, 14, 30), transform: `scale(${1.3 - ease(f, 14, 30) * 0.3})` }}>১৯৪টি কী চাপ, এক পলকেই সামলানো</div>
      <Sub o={ease(f, 26, 40)}>Ekushe processes all 194 keystrokes of these 24 words within one blink</Sub>
    </AbsoluteFill>
  );
};

/** 11.7–16 s: the numbers. A blink's bar crawls; Ekushe's is done instantly; the per-key time
 * counts down to 0.98 ms. */
const Race: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const blinkW = ease(f, 30, 220);
  const ekW = ease(f, 30, 34);
  const n = interpolate(ease(f, 40, 150), [0, 1], [400, 0.98]);
  const label = n > 10 ? bn(Math.round(n)) : bn(n.toFixed(2));
  const BW = (V ? 1200 : 1800) * u;
  const row = (title: string, sub: string, w: number, color: string, glow: boolean) => (
    <div style={{ width: BW }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', marginBottom: 14 * u }}>
        <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 50 : 46) * u, color: C.white }}>{title}</span>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 34 * u, color: 'rgba(234,243,238,0.75)' }}>{sub}</span>
      </div>
      <div style={{ height: 30 * u, borderRadius: 99, background: 'rgba(255,255,255,0.07)', overflow: 'hidden' }}>
        <div style={{ width: `${Math.max(w * 100, glow ? 0.6 : 0)}%`, height: '100%', borderRadius: 99, background: color, boxShadow: glow ? `0 0 ${30 * u}px ${color}` : 'none' }} />
      </div>
    </div>
  );
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: (V ? 70 : 50) * u, opacity: interpolate(f, [244, 260], [1, 0], clamp) }}>
      <div style={{ textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_LATIN, fontWeight: 800, fontSize: (V ? 300 : 280) * u, lineHeight: 1, color: C.white, fontVariantNumeric: 'tabular-nums', textShadow: `0 0 ${80 * u}px rgba(47,209,142,0.45)` }}>{label}<span style={{ fontSize: '0.3em', marginLeft: 16 * u, color: C.greenBright }}>ms</span></div>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 56 : 50) * u, color: C.white, marginTop: 10 * u, opacity: ease(f, 150, 170) }}>প্রতিটি কী চাপে, গড়ে</div>
        <Sub o={ease(f, 160, 180)}>per keystroke, on average</Sub>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 40 * u, opacity: ease(f, 10, 30) }}>
        {row('চোখের এক পলক', '100–400 ms', blinkW, `linear-gradient(90deg, #8a6b2a, ${C.mustard})`, false)}
        {row('একুশে নেক্সট', '0.98 ms', ekW * 0.006, C.greenBright, true)}
      </div>
    </AbsoluteFill>
  );
};

/** 16–17.7 s: the line before the ending. */
const Tagline: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const t = spring({ frame: f - 4, fps: 60, config: { damping: 10, stiffness: 170 } });
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [84, 100], [1, 0], clamp) }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 150 : 140) * u, lineHeight: 1.15, textAlign: 'center', color: C.white, transform: `scale(${1.8 - t * 0.8}) skewX(${(1 - t) * -12}deg)`, opacity: Math.min(1, t * 1.5), textShadow: `0 0 ${70 * u}px rgba(47,209,142,0.5)` }}>ভাবনার গতিতে<br />লিখুন</div>
      <Sub o={ease(f, 24, 44)} size={40} style={{ marginTop: 22 * u }}>Write at the speed of thought</Sub>
    </AbsoluteFill>
  );
};

export const Teaser4: React.FC<{ audio?: boolean }> = ({ audio = true }) => {
  const f = useCurrentFrame();
  const sun = interpolate(f, [0, T4.rapid, T4.open, T4.race, T4.tag, T4.end], [0.25, 0.1, 0.15, 0.55, 0.9, 1], clamp);
  const dark = interpolate(f, [T4.rapid - 10, T4.rapid + 10, T4.open, T4.open + 10], [0, 0.55, 0.55, 0], clamp);
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Backdrop sun={sun} field={interpolate(f, [0, T4.race, TEASER4_FRAMES], [0.4, 0.7, 1], clamp)} warmth={interpolate(f, [T4.tag, TEASER4_FRAMES], [0, 0.4], clamp)} />
      <AbsoluteFill style={{ background: '#020806', opacity: dark }} />
      <Streaks from={T4.rapid} to={T4.open} count={50} seed="r" />
      <Streaks from={T4.tag - 20} to={T4.tag + 30} count={40} color="rgba(233,185,73,0.85)" seed="t" />
      <Sequence from={T4.blink} durationInFrames={240}><SlowBlink /></Sequence>
      <Sequence from={T4.rapid} durationInFrames={360}><Rapid /></Sequence>
      <Sequence from={T4.open} durationInFrames={100}><Open /></Sequence>
      <Burst at={T4.open + 4} y={0.45} count={60} seed="o" spread={0.8} />
      <Sequence from={T4.race} durationInFrames={260}><Race /></Sequence>
      <Sequence from={T4.tag} durationInFrames={100}><Tagline /></Sequence>
      <Burst at={T4.tag + 8} y={0.45} count={40} seed="g" />
      <Sequence from={T4.end}><EndCard /></Sequence>
      {audio && <Audio src={staticFile('music/teaser4.wav')} />}
    </AbsoluteFill>
  );
};
