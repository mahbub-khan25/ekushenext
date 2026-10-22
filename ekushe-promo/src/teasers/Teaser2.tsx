import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, random, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { ComingSoon } from './Teaser1';

/** Teaser 2 of 5: "Why is typing Bangla still this hard?" 20 s at 60 fps.
 * Act 1 the hook, act 2 three frustrations, "আর না।", then the real app fixes each one. */
export const TEASER2_FRAMES = 1200;
export const T2 = { hook: 0, p1: 150, p2: 290, p3: 430, slam: 570, reveal: 660, r2: 800, r3: 930, logo: 1060 } as const;
export const CONV_AT = T2.reveal + 40, CONV_EVERY = 4; // Bijoy typed into the converter, 20 keys
export const PAD_AT = T2.r2 + 30, PAD_EVERY = 10;     // "kkhoma", 6 keys
export const FIX_AT = T2.r3 + 62;                      // the spelling fix is clicked

const Sub: React.FC<{ children: React.ReactNode; o?: number; size?: number; style?: React.CSSProperties }> = ({ children, o = 1, size = 34, style }) => {
  const { u } = useLayout();
  return <div style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: size * u, color: 'rgba(234,243,238,0.72)', opacity: o, letterSpacing: '0.01em', textAlign: 'center', ...style }}>{children}</div>;
};
const io = (f: number, a: number, b: number, d = 14) => Math.min(interpolate(f, [a, a + d], [0, 1], clamp), interpolate(f, [b - d, b], [1, 0], clamp));
const graphemes = (s: string) => Array.from(new Intl.Segmenter('bn', { granularity: 'grapheme' }).segment(s), (g) => g.segment);

/** Text that glitches: red and cyan ghosts, sliced bands that jump sideways. */
const Glitch: React.FC<{ text: string; size: number; font: string; color?: string; amount: number; seed: string }> = ({ text, size, font, color = '#fff', amount, seed }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const r = (k: string) => random(`${seed}${k}${Math.floor(f / 2)}`) - 0.5;
  const base: React.CSSProperties = { fontFamily: font, fontWeight: 700, fontSize: size, lineHeight: 1.1, whiteSpace: 'nowrap', letterSpacing: '0.01em' };
  const slices = [0, 1, 2, 3, 4];
  return (
    <div style={{ position: 'relative' }}>
      <div style={{ ...base, color: 'transparent' }}>{text}</div>
      <div style={{ ...base, position: 'absolute', inset: 0, color: '#FF3B4E', opacity: 0.8, transform: `translate(${r('rx') * 26 * u * amount}px, ${r('ry') * 8 * u * amount}px)`, mixBlendMode: 'screen' }}>{text}</div>
      <div style={{ ...base, position: 'absolute', inset: 0, color: '#2FD1E8', opacity: 0.8, transform: `translate(${r('cx') * 26 * u * amount}px, ${r('cy') * 8 * u * amount}px)`, mixBlendMode: 'screen' }}>{text}</div>
      {slices.map((s) => {
        const top = s * 20, jump = random(`${seed}j${s}${Math.floor(f / 3)}`) > 0.72 ? r(`s${s}`) * 60 * u * amount : 0;
        return <div key={s} style={{ ...base, position: 'absolute', inset: 0, color, clipPath: `inset(${top}% 0 ${100 - top - 20}% 0)`, transform: `translateX(${jump}px)` }}>{text}</div>;
      })}
    </div>
  );
};

/** A pain point: caption below a big animated thing, with a whip-in and whip-out. */
const Pain: React.FC<{ len: number; caption: string; en: string; children: React.ReactNode }> = ({ len, caption, en, children }) => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const inn = spring({ frame: f, fps: 60, config: { damping: 15, stiffness: 140 } });
  const out = ease(f, len - 16, len);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: Math.min(1, inn * 1.4) * (1 - out), transform: `translateX(${(1 - inn) * 260 * u - out * 300 * u}px) scale(${0.9 + inn * 0.1 + out * 0.15})`, filter: `blur(${(1 - inn) * 14 * u + out * 18 * u}px)` }}>
      <div style={{ height: (V ? 700 : 520) * u, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>{children}</div>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 76 : 70) * u, color: C.white, textAlign: 'center', padding: `0 ${60 * u}px`, marginTop: 10 * u }}>{caption}</div>
      <Sub style={{ marginTop: 8 * u }}>{en}</Sub>
    </AbsoluteFill>
  );
};

