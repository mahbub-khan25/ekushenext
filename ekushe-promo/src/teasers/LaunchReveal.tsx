import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, random, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { clamp, ease, Petals } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { AddressPill, Burst, Sub, Sweep, TypeOn, graphemes } from './common';
import { OsIcon } from './FullPromo';
import L from './launch_lines.json';

/** Launch-day reveal: the Shaheed Minar in the dark, a red sun rises and lights it, 30 October,
 * then a welcome typed on Windows, macOS and Linux (all out today), a message from the phones
 * (Android and iPhone, coming soon), a celebration and where to download. 9:16, 2K, 60 fps, 84 s.
 * Every typed line is what Ekushe's engine really commits (launch_lines.json).
 * music/launch_reveal.py is composed to the same marks. */
export const R = L.R;
export const LAUNCH_FRAMES = R.total;
const MONO = "'DejaVu Sans Mono', 'Menlo', 'Consolas', monospace";

type Rgb = [number, number, number];
const mix = (a: Rgb, b: Rgb, t: number) => `rgb(${a.map((v, i) => Math.round(v + (b[i] - v) * t)).join(',')})`;

/* ───────────────────────── typing, word by word ───────────────────────── */
type Tok = [string, string];
type Planned = { r: string; b: string; s: number; e: number; c: number };
const plan = (tokens: Tok[], start: number): Planned[] => {
  let t = start;
  return tokens.map(([r, b]) => {
    const s = t;
    const e = s + r.length * L.every;
    t = e + L.gap;
    return { r, b, s, e, c: e + L.gap - 4 };
  });
};
const typing = (f: number, P: Planned[]) => {
  let done = '';
  for (const w of P) {
    if (f >= w.c) { done += (done ? ' ' : '') + w.b; continue; }
    if (f < w.s) break;
    const n = Math.min(w.r.length, Math.floor((f - w.s) / L.every) + 1);
    const g = graphemes(w.b);
    const k = f >= w.e ? g.length : Math.max(1, Math.round((n / w.r.length) * g.length));
    return { done, live: g.slice(0, k).join(''), roman: w.r, n, picked: f >= w.e, complete: false };
  }
  const last = P[P.length - 1];
  return { done, live: '', roman: '', n: 0, picked: false, complete: f >= last.c };
};
const endOf = (P: Planned[]) => P[P.length - 1].c;

/** The Roman letters of the word being typed, in small key caps */
const KeyStrip: React.FC<{ roman: string; n: number; light?: boolean }> = ({ roman, n, light }) => {
  const { u } = useLayout();
  return (
    <div style={{ display: 'flex', gap: 10 * u, justifyContent: 'center', minHeight: 84 * u }}>
      {Array.from(roman).slice(0, n).map((c, i) => {
        const last = i === n - 1;
        return (
          <div key={i} style={{ minWidth: 66 * u, height: 80 * u, padding: `0 ${12 * u}px`, display: 'grid', placeItems: 'center', borderRadius: 18 * u, background: last ? 'rgba(47,209,142,0.32)' : light ? 'rgba(255,255,255,0.16)' : 'rgba(255,255,255,0.09)', border: `${2 * u}px solid rgba(255,255,255,0.2)`, boxShadow: `0 ${last ? 2 : 6}px 0 rgba(0,0,0,0.45)`, transform: `translateY(${last ? 4 * u : 0}px)`, fontFamily: MONO, fontWeight: 700, fontSize: 42 * u, color: C.white }}>{c}</div>
        );
      })}
    </div>
  );
};

/** Text being written: what's committed, the word forming (underlined), a caret */
const Written: React.FC<{ done: string; live: string; ink: string; accent: string; size: number; align?: 'left' | 'center' }> = ({ done, live, ink, accent, size, align = 'left' }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  return (
    <div style={{ fontFamily: FONT_BN, fontWeight: 500, fontSize: size * u, lineHeight: 1.55, color: ink, textAlign: align, wordBreak: 'keep-all' }}>
      {done}{done && live ? ' ' : ''}
      {live && <span style={{ color: accent, borderBottom: `${4 * u}px solid ${accent}` }}>{live}</span>}
      <span style={{ display: 'inline-block', width: 4 * u, height: size * 1.0 * u, marginLeft: 4 * u, verticalAlign: '-0.15em', background: accent, opacity: Math.floor(f / 18) % 2 ? 0.2 : 1 }} />
    </div>
  );
};

/** Ekushe's suggestion bar, floating under the text */
const Cands: React.FC<{ word: string; picked: boolean; dark: boolean }> = ({ word, picked, dark }) => {
  const { u } = useLayout();
  if (!word) return <div style={{ height: 96 * u }} />;
  return (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap: 14 * u, padding: `${10 * u}px ${12 * u}px`, borderRadius: 24 * u, background: dark ? 'rgba(30,36,34,0.96)' : 'rgba(255,255,255,0.98)', border: `${2 * u}px solid ${dark ? 'rgba(255,255,255,0.12)' : 'rgba(0,0,0,0.1)'}`, boxShadow: `0 ${18 * u}px ${50 * u}px rgba(0,0,0,0.35)` }}>
      <Img src={staticFile('assets/logo_64.png')} style={{ width: 44 * u, height: 44 * u, borderRadius: 10 * u, marginLeft: 6 * u }} />
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 10 * u, padding: `${8 * u}px ${24 * u}px`, borderRadius: 16 * u, background: C.greenBright, color: '#03170F', transform: `scale(${picked ? 1.06 : 1})`, boxShadow: picked ? `0 0 ${36 * u}px rgba(47,209,142,0.6)` : 'none' }}>
        <span style={{ fontFamily: MONO, fontSize: 24 * u, opacity: 0.6 }}>1</span>
        <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 50 * u }}>{word}</span>
      </div>
    </div>
  );
};

/* ───────────────────────── the Minar, the sun, the sky ───────────────────────── */
const MB = 2080; // the Minar stands on this line (in u)
const MH = 900;
const SUN_Y = MB - MH + MH * 0.47;

