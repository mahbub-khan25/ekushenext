import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals, ShaheedMinar } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { AddressPill, Burst, Streaks, Sub, Sweep, TypeOn, graphemes } from './common';

/** The launch-date reveal: the remembrance, the name, 30 October 2026 rolled in like a slot machine,
 * every top feature, and the website and support email. 50 s at 60 fps, 2K, 16:9 and 9:16.
 * music/reveal.py is composed to the same marks. */
export const R = {
  prologue: 0,
  logo: 420,
  date: 660,
  features: 1140,
  finale: 2490,
  end: 2730,
} as const;
export const FEAT_LEN = 150;
export const REVEAL_FRAMES = 3000;
/** The date reels land one by one, then the whole date flares */
export const LOCK = { day: 800, month: 862, year: 924, flare: 944 } as const;

const W = (f: number, a: number, b: number, d = 16) =>
  Math.min(interpolate(f, [a, a + d], [0, 1], clamp), interpolate(f, [b - d, b], [1, 0], clamp));

const Big: React.FC<{ children: React.ReactNode; size: number; style?: React.CSSProperties }> = ({ children, size, style }) => {
  const { u } = useLayout();
  return <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: size * u, lineHeight: 1.2, color: C.white, textAlign: 'center', ...style }}>{children}</div>;
};

/* ───────────────────────── 0–7 s: remembrance ───────────────────────── */
const Prologue: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const l1 = W(f, 40, 215, 26), l2 = W(f, 230, 410, 26);
  const minar = ease(f, 110, 300);
  const rise = (a: number) => (1 - ease(f, a, a + 40)) * 30 * u;
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center' }}>
      <div style={{ position: 'absolute', bottom: (V ? 260 : -40) * u, opacity: 0.55 * minar, filter: `blur(${(1 - minar) * 10 * u}px)`, transform: `scale(${1 + f * 0.0004})` }}>
        <ShaheedMinar size={(V ? 900 : 720) * u} rise={minar} glow={0.7} />
      </div>
      <AbsoluteFill style={{ background: 'radial-gradient(ellipse at 50% 45%, rgba(0,0,0,0) 0%, rgba(1,8,5,0.75) 75%)' }} />
      <div style={{ position: 'absolute', width: '86%', textAlign: 'center', opacity: l1, transform: `translateY(${rise(40)}px)` }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, letterSpacing: '0.12em', fontSize: 46 * u, color: C.mustard, marginBottom: 22 * u }}>২১ ফেব্রুয়ারি ১৯৫২</div>
        <Big size={V ? 104 : 96} style={{ fontWeight: 600 }}>মায়ের ভাষার জন্য<br />যাঁরা প্রাণ দিয়েছিলেন</Big>
        <Sub style={{ marginTop: 26 * u }}>For those who gave their lives for their mother tongue</Sub>
      </div>
      <div style={{ position: 'absolute', width: '86%', textAlign: 'center', opacity: l2, transform: `translateY(${rise(230)}px)` }}>
        <Big size={V ? 112 : 104}>তাঁদের ভাষা লেখা হোক<br /><span style={{ color: C.greenBright }}>সবার জন্য সহজ</span></Big>
        <Sub style={{ marginTop: 26 * u }}>Their language deserves to be easy to write, for everyone</Sub>
      </div>
    </AbsoluteFill>
  );
};