/** 0–2.5 s: a cursor in the dark, and a line typing itself. */
const Hook: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const g = graphemes('বাংলায় লিখতে বসলেন…');
  const n = Math.max(0, Math.min(g.length, Math.floor((f - 22) / 4)));
  const o = interpolate(f, [130, 150], [1, 0], clamp);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: o }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 110 : 104) * u, color: C.white, display: 'flex', alignItems: 'center' }}>
        <span>{g.slice(0, n).join('')}</span>
        <span style={{ display: 'inline-block', width: 8 * u, height: 110 * u, marginLeft: 8 * u, background: C.greenBright, opacity: Math.floor(f / 18) % 2 || n < g.length ? 1 : 0, borderRadius: 4 * u }} />
      </div>
      <Sub o={ease(f, 70, 100)} style={{ marginTop: 16 * u }}>You sit down to write in Bangla…</Sub>
    </AbsoluteFill>
  );
};

/** Pain 1: Bijoy text breaking apart. */
const PainBijoy: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const amount = 0.3 + 0.7 * Math.abs(Math.sin(f / 9));
  return (
    <Pain len={140} caption="পুরনো বিজয় ফাইল খুললেই হিজিবিজি" en="Open an old Bijoy file and it's gibberish">
      <div style={{ transform: `rotate(${(random(`rb${Math.floor(f / 4)}`) - 0.5) * 2}deg)` }}>
        <Glitch text="Avwg evsjvq Mvb MvB" size={(V ? 120 : 150) * u} font={FONT_LATIN} color="#FFC2C7" amount={amount} seed="bij" />
      </div>
    </Pain>
  );
};

/** Pain 2: shortcut soup around a conjunct that keeps coming out wrong. */
const SHORTCUTS = ['k+Sh', 'kkh', 'kSh', 'k+S', 'kS', 'ksh', 'k+Sh?', 'kkh?'];
const PainConjunct: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const wrong = ['কষ', 'ক্‌ষ', 'কশ', 'ক্ষ', 'কস'][Math.floor(f / 12) % 5];
  const R = (V ? 420 : 300) * u;
  return (
    <Pain len={140} caption="প্রতিটি যুক্তবর্ণের শর্টকাট মুখস্থ?" en="Memorising a shortcut for every conjunct?">
      <div style={{ position: 'relative', width: R * 2.6, height: R * 1.6 }}>
        {SHORTCUTS.map((s, i) => {
          const a = (i / SHORTCUTS.length) * Math.PI * 2 + f / 40;
          const t = spring({ frame: f - 6 - i * 4, fps: 60, config: { damping: 12 } });
          const x = Math.cos(a) * R * 1.05 * t, y = Math.sin(a) * R * 0.62 * t;
          return (
            <div key={s + i} style={{ position: 'absolute', left: '50%', top: '50%', transform: `translate(-50%,-50%) translate(${x}px, ${y}px) rotate(${Math.sin(f / 15 + i) * 8}deg)`, padding: `${12 * u}px ${22 * u}px`, borderRadius: 16 * u, background: 'rgba(255,255,255,0.07)', border: `${2 * u}px solid rgba(255,255,255,0.18)`, boxShadow: `inset 0 -${5 * u}px 0 rgba(0,0,0,0.35)`, fontFamily: 'ui-monospace, Menlo, Consolas, monospace', fontWeight: 700, fontSize: 46 * u, color: i % 3 === 0 ? '#FFB4BA' : C.mist, opacity: t }}>
              {s}
            </div>
          );
        })}
        <div style={{ position: 'absolute', left: '50%', top: '50%', transform: `translate(-50%,-55%) scale(${1 + Math.sin(f / 6) * 0.03})`, fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 300 : 230) * u, lineHeight: 1, color: wrong === 'ক্ষ' ? C.white : '#FF6B78', textShadow: `0 0 ${50 * u}px rgba(224,48,63,0.6)` }}>
          {wrong}
          <span style={{ fontFamily: FONT_LATIN, fontSize: 120 * u, color: '#FF6B78', marginLeft: 10 * u, verticalAlign: 'top' }}>?</span>
        </div>
      </div>
    </Pain>
  );
};

