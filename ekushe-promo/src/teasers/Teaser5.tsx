import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, random, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { Burst, END_LEN, EndCard, Streaks, Sub, Sweep } from './common';

/** Teaser 5 of 5: "আপনার লেখা, শুধু আপনার" and the first full look at the app. 22 s at 60 fps.
 * Privacy (connections reaching for a lock are cut one by one), then the whole of Ekushe Studio
 * for the first time (real screenshots, page by page, dark then light), the platforms, the ending. */
export const T5 = { lock: 0, title: 330, look: 430, platforms: 1000, end: 1140 } as const;
export const TEASER5_FRAMES = T5.end + END_LEN;
export const CUT_AT = [60, 95, 130, 165, 200, 235]; // each connection is cut (the music snaps on each)
export const PAGE_AT = (i: number) => T5.look + 40 + i * 88; // each Studio page turns in
export const THEME_AT = PAGE_AT(3) + 40;

const Lock: React.FC<{ size: number; shut: number }> = ({ size, shut }) => (
  <svg width={size} height={size * 1.2} viewBox="0 0 100 120" style={{ overflow: 'visible' }}>
    <defs><linearGradient id="lockg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stopColor="#5BE3AA" /><stop offset="1" stopColor="#0B6B4A" /></linearGradient></defs>
    <path d={`M28 ${54 - (1 - shut) * 14} V36 a22 22 0 0 1 44 0 V${54 - (1 - shut) * 2}`} fill="none" stroke="#EEF4F1" strokeWidth={10} strokeLinecap="round" />
    <rect x={14} y={52} width={72} height={60} rx={14} fill="url(#lockg)" stroke="rgba(255,255,255,0.5)" strokeWidth={2} />
    <circle cx={50} cy={78} r={8} fill="#03170F" /><rect x={46} y={80} width={8} height={16} rx={4} fill="#03170F" />
  </svg>
);

/** 0–5.5 s: lines reach in from the outside world; one by one they are cut. */
const LINES = ['cloud', 'ad', 'track', 'account', 'server', 'upload'];
const LABELS: [number, string, string][] = [
  [0, 'ইন্টারনেট লাগে না', 'No internet needed'],
  [115, 'অ্যাকাউন্ট লাগে না', 'No account'],
  [215, 'কোনো ট্র্যাকিং নেই', 'No tracking'],
];
const Connections: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width, height } = useLayout();
  const cx = width / 2, cy = height * (V ? 0.42 : 0.44);
  const R = Math.min(width, height) * (V ? 0.42 : 0.4);
  const label = [...LABELS].reverse().find(([at]) => f >= at)!;
  const lt = spring({ frame: f - label[0], fps: 60, config: { damping: 14 } });
  return (
    <AbsoluteFill style={{ opacity: interpolate(f, [312, 330], [1, 0], clamp) }}>
      <svg width={width} height={height} style={{ position: 'absolute', inset: 0 }}>
        {LINES.map((k, i) => {
          const a = (i / LINES.length) * Math.PI * 2 - Math.PI / 2 + 0.3;
          const x0 = cx + Math.cos(a) * R * 1.25, y0 = cy + Math.sin(a) * R * (V ? 1.5 : 1.05);
          const grow = ease(f, 6 + i * 6, 50 + i * 6);
          const cut = f >= CUT_AT[i];
          const back = ease(f, CUT_AT[i], CUT_AT[i] + 24);
          const reach = grow * (1 - back * 0.7);
          const x1 = x0 + (cx - x0) * reach * 0.82, y1 = y0 + (cy - y0) * reach * 0.82;
          return (
            <g key={k} opacity={cut ? 1 - back * 0.85 : 1}>
              <line x1={x0} y1={y0} x2={x1} y2={y1} stroke={cut ? '#FF5A66' : 'rgba(234,243,238,0.55)'} strokeWidth={5 * u} strokeDasharray={`${18 * u} ${14 * u}`} strokeDashoffset={-f * 3 * u} />
              <circle cx={x0} cy={y0} r={26 * u} fill={cut ? 'rgba(255,90,102,0.25)' : 'rgba(255,255,255,0.12)'} stroke={cut ? '#FF5A66' : 'rgba(255,255,255,0.5)'} strokeWidth={3 * u} />
              {cut && <g stroke="#FF5A66" strokeWidth={6 * u} strokeLinecap="round" opacity={1 - back}><line x1={x1 - 22 * u} y1={y1 - 22 * u} x2={x1 + 22 * u} y2={y1 + 22 * u} /><line x1={x1 + 22 * u} y1={y1 - 22 * u} x2={x1 - 22 * u} y2={y1 + 22 * u} /></g>}
            </g>
          );
        })}
      </svg>
      <div style={{ position: 'absolute', left: cx, top: cy, transform: `translate(-50%,-50%) scale(${1 + Math.sin(f / 10) * 0.02})`, filter: `drop-shadow(0 0 ${50 * u}px rgba(47,209,142,0.6))` }}>
        <Lock size={(V ? 280 : 240) * u} shut={ease(f, 240, 270)} />
      </div>
      <div style={{ position: 'absolute', left: 0, right: 0, top: cy + (V ? 520 : 330) * u, textAlign: 'center', opacity: lt, transform: `translateY(${(1 - lt) * 40 * u}px)` }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 96 : 84) * u, color: C.white }}>{label[1]}</div>
        <Sub style={{ marginTop: 6 * u }}>{label[2]}</Sub>
      </div>
      {CUT_AT.map((at, i) => {
        const a = (i / LINES.length) * Math.PI * 2 - Math.PI / 2 + 0.3;
        const x0 = cx + Math.cos(a) * R * 1.25, y0 = cy + Math.sin(a) * R * (V ? 1.5 : 1.05);
        return <Burst key={i} at={at} x={(x0 + (cx - x0) * 0.82) / width} y={(y0 + (cy - y0) * 0.82) / height} count={12} spread={0.12} seed={`cut${i}`} colors={['#FF5A66', '#ffffff', C.coral]} />;
      })}
    </AbsoluteFill>
  );
};