/* ───────────────────────── 7–11 s: the name ───────────────────────── */
const Logo: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const pop = spring({ frame: f - 30, fps: 60, config: { damping: 11, mass: 0.9 } });
  const name = graphemes('একুশে নেক্সট');
  const ring = ease(f, 30, 90);
  const out = ease(f, 222, 240);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out, transform: `scale(${1 + out * 0.15})`, filter: `blur(${out * 14 * u}px)` }}>
      <Streaks from={0} to={44} count={34} color="rgba(255,255,255,0.9)" seed="lg" />
      <div style={{ position: 'relative' }}>
        {[0, 14].map((d) => {
          const r = ease(f, 30 + d, 100 + d);
          return <div key={d} style={{ position: 'absolute', left: '50%', top: '50%', width: 340 * u, height: 340 * u, borderRadius: 999, border: `${3 * u}px solid rgba(255,255,255,${0.55 * (1 - r)})`, transform: `translate(-50%,-50%) scale(${0.7 + r * 2.2})` }} />;
        })}
        <Img src={staticFile('assets/logo_512.png')} style={{ width: (V ? 300 : 260) * u, height: (V ? 300 : 260) * u, borderRadius: 64 * u, opacity: Math.min(1, pop * 1.4), transform: `scale(${0.4 + pop * 0.6}) rotate(${(1 - pop) * -14}deg)`, boxShadow: `0 ${30 * u}px ${90 * u}px rgba(0,0,0,0.6), 0 0 ${160 * u * (0.5 + ring)}px rgba(224,48,63,0.55)` }} />
      </div>
      <div style={{ display: 'flex', marginTop: 34 * u }}>
        {name.map((g, i) => {
          const t = spring({ frame: f - 52 - i * 3, fps: 60, config: { damping: 14 } });
          return <span key={i} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 170 : 150) * u, color: C.white, whiteSpace: 'pre', display: 'inline-block', opacity: t, transform: `translateY(${(1 - t) * 60 * u}px)`, textShadow: `0 0 ${60 * u}px rgba(224,48,63,0.5)` }}>{g}</span>;
        })}
      </div>
      <div style={{ opacity: ease(f, 96, 126), transform: `translateY(${(1 - ease(f, 96, 126)) * 20 * u}px)`, textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 64 : 58) * u, color: 'rgba(234,243,238,0.9)' }}>বাংলা লেখার নতুন কিবোর্ড</div>
        <Sub style={{ marginTop: 4 * u }}>A new keyboard for writing Bangla</Sub>
      </div>
      <Burst at={30} y={V ? 0.4 : 0.36} count={40} seed="lg" spread={0.7} />
      <Sweep at={60} dur={50} strength={0.14} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 11–19 s: the date ───────────────────────── */
const MONTHS = ['জানুয়ারি', 'ফেব্রুয়ারি', 'মার্চ', 'এপ্রিল', 'মে', 'জুন', 'জুলাই', 'আগস্ট', 'সেপ্টেম্বর', 'অক্টোবর', 'নভেম্বর', 'ডিসেম্বর'];
const bnNum = (n: number | string) => String(n).replace(/[0-9]/g, (d) => '০১২৩৪৫৬৭৮৯'[+d]);

/** A slot-machine reel that spins and lands on `target`. */
const Reel: React.FC<{ items: string[]; target: number; spins: number; start: number; land: number; size: number; row: number; width: number; color?: string }> = ({ items, target, spins, start, land, size, row, width, color = C.white }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const n = items.length;
  const total = spins * n + target;
  const t = interpolate(f, [start, land], [0, 1], clamp);
  const k = 1 - Math.pow(1 - t, 4);
  const pos = total * k;
  const vel = t < 1 ? (4 * Math.pow(1 - t, 3) * total) / (land - start) : 0;
  const rowH = row;
  const base = Math.floor(pos);
  const frac = pos - base;
  const settle = spring({ frame: f - land, fps: 60, config: { damping: 9, stiffness: 220 } });
  const landed = f >= land;
  const glow = interpolate(f, [land, land + 6, land + 40], [0, 1, 0.3], clamp);
  return (
    <div style={{ position: 'relative', width, height: rowH * 1.08, overflow: 'hidden', borderRadius: 26 * u, background: 'linear-gradient(180deg, rgba(0,0,0,0.5), rgba(255,255,255,0.06) 50%, rgba(0,0,0,0.5))', border: `${2 * u}px solid rgba(255,255,255,${0.16 + glow * 0.5})`, boxShadow: `0 0 ${80 * u * glow}px rgba(233,185,73,${0.6 * glow}), inset 0 0 ${40 * u}px rgba(0,0,0,0.6)` }}>
      {[-1, 0, 1].map((d) => {
        const idx = (((base + d) % n) + n) % n;
        const y = (d - frac) * rowH + (landed ? (1 - settle) * -0.12 * rowH : 0);
        return (
          <div key={d} style={{ position: 'absolute', left: 0, right: 0, top: '50%', height: rowH, marginTop: -rowH / 2, display: 'flex', alignItems: 'center', justifyContent: 'center', transform: `translateY(${y}px)`, fontFamily: FONT_BN, fontWeight: 700, fontSize: size, color: landed && d === 0 ? color : C.white, filter: `blur(${Math.min(10, vel * 0.25) * u}px)`, opacity: d === 0 ? 1 : 0.5 }}>
            {items[idx]}
          </div>
        );
      })}
      <div style={{ position: 'absolute', inset: 0, background: 'linear-gradient(180deg, rgba(3,18,12,0.85), transparent 30%, transparent 70%, rgba(3,18,12,0.85))' }} />
    </div>
  );
};