/** Pain 3: a misspelling, its red underline wiggling, unnoticed. */
const PainSpelling: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const W = (V ? 900 : 1000) * u;
  const pts = Array.from({ length: 40 }, (_, i) => `${(i / 39) * W},${Math.sin(i * 1.3 + f / 3) * 12 * u + 20 * u}`).join(' ');
  return (
    <Pain len={140} caption="ভুল বানান, কেউ ধরিয়ে দেয় না" en="Spelling mistakes nobody catches">
      <div style={{ position: 'relative', transform: `scale(${1 + ease(f, 0, 140) * 0.06})` }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 220 : 230) * u, color: C.white, lineHeight: 1.15, textAlign: 'center' }}>বাংলাদেস</div>
        <svg width={W} height={40 * u} style={{ display: 'block', margin: '0 auto' }}><polyline points={pts} fill="none" stroke="#FF4D5B" strokeWidth={8 * u} strokeLinecap="round" strokeDasharray={W * 2} strokeDashoffset={W * 2 * (1 - ease(f, 14, 50))} /></svg>
      </div>
    </Pain>
  );
};

/** "আর না।": the three problems pile up, then shatter. */
const Slam: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width, height } = useLayout();
  const hit = 34;
  const burst = ease(f, hit, hit + 40);
  const words = ['Avwg evsjvq', 'kkh? kSh?', 'বাংলাদেস'];
  const big = spring({ frame: f - hit, fps: 60, config: { damping: 9, stiffness: 160 } });
  const shake = f > hit && f < hit + 16 ? (random(`sh${f}`) - 0.5) * 30 * u : 0;
  return (
    <AbsoluteFill style={{ transform: `translate(${shake}px, ${shake * 0.6}px)` }}>
      {words.map((w, i) => {
        const pile = spring({ frame: f - i * 6, fps: 60, config: { damping: 14 } });
        const ang = (i - 1) * 0.9 + 0.3;
        return (
          <div key={w} style={{ position: 'absolute', left: width / 2, top: height / 2 + (i - 1) * 110 * u, transform: `translate(-50%,-50%) translate(${Math.cos(ang) * burst * width * 0.8}px, ${Math.sin(ang) * burst * height * 0.5}px) rotate(${(i - 1) * 6 + burst * (i - 1) * 90}deg) scale(${pile * (1 - burst * 0.4)})`, opacity: pile * (1 - burst), filter: `blur(${burst * 20 * u}px)`, fontFamily: i === 2 ? FONT_BN : FONT_LATIN, fontWeight: 700, fontSize: 90 * u, color: '#FF8A94', whiteSpace: 'nowrap' }}>
            {w}
          </div>
        );
      })}
      {Array.from({ length: 46 }).map((_, i) => {
        const a = random(`pa${i}`) * Math.PI * 2, d = (0.3 + random(`pd${i}`)) * width * 0.6 * burst, s = (8 + random(`ps${i}`) * 26) * u;
        return <div key={i} style={{ position: 'absolute', left: width / 2 + Math.cos(a) * d, top: height / 2 + Math.sin(a) * d, width: s, height: s, background: i % 3 ? '#FF4D5B' : '#fff', opacity: f > hit ? 1 - burst : 0, transform: `rotate(${a * 180}deg)` }} />;
      })}
      <AbsoluteFill style={{ background: '#fff', opacity: interpolate(f, [hit, hit + 3, hit + 14], [0, 0.55, 0], clamp) }} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: f >= hit ? interpolate(f, [80, 90], [1, 0], clamp) : 0 }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 330 : 300) * u, lineHeight: 1, color: C.white, transform: `scale(${1.6 - big * 0.6})`, textShadow: `0 0 ${80 * u}px rgba(47,209,142,0.55)` }}>আর না।</div>
        <Sub o={ease(f, hit + 10, hit + 30)} size={40} style={{ marginTop: 20 * u }}>Not anymore.</Sub>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