/** 5.5–7.2 s: the promise. */
const Title: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const t = spring({ frame: f - 4, fps: 60, config: { damping: 10, stiffness: 170 } });
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [84, 100], [1, 0], clamp) }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 150 : 140) * u, lineHeight: 1.15, textAlign: 'center', color: C.white, transform: `scale(${1.8 - t * 0.8})`, opacity: Math.min(1, t * 1.5), textShadow: `0 0 ${70 * u}px rgba(47,209,142,0.5)` }}>আপনার লেখা,<br />শুধু আপনার</div>
      <Sub o={ease(f, 24, 44)} size={40} style={{ marginTop: 22 * u }}>What you write stays yours</Sub>
    </AbsoluteFill>
  );
};

/** 7.2–16.7 s: the first full look, page by page. */
const PAGES: [string, string, string][] = [
  ['playground', 'লেখার প্যাড', 'Writing pad'],
  ['xray', 'শব্দ এক্স-রে', 'Word X-ray'],
  ['signs', 'যুক্তবর্ণ গাইড', 'Conjunct guide'],
  ['converter', 'বিজয় ⇄ ইউনিকোড', 'Bijoy ⇄ Unicode'],
  ['spell', 'বানান পরীক্ষা', 'Spell check'],
  ['settings', 'সেটিংস', 'Settings'],
];
const FullLook: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width } = useLayout();
  const G = f + T5.look; // global frame
  const idx = Math.max(0, Math.min(PAGES.length - 1, Math.floor((G - PAGE_AT(0)) / 88)));
  const pageStart = PAGE_AT(idx);
  const flip = spring({ frame: G - pageStart, fps: 60, config: { damping: 16, stiffness: 120 } });
  const theme = G >= THEME_AT ? 'light' : 'dark';
  const enter = spring({ frame: f, fps: 60, config: { damping: 18, stiffness: 70 } });
  const out = ease(f, 552, 570);
  const W = V ? width * 0.94 : 1560 * u;
  const H = W * (1500 / 2400);
  const [key, bnName, enName] = PAGES[idx];
  const prev = idx > 0 ? PAGES[idx - 1][0] : null;
  const prevTheme = PAGE_AT(idx) > THEME_AT ? 'light' : 'dark';
  const pt = spring({ frame: G - pageStart - 8, fps: 60, config: { damping: 14 } });
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out, perspective: 2800 * u }}>
      <div style={{ position: 'absolute', top: (V ? 380 : 50) * u, textAlign: 'center', width: '100%', opacity: ease(f, 10, 30) }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 84 : 72) * u, color: C.white }}>প্রথম পূর্ণ দর্শন: একুশে স্টুডিও</div>
        <Sub style={{ marginTop: 6 * u }}>The first full look: Ekushe Studio</Sub>
      </div>
      <div style={{ position: 'relative', width: W, height: H, marginTop: (V ? 40 : 150) * u, transformStyle: 'preserve-3d', transform: `translateY(${(1 - enter) * 600 * u}px) rotateX(${(1 - enter) * 50 + Math.sin(f / 70) * 2}deg) rotateY(${Math.sin(f / 90) * 4}deg) scale(${0.6 + enter * 0.4})` }}>
        {prev && flip < 0.999 && (
          <Img src={staticFile(`teaser5/${prev}-${prevTheme}.jpg`)} style={{ position: 'absolute', inset: 0, width: W, height: H, borderRadius: 34 * u, opacity: 1 - flip, transform: `translateX(${-flip * 30}%) rotateY(${flip * 35}deg) scale(${1 - flip * 0.15})` }} />
        )}
        <div style={{ position: 'absolute', inset: 0, borderRadius: 34 * u, overflow: 'hidden', opacity: idx === 0 ? 1 : flip, transform: idx === 0 ? 'none' : `translateX(${(1 - flip) * 40}%) rotateY(${(1 - flip) * -40}deg) scale(${0.85 + flip * 0.15})`, boxShadow: `0 0 0 ${2 * u}px rgba(255,255,255,0.16), 0 ${60 * u}px ${140 * u}px -${30 * u}px rgba(0,0,0,0.9), 0 0 ${160 * u}px -${20 * u}px rgba(224,48,63,0.4)` }}>
          <Img src={staticFile(`teaser5/${key}-${theme}.jpg`)} style={{ width: W, height: H, display: 'block' }} />
          <Sweep at={pageStart - T5.look + 6} dur={46} strength={0.14} />
        </div>
      </div>
      <div style={{ marginTop: (V ? 44 : 34) * u, display: 'flex', alignItems: 'center', gap: 16 * u, padding: `${16 * u}px ${32 * u}px`, borderRadius: 999, background: 'rgba(47,209,142,0.14)', border: `${2 * u}px solid rgba(47,209,142,0.5)`, opacity: pt, transform: `translateY(${(1 - pt) * 30 * u}px)` }}>
        <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 56 : 50) * u, color: C.white }}>{bnName}</span>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: 32 * u, color: 'rgba(234,243,238,0.7)' }}>{enName}</span>
      </div>
      <div style={{ position: 'absolute', bottom: (V ? 330 : 70) * u, display: 'flex', gap: 12 * u }}>
        {PAGES.map((p, i) => <div key={p[0]} style={{ width: (i === idx ? 44 : 14) * u, height: 14 * u, borderRadius: 99, background: i === idx ? C.greenBright : 'rgba(255,255,255,0.3)' }} />)}
      </div>
      {G >= THEME_AT && G < THEME_AT + 30 && <AbsoluteFill style={{ background: '#fff', opacity: interpolate(G, [THEME_AT, THEME_AT + 4, THEME_AT + 30], [0, 0.35, 0], clamp) }} />}
    </AbsoluteFill>
  );
};