/** Shaheed Minar drawn as stone that the sun lights: `lit` 0 is a dim outline in the dark, 1 is white stone */
const Minar: React.FC<{ size: number; lit: number }> = ({ size, lit }) => {
  const h = size;
  const w = size * 1.3;
  const sw = size * 0.018;
  const stone = mix([38, 58, 50], [255, 255, 255], lit);
  const stoneLow = mix([26, 40, 35], [214, 226, 220], lit);
  const fill = `rgba(255,255,255,${0.02 + lit * 0.05})`;
  const frame = (x: number, cw: number, ch: number, key: string) => {
    const top = h - ch;
    return (
      <g key={key}>
        <rect x={x} y={top} width={cw} height={ch} fill={fill} stroke="url(#mstone)" strokeWidth={sw} />
        {[0.25, 0.5, 0.75].map((k) => <line key={k} x1={x + cw * k} x2={x + cw * k} y1={top} y2={h} stroke="url(#mstone)" strokeWidth={sw * 0.45} />)}
      </g>
    );
  };
  const cx = w * 0.4, cw = w * 0.2, ch = h * 0.7, top = h - ch, lean = h * 0.22;
  return (
    <svg width={w} height={h * 1.05} viewBox={`0 ${-h * 0.05} ${w} ${h * 1.05}`} style={{ overflow: 'visible', filter: `drop-shadow(0 0 ${size * 0.03 * lit}px rgba(255,120,110,${0.55 * lit}))` }}>
      <defs>
        <linearGradient id="mstone" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={stone} />
          <stop offset="100%" stopColor={stoneLow} />
        </linearGradient>
      </defs>
      {frame(w * 0.05, w * 0.1, h * 0.42, 'l2')}
      {frame(w * 0.2, w * 0.12, h * 0.56, 'l1')}
      {frame(w * 0.68, w * 0.12, h * 0.56, 'r1')}
      {frame(w * 0.85, w * 0.1, h * 0.42, 'r2')}
      <rect x={cx} y={top} width={cw} height={ch} fill={fill} stroke="url(#mstone)" strokeWidth={sw} />
      {[0.25, 0.5, 0.75].map((k) => <line key={k} x1={cx + cw * k} x2={cx + cw * k} y1={top} y2={h} stroke="url(#mstone)" strokeWidth={sw * 0.45} />)}
      <polygon points={`${cx},${top} ${cx + cw},${top} ${cx + cw + lean * 0.55},${top - lean * 0.85} ${cx + lean * 0.55},${top - lean * 0.85}`} fill={fill} stroke="url(#mstone)" strokeWidth={sw} strokeLinejoin="round" />
      {[0.25, 0.5, 0.75].map((k) => <line key={`t${k}`} x1={cx + cw * k} y1={top} x2={cx + cw * k + lean * 0.55} y2={top - lean * 0.85} stroke="url(#mstone)" strokeWidth={sw * 0.45} />)}
      <rect x={-w * 0.02} y={h * 0.99} width={w * 1.04} height={h * 0.025} rx={h * 0.01} fill={stone} opacity={0.4 + lit * 0.2} />
    </svg>
  );
};

/** The whole backdrop: night sky and stars, the sun coming up from below, the Minar it lights.
 * After the name it all settles low behind the scenes, and comes up a little for the ending. */
const Scenery: React.FC = () => {
  const f = useCurrentFrame();
  const { width, height, u } = useLayout();
  const rise = ease(f, R.sun, R.sun + 440);
  const lit = ease(f, R.sun + 140, R.date);
  const down = ease(f, R.name, R.name + 140) - ease(f, R.end, R.end + 120) * 0.45;
  const sx = width / 2;
  const sy = interpolate(rise, [0, 1], [height + 420 * u, SUN_Y * u]) + Math.sin(f / 200) * 6 * u;
  const sr = 300 * u;
  const pulse = 1 + Math.sin(f / 30) * 0.015;
  const dim = 1 - down * 0.5;
  return (
    <AbsoluteFill style={{ overflow: 'hidden' }}>
      {/* sky */}
      <AbsoluteFill style={{ background: `radial-gradient(120% 80% at 50% 10%, ${mix([3, 14, 10], [12, 44, 32], lit)} 0%, ${mix([1, 6, 4], [3, 18, 12], lit)} 60%, #010503 100%)` }} />
      <AbsoluteFill style={{ background: `radial-gradient(70% 45% at 50% ${(sy / height) * 100}%, rgba(224,48,63,${0.5 * rise}) 0%, rgba(242,107,79,${0.18 * rise}) 40%, rgba(0,0,0,0) 75%)` }} />
      {/* stars */}
      {Array.from({ length: 110 }).map((_, i) => {
        const tw = 0.5 + 0.5 * Math.sin(f / (14 + random(`tw${i}`) * 30) + i);
        const s = (2 + random(`ss${i}`) * 4) * u;
        return <div key={i} style={{ position: 'absolute', left: random(`sx${i}`) * width, top: random(`sy${i}`) * height * 0.75, width: s, height: s, borderRadius: 9, background: '#fff', opacity: (0.25 + 0.6 * tw) * (1 - lit * 0.85), boxShadow: `0 0 ${s * 2}px #fff` }} />;
      })}
      {/* rays */}
      <AbsoluteFill style={{ opacity: lit * 0.35 * dim, background: `repeating-conic-gradient(from ${f * 0.04}deg at ${sx}px ${sy}px, rgba(255,140,120,0.16) 0deg 3deg, rgba(0,0,0,0) 3deg 11deg)`, WebkitMaskImage: `radial-gradient(circle at ${sx}px ${sy}px, black 0%, rgba(0,0,0,0.6) 30%, transparent 62%)` }} />
      {/* green field at the bottom */}
      <div style={{ position: 'absolute', left: -width * 0.3, bottom: -height * 0.3, width: width * 1.6, height: height * 0.62, borderRadius: '50%', background: 'radial-gradient(ellipse, #0F8A5F 0%, rgba(11,107,74,0.5) 45%, rgba(11,107,74,0) 72%)', opacity: 0.15 + lit * 0.75, filter: `blur(${40 * u}px)` }} />
      {/* sun and Minar move together */}
      <AbsoluteFill style={{ transform: `translateY(${down * 430 * u}px) scale(${1 - down * 0.38})`, transformOrigin: `50% ${MB * u}px`, opacity: dim }}>
        <div style={{ position: 'absolute', left: sx - sr * pulse, top: sy - sr * pulse, width: sr * 2 * pulse, height: sr * 2 * pulse, borderRadius: '50%', background: `radial-gradient(circle at 45% 40%, #FF4B57 0%, ${C.red} 55%, #B81E2C 100%)`, boxShadow: `0 0 ${160 * u}px ${40 * u}px rgba(224,48,63,0.55), 0 0 ${420 * u}px ${120 * u}px rgba(224,48,63,0.25)` }} />
        <div style={{ position: 'absolute', left: width / 2 - MH * 1.3 * u / 2, top: (MB - MH) * u }}>
          <Minar size={MH * u} lit={lit} />
        </div>
        {/* ground mist in front of the sun's lower half */}
        <div style={{ position: 'absolute', left: 0, right: 0, top: (MB - 10) * u, height: 700 * u, background: `linear-gradient(180deg, ${mix([2, 10, 7], [5, 30, 21], lit)} 0%, #010503 100%)` }} />
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

/* ───────────────────────── 0–16 s: dark, then the sun ───────────────────────── */
const Dark: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const a = Math.min(ease(f, 90, 150), 1 - ease(f, 400, 450));
  const b = Math.min(ease(f, 640, 700), 1 - ease(f, 900, 950));
  return (
    <AbsoluteFill>
      <div style={{ position: 'absolute', top: 620 * u, left: 0, right: 0, textAlign: 'center', opacity: a, transform: `translateY(${(1 - a) * 30 * u}px)`, filter: `blur(${(1 - a) * 8 * u}px)` }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 92 * u, color: 'rgba(234,243,238,0.88)' }}>অনেক অপেক্ষার পর…</div>
        <Sub size={36} style={{ marginTop: 16 * u }}>After a long wait…</Sub>
      </div>
      <div style={{ position: 'absolute', top: 560 * u, left: 0, right: 0, textAlign: 'center', opacity: b, transform: `translateY(${(1 - b) * 30 * u}px) scale(${0.96 + b * 0.04})` }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 140 * u, color: C.white, textShadow: `0 0 ${70 * u}px rgba(224,48,63,0.7)` }}>নতুন ভোর</div>
        <Sub size={38} style={{ marginTop: 10 * u }}>A new dawn</Sub>
      </div>
      <Sweep at={R.date - 40} dur={60} strength={0.16} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 16–22 s: 30 October ───────────────────────── */
const DateScene: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const roman = '30 october';
  const n = Math.max(0, Math.min(roman.length, Math.floor((f - 20) / 6) + 1));
  const morph = ease(f, 104, 130);
  const out = ease(f, 336, 360);
  const typedIn = f >= 20 ? 1 : 0;
  return (
    <AbsoluteFill style={{ opacity: 1 - out, filter: `blur(${out * 10 * u}px)` }}>
      <div style={{ position: 'absolute', top: 360 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        <div style={{ opacity: typedIn * (1 - morph) }}><KeyStrip roman={roman} n={n} /></div>
        <div style={{ position: 'relative', height: 250 * u, width: '100%', display: 'grid', placeItems: 'center', marginTop: -40 * u }}>
          <div style={{ position: 'absolute', fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 130 * u, color: C.mustard, opacity: (1 - morph) * (f >= 60 ? 1 : 0), filter: `blur(${morph * 12 * u}px)` }}>{f >= 60 ? roman.slice(0, n) : ''}</div>
          <div style={{ position: 'absolute', fontFamily: FONT_BN, fontWeight: 700, fontSize: 200 * u, color: C.white, opacity: morph, transform: `scale(${0.8 + morph * 0.2})`, textShadow: `0 0 ${90 * u * morph}px rgba(224,48,63,0.85)` }}>৩০ অক্টোবর</div>
        </div>
        <div style={{ marginTop: 30 * u, height: 130 * u }}>
          <TypeOn text="অবশেষে এসে গেছে!" start={170} every={4} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 104 * u, color: C.greenBright, textShadow: `0 0 ${50 * u}px rgba(47,209,142,0.55)` }} />
        </div>
        <Sub size={38} o={ease(f, 230, 260)} style={{ marginTop: 16 * u }}>30 October · It’s finally here!</Sub>
      </div>
      <Burst at={110} y={0.2} count={40} seed="dt" spread={0.6} />
      <Burst at={224} y={0.28} count={26} seed="dt2" spread={0.4} colors={[C.greenBright, '#ffffff', C.mustard]} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 22–27 s: the name, out today everywhere ───────────────────────── */
const OsPill: React.FC<{ k: 'win' | 'mac' | 'linux'; at: number; size?: number }> = ({ k, at, size = 40 }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const p = spring({ frame: f - at, fps: 60, config: { damping: 12, stiffness: 160 } });
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 14 * u, padding: `${12 * u}px ${28 * u}px`, borderRadius: 99, background: C.greenBright, color: '#03170F', fontFamily: FONT_LATIN, fontWeight: 700, fontSize: size * u, opacity: Math.min(1, p * 1.4), transform: `scale(${0.6 + p * 0.4}) translateY(${(1 - p) * 30 * u}px)` }}>
      <OsIcon kind={k} size={size * 1.05 * u} />{k === 'win' ? 'Windows' : k === 'mac' ? 'macOS' : 'Linux'}
    </div>
  );
};

