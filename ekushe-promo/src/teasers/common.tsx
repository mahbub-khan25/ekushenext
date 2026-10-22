import React from 'react';
import { AbsoluteFill, Img, interpolate, random, spring, staticFile, useCurrentFrame } from 'remotion';
import { clamp, ease } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';

/** Shared pieces for the five teasers. */

export const io = (f: number, a: number, b: number, d = 14) =>
  Math.min(interpolate(f, [a, a + d], [0, 1], clamp), interpolate(f, [b - d, b], [1, 0], clamp));
export const graphemes = (s: string) => Array.from(new Intl.Segmenter('bn', { granularity: 'grapheme' }).segment(s), (g) => g.segment);
export const pad3 = (n: number) => String(n).padStart(3, '0');

export const Sub: React.FC<{ children: React.ReactNode; o?: number; size?: number; style?: React.CSSProperties }> = ({ children, o = 1, size = 34, style }) => {
  const { u } = useLayout();
  return <div style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: size * u, color: 'rgba(234,243,238,0.72)', opacity: o, letterSpacing: '0.01em', textAlign: 'center', ...style }}>{children}</div>;
};

/** A burst of particles (petals and sparks) from a point, starting at frame `at`. */
export const Burst: React.FC<{ at: number; x?: number; y?: number; count?: number; spread?: number; seed?: string; colors?: string[] }> = ({ at, x = 0.5, y = 0.5, count = 36, spread = 0.55, seed = 'b', colors = [C.red, C.coral, C.greenBright, '#ffffff', C.mustard] }) => {
  const f = useCurrentFrame() - at;
  const { width, height, u } = useLayout();
  if (f < 0 || f > 80) return null;
  const t = ease(f, 0, 70);
  return (
    <AbsoluteFill style={{ pointerEvents: 'none' }}>
      {Array.from({ length: count }).map((_, i) => {
        const a = random(`${seed}a${i}`) * Math.PI * 2;
        const d = (0.25 + random(`${seed}d${i}`) * 0.75) * Math.min(width, height) * spread * t;
        const s = (8 + random(`${seed}s${i}`) * 22) * u;
        const petal = random(`${seed}p${i}`) > 0.45;
        return (
          <div key={i} style={{ position: 'absolute', left: width * x + Math.cos(a) * d, top: height * y + Math.sin(a) * d + t * t * 120 * u, width: s, height: petal ? s * 0.62 : s * 0.4, borderRadius: petal ? '70% 10% 70% 10%' : 99, background: colors[i % colors.length], opacity: 1 - t, transform: `rotate(${a * 57 + f * 9}deg)`, boxShadow: petal ? 'none' : `0 0 ${10 * u}px ${colors[i % colors.length]}` }} />
        );
      })}
    </AbsoluteFill>
  );
};

/** A diagonal sheen that sweeps across whatever it is placed over. */
export const Sweep: React.FC<{ at: number; dur?: number; strength?: number }> = ({ at, dur = 50, strength = 0.18 }) => {
  const f = useCurrentFrame();
  const p = interpolate(f, [at, at + dur], [-30, 130], clamp);
  if (f < at || f > at + dur) return null;
  return <AbsoluteFill style={{ pointerEvents: 'none', background: `linear-gradient(105deg, transparent ${p - 12}%, rgba(255,255,255,${strength}) ${p}%, transparent ${p + 12}%)`, mixBlendMode: 'screen' }} />;
};