const DateReveal: React.FC = () => {
  const f = useCurrentFrame();
  const G = f + R.date;
  const { u, V } = useLayout();
  const enter = spring({ frame: f, fps: 60, config: { damping: 16, stiffness: 90 } });
  const flare = ease(G, LOCK.flare, LOCK.flare + 30);
  const out = ease(f, 462, 480);
  const days = Array.from({ length: 31 }, (_, i) => bnNum(i + 1));
  const years = Array.from({ length: 10 }, (_, i) => bnNum(2020 + i));
  const s = V ? 150 : 170;
  const shake = G >= LOCK.flare && G < LOCK.flare + 14 ? Math.sin(G * 3.1) * (14 - (G - LOCK.flare)) * u : 0;
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out, transform: `translateX(${shake}px)` }}>
      <AbsoluteFill style={{ background: `radial-gradient(circle at 50% ${V ? 46 : 52}%, rgba(233,185,73,${0.35 * flare * (1 - ease(G, LOCK.flare + 30, LOCK.flare + 160))}) 0%, rgba(224,48,63,${0.25 * flare}) 25%, transparent 60%)` }} />
      <div style={{ opacity: enter, transform: `translateY(${(1 - enter) * 40 * u}px)`, textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_LATIN, fontWeight: 700, letterSpacing: '0.42em', fontSize: 30 * u, color: C.mustard }}>LAUNCHING</div>
        <Big size={V ? 96 : 84} style={{ marginTop: 8 * u }}>উদ্বোধনের দিন</Big>
      </div>
      <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 26 : 30) * u, marginTop: 50 * u, opacity: enter, transform: `scale(${0.9 + enter * 0.1 + flare * 0.04})` }}>
        <Reel items={days} target={29} spins={3} start={40} land={LOCK.day - R.date} size={s * u} row={s * 1.4 * u} width={(V ? 420 : 300) * u} color={C.mustard} />
        <Reel items={MONTHS} target={9} spins={2} start={52} land={LOCK.month - R.date} size={s * 0.82 * u} row={s * 1.4 * u} width={(V ? 900 : 760) * u} color={C.mustard} />
        <Reel items={years} target={6} spins={2} start={64} land={LOCK.year - R.date} size={s * u} row={s * 1.4 * u} width={(V ? 560 : 480) * u} color={C.mustard} />
      </div>
      <div style={{ marginTop: 46 * u, opacity: ease(G, LOCK.flare, LOCK.flare + 30), transform: `translateY(${(1 - ease(G, LOCK.flare, LOCK.flare + 30)) * 30 * u}px)`, textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: (V ? 70 : 64) * u, color: C.white, letterSpacing: '0.02em' }}>October 30, 2026</div>
        <Sub style={{ marginTop: 8 * u }}>Free for Windows, macOS and Linux</Sub>
      </div>
      <Burst at={LOCK.day - R.date} x={V ? 0.5 : 0.24} y={V ? 0.36 : 0.52} count={14} spread={0.16} seed="d1" />
      <Burst at={LOCK.month - R.date} x={0.5} y={V ? 0.5 : 0.52} count={14} spread={0.16} seed="d2" />
      <Burst at={LOCK.year - R.date} x={V ? 0.5 : 0.78} y={V ? 0.64 : 0.52} count={14} spread={0.16} seed="d3" />
      <Burst at={LOCK.flare - R.date} y={V ? 0.5 : 0.52} count={70} spread={1.0} seed="fl" />
      <Sweep at={LOCK.flare - R.date + 10} dur={60} strength={0.2} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 19–41.5 s: the features ───────────────────────── */
const FEATURES = 9;
const Feature: React.FC<{ i: number; title: React.ReactNode; sub: string; children: React.ReactNode }> = ({ i, title, sub, children }) => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const t = spring({ frame: f, fps: 60, config: { damping: 15, stiffness: 120 } });
  const out = ease(f, FEAT_LEN - 16, FEAT_LEN);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out, transform: `translateX(${-out * 120 * u}px)`, filter: `blur(${out * 10 * u}px)` }}>
      <div style={{ position: 'absolute', top: (V ? 330 : 90) * u, display: 'flex', alignItems: 'center', gap: 16 * u, opacity: t }}>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 30 * u, color: C.greenBright, letterSpacing: '0.2em' }}>{String(i + 1).padStart(2, '0')} / {String(FEATURES).padStart(2, '0')}</span>
        <div style={{ width: 200 * u, height: 4 * u, borderRadius: 9, background: 'rgba(255,255,255,0.15)', overflow: 'hidden' }}>
          <div style={{ width: `${((i + ease(f, 0, FEAT_LEN)) / FEATURES) * 100}%`, height: '100%', background: C.greenBright }} />
        </div>
      </div>
      <div style={{ textAlign: 'center', opacity: t, transform: `translateY(${(1 - t) * 50 * u}px)`, marginBottom: (V ? 70 : 50) * u, padding: `0 ${60 * u}px` }}>
        <Big size={V ? 108 : 96}>{title}</Big>
        <Sub size={38} style={{ marginTop: 12 * u }}>{sub}</Sub>
      </div>
      <div style={{ height: (V ? 760 : 500) * u, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>{children}</div>
    </AbsoluteFill>
  );
};