const NameScene: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const pop = spring({ frame: f - 8, fps: 60, config: { damping: 11, mass: 0.8 } });
  const out = ease(f, 278, 300);
  const name = graphemes('একুশে নেক্সট');
  return (
    <AbsoluteFill style={{ alignItems: 'center', flexDirection: 'column', paddingTop: 430 * u, opacity: 1 - out, filter: `blur(${out * 10 * u}px)` }}>
      <Img src={staticFile('assets/logo_512.png')} style={{ width: 300 * u, height: 300 * u, borderRadius: 68 * u, transform: `scale(${0.5 + pop * 0.5}) rotate(${(1 - pop) * -14}deg)`, opacity: Math.min(1, pop * 1.4), boxShadow: `0 ${30 * u}px ${90 * u}px rgba(0,0,0,0.6), 0 0 ${160 * u}px rgba(224,48,63,0.6)` }} />
      <div style={{ display: 'flex', marginTop: 40 * u }}>
        {name.map((g, i) => {
          const t = spring({ frame: f - 24 - i * 3, fps: 60, config: { damping: 14 } });
          return <span key={i} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 170 * u, color: C.white, whiteSpace: 'pre', display: 'inline-block', opacity: t, transform: `translateY(${(1 - t) * 60 * u}px)`, textShadow: `0 0 ${60 * u}px rgba(224,48,63,0.5)` }}>{g}</span>;
        })}
      </div>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 76 * u, color: C.mustard, marginTop: 20 * u, opacity: ease(f, 90, 115) }}>আজ থেকে পাওয়া যাচ্ছে</div>
      <div style={{ display: 'flex', gap: 22 * u, marginTop: 30 * u }}>
        <OsPill k="win" at={110} /><OsPill k="mac" at={122} /><OsPill k="linux" at={134} />
      </div>
      <Sub size={36} o={ease(f, 150, 175)} style={{ marginTop: 22 * u }}>Available today on Windows, macOS and Linux</Sub>
      <Burst at={8} y={0.27} count={46} seed="nm" spread={0.7} />
      <Sweep at={30} dur={50} strength={0.14} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 27–56 s: the welcome on each computer ───────────────────────── */
type Os = 'win' | 'mac' | 'linux';
const CLOCK = { win: ['10:30 AM', '30-10-2026'], mac: 'Fri 30 Oct 10:30', linux: 'Oct 30  10:30' } as const;

/** A small badge in the system bar showing Ekushe is on and writing Bangla */
const LangBadge: React.FC<{ on: boolean; dark: boolean; size: number }> = ({ on, dark, size }) => {
  const { u } = useLayout();
  return (
    <div style={{ display: 'grid', placeItems: 'center', minWidth: size * 1.25 * u, height: size * u, padding: `0 ${8 * u}px`, borderRadius: 8 * u, background: on ? C.greenBright : 'transparent', border: on ? 'none' : `${2 * u}px solid ${dark ? 'rgba(255,255,255,0.7)' : 'rgba(0,0,0,0.6)'}`, color: on ? '#03170F' : dark ? '#fff' : '#111', fontFamily: on ? FONT_BN : FONT_LATIN, fontWeight: 700, fontSize: size * 0.62 * u, boxShadow: on ? `0 0 ${24 * u}px rgba(47,209,142,0.7)` : 'none' }}>{on ? 'এ' : 'EN'}</div>
  );
};