/** A real piece of the app, in a floating glass card that swings into place. */
const AppCard: React.FC<{ src: string; width: number; crop?: { x: number; y: number; w: number; h: number; iw: number; ih: number }; delay?: number; accent?: boolean; style?: React.CSSProperties }> = ({ src, width, crop, delay = 0, accent, style }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const t = spring({ frame: f - delay, fps: 60, config: { damping: 16, stiffness: 110 } });
  const k = crop ? width / crop.w : 1;
  return (
    <div style={{ width, height: crop ? crop.h * k : undefined, overflow: 'hidden', position: 'relative', borderRadius: 30 * u, background: '#16231e', opacity: t, transform: `perspective(${2400 * u}px) rotateX(${(1 - t) * 24}deg) translateY(${(1 - t) * 80 * u}px)`, boxShadow: `0 0 0 ${2 * u}px ${accent ? 'rgba(47,209,142,0.55)' : 'rgba(255,255,255,0.14)'}, 0 ${40 * u}px ${100 * u}px -${30 * u}px rgba(0,0,0,0.9)${accent ? `, 0 0 ${80 * u}px rgba(47,209,142,0.25)` : ''}`, ...style }}>
      {crop
        ? <Img src={src} style={{ position: 'absolute', left: -crop.x * k, top: -crop.y * k, width: crop.iw * k, height: crop.ih * k, maxWidth: 'none' }} />
        : <Img src={src} style={{ width: '100%', display: 'block' }} />}
    </div>
  );
};

const RevealTitle: React.FC<{ bn: string; en: string; len: number }> = ({ bn, en, len }) => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const t = spring({ frame: f, fps: 60, config: { damping: 14 } });
  return (
    <div style={{ textAlign: 'center', opacity: t * (1 - ease(f, len - 14, len)), transform: `translateY(${(1 - t) * 30 * u}px)` }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 84 : 76) * u, color: C.white }}>{bn}</div>
      <Sub style={{ marginTop: 6 * u }}>{en}</Sub>
    </div>
  );
};
const pad3 = (n: number) => String(n).padStart(3, '0');
const step = (f: number, at: number, every: number, max: number) => Math.max(0, Math.min(max, Math.floor((f - at) / every) + 1));

/** Fix 1: Bijoy in, Unicode out, as it is typed (the real converter). */
const RevealBijoy: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const k = f < CONV_AT - T2.reveal ? 0 : step(f, CONV_AT - T2.reveal, CONV_EVERY, 20);
  const W = (V ? 1300 : 1500) * u;
  const crop = { x: 0, y: 0, w: 1050, h: 222, iw: 1533, ih: 222 };
  const out = ease(f, 126, 140);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 44 * u, opacity: 1 - out, transform: `translateY(${-out * 60 * u}px)` }}>
      <RevealTitle bn="বিজয় থেকে ইউনিকোড, মুহূর্তে" en="Bijoy to Unicode, instantly" len={140} />
      <AppCard src={staticFile(`teaser2/conv/i${pad3(k)}.jpg`)} width={W} crop={crop} delay={10} />
      <div style={{ color: C.greenBright, fontSize: 64 * u, lineHeight: 1, opacity: ease(f, 30, 50), transform: `translateY(${Math.sin(f / 8) * 6 * u}px)` }}>↓</div>
      <AppCard src={staticFile(`teaser2/conv/o${pad3(k)}.jpg`)} width={W} crop={crop} delay={22} accent />
    </AbsoluteFill>
  );
};

/** Fix 2: "kkhoma" (or any near spelling) gives ক্ষমা straight away (the real suggestion bar). */
const RevealConjunct: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const k = f < PAD_AT - T2.r2 ? 0 : step(f, PAD_AT - T2.r2, PAD_EVERY, 6);
  const W = (V ? 1360 : 1700) * u;
  const done = ease(f, 98, 118);
  const out = ease(f, 116, 130);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 50 * u, opacity: 1 - out, transform: `translateY(${-out * 60 * u}px)` }}>
      <RevealTitle bn="শর্টকাট মুখস্থ লাগবে না" en="No shortcuts to memorise" len={130} />
      <AppCard src={staticFile(`teaser2/pad/a${pad3(k)}.jpg`)} width={W} crop={{ x: 0, y: 0, w: 1500, h: 378, iw: 2160, ih: 378 }} delay={8} style={{ WebkitMaskImage: 'linear-gradient(90deg, #000 75%, transparent)', maskImage: 'linear-gradient(90deg, #000 75%, transparent)' }} />
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 230 : 200) * u, lineHeight: 1, color: C.white, opacity: done, transform: `scale(${0.7 + done * 0.3})`, textShadow: `0 0 ${60 * u}px rgba(47,209,142,0.5)` }}>ক্ষমা</div>
    </AbsoluteFill>
  );
};