const chip = (u: number, on = false): React.CSSProperties => ({ fontFamily: FONT_BN, fontWeight: 600, color: on ? '#03170F' : C.white, background: on ? C.greenBright : 'rgba(255,255,255,0.08)', border: `${2 * u}px solid ${on ? C.greenBright : 'rgba(255,255,255,0.2)'}`, borderRadius: 24 * u, padding: `${10 * u}px ${30 * u}px`, boxShadow: on ? `0 0 ${50 * u}px rgba(47,209,142,0.5)` : 'none' });
const roman = (u: number, size: number): React.CSSProperties => ({ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: size * u, color: 'rgba(234,243,238,0.75)', letterSpacing: '0.01em' });

/** Roman words that turn into Bangla one by one */
const Phonetic: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const words: [string, string][] = [['amar', 'আমার'], ['sonar', 'সোনার'], ['bangla', 'বাংলা']];
  return (
    <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', gap: (V ? 40 : 60) * u, alignItems: 'center' }}>
      {words.map(([r, b], i) => {
        const at = 18 + i * 24;
        const m = spring({ frame: f - at - 18, fps: 60, config: { damping: 12 } });
        return (
          <div key={r} style={{ position: 'relative', textAlign: 'center', minWidth: 300 * u }}>
            <div style={{ ...roman(u, 54), opacity: 1 - m, transform: `translateY(${-m * 40 * u}px)` }}><TypeOn text={r} start={at - 12} every={3} style={{}} /></div>
            <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 150 : 140) * u, color: C.white, marginTop: -40 * u, opacity: m, transform: `scale(${0.6 + m * 0.4})`, textShadow: `0 0 ${50 * u}px rgba(47,209,142,${0.5 * m})` }}>{b}</div>
          </div>
        );
      })}
    </div>
  );
};

/** Three spellings, one right word */
const Converge: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const ways = ['kkhoma', 'kShoma', 'khoma'];
  const m = ease(f, 50, 80);
  const pop = spring({ frame: f - 74, fps: 60, config: { damping: 10 } });
  return (
    <div style={{ position: 'relative', width: (V ? 1200 : 1400) * u, height: 460 * u }}>
      {ways.map((w, i) => {
        const x0 = (i - 1) * (V ? 360 : 440) * u, y0 = -150 * u + (i % 2) * 40 * u;
        const t = ease(f, 6 + i * 8, 28 + i * 8);
        return <div key={w} style={{ position: 'absolute', left: '50%', top: '50%', transform: `translate(-50%,-50%) translate(${x0 * (1 - m)}px, ${y0 * (1 - m) + (1 - t) * 40 * u}px) scale(${1 - m * 0.6})`, opacity: t * (1 - m), ...chip(u), ...roman(u, 50), color: C.white }}>{w}</div>;
      })}
      <div style={{ position: 'absolute', left: '50%', top: '58%', transform: `translate(-50%,-50%) scale(${0.5 + pop * 0.5})`, opacity: Math.min(1, pop * 1.4), fontFamily: FONT_BN, fontWeight: 700, fontSize: 210 * u, color: C.white, textShadow: `0 0 ${80 * u}px rgba(47,209,142,0.6)` }}>ক্ষমা</div>
      <Burst at={74} y={0.5} count={22} spread={0.3} seed="cv" />
    </div>
  );
};