const Desktop: React.FC<{ os: Os; t: ReturnType<typeof typing>; on: boolean; glow: number; note: number }> = ({ os, t, on, glow, note }) => {
  const { u } = useLayout();
  const W = 1280, H = 1240;
  const dark = os !== 'mac';
  const ink = dark ? '#F2F5F3' : '#1B1F1D';
  const accent = dark ? C.greenBright : '#0B8A5C';
  const wall = os === 'win'
    ? 'radial-gradient(90% 70% at 30% 30%, #2c6bd6 0%, #0d3a8f 45%, #06173d 100%)'
    : os === 'mac'
      ? 'linear-gradient(160deg, #f6a35b 0%, #e2546e 38%, #6b3fa0 75%, #2a2160 100%)'
      : 'radial-gradient(90% 70% at 70% 20%, #4a2a6e 0%, #241a44 45%, #0e0c1f 100%)';
  const win = { win: { x: 70, y: 90, w: W - 140, h: H - 270 }, mac: { x: 70, y: 130, w: W - 140, h: H - 330 }, linux: { x: 70, y: 120, w: W - 140, h: H - 210 } }[os];
  const title = os === 'win' ? 'শুভেচ্ছা.txt – Notepad' : os === 'mac' ? 'শুভেচ্ছা' : 'শুভেচ্ছা — Text Editor';
  return (
    <div style={{ position: 'relative', width: W * u, height: H * u, borderRadius: 44 * u, overflow: 'hidden', background: wall, boxShadow: `0 ${50 * u}px ${140 * u}px -${30 * u}px rgba(0,0,0,0.9), 0 0 ${120 * u * glow}px rgba(47,209,142,${0.45 * glow})`, border: `${3 * u}px solid rgba(255,255,255,${0.12 + glow * 0.3})` }}>
      {/* system bars */}
      {os === 'mac' && (
        <div style={{ position: 'absolute', top: 0, left: 0, right: 0, height: 60 * u, display: 'flex', alignItems: 'center', gap: 34 * u, padding: `0 ${30 * u}px`, background: 'rgba(255,255,255,0.32)', backdropFilter: `blur(${20 * u}px)`, color: '#111', fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 28 * u }}>
          <OsIcon kind="mac" size={32 * u} /><span style={{ fontWeight: 800 }}>TextEdit</span><span style={{ fontWeight: 500 }}>File</span><span style={{ fontWeight: 500 }}>Edit</span><span style={{ fontWeight: 500 }}>Format</span>
          <div style={{ flex: 1 }} /><LangBadge on={on} dark={false} size={40} /><span style={{ fontWeight: 500 }}>{CLOCK.mac}</span>
        </div>
      )}
      {os === 'linux' && (
        <div style={{ position: 'absolute', top: 0, left: 0, right: 0, height: 60 * u, display: 'flex', alignItems: 'center', padding: `0 ${26 * u}px`, background: '#0b0b0d', color: '#fff', fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 28 * u }}>
          <div style={{ display: 'flex', gap: 8 * u }}>{[1, 0, 0].map((a, i) => <div key={i} style={{ width: (a ? 44 : 12) * u, height: 12 * u, borderRadius: 9, background: a ? '#fff' : 'rgba(255,255,255,0.45)' }} />)}</div>
          <div style={{ flex: 1, textAlign: 'center' }}>{CLOCK.linux}</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 20 * u }}><LangBadge on={on} dark size={40} />{[0, 1, 2].map((i) => <div key={i} style={{ width: 24 * u, height: 24 * u, borderRadius: i === 2 ? 6 * u : 99, border: `${3 * u}px solid #fff` }} />)}</div>
        </div>
      )}
      {/* the window */}
      <div style={{ position: 'absolute', left: win.x * u, top: win.y * u, width: win.w * u, height: win.h * u, borderRadius: (os === 'win' ? 18 : 26) * u, overflow: 'hidden', background: dark ? (os === 'win' ? '#202020' : '#242424') : '#FFFFFF', boxShadow: `0 ${30 * u}px ${80 * u}px rgba(0,0,0,0.55)`, border: `${1.5 * u}px solid ${dark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.12)'}` }}>
        <div style={{ height: 76 * u, display: 'flex', alignItems: 'center', padding: `0 ${26 * u}px`, gap: 18 * u, background: os === 'win' ? '#1b1b1b' : os === 'mac' ? '#ECECEC' : '#303030', color: dark ? '#ddd' : '#333', fontFamily: FONT_BN, fontWeight: 500, fontSize: 30 * u, borderBottom: `${1.5 * u}px solid ${dark ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)'}` }}>
          {os === 'mac' && <div style={{ display: 'flex', gap: 14 * u }}>{['#FF5F57', '#FEBC2E', '#28C840'].map((c) => <div key={c} style={{ width: 24 * u, height: 24 * u, borderRadius: 99, background: c }} />)}</div>}
          {os === 'win' && <div style={{ width: 30 * u, height: 34 * u, borderRadius: 4 * u, background: '#4CC2FF' }} />}
          <div style={{ flex: 1, textAlign: os === 'win' ? 'left' : 'center', marginRight: os === 'mac' ? 100 * u : 0 }}>{title}</div>
          {os === 'win' && <div style={{ display: 'flex', gap: 40 * u, fontFamily: FONT_LATIN, fontSize: 30 * u, color: '#ccc' }}><span>—</span><span>▢</span><span>✕</span></div>}
          {os === 'linux' && <div style={{ width: 40 * u, height: 40 * u, borderRadius: 99, background: 'rgba(255,255,255,0.1)', display: 'grid', placeItems: 'center', color: '#ddd', fontFamily: FONT_LATIN, fontSize: 24 * u }}>✕</div>}
        </div>
        {os === 'win' && <div style={{ display: 'flex', gap: 36 * u, padding: `${10 * u}px ${30 * u}px`, fontFamily: FONT_LATIN, fontSize: 26 * u, color: '#bbb', background: '#202020', borderBottom: `${1.5 * u}px solid rgba(255,255,255,0.06)` }}><span>File</span><span>Edit</span><span>View</span></div>}
        <div style={{ padding: `${50 * u}px ${56 * u}px` }}>
          <Written done={t.done} live={t.live} ink={ink} accent={accent} size={74} />
          <div style={{ marginTop: 24 * u }}><Cands word={t.live} picked={t.picked} dark={dark} /></div>
        </div>
        {/* Linux: the note at the caret when you switch with double Shift */}
        {os === 'linux' && note > 0 && (
          <div style={{ position: 'absolute', left: 60 * u, top: 150 * u, padding: `${10 * u}px ${22 * u}px`, borderRadius: 14 * u, background: 'rgba(20,20,22,0.96)', border: `${1.5 * u}px solid rgba(255,255,255,0.15)`, fontFamily: FONT_BN, fontWeight: 600, fontSize: 34 * u, color: '#fff', opacity: note, transform: `translateY(${(1 - note) * 10 * u}px)` }}>এ · বাংলা</div>
        )}
      </div>
      {/* Windows taskbar */}
      {os === 'win' && (
        <div style={{ position: 'absolute', left: 0, right: 0, bottom: 0, height: 96 * u, display: 'flex', alignItems: 'center', padding: `0 ${26 * u}px`, background: 'rgba(24,24,28,0.86)', backdropFilter: `blur(${20 * u}px)`, color: '#fff', fontFamily: FONT_LATIN }}>
          <div style={{ flex: 1 }} />
          <div style={{ display: 'flex', gap: 22 * u, alignItems: 'center' }}>
            <div style={{ color: '#4CC2FF' }}><OsIcon kind="win" size={46 * u} /></div>
            {['#ffd34e', '#4CC2FF', '#7ee787'].map((c) => <div key={c} style={{ width: 46 * u, height: 46 * u, borderRadius: 10 * u, background: c, opacity: 0.85 }} />)}
          </div>
          <div style={{ flex: 1, display: 'flex', justifyContent: 'flex-end', alignItems: 'center', gap: 20 * u }}>
            <Img src={staticFile('assets/logo_64.png')} style={{ width: 40 * u, height: 40 * u, borderRadius: 8 * u }} />
            <LangBadge on={on} dark size={42} />
            <div style={{ textAlign: 'right', fontSize: 22 * u, lineHeight: 1.25 }}>{CLOCK.win[0]}<br />{CLOCK.win[1]}</div>
          </div>
        </div>
      )}
      {/* macOS dock */}
      {os === 'mac' && (
        <div style={{ position: 'absolute', left: '50%', bottom: 26 * u, transform: 'translateX(-50%)', display: 'flex', gap: 18 * u, padding: `${14 * u}px ${20 * u}px`, borderRadius: 30 * u, background: 'rgba(255,255,255,0.3)', backdropFilter: `blur(${20 * u}px)`, border: `${1.5 * u}px solid rgba(255,255,255,0.4)` }}>
          {['#3a8ef6', '#f5f5f5', '#34c759', '#ff9f0a'].map((c) => <div key={c} style={{ width: 92 * u, height: 92 * u, borderRadius: 22 * u, background: c }} />)}
          <Img src={staticFile('assets/logo_128.png')} style={{ width: 92 * u, height: 92 * u, borderRadius: 22 * u }} />
        </div>
      )}
    </div>
  );
};