/** 16.7–19 s: Windows, macOS and Linux, free and open source. */
const OS: { name: string; path: string }[] = [
  { name: 'Windows', path: 'M3 5.6l7.4-1v7H3zm8.4-1.2L21 3v8.6h-9.6zM3 12.6h7.4v7l-7.4-1zm8.4 0H21V21l-9.6-1.4z' },
  { name: 'macOS', path: 'M16.4 12.6c0-2.4 2-3.6 2.1-3.7-1.2-1.7-3-1.9-3.6-2-1.5-.2-3 .9-3.8.9-.8 0-2-.9-3.3-.9-1.7 0-3.3 1-4.2 2.5-1.8 3.1-.5 7.7 1.3 10.2.9 1.2 1.9 2.6 3.2 2.6 1.3-.1 1.8-.8 3.3-.8s2 .8 3.3.8c1.4 0 2.3-1.2 3.1-2.5 1-1.4 1.4-2.8 1.4-2.9 0 0-2.8-1.1-2.8-4.2zM14 5.3c.7-.9 1.2-2 1-3.3-1 0-2.3.7-3 1.6-.6.7-1.2 1.9-1 3.1 1.1.1 2.3-.6 3-1.4z' },
  { name: 'Linux', path: 'M12 2c-2.2 0-3.4 1.9-3.4 4.3 0 1.4.3 2.3-.6 3.7-1 1.6-2.5 3.6-2.5 6.1 0 .6.1 1.1.2 1.6-.9.4-1.7.9-1.7 1.7 0 1.1 1.7 1.3 3.2 1.7 1.2.3 2 .9 3 .9.8 0 1.4-.4 1.8-.9.4.5 1 .9 1.8.9 1 0 1.8-.6 3-.9 1.5-.4 3.2-.6 3.2-1.7 0-.8-.8-1.3-1.7-1.7.1-.5.2-1 .2-1.6 0-2.5-1.5-4.5-2.5-6.1-.9-1.4-.6-2.3-.6-3.7C15.4 3.9 14.2 2 12 2z' },
];
const Platforms: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 50 * u, opacity: interpolate(f, [124, 140], [1, 0], clamp) }}>
      <div style={{ display: 'flex', gap: (V ? 30 : 46) * u }}>
        {OS.map((o, i) => {
          const t = spring({ frame: f - 6 - i * 9, fps: 60, config: { damping: 11, stiffness: 160 } });
          const bob = Math.sin((f + i * 20) / 18) * 8 * u;
          return (
            <div key={o.name} style={{ width: (V ? 380 : 360) * u, padding: `${40 * u}px 0`, borderRadius: 40 * u, background: 'rgba(255,255,255,0.08)', border: `${2 * u}px solid rgba(255,255,255,0.2)`, backdropFilter: `blur(${16 * u}px)`, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 18 * u, opacity: t, transform: `translateY(${(1 - t) * 120 * u + bob}px) rotate(${(1 - t) * (i - 1) * 14}deg) scale(${0.7 + t * 0.3})` }}>
              <svg width={130 * u} height={130 * u} viewBox="0 0 24 24"><path d={o.path} fill={C.white} /></svg>
              <div style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: (V ? 50 : 46) * u, color: C.white }}>{o.name}</div>
            </div>
          );
        })}
      </div>
      <div style={{ textAlign: 'center', opacity: ease(f, 40, 60), transform: `translateY(${(1 - ease(f, 40, 60)) * 30 * u}px)` }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 96 : 86) * u, color: C.white }}>সম্পূর্ণ ফ্রি, ওপেন সোর্স</div>
        <Sub style={{ marginTop: 6 * u }}>Completely free and open source</Sub>
      </div>
    </AbsoluteFill>
  );
};