const Dictionary: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const n = Math.round(418743 * ease(f, 8, 80));
  const fmt = (x: number) => bnNum(x.toLocaleString('en-IN'));
  const p = spring({ frame: f - 60, fps: 60, config: { damping: 13 } });
  return (
    <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 60 : 110) * u }}>
      <div style={{ textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 170 * u, color: C.mustard, fontVariantNumeric: 'tabular-nums' }}>{fmt(n)}</div>
        <Sub size={36}>শব্দের নিজস্ব অভিধান · 418,743 words</Sub>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 28 * u, opacity: p, transform: `translateX(${(1 - p) * 60 * u}px)` }}>
        <span style={{ ...roman(u, 56) }}>shohid</span>
        <span style={{ fontSize: 60 * u, color: C.greenBright }}>→</span>
        <span style={{ ...chip(u, true), fontSize: 96 * u }}>শহীদ</span>
      </div>
    </div>
  );
};

const NextWord: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const nexts = ['সকাল', 'জন্মদিন', 'নববর্ষ'];
  const sel = f > 104 ? 2 : -1;
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 50 * u }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 170 * u, color: C.white }}>শুভ {sel >= 0 && <span style={{ color: C.greenBright, opacity: ease(f, 104, 116) }}>{nexts[sel]}</span>}</div>
      <div style={{ display: 'flex', gap: 24 * u, flexWrap: 'wrap', justifyContent: 'center' }}>
        {nexts.map((w, i) => {
          const t = spring({ frame: f - 30 - i * 10, fps: 60, config: { damping: 12 } });
          return <div key={w} style={{ ...chip(u, i === 2 && f > 92), fontSize: (V ? 76 : 70) * u, opacity: t * (sel >= 0 ? 0.6 : 1), transform: `translateY(${(1 - t) * 40 * u}px) scale(${0.8 + t * 0.2})` }}>{w}</div>;
        })}
      </div>
    </div>
  );
};

const Speed: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const v = (0.98 * ease(f, 6, 70)).toFixed(2);
  return (
    <div style={{ position: 'relative', textAlign: 'center' }}>
      <Streaks from={0} to={FEAT_LEN} count={30} seed="sp" />
      <div style={{ fontFamily: FONT_LATIN, fontWeight: 800, fontSize: 280 * u, color: C.white, lineHeight: 1, letterSpacing: '-0.03em', textShadow: `0 0 ${80 * u}px rgba(47,209,142,0.55)` }}>{v}<span style={{ fontSize: 110 * u, color: C.greenBright, marginLeft: 16 * u }}>ms</span></div>
      <Sub size={38} style={{ marginTop: 20 * u }}>প্রতিটি কী-তে গড়ে · per keystroke, on average</Sub>
    </div>
  );
};

const Bijoy: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const m = ease(f, 64, 92);
  const keys = ['Ctrl', 'Alt', 'C'];
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 46 * u }}>
      <div style={{ position: 'relative', height: 180 * u, minWidth: (V ? 1200 : 1500) * u, textAlign: 'center' }}>
        <div style={{ position: 'absolute', inset: 0, fontFamily: FONT_LATIN, fontWeight: 500, fontSize: (V ? 84 : 100) * u, color: 'rgba(242,107,79,0.95)', opacity: 1 - m, filter: `blur(${m * 12 * u}px)`, letterSpacing: '0.02em' }}>Avgvi †mvbvi evsjv</div>
        <div style={{ position: 'absolute', inset: 0, fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 110 : 130) * u, color: C.white, opacity: m, transform: `scale(${0.85 + m * 0.15})` }}>আমার সোনার বাংলা</div>
        <Sweep at={64} dur={40} />
      </div>
      <div style={{ display: 'flex', gap: 18 * u, alignItems: 'center' }}>
        {keys.map((k, i) => {
          const press = interpolate(f, [40 + i * 6, 46 + i * 6, 70], [0, 1, 0], clamp);
          return <React.Fragment key={k}>{i > 0 && <span style={{ ...roman(u, 50) }}>+</span>}<span style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 54 * u, color: C.white, padding: `${14 * u}px ${30 * u}px`, borderRadius: 18 * u, background: 'linear-gradient(#1D2E27, #15231D)', boxShadow: `inset 0 0 0 ${2 * u}px rgba(255,255,255,0.25), 0 ${(1 - press) * 8 * u}px 0 rgba(0,0,0,0.5), 0 0 ${press * 50 * u}px rgba(47,209,142,0.7)`, transform: `translateY(${press * 6 * u}px)` }}>{k}</span></React.Fragment>;
        })}
      </div>
      <Sub size={34}>পুরনো বিজয় বা বাংলিশ লেখা বেছে নিয়ে এক চাপে · Select, press, done</Sub>
    </div>
  );
};