const HEAD: Record<Os, { bn: string; en: string; tag?: string }> = {
  win: { bn: 'আজ থেকে Windows-এ', en: 'Out today on Windows 10 and 11' },
  mac: { bn: 'এবার Mac-এও!', en: 'New: Ekushe Next is on the Mac too', tag: 'নতুন' },
  linux: { bn: 'Linux-এ, সব ডেস্কটপে', en: 'Ubuntu, Fedora · GNOME, KDE and more' },
};
const EN_LINE: Record<Os, string> = {
  win: 'Welcome, everyone! From today, may writing Bangla be easier and more beautiful.',
  mac: 'Dear Bangla-speaking friends, warm wishes to you.',
  linux: 'Whatever your computer, write Bangla just the way you like.',
};

const ComputerScene: React.FC<{ os: Os; len: number }> = ({ os, len }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const P = plan(L[os].tokens as Tok[], L[os].start);
  const t = typing(f, P);
  const fin = endOf(P);
  const inT = spring({ frame: f - 4, fps: 60, config: { damping: 16, stiffness: 90 } });
  const scr = spring({ frame: f - 22, fps: 60, config: { damping: 17, stiffness: 85 } });
  const out = ease(f, len - 24, len);
  const glow = interpolate(f, [fin, fin + 10, fin + 60], [0, 1, 0.45], clamp);
  const on = f >= L[os].start - 40;
  const note = Math.min(ease(f, L[os].start - 40, L[os].start - 28), 1 - ease(f, L[os].start + 4, L[os].start + 16));
  const head = HEAD[os];
  const tagPop = spring({ frame: f - 40, fps: 60, config: { damping: 9, stiffness: 180 } });
  return (
    <AbsoluteFill style={{ opacity: 1 - out, transform: `translateY(${-out * 50 * u}px)`, filter: `blur(${out * 10 * u}px)` }}>
      <div style={{ position: 'absolute', top: 250 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', opacity: inT, transform: `translateY(${(1 - inT) * 60 * u}px)` }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 22 * u, color: C.white }}>
          <OsIcon kind={os} size={78 * u} />
          <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 100 * u }}>{head.bn}</span>
          {head.tag && <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 44 * u, padding: `${6 * u}px ${22 * u}px`, borderRadius: 99, background: C.mustard, color: '#2A1C00', transform: `scale(${tagPop}) rotate(${(1 - tagPop) * 30 - 6}deg)`, boxShadow: `0 0 ${40 * u}px rgba(233,185,73,0.6)` }}>{head.tag}</span>}
        </div>
        <Sub size={36} style={{ marginTop: 14 * u }}>{head.en}</Sub>
      </div>
      <div style={{ position: 'absolute', top: 560 * u, left: 0, right: 0, display: 'flex', justifyContent: 'center', opacity: scr, transform: `translateY(${(1 - scr) * 90 * u}px) scale(${0.94 + scr * 0.06})` }}>
        <Desktop os={os} t={t} on={on} glow={glow} note={os === 'linux' ? note : 0} />
      </div>
      <div style={{ position: 'absolute', top: 1860 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', padding: `0 ${90 * u}px` }}>
        <div style={{ opacity: t.roman ? 1 : 0 }}><KeyStrip roman={t.roman} n={t.n} /></div>
        <Sub size={36} o={ease(f, fin, fin + 25)} style={{ marginTop: -60 * u, lineHeight: 1.4 }}>{EN_LINE[os]}</Sub>
      </div>
      {os === 'linux' && <Sub size={30} o={note} style={{ position: 'absolute', top: 470 * u, left: 0, right: 0 }}>double-tap Shift to switch</Sub>}
      <Burst at={fin} y={0.36} count={34} seed={`c${os}`} spread={0.5} colors={[C.greenBright, '#ffffff', C.mustard, C.red]} />
      <Sweep at={fin + 4} dur={46} strength={0.12} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 56–66 s: the phones, coming soon ───────────────────────── */
const Phone: React.FC<{ kind: 'android' | 'ios'; start: number }> = ({ kind, start }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const P = plan(L.phone.tokens as Tok[], start);
  const t = typing(f, P);
  const fin = endOf(P);
  const sent = f >= fin + 16;
  const fly = spring({ frame: f - fin - 16, fps: 60, config: { damping: 14, stiffness: 120 } });
  const W = 600, H = 1240, B = 18;
  const sw = W - B * 2;
  const kb = kind === 'android' ? { src: 'android/kb-ekushe.jpg', h: sw * (708 / 1080) } : { src: 'ios/ekushe.jpg', h: sw * (798 / 1170) };
  const ios = kind === 'ios';
  const ribbon = spring({ frame: f - 60, fps: 60, config: { damping: 12 } });
  return (
    <div style={{ position: 'relative', width: W * u, height: H * u, borderRadius: 92 * u, background: '#0a0a0a', padding: B * u, boxShadow: `0 ${40 * u}px ${110 * u}px rgba(0,0,0,0.8), inset 0 0 0 ${3 * u}px rgba(255,255,255,0.14)` }}>
      <div style={{ position: 'relative', width: '100%', height: '100%', borderRadius: 76 * u, overflow: 'hidden', background: ios ? '#F2F2F7' : '#101614', display: 'flex', flexDirection: 'column' }}>
        {/* status bar and camera */}
        <div style={{ height: 70 * u, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: `0 ${48 * u}px`, fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 24 * u, color: ios ? '#111' : '#eee' }}><span>10:30</span><span>●●● ▮</span></div>
        {ios ? <div style={{ position: 'absolute', top: 16 * u, left: '50%', transform: 'translateX(-50%)', width: 150 * u, height: 42 * u, borderRadius: 99, background: '#000' }} /> : <div style={{ position: 'absolute', top: 22 * u, left: '50%', transform: 'translateX(-50%)', width: 26 * u, height: 26 * u, borderRadius: 99, background: '#000' }} />}
        {/* chat header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 * u, padding: `${12 * u}px ${30 * u}px`, borderBottom: `${1.5 * u}px solid ${ios ? 'rgba(0,0,0,0.08)' : 'rgba(255,255,255,0.08)'}` }}>
          <div style={{ width: 62 * u, height: 62 * u, borderRadius: 99, background: `linear-gradient(135deg, ${C.coral}, ${C.red})`, display: 'grid', placeItems: 'center', fontFamily: FONT_BN, fontWeight: 700, fontSize: 30 * u, color: '#fff' }}>ব</div>
          <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 32 * u, color: ios ? '#111' : '#eee' }}>বন্ধু</div>
        </div>
        {/* messages */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'flex-end', gap: 16 * u, padding: `${20 * u}px ${24 * u}px` }}>
          <div style={{ alignSelf: 'flex-start', maxWidth: '78%', padding: `${14 * u}px ${24 * u}px`, borderRadius: 30 * u, background: ios ? '#E5E5EA' : '#25302c', fontFamily: FONT_BN, fontSize: 36 * u, color: ios ? '#111' : '#eee' }}>ফোনে কবে আসছে?</div>
          {sent && <div style={{ alignSelf: 'flex-end', maxWidth: '84%', padding: `${14 * u}px ${24 * u}px`, borderRadius: 30 * u, background: ios ? '#0A84FF' : '#0B6B4A', fontFamily: FONT_BN, fontSize: 36 * u, lineHeight: 1.4, color: '#fff', transform: `translateY(${(1 - fly) * 60 * u}px) scale(${0.85 + fly * 0.15})`, opacity: fly, transformOrigin: 'right bottom' }}>{P.map((w) => w.b).join(' ')}</div>}
        </div>
        {/* input */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 * u, padding: `${12 * u}px ${20 * u}px` }}>
          <div style={{ flex: 1, minHeight: 74 * u, borderRadius: 40 * u, padding: `${8 * u}px ${24 * u}px`, background: ios ? '#fff' : '#1d2724', border: `${1.5 * u}px solid ${ios ? 'rgba(0,0,0,0.12)' : 'rgba(255,255,255,0.12)'}`, display: 'flex', alignItems: 'center' }}>
            {!sent && <Written done={t.done} live={t.live} ink={ios ? '#111' : '#eee'} accent={ios ? '#0B8A5C' : C.greenBright} size={34} />}
          </div>
          <div style={{ width: 70 * u, height: 70 * u, borderRadius: 99, background: ios ? '#0A84FF' : C.greenBright, display: 'grid', placeItems: 'center', color: '#fff', fontSize: 34 * u, transform: `scale(${f >= fin && f < fin + 20 ? 0.88 : 1})` }}>➤</div>
        </div>
        <Img src={staticFile(kb.src)} style={{ width: sw * u, height: kb.h * u, display: 'block' }} />
      </div>
      {/* coming soon ribbon */}
      <div style={{ position: 'absolute', top: 70 * u, right: -36 * u, transform: `rotate(${12 - (1 - ribbon) * 20}deg) scale(${ribbon})`, padding: `${12 * u}px ${30 * u}px`, borderRadius: 20 * u, background: C.mustard, color: '#2A1C00', fontFamily: FONT_BN, fontWeight: 700, fontSize: 44 * u, boxShadow: `0 ${14 * u}px ${40 * u}px rgba(0,0,0,0.5), 0 0 ${40 * u}px rgba(233,185,73,0.5)` }}>শীঘ্রই</div>
    </div>
  );
};

