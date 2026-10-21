import React from 'react';
import { AbsoluteFill, interpolate, random, useCurrentFrame } from 'remotion';
import { C, useLayout } from './theme';

export const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;

/** 0→1 over [a, b] with an ease-out curve. */
export const ease = (f: number, a: number, b: number) => {
  const t = interpolate(f, [a, b], [0, 1], clamp);
  return 1 - Math.pow(1 - t, 3);
};

/** Fade in at [a, a+d], out at [b-d, b]. */
export const window = (f: number, a: number, b: number, d = 18) =>
  Math.min(interpolate(f, [a, a + d], [0, 1], clamp), interpolate(f, [b - d, b], [1, 0], clamp));

/** Night sky with the flag's red sun and green field, both blurred, drifting slowly. */
export const Backdrop: React.FC<{ sun?: number; field?: number; warmth?: number }> = ({ sun = 1, field = 1, warmth = 0 }) => {
  const f = useCurrentFrame();
  const { width, height, u } = useLayout();
  const sx = width * 0.5 + Math.sin(f / 300) * 60 * u;
  const sy = height * 0.36 + Math.cos(f / 420) * 40 * u;
  const r = Math.min(width, height) * 0.42;
  return (
    <AbsoluteFill style={{ background: `radial-gradient(120% 90% at 50% 0%, #0A2E21 0%, ${C.night} 60%, #010805 100%)`, overflow: 'hidden' }}>
      <div style={{ position: 'absolute', left: sx - r, top: sy - r, width: r * 2, height: r * 2, borderRadius: '50%', background: `radial-gradient(circle, ${C.red} 0%, rgba(224,48,63,0.55) 38%, rgba(224,48,63,0) 70%)`, opacity: 0.85 * sun, filter: `blur(${30 * u}px)` }} />
      <div style={{ position: 'absolute', left: -width * 0.3, bottom: -height * 0.32, width: width * 1.6, height: height * 0.75, borderRadius: '50%', background: `radial-gradient(ellipse, #0F8A5F 0%, rgba(11,107,74,0.5) 45%, rgba(11,107,74,0) 72%)`, opacity: 0.9 * field, filter: `blur(${40 * u}px)` }} />
      <div style={{ position: 'absolute', right: -width * 0.15, top: height * 0.55, width: width * 0.5, height: width * 0.5, borderRadius: '50%', background: `radial-gradient(circle, ${C.mustard} 0%, rgba(233,185,73,0) 65%)`, opacity: 0.25 * warmth, filter: `blur(${50 * u}px)` }} />
      <Grain />
    </AbsoluteFill>
  );
};

/** Fine film grain so the gradients do not band. */
const Grain: React.FC = () => (
  <AbsoluteFill style={{ opacity: 0.07, mixBlendMode: 'overlay', backgroundImage: "url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>\")" }} />
);

/** Krishnachura petals drifting down. */
export const Petals: React.FC<{ count?: number; from?: number; opacity?: number }> = ({ count = 28, from = 0, opacity = 1 }) => {
  const f = useCurrentFrame() - from;
  const { width, height, u } = useLayout();
  return (
    <AbsoluteFill style={{ pointerEvents: 'none', opacity }}>
      {Array.from({ length: count }).map((_, i) => {
        const x0 = random(`px${i}`) * width;
        const speed = 1.2 + random(`ps${i}`) * 2.2;
        const size = (14 + random(`pz${i}`) * 26) * u;
        const y = ((f * speed * u + random(`py${i}`) * (height + 200)) % (height + 200)) - 100;
        const x = x0 + Math.sin((f + i * 40) / 45) * 40 * u;
        const rot = f * (1 + random(`pr${i}`) * 2) + i * 37;
        const tint = random(`pc${i}`) > 0.3 ? C.red : C.coral;
        return (
          <div key={i} style={{ position: 'absolute', left: x, top: y, width: size, height: size * 0.62, background: tint, borderRadius: '70% 10% 70% 10%', transform: `rotate(${rot}deg)`, opacity: 0.55 + random(`po${i}`) * 0.4, filter: `blur(${random(`pb${i}`) > 0.7 ? 2 * u : 0}px)` }} />
        );
      })}
    </AbsoluteFill>
  );
};

/** Stylised Shaheed Minar: five open frames with vertical bars, the tall centre frame bowing forward
 * at the top (the mother sheltering her children), the red sun behind. */