const Studio: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width } = useLayout();
  const pages = ['playground', 'xray', 'signs', 'spell'];
  const idx = Math.min(pages.length - 1, Math.floor(f / 34));
  const t = spring({ frame: f, fps: 60, config: { damping: 18, stiffness: 80 } });
  const flip = spring({ frame: f - idx * 34, fps: 60, config: { damping: 16 } });
  const Wd = V ? width * 0.9 : 880 * u;
  return (
    <div style={{ perspective: 2600 * u }}>
      <div style={{ width: Wd, height: Wd * 0.625, borderRadius: 26 * u, overflow: 'hidden', position: 'relative', transform: `rotateX(${(1 - t) * 40 + 6}deg) rotateY(${Math.sin(f / 60) * 6}deg) translateY(${(1 - t) * 200 * u}px)`, boxShadow: `0 0 0 ${2 * u}px rgba(255,255,255,0.16), 0 ${50 * u}px ${120 * u}px -${30 * u}px rgba(0,0,0,0.9), 0 0 ${100 * u}px rgba(47,209,142,0.25)` }}>
        {idx > 0 && <Img src={staticFile(`teaser5/${pages[idx - 1]}-dark.jpg`)} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%' }} />}
        <Img src={staticFile(`teaser5/${pages[idx]}-dark.jpg`)} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', opacity: flip, transform: `translateX(${(1 - flip) * 8}%)` }} />
        <Sweep at={idx * 34} dur={30} strength={0.12} />
      </div>
    </div>
  );
};

const Lock: React.FC<{ size: number; shut: number }> = ({ size, shut }) => (
  <svg width={size} height={size * 1.2} viewBox="0 0 100 120" style={{ overflow: 'visible' }}>
    <defs><linearGradient id="rlock" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stopColor="#5BE3AA" /><stop offset="1" stopColor="#0B6B4A" /></linearGradient></defs>
    <path d={`M28 ${54 - (1 - shut) * 14} V36 a22 22 0 0 1 44 0 V${54 - (1 - shut) * 2}`} fill="none" stroke="#EEF4F1" strokeWidth={10} strokeLinecap="round" />
    <rect x={14} y={52} width={72} height={60} rx={14} fill="url(#rlock)" stroke="rgba(255,255,255,0.5)" strokeWidth={2} />
    <circle cx={50} cy={78} r={8} fill="#03170F" /><rect x={46} y={80} width={8} height={16} rx={4} fill="#03170F" />
  </svg>
);
const Private: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const items = ['ইন্টারনেট লাগে না', 'অ্যাকাউন্ট লাগে না', 'কোনো ট্র্যাকিং নেই'];
  return (
    <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 50 : 90) * u }}>
      <div style={{ filter: `drop-shadow(0 0 ${60 * u}px rgba(47,209,142,0.6))`, transform: `scale(${1 + Math.sin(f / 12) * 0.02})` }}><Lock size={200 * u} shut={ease(f, 30, 50)} /></div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 22 * u, alignItems: V ? 'center' : 'flex-start' }}>
        {items.map((w, i) => {
          const t = spring({ frame: f - 40 - i * 14, fps: 60, config: { damping: 13 } });
          return <div key={w} style={{ display: 'flex', alignItems: 'center', gap: 20 * u, opacity: t, transform: `translateX(${(1 - t) * 60 * u}px)` }}><span style={{ display: 'grid', placeItems: 'center', width: 60 * u, height: 60 * u, borderRadius: 99, background: C.greenBright, color: '#03170F', fontSize: 36 * u, fontWeight: 800 }}>✓</span><span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 68 * u, color: C.white }}>{w}</span></div>;
        })}
      </div>
    </div>
  );
};