/** Fast streaks of light, for speed and transitions. */
export const Streaks: React.FC<{ from: number; to: number; count?: number; color?: string; seed?: string }> = ({ from, to, count = 26, color = 'rgba(47,209,142,0.8)', seed = 'st' }) => {
  const f = useCurrentFrame();
  const { width, height, u } = useLayout();
  if (f < from || f > to) return null;
  const o = io(f, from, to, 12);
  return (
    <AbsoluteFill style={{ pointerEvents: 'none', opacity: o }}>
      {Array.from({ length: count }).map((_, i) => {
        const y = random(`${seed}y${i}`) * height;
        const speed = 0.6 + random(`${seed}v${i}`) * 1.6;
        const len = (200 + random(`${seed}l${i}`) * 600) * u;
        const x = ((f - from) * speed * 40 * u + random(`${seed}x${i}`) * (width + len)) % (width + len) - len;
        return <div key={i} style={{ position: 'absolute', left: x, top: y, width: len, height: (2 + random(`${seed}h${i}`) * 4) * u, borderRadius: 9, background: `linear-gradient(90deg, transparent, ${color})`, opacity: 0.25 + random(`${seed}o${i}`) * 0.6 }} />;
      })}
    </AbsoluteFill>
  );
};

/** Text that types itself in, with a caret. */
export const TypeOn: React.FC<{ text: string; start: number; every?: number; style: React.CSSProperties; caret?: string }> = ({ text, start, every = 2, style, caret = C.greenBright }) => {
  const f = useCurrentFrame();
  const g = graphemes(text);
  const n = Math.max(0, Math.min(g.length, Math.floor((f - start) / every)));
  const done = n >= g.length;
  return (
    <span style={{ ...style, whiteSpace: 'nowrap' }}>
      {g.slice(0, n).join('')}
      <span style={{ display: 'inline-block', width: '0.08em', height: '0.95em', marginLeft: '0.06em', verticalAlign: '-0.1em', background: caret, borderRadius: 3, opacity: f < start ? 0 : done ? (Math.floor(f / 20) % 2 ? 0 : 1) : 1 }} />
    </span>
  );
};

const Icon: React.FC<{ kind: 'web' | 'mail'; size: number }> = ({ kind, size }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
    {kind === 'web'
      ? <><circle cx="12" cy="12" r="9" /><path d="M3 12h18M12 3c2.6 2.8 2.6 15.2 0 18M12 3c-2.6 2.8-2.6 15.2 0 18" /></>
      : <><rect x="3" y="5" width="18" height="14" rx="2.5" /><path d="M3.5 6.5l8.5 6.5 8.5-6.5" /></>}
  </svg>
);

/** One address pill: icon, the address typing itself in, a glow and a sheen when it completes. */
const AddressPill: React.FC<{ kind: 'web' | 'mail'; text: string; at: number }> = ({ kind, text, at }) => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const pop = spring({ frame: f - at, fps: 60, config: { damping: 13, stiffness: 150 } });
  const every = 2;
  const done = at + 10 + text.length * every;
  const glow = interpolate(f, [done, done + 8, done + 40], [0, 1, 0.35], clamp);
  return (
    <div style={{ position: 'relative', overflow: 'hidden', display: 'flex', alignItems: 'center', gap: 22 * u, padding: `${18 * u}px ${36 * u}px ${18 * u}px ${26 * u}px`, borderRadius: 999, background: 'rgba(8,28,20,0.6)', border: `${2 * u}px solid rgba(47,209,142,${0.35 + glow * 0.5})`, boxShadow: `0 0 ${60 * u * glow}px rgba(47,209,142,${0.55 * glow}), inset 0 1px 0 rgba(255,255,255,0.2)`, backdropFilter: `blur(${18 * u}px)`, opacity: pop, transform: `translateY(${(1 - pop) * 50 * u}px) scale(${0.85 + pop * 0.15})` }}>
      <div style={{ display: 'grid', placeItems: 'center', width: 64 * u, height: 64 * u, borderRadius: 999, background: C.greenBright, color: '#03170F' }}><Icon kind={kind} size={36 * u} /></div>
      <TypeOn text={text} start={at + 10} every={every} style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: (V ? 56 : 52) * u, color: C.white, letterSpacing: '0.005em' }} />
      <Sweep at={done} dur={40} strength={0.25} />
    </div>
  );
};

/** The series ending, the same in every teaser: logo, name, "coming soon", then the website and the
 * support email typing themselves in. Lasts about 3 s (180 frames). */