export const ShaheedMinar: React.FC<{ size: number; rise: number; glow?: number }> = ({ size, rise, glow = 1 }) => {
  const h = size;
  const w = size * 1.3;
  const sw = size * 0.018;
  const frame = (x: number, cw: number, ch: number, key: string) => {
    const top = h - ch;
    const bars = [0.25, 0.5, 0.75].map((k) => x + cw * k);
    return (
      <g key={key}>
        <rect x={x} y={top} width={cw} height={ch} fill="rgba(255,255,255,0.06)" stroke="url(#stone)" strokeWidth={sw} />
        {bars.map((bx) => <line key={bx} x1={bx} x2={bx} y1={top} y2={h} stroke="url(#stone)" strokeWidth={sw * 0.45} />)}
      </g>
    );
  };
  // centre: vertical part, then the top section leaning forward (drawn as a sheared frame)
  const cx = w * 0.4;
  const cw = w * 0.2;
  const ch = h * 0.7;
  const top = h - ch;
  const lean = h * 0.22;
  const centreBars = [0.25, 0.5, 0.75];
  return (
    <svg width={w} height={h} viewBox={`0 ${-h * 0.05} ${w} ${h * 1.05}`} style={{ overflow: 'visible', transform: `translateY(${(1 - rise) * h * 0.6}px)`, opacity: rise }}>
      <defs>
        <linearGradient id="stone" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#FFFFFF" />
          <stop offset="100%" stopColor="#C7D6CF" />
        </linearGradient>
        <radialGradient id="sunfill">
          <stop offset="0%" stopColor={C.red} />
          <stop offset="72%" stopColor={C.red} />
          <stop offset="100%" stopColor="#B81E2C" />
        </radialGradient>
      </defs>
      <circle cx={w / 2} cy={h * 0.45} r={h * 0.3} fill="url(#sunfill)" opacity={0.95 * glow} />
      {frame(w * 0.05, w * 0.1, h * 0.42, 'l2')}
      {frame(w * 0.2, w * 0.12, h * 0.56, 'l1')}
      {frame(w * 0.68, w * 0.12, h * 0.56, 'r1')}
      {frame(w * 0.85, w * 0.1, h * 0.42, 'r2')}
      <rect x={cx} y={top} width={cw} height={ch} fill="rgba(255,255,255,0.06)" stroke="url(#stone)" strokeWidth={sw} />
      {centreBars.map((k) => <line key={k} x1={cx + cw * k} x2={cx + cw * k} y1={top} y2={h} stroke="url(#stone)" strokeWidth={sw * 0.45} />)}
      <polygon points={`${cx},${top} ${cx + cw},${top} ${cx + cw + lean * 0.55},${top - lean * 0.85} ${cx + lean * 0.55},${top - lean * 0.85}`} fill="rgba(255,255,255,0.06)" stroke="url(#stone)" strokeWidth={sw} strokeLinejoin="round" />
      {centreBars.map((k) => <line key={`t${k}`} x1={cx + cw * k} y1={top} x2={cx + cw * k + lean * 0.55} y2={top - lean * 0.85} stroke="url(#stone)" strokeWidth={sw * 0.45} />)}
      <rect x={-w * 0.02} y={h * 0.99} width={w * 1.04} height={h * 0.025} rx={h * 0.01} fill="rgba(255,255,255,0.55)" />
    </svg>
  );
};

/** Frosted glass panel. */
export const Glass: React.FC<React.PropsWithChildren<{ style?: React.CSSProperties; strong?: boolean }>> = ({ children, style, strong }) => {
  const { u } = useLayout();
  return (
    <div
      style={{
        background: strong ? 'rgba(255,255,255,0.14)' : 'rgba(255,255,255,0.08)',
        border: '1px solid rgba(255,255,255,0.22)',
        boxShadow: `inset 0 1px 0 rgba(255,255,255,0.35), 0 ${30 * u}px ${80 * u}px -${30 * u}px rgba(0,0,0,0.7)`,
        backdropFilter: `blur(${28 * u}px) saturate(160%)`,
        WebkitBackdropFilter: `blur(${28 * u}px) saturate(160%)`,
        borderRadius: 32 * u,
        ...style,
      }}
    >
      {children}
    </div>
  );
};

/** Letters revealed one by one (grapheme-safe enough for Bangla: whole string fades per word). */
export const RiseText: React.FC<{ text: string; start: number; style: React.CSSProperties; stagger?: number; dy?: number }> = ({ text, start, style, stagger = 5, dy = 40 }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const words = text.split(' ');
  return (
    <div style={{ ...style, display: 'flex', flexWrap: 'wrap', justifyContent: style.textAlign === 'left' ? 'flex-start' : 'center', columnGap: '0.28em' }}>
      {words.map((w, i) => {
        const t = ease(f, start + i * stagger, start + i * stagger + 24);
        return (
          <span key={i} style={{ display: 'inline-block', opacity: t, transform: `translateY(${(1 - t) * dy * u}px)`, filter: `blur(${(1 - t) * 8 * u}px)` }}>
            {w}
          </span>
        );
      })}
    </div>
  );
};