const OsIcon: React.FC<{ kind: 'win' | 'mac' | 'linux'; size: number }> = ({ kind, size }) => (
  <svg width={size} height={size} viewBox="0 0 24 24">
    {kind === 'win' && <path fill="currentColor" d="M3 5.6l7.4-1v7H3zm8.4-1.2L21 3v8.6h-9.6zM3 12.6h7.4v7l-7.4-1zm8.4 0H21V21l-9.6-1.4z" />}
    {kind === 'mac' && <path fill="currentColor" d="M16.4 12.6c0-2.4 2-3.6 2.1-3.7-1.2-1.7-3-1.9-3.6-2-1.5-.2-3 .9-3.8.9-.8 0-2-.9-3.3-.9-1.7 0-3.3 1-4.2 2.5-1.8 3.1-.5 7.7 1.3 10.2.9 1.2 1.9 2.6 3.2 2.6 1.3-.1 1.8-.8 3.3-.8s2 .8 3.3.8c1.4 0 2.3-1.2 3.1-2.5 1-1.4 1.4-2.8 1.4-2.9 0 0-2.8-1.1-2.8-4.2zM14 5.3c.7-.9 1.2-2 1-3.3-1 0-2.3.7-3 1.6-.6.7-1.2 1.9-1 3.1 1.1.1 2.3-.6 3-1.4z" />}
    {kind === 'linux' && <path fill="currentColor" d="M12 2c-2.2 0-3.4 1.9-3.4 4.3 0 1.4.3 2.3-.6 3.7-1 1.6-2.5 3.6-2.5 6.1 0 .6.1 1.1.2 1.6-.9.4-1.7.9-1.7 1.7 0 1.1 1.7 1.3 3.2 1.7 1.2.3 2 .9 3 .9.8 0 1.4-.4 1.8-.9.4.5 1 .9 1.8.9 1 0 1.8-.6 3-.9 1.5-.4 3.2-.6 3.2-1.7 0-.8-.8-1.3-1.7-1.7.1-.5.2-1 .2-1.6 0-2.5-1.5-4.5-2.5-6.1-.9-1.4-.6-2.3-.6-3.7C15.4 3.9 14.2 2 12 2z" />}
  </svg>
);
const Platforms: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const os: ['win' | 'mac' | 'linux', string][] = [['win', 'Windows'], ['mac', 'macOS'], ['linux', 'Linux']];
  const free = spring({ frame: f - 70, fps: 60, config: { damping: 9 } });
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 60 * u }}>
      <div style={{ display: 'flex', gap: (V ? 70 : 120) * u }}>
        {os.map(([k, name], i) => {
          const t = spring({ frame: f - 8 - i * 12, fps: 60, config: { damping: 11 } });
          return <div key={k} style={{ textAlign: 'center', color: C.white, opacity: t, transform: `translateY(${(1 - t) * 60 * u}px) scale(${0.7 + t * 0.3})` }}><div style={{ display: 'grid', placeItems: 'center', width: 190 * u, height: 190 * u, borderRadius: 48 * u, background: 'rgba(255,255,255,0.08)', border: `${2 * u}px solid rgba(255,255,255,0.2)` }}><OsIcon kind={k} size={100 * u} /></div><div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 42 * u, marginTop: 18 * u }}>{name}</div></div>;
        })}
      </div>
      <div style={{ ...chip(u, true), fontSize: 64 * u, opacity: Math.min(1, free * 1.4), transform: `scale(${0.6 + free * 0.4})` }}>সম্পূর্ণ ফ্রি · ওপেন সোর্স</div>
      <Burst at={70} y={0.7} count={24} spread={0.3} seed="pf" />
    </div>
  );
};

const FEATURE_SCENES: { title: React.ReactNode; sub: string; body: React.ReactNode }[] = [
  { title: 'যেভাবে বলেন, সেভাবে লিখুন', sub: 'Type the way you speak', body: <Phonetic /> },
  { title: 'যুক্তবর্ণ মুখস্থ করতে হবে না', sub: 'No conjunct rules to memorise', body: <Converge /> },
  { title: 'প্রমিত বানান সবার আগে', sub: 'The Bangla Academy standard spelling comes first', body: <Dictionary /> },
  { title: 'পরের শব্দটাও বুঝে নেয়', sub: 'It suggests the next word', body: <NextWord /> },
  { title: 'চোখের পলকের চেয়েও দ্রুত', sub: 'Faster than the blink of an eye', body: <Speed /> },
  { title: 'বিজয় থেকে ইউনিকোড, এক চাপে', sub: 'Old Bijoy text, fixed in one keystroke', body: <Bijoy /> },
  { title: 'একুশে স্টুডিও', sub: 'Writing pad, word X-ray, conjunct guide, spell check', body: <Studio /> },
  { title: 'আপনার লেখা, শুধু আপনার', sub: 'Nothing you type leaves your computer', body: <Private /> },
  { title: 'সব কম্পিউটারে, সবার জন্য', sub: 'Windows, macOS and Linux', body: <Platforms /> },
];