export const Teaser5: React.FC = () => {
  const f = useCurrentFrame();
  const sun = interpolate(f, [0, T5.title, T5.look, T5.platforms, T5.end], [0.25, 0.7, 0.55, 0.85, 1], clamp);
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Backdrop sun={sun} field={interpolate(f, [0, T5.look, TEASER5_FRAMES], [0.45, 0.75, 1], clamp)} warmth={interpolate(f, [T5.platforms, TEASER5_FRAMES], [0, 0.4], clamp)} />
      <Petals count={24} opacity={interpolate(f, [T5.title, T5.title + 40], [0.2, 0.7], clamp)} />
      <Streaks from={T5.look - 18} to={T5.look + 30} count={40} seed="l" />
      <Streaks from={T5.platforms - 18} to={T5.platforms + 24} count={34} seed="p" />
      <Sequence from={T5.lock} durationInFrames={330}><Connections /></Sequence>
      <Sequence from={T5.title} durationInFrames={100}><Title /></Sequence>
      <Burst at={T5.title + 8} y={0.45} count={40} seed="t" />
      <Sequence from={T5.look} durationInFrames={570}><FullLook /></Sequence>
      <Sequence from={T5.platforms} durationInFrames={140}><Platforms /></Sequence>
      <Sequence from={T5.end}><EndCard /></Sequence>
      <Audio src={staticFile('music/teaser5.wav')} />
    </AbsoluteFill>
  );
};

void random;