const PhonesScene: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const len = R.party - R.phones;
  const inT = spring({ frame: f - 4, fps: 60, config: { damping: 16, stiffness: 90 } });
  const a = spring({ frame: f - 24, fps: 60, config: { damping: 15, stiffness: 80 } });
  const b = spring({ frame: f - 44, fps: 60, config: { damping: 15, stiffness: 80 } });
  const out = ease(f, len - 24, len);
  const fin = endOf(plan(L.phone.tokens as Tok[], L.phone.ios));
  const bar = ease(f, 80, 520) * 0.82;
  return (
    <AbsoluteFill style={{ opacity: 1 - out, filter: `blur(${out * 10 * u}px)` }}>
      <div style={{ position: 'absolute', top: 230 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', opacity: inT, transform: `translateY(${(1 - inT) * 60 * u}px)` }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 20 * u, color: C.white }}>
          <OsIcon kind="android" size={72 * u} /><OsIcon kind="ios" size={72 * u} />
          <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 92 * u }}>ফোনেও আসছে</span>
        </div>
        <Sub size={36} style={{ marginTop: 12 * u }}>Android and iPhone are coming soon</Sub>
      </div>
      <div style={{ position: 'absolute', top: 520 * u, left: 0, right: 0, display: 'flex', justifyContent: 'center', gap: 60 * u }}>
        <div style={{ opacity: a, transform: `translateY(${(1 - a) * 120 * u}px) rotate(${-4 + (1 - a) * -6}deg)` }}><Phone kind="android" start={L.phone.android} /></div>
        <div style={{ opacity: b, transform: `translateY(${(1 - b) * 120 * u + 40 * u}px) rotate(${4 + (1 - b) * 6}deg)` }}><Phone kind="ios" start={L.phone.ios} /></div>
      </div>
      <div style={{ position: 'absolute', top: 1900 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 52 * u, color: C.mustard }}>তৈরি হচ্ছে…</div>
        <div style={{ marginTop: 18 * u, width: 820 * u, height: 22 * u, borderRadius: 99, background: 'rgba(255,255,255,0.12)', overflow: 'hidden' }}>
          <div style={{ width: `${bar * 100}%`, height: '100%', borderRadius: 99, background: `linear-gradient(90deg, ${C.mustard}, ${C.greenBright})`, boxShadow: `0 0 ${24 * u}px rgba(47,209,142,0.6)` }} />
        </div>
        <Sub size={34} o={ease(f, fin + 20, fin + 45)} style={{ marginTop: 20 * u }}>“Coming to your phone too, wait a little.”</Sub>
      </div>
    </AbsoluteFill>
  );
};