/* ───────────────────────── 41.5–45.5 s: the promise ───────────────────────── */
const Finale: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const m = ease(f, 0, 120);
  const t = spring({ frame: f - 30, fps: 60, config: { damping: 14, stiffness: 80 } });
  const out = ease(f, 222, 240);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', opacity: 1 - out }}>
      <div style={{ position: 'absolute', bottom: (V ? 320 : -20) * u, opacity: 0.8 * m }}><ShaheedMinar size={(V ? 1000 : 820) * u} rise={m} glow={1} /></div>
      <AbsoluteFill style={{ background: 'linear-gradient(180deg, rgba(1,8,5,0.7), rgba(1,8,5,0.1) 45%, rgba(1,8,5,0.6))' }} />
      <div style={{ position: 'absolute', top: (V ? 420 : 170) * u, textAlign: 'center', opacity: t, transform: `translateY(${(1 - t) * 40 * u}px)` }}>
        <Big size={V ? 104 : 96} style={{ fontWeight: 600 }}>দেখা হবে</Big>
        <Big size={V ? 200 : 190} style={{ color: C.mustard, textShadow: `0 0 ${80 * u}px rgba(233,185,73,0.5)` }}>৩০ অক্টোবর</Big>
        <Sub size={40} style={{ marginTop: 8 * u }}>See you on October 30</Sub>
      </div>
    </AbsoluteFill>
  );
};

/* ───────────────────────── 45.5–50 s: where to find it ───────────────────────── */
const End: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const pop = spring({ frame: f - 4, fps: 60, config: { damping: 11, mass: 0.8 } });
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [0, 12], [0, 1], clamp) }}>
      <Img src={staticFile('assets/logo_512.png')} style={{ width: (V ? 240 : 200) * u, height: (V ? 240 : 200) * u, borderRadius: 50 * u, transform: `scale(${0.6 + pop * 0.4})`, opacity: Math.min(1, pop * 1.5), boxShadow: `0 ${30 * u}px ${80 * u}px rgba(0,0,0,0.55), 0 0 ${120 * u}px rgba(224,48,63,0.5)` }} />
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 130 : 112) * u, color: C.white, marginTop: 22 * u, lineHeight: 1.1, opacity: ease(f, 10, 30) }}>একুশে নেক্সট</div>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 18 * u, marginTop: 14 * u, opacity: ease(f, 22, 42), flexWrap: 'wrap', justifyContent: 'center' }}>
        <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 58 : 52) * u, color: C.mustard }}>উদ্বোধন ৩০ অক্টোবর ২০২৬</span>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: 34 * u, color: 'rgba(234,243,238,0.7)' }}>Launching October 30, 2026</span>
      </div>
      <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 26 : 30) * u, marginTop: (V ? 56 : 46) * u }}>
        <AddressPill kind="web" text="www.ekushenext.com" at={58} />
        <AddressPill kind="mail" text="support@ekushenext.com" at={96} />
      </div>
      <Burst at={4} y={V ? 0.36 : 0.3} count={34} seed="rend" spread={0.6} />
    </AbsoluteFill>
  );
};

export const Reveal: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const sun = interpolate(f, [0, 380, 420, 660, 944, 1100, 2490, 2600], [0.15, 0.5, 1, 0.8, 1.2, 0.6, 0.6, 1], clamp);
  const fade = interpolate(f, [0, 30, REVEAL_FRAMES - 30, REVEAL_FRAMES], [0, 1, 1, 0], clamp);
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Audio src={staticFile('music/reveal.wav')} />
      <AbsoluteFill style={{ opacity: fade }}>
        <Backdrop sun={sun} field={interpolate(f, [0, 420], [0.3, 1], clamp)} warmth={f > R.finale ? 1 : 0.4} />
        <Petals count={22} opacity={interpolate(f, [0, 200, R.features, R.features + 60, R.finale, R.finale + 60], [0.2, 0.6, 0.6, 0.2, 0.2, 0.8], clamp)} />
        <Sequence from={R.prologue} durationInFrames={R.logo - R.prologue}><Prologue /></Sequence>
        <Sequence from={R.logo} durationInFrames={R.date - R.logo}><Logo /></Sequence>
        <Sequence from={R.date} durationInFrames={R.features - R.date}><DateReveal /></Sequence>
        {FEATURE_SCENES.map((s, i) => (
          <Sequence key={i} from={R.features + i * FEAT_LEN} durationInFrames={FEAT_LEN}>
            <Feature i={i} title={s.title} sub={s.sub}>{s.body}</Feature>
          </Sequence>
        ))}
        <Sequence from={R.finale} durationInFrames={R.end - R.finale}><Finale /></Sequence>
        <Sequence from={R.end} durationInFrames={REVEAL_FRAMES - R.end}><End /></Sequence>
        <AbsoluteFill style={{ pointerEvents: 'none', boxShadow: `inset 0 0 ${260 * u}px rgba(0,0,0,0.65)` }} />
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