/** Fix 3: the spell checker finds বাংলাদেস and one click makes it বাংলাদেশ. */
const RevealSpelling: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const fixed = f >= FIX_AT - T2.r3;
  const W = (V ? 1300 : 1500) * u;
  const out = ease(f, 116, 130);
  const click = interpolate(f, [FIX_AT - T2.r3 - 14, FIX_AT - T2.r3 - 2, FIX_AT - T2.r3 + 8], [0, 1, 0], clamp);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 36 * u, opacity: 1 - out, transform: `translateY(${-out * 60 * u}px)` }}>
      <RevealTitle bn="ভুল বানান? ঠিক করে দেয়" en="Misspelled? Fixed in one click" len={130} />
      <AppCard src={staticFile(`teaser2/spell/${fixed ? 'text1' : 'text0'}.jpg`)} width={W} crop={{ x: 0, y: 0, w: 1200, h: 150, iw: 3108, ih: 210 }} delay={6} accent={fixed} />
      <div style={{ position: 'relative' }}>
        <AppCard src={staticFile(`teaser2/spell/${fixed ? 'ok' : 'issue'}.jpg`)} width={W} crop={fixed ? { x: 0, y: 0, w: 1200, h: 150, iw: 3108, ih: 150 } : { x: 0, y: 0, w: 1700, h: 219, iw: 3108, ih: 219 }} delay={20} />
        {/* where the click lands: the first suggestion, বাংলাদেশ */}
        <div style={{ position: 'absolute', left: W * 0.215, top: W * 0.058, width: 60 * u, height: 60 * u, borderRadius: 999, border: `${4 * u}px solid ${C.greenBright}`, transform: `translate(-50%,-50%) scale(${0.6 + click})`, opacity: click }} />
      </div>
    </AbsoluteFill>
  );
};

export const Teaser2: React.FC = () => {
  const f = useCurrentFrame();
  // cold and tense while it's hard, warm again once it's fixed
  const sun = interpolate(f, [0, T2.p1, T2.slam, T2.slam + 40, T2.reveal, T2.logo, 1200], [0, 0.18, 0.3, 0.05, 0.6, 0.6, 1], clamp);
  const field = interpolate(f, [0, T2.reveal - 10, T2.reveal + 30, 1200], [0.08, 0.12, 0.75, 1], clamp);
  const cold = interpolate(f, [T2.p1, T2.p1 + 20, T2.slam + 30, T2.reveal], [0, 1, 1, 0], clamp);
  const alarm = f > T2.p1 && f < T2.slam ? 0.06 + 0.06 * Math.max(0, Math.sin(f / 7)) : 0;
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Backdrop sun={sun} field={field} warmth={interpolate(f, [T2.reveal, 1200], [0, 0.4], clamp)} />
      <AbsoluteFill style={{ background: 'radial-gradient(120% 90% at 50% 50%, rgba(40,60,80,0.35), rgba(5,10,16,0.75))', opacity: cold * 0.9 }} />
      <AbsoluteFill style={{ boxShadow: 'inset 0 0 300px rgba(224,48,63,1)', opacity: alarm * 4 }} />
      <Petals count={26} opacity={interpolate(f, [T2.reveal, T2.reveal + 60], [0, 0.7], clamp)} />
      <Sequence from={T2.hook} durationInFrames={150}><Hook /></Sequence>
      <Sequence from={T2.p1} durationInFrames={140}><PainBijoy /></Sequence>
      <Sequence from={T2.p2} durationInFrames={140}><PainConjunct /></Sequence>
      <Sequence from={T2.p3} durationInFrames={140}><PainSpelling /></Sequence>
      <Sequence from={T2.slam - 34} durationInFrames={124}><Slam /></Sequence>
      <Sequence from={T2.reveal} durationInFrames={140}><RevealBijoy /></Sequence>
      <Sequence from={T2.r2} durationInFrames={130}><RevealConjunct /></Sequence>
      <Sequence from={T2.r3} durationInFrames={130}><RevealSpelling /></Sequence>
      <Sequence from={T2.logo}><ComingSoon /></Sequence>
      <Audio src={staticFile('music/teaser2.wav')} />
    </AbsoluteFill>
  );
};