/* ───────────────────────── 66–76 s: celebration ───────────────────────── */
const LETTERS = graphemes('অআইঈউঊঋএঐওঔকখগঘঙচছজঝঞটঠডঢণতথদধনপফবভমযরলশষসহ');
/** A rocket from the bottom that bursts into Bangla letters */
const LetterFirework: React.FC<{ at: number; x: number; y: number; seed: string; color: string }> = ({ at, x, y, seed, color }) => {
  const f = useCurrentFrame() - at;
  const { width, height, u } = useLayout();
  if (f < 0 || f > 130) return null;
  const up = 34;
  if (f < up) {
    const p = ease(f, 0, up);
    return <div style={{ position: 'absolute', left: width * x - 5 * u, top: interpolate(p, [0, 1], [height + 20, height * y]), width: 10 * u, height: 60 * u, borderRadius: 9, background: `linear-gradient(180deg, #fff, ${color}00)`, boxShadow: `0 0 ${20 * u}px ${color}` }} />;
  }
  const g = f - up;
  const t = ease(g, 0, 70);
  return (
    <>
      <div style={{ position: 'absolute', left: width * x - 140 * u, top: height * y - 140 * u, width: 280 * u, height: 280 * u, borderRadius: '50%', background: `radial-gradient(circle, #fff 0%, ${color} 30%, transparent 70%)`, opacity: interpolate(g, [0, 4, 20], [0, 0.9, 0], clamp) }} />
      {Array.from({ length: 16 }).map((_, i) => {
        const a = (i / 16) * Math.PI * 2 + random(`${seed}a${i}`) * 0.3;
        const d = (300 + random(`${seed}d${i}`) * 160) * u * t;
        return (
          <div key={i} style={{ position: 'absolute', left: width * x + Math.cos(a) * d, top: height * y + Math.sin(a) * d + g * g * 0.03 * u, transform: 'translate(-50%, -50%)', fontFamily: FONT_BN, fontWeight: 700, fontSize: (44 + random(`${seed}s${i}`) * 30) * u, color: i % 3 === 0 ? '#fff' : color, opacity: interpolate(g, [0, 60, 96], [1, 0.9, 0], clamp), textShadow: `0 0 ${24 * u}px ${color}` }}>{LETTERS[Math.floor(random(`${seed}l${i}`) * LETTERS.length)]}</div>
        );
      })}
    </>
  );
};

const FIREWORKS = [
  { at: 180, x: 0.5, y: 0.2, c: C.red }, { at: 200, x: 0.22, y: 0.3, c: C.greenBright }, { at: 214, x: 0.78, y: 0.26, c: C.mustard },
  { at: 262, x: 0.3, y: 0.14, c: C.coral }, { at: 290, x: 0.72, y: 0.12, c: C.greenBright }, { at: 330, x: 0.5, y: 0.3, c: C.mustard },
  { at: 372, x: 0.18, y: 0.2, c: C.red }, { at: 396, x: 0.82, y: 0.34, c: C.coral }, { at: 440, x: 0.5, y: 0.16, c: C.greenBright },
];
const BUBBLES = ['অভিনন্দন!', 'অবশেষে!', 'আলহামদুলিল্লাহ', 'শুভেচ্ছা রইল', 'স্বাগতম', 'অপেক্ষা শেষ', 'অভিনন্দন!', 'শুভেচ্ছা রইল'];
export const PARTY = { count: [0, 60, 120], boom: 180, flip: 250, bubbles: 290 } as const;

const FlipCap: React.FC<{ from: string; to: string; at: number }> = ({ from, to, at }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const p = interpolate(f, [at, at + 18], [0, 180], clamp);
  const pop = spring({ frame: f - at - 18, fps: 60, config: { damping: 9, stiffness: 200 } });
  const back = p > 90;
  return (
    <div style={{ perspective: 800 * u }}>
      <div style={{ minWidth: 150 * u, height: 170 * u, padding: `0 ${24 * u}px`, display: 'grid', placeItems: 'center', borderRadius: 30 * u, background: back ? C.greenBright : 'rgba(255,255,255,0.12)', border: `${3 * u}px solid ${back ? C.greenBright : 'rgba(255,255,255,0.3)'}`, boxShadow: back ? `0 0 ${60 * u * pop}px rgba(47,209,142,0.7), 0 ${10 * u}px 0 #0B6B4A` : `0 ${10 * u}px 0 rgba(0,0,0,0.5)`, transform: `rotateX(${back ? p - 180 : p}deg) scale(${back ? 1 + (1 - pop) * 0.1 : 1})`, color: back ? '#03170F' : C.white, fontFamily: back ? FONT_BN : MONO, fontWeight: 700, fontSize: (back ? 96 : 76) * u }}>{back ? to : from}</div>
    </div>
  );
};

const PartyScene: React.FC = () => {
  const f = useCurrentFrame();
  const { width, height, u } = useLayout();
  const len = R.end - R.party;
  const out = ease(f, len - 24, len);
  const caps: [string, string][] = [['e', 'এ'], ['ku', 'কু'], ['she', 'শে'], ['next', 'নেক্সট']];
  return (
    <AbsoluteFill style={{ opacity: 1 - out }}>
      {/* countdown */}
      {PARTY.count.map((at, i) => {
        const g = f - at;
        if (g < 0 || g >= 60) return null;
        const p = spring({ frame: g, fps: 60, config: { damping: 10, stiffness: 160 } });
        const fade = 1 - ease(g, 40, 60);
        return (
          <AbsoluteFill key={i} style={{ alignItems: 'center', justifyContent: 'center' }}>
            <div style={{ position: 'absolute', width: 700 * u * (0.6 + ease(g, 0, 60) * 0.8), height: 700 * u * (0.6 + ease(g, 0, 60) * 0.8), borderRadius: '50%', border: `${8 * u}px solid rgba(224,48,63,${0.7 * fade})`, boxShadow: `0 0 ${80 * u}px rgba(224,48,63,${0.5 * fade})` }} />
            <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 520 * u, color: C.white, opacity: fade, transform: `scale(${1.6 - p * 0.6})`, textShadow: `0 0 ${100 * u}px rgba(224,48,63,0.9)` }}>{['৩', '২', '১'][i]}</div>
          </AbsoluteFill>
        );
      })}
      {/* the boom */}
      <AbsoluteFill style={{ background: '#fff', opacity: interpolate(f, [PARTY.boom, PARTY.boom + 3, PARTY.boom + 26], [0, 0.65, 0], clamp) }} />
      {FIREWORKS.map((w, i) => <LetterFirework key={i} at={w.at - 34} x={w.x} y={w.y} seed={`fw${i}`} color={w.c} />)}
      <Burst at={PARTY.boom} y={0.45} count={70} seed="pb1" spread={0.9} />
      <Burst at={PARTY.boom + 60} y={0.5} count={50} seed="pb2" spread={0.8} />
      <Burst at={PARTY.boom + 160} y={0.4} count={50} seed="pb3" spread={0.8} />
      {f >= PARTY.boom && <Petals count={40} from={PARTY.boom} opacity={0.8} />}
      {/* title */}
      {f >= PARTY.boom && (
        <div style={{ position: 'absolute', top: 960 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          {[graphemes('একুশে নেক্সট'), graphemes('এসে গেছে!')].map((line, li) => (
            <div key={li} style={{ display: 'flex', justifyContent: 'center' }}>
              {line.map((g, j) => {
                const i = li * 12 + j;
                const t = spring({ frame: f - PARTY.boom - 6 - i * 2, fps: 60, config: { damping: 9, stiffness: 170 } });
                const bob = Math.sin((f + i * 8) / 14) * 6 * u;
                return <span key={j} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (li ? 150 : 140) * u, lineHeight: 1.25, color: li ? C.greenBright : C.white, whiteSpace: 'pre', display: 'inline-block', opacity: Math.min(1, t * 1.5), transform: `translateY(${(1 - t) * 120 * u + bob}px) scale(${0.4 + t * 0.6})`, textShadow: `0 0 ${60 * u}px ${li ? 'rgba(47,209,142,0.7)' : 'rgba(224,48,63,0.7)'}` }}>{g}</span>;
              })}
            </div>
          ))}
          <Sub size={38} o={ease(f, PARTY.boom + 40, PARTY.boom + 70)} style={{ marginTop: 14 * u }}>Ekushe Next is here!</Sub>
          <div style={{ display: 'flex', gap: 22 * u, marginTop: 70 * u, opacity: ease(f, PARTY.flip - 20, PARTY.flip) }}>
            {caps.map(([a, b], i) => <FlipCap key={i} from={a} to={b} at={PARTY.flip + i * 14} />)}
          </div>
          <Sub size={32} o={ease(f, PARTY.flip + 70, PARTY.flip + 95)} style={{ marginTop: 26 * u }}>e·ku·she next → একুশে নেক্সট</Sub>
        </div>
      )}
      {/* wishes floating up */}
      {BUBBLES.map((w, i) => {
        const at = PARTY.bubbles + i * 30;
        const g = f - at;
        if (g < 0) return null;
        const x = (i % 2 ? 0.7 : 0.3) + (random(`bx${i}`) - 0.5) * 0.25;
        const y = height + 100 * u - g * (5.5 + random(`bv${i}`) * 2) * u;
        const p = spring({ frame: g, fps: 60, config: { damping: 10 } });
        return <div key={i} style={{ position: 'absolute', left: width * x + Math.sin(g / 24 + i) * 40 * u, top: y, transform: `translate(-50%, 0) scale(${p})`, padding: `${16 * u}px ${34 * u}px`, borderRadius: 60 * u, background: ['rgba(224,48,63,0.88)', 'rgba(11,107,74,0.92)', 'rgba(233,185,73,0.95)'][i % 3], color: i % 3 === 2 ? '#2A1C00' : '#fff', fontFamily: FONT_BN, fontWeight: 700, fontSize: 52 * u, whiteSpace: 'nowrap', boxShadow: `0 ${14 * u}px ${40 * u}px rgba(0,0,0,0.4)`, opacity: interpolate(y, [-200 * u, 400 * u], [0, 1], clamp) }}>{w}</div>;
      })}
    </AbsoluteFill>
  );
};