export const END_LEN = 180;
export const END_WEB_AT = 58, END_MAIL_AT = 96; // the music places a sparkle on each
export const EndCard: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const pop = spring({ frame: f - 4, fps: 60, config: { damping: 11, mass: 0.8 } });
  const ring = ease(f, 4, 50);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [0, 12], [0, 1], clamp) }}>
      <div style={{ position: 'relative' }}>
        <div style={{ position: 'absolute', left: '50%', top: '50%', width: 300 * u, height: 300 * u, borderRadius: 999, border: `${3 * u}px solid rgba(255,255,255,${0.5 * (1 - ring)})`, transform: `translate(-50%,-50%) scale(${0.8 + ring * 1.6})` }} />
        <Img src={staticFile('assets/logo_512.png')} style={{ width: (V ? 260 : 220) * u, height: (V ? 260 : 220) * u, borderRadius: 56 * u, transform: `scale(${0.6 + pop * 0.4}) rotate(${(1 - pop) * -10}deg)`, opacity: Math.min(1, pop * 1.5), boxShadow: `0 ${30 * u}px ${80 * u}px rgba(0,0,0,0.55), 0 0 ${120 * u}px rgba(224,48,63,0.5)` }} />
      </div>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 140 : 120) * u, color: C.white, marginTop: 26 * u, lineHeight: 1.1, opacity: ease(f, 10, 30), transform: `translateY(${(1 - ease(f, 10, 30)) * 24 * u}px)`, textShadow: `0 0 ${50 * u}px rgba(224,48,63,0.45)` }}>একুশে নেক্সট</div>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 18 * u, marginTop: 10 * u, opacity: ease(f, 24, 44) }}>
        <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 60 : 54) * u, color: C.mustard }}>আসছে শীঘ্রই</span>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: 34 * u, color: 'rgba(234,243,238,0.7)' }}>Coming soon</span>
      </div>
      <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 26 : 30) * u, marginTop: (V ? 56 : 46) * u }}>
        <AddressPill kind="web" text="www.ekushenext.com" at={END_WEB_AT} />
        <AddressPill kind="mail" text="support@ekushenext.com" at={END_MAIL_AT} />
      </div>
      <Burst at={4} y={V ? 0.36 : 0.32} count={34} seed="end" spread={0.6} />
    </AbsoluteFill>
  );
};

/** A real piece of the app in a floating glass card that swings into place. */
export const AppCard: React.FC<{ src: string; width: number; crop?: { x: number; y: number; w: number; h: number; iw: number; ih: number }; delay?: number; accent?: boolean; style?: React.CSSProperties; children?: React.ReactNode }> = ({ src, width, crop, delay = 0, accent, style, children }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const t = spring({ frame: f - delay, fps: 60, config: { damping: 16, stiffness: 110 } });
  const k = crop ? width / crop.w : 1;
  return (
    <div style={{ width, height: crop ? crop.h * k : undefined, overflow: 'hidden', position: 'relative', borderRadius: 30 * u, background: '#16231e', opacity: t, transform: `perspective(${2400 * u}px) rotateX(${(1 - t) * 24}deg) translateY(${(1 - t) * 80 * u}px)`, boxShadow: `0 0 0 ${2 * u}px ${accent ? 'rgba(47,209,142,0.6)' : 'rgba(255,255,255,0.14)'}, 0 ${40 * u}px ${100 * u}px -${30 * u}px rgba(0,0,0,0.9)${accent ? `, 0 0 ${90 * u}px rgba(47,209,142,0.3)` : ''}`, ...style }}>
      {crop
        ? <Img src={src} style={{ position: 'absolute', left: -crop.x * k, top: -crop.y * k, width: crop.iw * k, height: crop.ih * k, maxWidth: 'none' }} />
        : <Img src={src} style={{ width: '100%', display: 'block' }} />}
      {children}
    </div>
  );
};