/* ───────────────────────── 76–84 s: download today ───────────────────────── */
export const END = { logo: 6, name: 22, cta: 70, pills: 100, later: 150, web: 200 } as const;
const Ending: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const pop = spring({ frame: f - END.logo, fps: 60, config: { damping: 11, mass: 0.8 } });
  const cta = spring({ frame: f - END.cta, fps: 60, config: { damping: 12 } });
  const later = ease(f, END.later, END.later + 30);
  const name = graphemes('একুশে নেক্সট');
  const beat = 1 + Math.max(0, Math.sin((f - END.cta) / 9)) * 0.025 * (f > END.cta + 30 ? 1 : 0);
  return (
    <AbsoluteFill style={{ alignItems: 'center', flexDirection: 'column', paddingTop: 300 * u, opacity: interpolate(f, [0, 14], [0, 1], clamp) }}>
      <Img src={staticFile('assets/logo_512.png')} style={{ width: 280 * u, height: 280 * u, borderRadius: 64 * u, transform: `scale(${0.5 + pop * 0.5}) rotate(${(1 - pop) * -12}deg)`, opacity: Math.min(1, pop * 1.4), boxShadow: `0 ${30 * u}px ${90 * u}px rgba(0,0,0,0.6), 0 0 ${150 * u}px rgba(224,48,63,0.55)` }} />
      <div style={{ display: 'flex', marginTop: 34 * u }}>
        {name.map((g, i) => {
          const t = spring({ frame: f - END.name - i * 3, fps: 60, config: { damping: 14 } });
          return <span key={i} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 150 * u, color: C.white, whiteSpace: 'pre', display: 'inline-block', opacity: t, transform: `translateY(${(1 - t) * 60 * u}px)`, textShadow: `0 0 ${60 * u}px rgba(224,48,63,0.5)` }}>{g}</span>;
        })}
      </div>
      <div style={{ marginTop: 40 * u, padding: `${22 * u}px ${60 * u}px`, borderRadius: 99, background: C.greenBright, color: '#03170F', fontFamily: FONT_BN, fontWeight: 700, fontSize: 84 * u, opacity: cta, transform: `scale(${(0.7 + cta * 0.3) * beat})`, boxShadow: `0 0 ${90 * u}px rgba(47,209,142,0.6)` }}>ডাউনলোড করুন আজই</div>
      <div style={{ display: 'flex', gap: 22 * u, marginTop: 34 * u }}>
        <OsPill k="win" at={END.pills} size={38} /><OsPill k="mac" at={END.pills + 10} size={38} /><OsPill k="linux" at={END.pills + 20} size={38} />
      </div>
      <Sub size={34} o={ease(f, END.pills + 20, END.pills + 45)} style={{ marginTop: 16 * u }}>Download today for Windows, macOS and Linux</Sub>
      <div style={{ marginTop: 34 * u, opacity: later, textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 52 * u, color: C.mustard }}>Android আর iPhone শীঘ্রই</div>
        <Sub size={30} style={{ marginTop: 8 * u }}>Android and iPhone coming soon</Sub>
      </div>
      <div style={{ marginTop: 50 * u }}><AddressPill kind="web" text="www.ekushenext.com" at={END.web} /></div>
      <Burst at={END.logo} y={0.2} count={44} seed="le" spread={0.7} />
      <Burst at={END.cta} y={0.36} count={36} seed="lc" spread={0.6} colors={[C.greenBright, '#ffffff', C.mustard]} />
      <Petals count={18} opacity={0.5} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── the film ───────────────────────── */
export const LaunchReveal: React.FC = () => (
  <AbsoluteFill style={{ background: '#010503' }}>
    <Fonts />
    <Scenery />
    <Sequence from={R.dark} durationInFrames={R.date}><Dark /></Sequence>
    <Sequence from={R.date} durationInFrames={R.name - R.date}><DateScene /></Sequence>
    <Sequence from={R.name} durationInFrames={R.win - R.name}><NameScene /></Sequence>
    <Sequence from={R.win} durationInFrames={R.mac - R.win}><ComputerScene os="win" len={R.mac - R.win} /></Sequence>
    <Sequence from={R.mac} durationInFrames={R.linux - R.mac}><ComputerScene os="mac" len={R.linux - R.mac} /></Sequence>
    <Sequence from={R.linux} durationInFrames={R.phones - R.linux}><ComputerScene os="linux" len={R.phones - R.linux} /></Sequence>
    <Sequence from={R.phones} durationInFrames={R.party - R.phones}><PhonesScene /></Sequence>
    <Sequence from={R.party} durationInFrames={R.end - R.party}><PartyScene /></Sequence>
    <Sequence from={R.end} durationInFrames={R.total - R.end}><Ending /></Sequence>
    <Audio src={staticFile('music/launch_reveal.wav')} />
  </AbsoluteFill>
);
