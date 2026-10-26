import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { AddressPill, Burst, Sub, Sweep, graphemes } from './common';
import { OsIcon } from './FullPromo';

/** "What Ekushe Next does differently": an opening, eight differences shown as live typing (every
 * example is what Ekushe's engine really gives), then when and where. 9:16, 2K, 60 fps, 70 s.
 * music/why_different.py is composed to the same marks. No other keyboard is named. */
export const W = {
  open: 0,
  ch: 360, // first chapter
  len: 420, // each chapter
  end: 3720,
} as const;
export const WHY_FRAMES = 4200;
const MONO = "'DejaVu Sans Mono', 'Menlo', 'Consolas', monospace";

/* ───────────────────────── shared pieces ───────────────────────── */

const Glass: React.FC<React.PropsWithChildren<{ style?: React.CSSProperties; glow?: number }>> = ({ children, style, glow = 0 }) => {
  const { u } = useLayout();
  return (
    <div style={{ position: 'relative', overflow: 'hidden', borderRadius: 48 * u, padding: `${40 * u}px ${44 * u}px`, background: 'linear-gradient(170deg, rgba(20,40,32,0.9), rgba(6,20,14,0.9))', border: `${2 * u}px solid rgba(255,255,255,${0.12 + glow * 0.25})`, boxShadow: `0 ${50 * u}px ${120 * u}px -${40 * u}px rgba(0,0,0,0.9), 0 0 ${110 * u * glow}px rgba(47,209,142,${0.3 * glow})`, ...style }}>{children}</div>
  );
};

/** Roman letters typed one by one (shown in small key caps) */
const Keys: React.FC<{ text: string; at: number; every: number }> = ({ text, at, every }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const chars = Array.from(text);
  return (
    <div style={{ display: 'flex', gap: 10 * u, justifyContent: 'center', flexWrap: 'wrap', minHeight: 86 * u }}>
      {chars.map((c, i) => {
        const t = at + i * every;
        const p = spring({ frame: f - t, fps: 60, config: { damping: 12, stiffness: 220 } });
        if (f < t) return null;
        const press = interpolate(f, [t, t + 4, t + 10], [0, 1, 0], clamp);
        return (
          <div key={i} style={{ minWidth: (c === ' ' ? 110 : 70) * u, height: 80 * u, padding: `0 ${14 * u}px`, display: 'grid', placeItems: 'center', borderRadius: 18 * u, background: `rgba(255,255,255,${0.08 + press * 0.25})`, border: `${2 * u}px solid rgba(255,255,255,0.18)`, boxShadow: `0 ${6 * u * (1 - press)}px 0 rgba(0,0,0,0.5)`, transform: `translateY(${press * 5 * u}px) scale(${0.6 + p * 0.4})`, fontFamily: MONO, fontWeight: 700, fontSize: 42 * u, color: C.white }}>
            {c === ' ' ? 'space' : c}
          </div>
        );
      })}
    </div>
  );
};

type Cand = { t: string; tag?: string; dim?: boolean };
/** Ekushe's suggestion bar: candidates, the first one picked, optional little tags */
const Bar: React.FC<{ cands: Cand[]; at: number; pick?: number; pickAt?: number }> = ({ cands, at, pick = 0, pickAt }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const show = spring({ frame: f - at, fps: 60, config: { damping: 15, stiffness: 160 } });
  if (f < at) return null;
  const moved = pickAt !== undefined ? ease(f, pickAt, pickAt + 10) : 0;
  const sel = pickAt !== undefined && f >= pickAt ? pick : 0;
  return (
    <div style={{ display: 'flex', gap: 14 * u, justifyContent: 'center', flexWrap: 'wrap', opacity: show, transform: `translateY(${(1 - show) * 30 * u}px)` }}>
      {cands.map((c, i) => {
        const on = i === sel;
        const pop = on && pickAt !== undefined && f >= pickAt ? interpolate(f, [pickAt, pickAt + 6, pickAt + 16], [1, 1.12, 1], clamp) : 1;
        return (
          <div key={i} style={{ position: 'relative', display: 'flex', alignItems: 'baseline', gap: 10 * u, padding: `${14 * u}px ${26 * u}px`, borderRadius: 26 * u, background: on ? C.greenBright : 'rgba(255,255,255,0.08)', border: `${2 * u}px solid ${on ? C.greenBright : 'rgba(255,255,255,0.16)'}`, boxShadow: on ? `0 0 ${50 * u}px rgba(47,209,142,0.55)` : 'none', color: on ? '#03170F' : c.dim ? 'rgba(234,243,238,0.45)' : C.white, transform: `scale(${pop})`, transition: 'none', opacity: on ? 1 : 0.85 + 0.15 * moved }}>
            <span style={{ fontFamily: MONO, fontSize: 26 * u, opacity: 0.6 }}>{i + 1}</span>
            <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 58 * u, textDecoration: c.dim ? 'line-through' : 'none', textDecorationThickness: 3 * u }}>{c.t}</span>
            {c.tag && <span style={{ position: 'absolute', top: -22 * u, right: -8 * u, fontFamily: FONT_BN, fontWeight: 700, fontSize: 24 * u, padding: `${2 * u}px ${12 * u}px`, borderRadius: 99, background: c.dim ? C.red : C.mustard, color: c.dim ? '#fff' : '#2A1C00' }}>{c.tag}</span>}
          </div>
        );
      })}
    </div>
  );
};

/** The text field: committed Bangla, then the word being written (underlined), with a caret */
const Field: React.FC<{ done?: string; live?: string; liveRoman?: boolean; size?: number; glow?: number }> = ({ done = '', live = '', liveRoman, size = 76, glow = 0 }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  return (
    <div style={{ minHeight: size * 1.7 * u, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: `${12 * u}px ${30 * u}px`, borderRadius: 30 * u, background: 'rgba(0,0,0,0.28)', border: `${2 * u}px solid rgba(47,209,142,${0.25 + glow * 0.5})`, boxShadow: `inset 0 0 ${40 * u * glow}px rgba(47,209,142,${0.3 * glow})` }}>
      <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: size * u, color: C.white, whiteSpace: 'pre' }}>{done}</span>
      <span style={{ fontFamily: liveRoman ? FONT_LATIN : FONT_BN, fontWeight: 600, fontSize: (liveRoman ? size * 0.9 : size) * u, color: liveRoman ? C.mustard : C.greenBright, borderBottom: live ? `${4 * u}px solid ${liveRoman ? C.mustard : C.greenBright}` : 'none', whiteSpace: 'pre' }}>{live}</span>
      <span style={{ display: 'inline-block', width: 5 * u, height: size * 1.05 * u, marginLeft: 6 * u, background: C.greenBright, borderRadius: 3, opacity: Math.floor(f / 18) % 2 ? 0.25 : 1 }} />
    </div>
  );
};

/** One typed word: keys appear, the field shows the Bangla as it forms, the bar offers words, Space picks */
const typedState = (f: number, roman: string, at: number, every: number, forming: string[]) => {
  const n = Math.max(0, Math.min(roman.length, Math.floor((f - at) / every) + 1));
  const typing = f >= at;
  const idx = Math.min(forming.length - 1, Math.max(0, Math.round(((n - 1) / Math.max(1, roman.length - 1)) * (forming.length - 1))));
  return { n, typing, live: typing ? forming[idx] : '', doneAt: at + roman.length * every };
};

/** A chapter: number, the headline in Bangla with a small English line, the demo, a closing line */
const Chapter: React.FC<React.PropsWithChildren<{ n: number; title: React.ReactNode; en: string; foot: string; footEn: string }>> = ({ n, title, en, foot, footEn, children }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const inT = spring({ frame: f - 2, fps: 60, config: { damping: 16, stiffness: 90 } });
  const out = ease(f, W.len - 24, W.len);
  const footT = ease(f, 200, 235);
  const cardT = spring({ frame: f - 10, fps: 60, config: { damping: 17, stiffness: 90 } });
  return (
    <AbsoluteFill style={{ opacity: 1 - out, transform: `translateY(${-out * 60 * u}px) scale(${1 - out * 0.03})`, filter: `blur(${out * 10 * u}px)` }}>
      <div style={{ position: 'absolute', top: 250 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', padding: `0 ${80 * u}px`, opacity: inT, transform: `translateY(${(1 - inT) * 70 * u}px)` }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 40 * u, letterSpacing: '0.08em', color: C.mustard }}>{['১', '২', '৩', '৪', '৫', '৬', '৭', '৮'][n - 1]} / ৮</div>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 104 * u, lineHeight: 1.2, color: C.white, textAlign: 'center', marginTop: 16 * u, textWrap: 'balance' as never, textShadow: `0 0 ${60 * u}px rgba(47,209,142,0.25)` }}>{title}</div>
        <Sub size={36} style={{ marginTop: 18 * u }}>{en}</Sub>
      </div>
      <div style={{ position: 'absolute', top: 880 * u, left: 110 * u, right: 110 * u, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 40 * u, transform: `translateY(${(1 - cardT) * 80 * u}px) scale(${1.04 + cardT * 0.06})`, transformOrigin: 'center top', opacity: cardT }}>{children}</div>
      <div style={{ position: 'absolute', bottom: 300 * u, left: 0, right: 0, padding: `0 ${90 * u}px`, textAlign: 'center', opacity: footT, transform: `translateY(${(1 - footT) * 30 * u}px)` }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 56 * u, lineHeight: 1.35, color: 'rgba(234,243,238,0.95)' }}>{foot}</div>
        <Sub size={32} style={{ marginTop: 12 * u }}>{footEn}</Sub>
      </div>
    </AbsoluteFill>
  );
};

const Hi: React.FC<React.PropsWithChildren> = ({ children }) => <span style={{ color: C.greenBright }}>{children}</span>;

/* ───────────────────────── opening (0–6 s) ───────────────────────── */
const Opening: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const a = spring({ frame: f - 10, fps: 60, config: { damping: 16, stiffness: 70 } });
  const aOut = ease(f, 160, 185);
  const b = spring({ frame: f - 190, fps: 60, config: { damping: 13 } });
  const out = ease(f, 336, 360);
  const roman = 'amar bhasha';
  const n = Math.max(0, Math.min(roman.length, Math.floor((f - 40) / 5)));
  const morph = ease(f, 110, 136);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', opacity: 1 - out, filter: `blur(${out * 12 * u}px)` }}>
      <div style={{ position: 'absolute', top: 560 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', opacity: Math.min(a, 1 - aOut), transform: `translateY(${(1 - a) * 60 * u - aOut * 50 * u}px)`, padding: `0 ${90 * u}px` }}>
        <div style={{ position: 'relative', height: 200 * u, display: 'grid', placeItems: 'center', width: '100%' }}>
          <div style={{ position: 'absolute', fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 120 * u, color: C.mustard, opacity: 1 - morph, filter: `blur(${morph * 12 * u}px)` }}>{roman.slice(0, n)}</div>
          <div style={{ position: 'absolute', fontFamily: FONT_BN, fontWeight: 700, fontSize: 150 * u, color: C.white, opacity: morph, transform: `scale(${0.85 + morph * 0.15})`, textShadow: `0 0 ${80 * u * morph}px rgba(47,209,142,0.7)` }}>আমার ভাষা</div>
          <Burst at={112} x={0.5} y={0.3} count={30} seed="wo" spread={0.35} colors={[C.greenBright, '#ffffff', C.mustard]} />
        </div>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 84 * u, lineHeight: 1.3, color: C.white, textAlign: 'center', marginTop: 60 * u }}>ইংরেজি অক্ষরে লিখি,<br />বাংলা চাই <Hi>ঠিক বানানে।</Hi></div>
        <Sub size={36} style={{ marginTop: 18 * u }}>We type in English letters. We want Bangla, spelled right.</Sub>
      </div>
      <div style={{ position: 'absolute', top: 640 * u, left: 0, right: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', opacity: b, transform: `translateY(${(1 - b) * 70 * u}px)` }}>
        <Img src={staticFile('assets/logo_512.png')} style={{ width: 280 * u, height: 280 * u, borderRadius: 64 * u, transform: `scale(${0.5 + b * 0.5}) rotate(${(1 - b) * -12}deg)`, boxShadow: `0 ${30 * u}px ${90 * u}px rgba(0,0,0,0.6), 0 0 ${150 * u}px rgba(224,48,63,0.5)` }} />
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 92 * u, lineHeight: 1.3, color: C.white, textAlign: 'center', marginTop: 60 * u }}>তাই <span style={{ color: C.red, textShadow: `0 0 ${50 * u}px rgba(224,48,63,0.6)` }}>একুশে নেক্সট</span><br />বানানো হয়েছে অন্যভাবে।</div>
        <Sub size={38} style={{ marginTop: 20 * u }}>So Ekushe Next was built differently. Here’s how.</Sub>
      </div>
      <Burst at={192} y={0.36} count={40} seed="wl" spread={0.7} />
      <Sweep at={200} dur={50} strength={0.14} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 1. its own dictionary, standard spelling first ───────────────────────── */
const Ch1: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const w1 = typedState(f, 'porikkha', 40, 6, ['প', 'পরি', 'পরিক', 'পরীক্ষ', 'পরীক্ষা']);
  const w2 = typedState(f, 'sompurno', 160, 6, ['স', 'সম', 'সম্প', 'সম্পূর', 'সম্পূর্ণ']);
  const first = f >= w1.doneAt + 40;
  const done = first ? 'পরীক্ষা ' : '';
  const live = f >= 160 ? w2.live : first ? '' : w1.live;
  const second = f >= w2.doneAt + 40;
  return (
    <Chapter n={1} title={<>নিজের অভিধান,<br /><Hi>প্রমিত বানান</Hi> আগে</>} en="Its own dictionary. The standard spelling comes first." foot="বাংলা একাডেমির প্রমিত বানান সবসময় প্রথমে আসে।" footEn="Bangla Academy standard spellings always rank first.">
      <Glass glow={first ? 0.6 : 0.2} style={{ width: '100%' }}>
        <Keys text={f >= 160 ? 'sompurno' : 'porikkha'} at={f >= 160 ? 160 : 40} every={6} />
        <div style={{ height: 30 * u }} />
        <Field done={second ? 'পরীক্ষা সম্পূর্ণ ' : done} live={second ? '' : live} />
        <div style={{ height: 50 * u }} />
        {f < 160 ? (
          <Bar at={w1.doneAt - 6} pickAt={w1.doneAt + 30} cands={[{ t: 'পরীক্ষা', tag: 'প্রমিত' }, { t: 'পরীক্ষায়' }, { t: 'পরিক্ষা', tag: 'ভুল', dim: true }]} />
        ) : (
          <Bar at={w2.doneAt - 6} pickAt={w2.doneAt + 30} cands={[{ t: 'সম্পূর্ণ', tag: 'প্রমিত' }, { t: 'সম্পূর্ন', tag: 'ভুল', dim: true }]} />
        )}
        <Sweep at={w1.doneAt + 30} dur={40} strength={0.15} />
        <Sweep at={w2.doneAt + 30} dur={40} strength={0.15} />
      </Glass>
    </Chapter>
  );
};

/* ───────────────────────── 2. understands everyday Bangla ───────────────────────── */
const Bubble: React.FC<React.PropsWithChildren<{ at: number; me?: boolean }>> = ({ at, me = true, children }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const t = spring({ frame: f - at, fps: 60, config: { damping: 14, stiffness: 130 } });
  if (f < at) return null;
  return <div style={{ alignSelf: me ? 'flex-end' : 'flex-start', padding: `${18 * u}px ${34 * u}px`, borderRadius: 38 * u, borderBottomRightRadius: me ? 10 * u : 38 * u, borderBottomLeftRadius: me ? 38 * u : 10 * u, background: me ? 'linear-gradient(160deg, #13A06A, #0B6B4A)' : 'rgba(255,255,255,0.1)', fontFamily: FONT_BN, fontWeight: 600, fontSize: 60 * u, color: C.white, opacity: t, transform: `translateY(${(1 - t) * 60 * u}px) scale(${0.85 + t * 0.15})`, transformOrigin: me ? 'right bottom' : 'left bottom' }}>{children}</div>;
};
const Ch2: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  // "kemon ase": after কেমন, "ase" is আছে (alone it could be আসে)
  const typing = f >= 40 && f < 196;
  const roman = 'kemon ase';
  const n = Math.max(0, Math.min(roman.length, Math.floor((f - 40) / 6) + 1));
  const live = !typing ? '' : n <= 5 ? ['ক', 'কে', 'কেম', 'কেমো', 'কেমন'][n - 1] : n === 6 ? '' : ['আ', 'আস', 'আছে'][Math.min(2, n - 7)];
  const done = typing && n >= 6 ? 'কেমন ' : '';
  return (
    <Chapter n={2} title={<>মুখের ভাষা <Hi>বোঝে</Hi></>} en="It understands Bangla the way we actually chat." foot="আগের শব্দ দেখে ঠিক শব্দটা বেছে নেয়: কেমন-এর পরে ase মানে আছে।" footEn="It reads the word before: after কেমন, “ase” means আছে.">
      <Glass glow={f > 196 ? 0.5 : 0.2} style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: 22 * u }}>
        <Bubble at={10} me={false}>কী খবর?</Bubble>
        <Bubble at={200}>কেমন আছে সবাই?</Bubble>
        <Bubble at={250} me={false}>ভালো আছি, তুমি?</Bubble>
        <div style={{ height: 10 * u }} />
        {f < 196 && <Keys text={roman} at={40} every={6} />}
        {f < 196 && <Field done={done} live={live} size={66} />}
        {f < 196 && <Bar at={40 + 8 * 6} pickAt={40 + 9 * 6 + 20} cands={[{ t: 'আছে', tag: 'কেমন-এর পরে' }, { t: 'আসে' }, { t: 'আছ' }]} />}
      </Glass>
    </Chapter>
  );
};

/* ───────────────────────── 3. exact spelling marks ───────────────────────── */
const MarkRow: React.FC<{ at: number; roman: string; mark: string; name: string; out: string; en: string }> = ({ at, roman, mark, name, out, en }) => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const t = spring({ frame: f - at, fps: 60, config: { damping: 14 } });
  const n = Math.max(0, Math.min(roman.length, Math.floor((f - at - 10) / 6) + 1));
  const done = f >= at + 10 + roman.length * 6 + 10;
  const pop = done ? spring({ frame: f - (at + 10 + roman.length * 6 + 10), fps: 60, config: { damping: 10 } }) : 0;
  const parts = roman.split(mark);
  const typed = roman.slice(0, n);
  return (
    <Glass glow={pop * 0.6} style={{ width: '100%', opacity: t, transform: `translateY(${(1 - t) * 50 * u}px)` }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 20 * u }}>
        <div style={{ fontFamily: MONO, fontWeight: 700, fontSize: 64 * u, color: C.white }}>
          {typed.length <= parts[0].length ? typed : <>{parts[0]}<span style={{ color: C.mustard, background: 'rgba(233,185,73,0.18)', borderRadius: 12 * u, padding: `0 ${8 * u}px` }}>{typed.slice(parts[0].length, parts[0].length + mark.length)}</span>{typed.slice(parts[0].length + mark.length)}</>}
        </div>
        <div style={{ fontFamily: FONT_BN, fontSize: 70 * u, color: 'rgba(234,243,238,0.5)' }}>→</div>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 96 * u, color: C.greenBright, opacity: pop, transform: `scale(${0.6 + pop * 0.4})`, textShadow: `0 0 ${50 * u}px rgba(47,209,142,0.6)` }}>{out}</div>
      </div>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 16 * u, marginTop: 14 * u, opacity: ease(f, at + 20, at + 40) }}>
        <span style={{ fontFamily: MONO, fontWeight: 700, fontSize: 40 * u, color: '#2A1C00', background: C.mustard, borderRadius: 12 * u, padding: `${2 * u}px ${14 * u}px` }}>{mark}</span>
        <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 44 * u, color: C.white }}>{name}</span>
        <span style={{ fontFamily: FONT_LATIN, fontSize: 30 * u, color: 'rgba(234,243,238,0.6)' }}>{en}</span>
      </div>
    </Glass>
  );
};
const Ch3: React.FC = () => (
  <Chapter n={3} title={<>যখন চান, <Hi>ঠিক যে বানান</Hi></>} en="Exact spelling, whenever you want it." foot="ছোট একটা চিহ্ন দিলেই হসন্ত, চন্দ্রবিন্দু, রেফ: বানান থাকে আপনার হাতে।" footEn="One small mark gives hasanta, chandrabindu or reph. You stay in control.">
    <MarkRow at={30} roman="bak,," mark=",," name="হসন্ত" out="বাক্‌" en="hasanta" />
    <MarkRow at={120} roman="ca^d" mark="^" name="চন্দ্রবিন্দু" out="চাঁদ" en="chandrabindu" />
  </Chapter>
);

/* ───────────────────────── 4. conjunct hints ───────────────────────── */
const Ch4: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const hints: [string, string][] = [['sw', 'স্ব'], ['st', 'স্ত'], ['sth', 'স্থ'], ['sm', 'স্ম'], ['sp', 'স্প'], ['sk', 'স্ক']];
  const pickAt = 230;
  return (
    <Chapter n={4} title={<>লিখতে লিখতে <Hi>যুক্তবর্ণ</Hi> শেখায়</>} en="It teaches conjuncts while you type." foot="পরের কোন কী চাপলে কোন যুক্তবর্ণ হবে, চাপার আগেই দেখায়।" footEn="It shows which key makes which conjunct, before you press it.">
      <Glass glow={f > pickAt ? 0.55 : 0.2} style={{ width: '100%' }}>
        <Keys text={f < pickAt ? 'as' : 'asth'} at={40} every={f < pickAt ? 14 : 14} />
        <div style={{ height: 30 * u }} />
        <Field live={f < 54 ? 'আ' : f < pickAt + 28 ? 'আস' : 'আস্থা'} />
        <div style={{ height: 40 * u }} />
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 18 * u }}>
          {hints.map(([k, b], i) => {
            const t = spring({ frame: f - 70 - i * 8, fps: 60, config: { damping: 13, stiffness: 160 } });
            const on = k === 'sth' && f >= pickAt;
            return (
              <div key={k} style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: `${18 * u}px 0`, borderRadius: 26 * u, background: on ? C.greenBright : 'rgba(255,255,255,0.07)', border: `${2 * u}px solid ${on ? C.greenBright : 'rgba(255,255,255,0.15)'}`, boxShadow: on ? `0 0 ${60 * u}px rgba(47,209,142,0.6)` : 'none', opacity: f < 70 ? 0 : t, transform: `scale(${(0.7 + t * 0.3) * (on ? interpolate(f, [pickAt, pickAt + 6, pickAt + 16], [1, 1.12, 1], clamp) : 1)})` }}>
                <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 82 * u, color: on ? '#03170F' : C.white, lineHeight: 1.2 }}>{b}</span>
                <span style={{ fontFamily: MONO, fontWeight: 700, fontSize: 34 * u, color: on ? '#03170F' : C.mustard }}>{k}</span>
              </div>
            );
          })}
        </div>
        <Burst at={pickAt + 28} x={0.5} y={0.45} count={24} seed="cj" spread={0.3} colors={[C.greenBright, '#ffffff', C.mustard]} />
      </Glass>
    </Chapter>
  );
};

/* ───────────────────────── 5. learns from you, fix in one click ───────────────────────── */
const Ch5: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const fixAt = 150;
  const learned = f >= 250;
  return (
    <Chapter n={5} title={<>আপনার মতো করে <Hi>শেখে</Hi></>} en="It learns the way you write." foot="ভুল শব্দ এক ক্লিকে ঠিক করুন; পরের বার ঠিক শব্দটাই আসবে।" footEn="Fix a wrong word in one click. Next time, the right word comes first.">
      <Glass glow={learned ? 0.6 : 0.2} style={{ width: '100%' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 40 * u, color: C.mustard, textAlign: 'center', marginBottom: 24 * u }}>ভুল শব্দ ঠিক করুন</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 20 * u, padding: `${22 * u}px ${28 * u}px`, borderRadius: 30 * u, background: 'rgba(255,255,255,0.06)', border: `${2 * u}px solid rgba(255,255,255,0.14)` }}>
          <div style={{ flex: 1 }}>
            <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 70 * u, color: f >= fixAt ? 'rgba(234,243,238,0.4)' : C.white, textDecoration: f >= fixAt ? 'line-through' : 'none' }}>আছে</div>
            <div style={{ fontFamily: FONT_LATIN, fontSize: 30 * u, color: 'rgba(234,243,238,0.6)' }}>typed: ase · meant আসে</div>
          </div>
          {['আসে', 'এসে'].map((w, i) => {
            const on = i === 0 && f >= fixAt;
            const t = spring({ frame: f - 40 - i * 10, fps: 60, config: { damping: 14 } });
            return <div key={w} style={{ padding: `${14 * u}px ${28 * u}px`, borderRadius: 24 * u, fontFamily: FONT_BN, fontWeight: 600, fontSize: 56 * u, background: on ? C.greenBright : 'rgba(255,255,255,0.08)', color: on ? '#03170F' : C.white, border: `${2 * u}px solid ${on ? C.greenBright : 'rgba(255,255,255,0.18)'}`, opacity: t, transform: `scale(${on ? interpolate(f, [fixAt, fixAt + 6, fixAt + 16], [1, 1.14, 1], clamp) : 0.8 + t * 0.2})` }}>{w}</div>;
          })}
        </div>
        {/* the pointer */}
        <div style={{ position: 'relative', height: 0 }}>
          <div style={{ position: 'absolute', right: interpolate(f, [80, fixAt - 4], [40, 330], clamp) * u, top: interpolate(f, [80, fixAt - 4], [60, -90], clamp) * u, fontSize: 80 * u, opacity: ease(f, 70, 90) * (1 - ease(f, fixAt + 40, fixAt + 60)), transform: `scale(${f >= fixAt - 4 && f < fixAt + 8 ? 0.85 : 1})` }}>👆</div>
        </div>
        <div style={{ height: 50 * u }} />
        <div style={{ opacity: ease(f, 200, 230), transform: `translateY(${(1 - ease(f, 200, 230)) * 30 * u}px)` }}>
          <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 38 * u, color: 'rgba(234,243,238,0.7)', textAlign: 'center', marginBottom: 14 * u }}>পরের বার ase লিখলে</div>
          <Bar at={210} pickAt={250} cands={[{ t: 'আসে', tag: 'শিখে নিয়েছে' }, { t: 'আছে' }]} />
        </div>
        <Burst at={fixAt} x={0.3} y={0.52} count={20} seed="fx" spread={0.25} colors={[C.greenBright, '#ffffff']} />
      </Glass>
    </Chapter>
  );
};

/* ───────────────────────── 6. Backspace brings the letters back ───────────────────────── */
const Ch6: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const roman = 'amar';
  const n = Math.max(0, Math.min(roman.length, Math.floor((f - 40) / 7) + 1));
  const space = 40 + roman.length * 7 + 12;
  const bs = 170;
  const forming = ['আ', 'আম', 'আমা', 'আমার'];
  let done = '', live = '', liveRoman = false;
  if (f < space) live = f >= 40 ? forming[n - 1] : '';
  else if (f < bs) done = 'আমার ';
  else { live = 'amar'; liveRoman = true; }
  const press = interpolate(f, [bs - 6, bs, bs + 12], [0, 1, 0], clamp);
  return (
    <Chapter n={6} title={<>এক <Hi>ব্যাকস্পেসে</Hi> ফিরে আসে</>} en="One Backspace brings your letters back." foot="ভুল রূপান্তর হলে আবার পুরো শব্দ টাইপ করতে হয় না।" footEn="If a word came out wrong, you don’t retype it.">
      <Glass glow={f > bs ? 0.55 : 0.2} style={{ width: '100%' }}>
        <Field done={done} live={live} liveRoman={liveRoman} size={90} glow={f > bs ? 0.8 : 0} />
        <div style={{ height: 60 * u }} />
        <div style={{ display: 'flex', justifyContent: 'center', gap: 30 * u }}>
          {[['space', space], ['⌫  Backspace', bs]].map(([label, at]) => {
            const p = interpolate(f, [(at as number) - 6, at as number, (at as number) + 12], [0, 1, 0], clamp);
            const lit = (label as string).startsWith('⌫') && f >= bs;
            return <div key={label as string} style={{ padding: `${24 * u}px ${44 * u}px`, borderRadius: 26 * u, fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 46 * u, color: lit ? '#03170F' : C.white, background: lit ? C.greenBright : `rgba(255,255,255,${0.08 + p * 0.3})`, border: `${2 * u}px solid rgba(255,255,255,0.2)`, boxShadow: `0 ${8 * u * (1 - p)}px 0 rgba(0,0,0,0.5)${lit ? `, 0 0 ${60 * u}px rgba(47,209,142,0.5)` : ''}`, transform: `translateY(${p * 6 * u}px)` }}>{label}</div>;
          })}
        </div>
        <Burst at={bs + 2} x={0.5} y={0.42} count={22} seed="bs" spread={0.3} colors={[C.mustard, '#ffffff']} />
        <Sweep at={bs} dur={40} strength={0.18 * (press + 0.5)} />
      </Glass>
    </Chapter>
  );
};

/* ───────────────────────── 7. private ───────────────────────── */
const Ch7: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const lock = spring({ frame: f - 20, fps: 60, config: { damping: 11 } });
  const pw = 'amar123';
  const n = Math.max(0, Math.min(pw.length, Math.floor((f - 150) / 6) + 1));
  return (
    <Chapter n={7} title={<>আপনার লেখা <Hi>আপনারই</Hi> থাকে</>} en="What you type stays with you." foot="ইন্টারনেট ছাড়াই চলে, কোথাও কিছু পাঠায় না; পাসওয়ার্ডের ঘরে নিজে থেকেই ইংরেজি।" footEn="Works offline and sends nothing anywhere. Password boxes stay English.">
      <div style={{ position: 'relative', width: 300 * u, height: 300 * u, display: 'grid', placeItems: 'center', transform: `scale(${0.5 + lock * 0.5})`, opacity: lock }}>
        {[0, 18, 36].map((d) => {
          const r = ((f - 20 - d + 600) % 90) / 90;
          return <div key={d} style={{ position: 'absolute', inset: 0, borderRadius: 999, border: `${3 * u}px solid rgba(47,209,142,${0.5 * (1 - r)})`, transform: `scale(${0.8 + r * 0.9})` }} />;
        })}
        <svg width={170 * u} height={170 * u} viewBox="0 0 24 24" fill="none" stroke={C.greenBright} strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" style={{ filter: `drop-shadow(0 0 ${30 * u}px rgba(47,209,142,0.7))` }}>
          <rect x="5" y="11" width="14" height="10" rx="2.5" /><path d={`M8 11V8a4 4 0 018 0v${3 - ease(f, 60, 80) * 0}`} /><circle cx="12" cy="16" r="1.4" fill={C.greenBright} />
        </svg>
      </div>
      <div style={{ display: 'flex', gap: 20 * u, opacity: ease(f, 60, 90) }}>
        {['ইন্টারনেট লাগে না', 'কিছু পাঠায় না'].map((t, i) => (
          <div key={t} style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 42 * u, color: C.white, padding: `${12 * u}px ${28 * u}px`, borderRadius: 99, background: 'rgba(47,209,142,0.14)', border: `${2 * u}px solid rgba(47,209,142,0.4)`, transform: `translateY(${(1 - ease(f, 60 + i * 10, 90 + i * 10)) * 30 * u}px)` }}>✓ {t}</div>
        ))}
      </div>
      <Glass style={{ width: '86%', opacity: ease(f, 120, 145) }}>
        <div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 34 * u, color: 'rgba(234,243,238,0.7)', marginBottom: 14 * u }}>Password</div>
        <div style={{ display: 'flex', alignItems: 'center', padding: `${18 * u}px ${26 * u}px`, borderRadius: 22 * u, background: 'rgba(0,0,0,0.3)', border: `${2 * u}px solid rgba(255,255,255,0.16)` }}>
          <span style={{ fontFamily: MONO, fontWeight: 700, fontSize: 58 * u, color: C.white, letterSpacing: '0.12em' }}>{f >= 150 ? '•'.repeat(Math.max(0, n - 1)) + (f < 150 + pw.length * 6 + 12 ? pw[n - 1] : '•') : ''}</span>
          <span style={{ display: 'inline-block', width: 4 * u, height: 58 * u, background: C.greenBright, marginLeft: 6 * u, opacity: Math.floor(f / 18) % 2 ? 0.25 : 1 }} />
          <span style={{ marginLeft: 'auto', fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 30 * u, color: '#03170F', background: C.mustard, borderRadius: 99, padding: `${4 * u}px ${16 * u}px`, opacity: ease(f, 200, 220) }}>EN</span>
        </div>
      </Glass>
    </Chapter>
  );
};

/* ───────────────────────── 8. one engine, every device ───────────────────────── */
const Ch8: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const core = spring({ frame: f - 20, fps: 60, config: { damping: 12 } });
  const devices: { k: 'win' | 'linux' | 'mac' | 'android' | 'ios'; name: string; ready: boolean }[] = [
    { k: 'win', name: 'Windows', ready: true }, { k: 'linux', name: 'Linux', ready: true },
    { k: 'mac', name: 'macOS', ready: false }, { k: 'android', name: 'Android', ready: false }, { k: 'ios', name: 'iPhone · iPad', ready: false },
  ];
  const R = 360;
  return (
    <Chapter n={8} title={<>এক ইঞ্জিন, <Hi>সব জায়গায়</Hi></>} en="One engine, on every device." foot="একই অভিধান, একই নিয়ম: কম্পিউটারে যেমন, ফোনেও তেমন।" footEn="The same dictionary and rules, on computer and phone alike.">
      <div style={{ position: 'relative', width: 1100 * u, height: 920 * u }}>
        <svg width={1100 * u} height={920 * u} style={{ position: 'absolute', inset: 0 }}>
          {devices.map((d, i) => {
            const a = -Math.PI / 2 + (i / devices.length) * Math.PI * 2;
            const x = 550 + Math.cos(a) * R, y = 460 + Math.sin(a) * R;
            const t = ease(f, 60 + i * 14, 100 + i * 14);
            return <line key={d.k} x1={550 * u} y1={460 * u} x2={(550 + (x - 550) * t) * u} y2={(460 + (y - 460) * t) * u} stroke={d.ready ? C.greenBright : 'rgba(234,243,238,0.35)'} strokeWidth={4 * u} strokeDasharray={d.ready ? 'none' : `${10 * u} ${10 * u}`} />;
          })}
        </svg>
        <div style={{ position: 'absolute', left: 550 * u, top: 460 * u, transform: `translate(-50%,-50%) scale(${0.5 + core * 0.5})`, opacity: core }}>
          <Img src={staticFile('assets/logo_512.png')} style={{ width: 220 * u, height: 220 * u, borderRadius: 52 * u, boxShadow: `0 0 ${120 * u}px rgba(224,48,63,0.6)` }} />
        </div>
        {devices.map((d, i) => {
          const a = -Math.PI / 2 + (i / devices.length) * Math.PI * 2;
          const x = 550 + Math.cos(a) * R, y = 460 + Math.sin(a) * R;
          const t = spring({ frame: f - 90 - i * 14, fps: 60, config: { damping: 12 } });
          return (
            <div key={d.k} style={{ position: 'absolute', left: x * u, top: y * u, transform: `translate(-50%,-50%) scale(${0.5 + t * 0.5})`, opacity: t, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8 * u }}>
              <div style={{ width: 150 * u, height: 150 * u, borderRadius: 40 * u, display: 'grid', placeItems: 'center', color: d.ready ? '#03170F' : C.white, background: d.ready ? C.greenBright : 'rgba(255,255,255,0.1)', border: `${2 * u}px solid ${d.ready ? C.greenBright : 'rgba(255,255,255,0.25)'}`, boxShadow: d.ready ? `0 0 ${60 * u}px rgba(47,209,142,0.55)` : 'none' }}><OsIcon kind={d.k} size={84 * u} /></div>
              <div style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 32 * u, color: C.white }}>{d.name}</div>
              <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 28 * u, color: d.ready ? C.greenBright : C.mustard }}>{d.ready ? '৩০ অক্টোবর' : 'শীঘ্রই'}</div>
            </div>
          );
        })}
      </div>
    </Chapter>
  );
};

/* ───────────────────────── ending (62–70 s) ───────────────────────── */
const Ending: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const pop = spring({ frame: f - 6, fps: 60, config: { damping: 11, mass: 0.8 } });
  const date = spring({ frame: f - 60, fps: 60, config: { damping: 13 } });
  const later = ease(f, 120, 150);
  const name = graphemes('একুশে নেক্সট');
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [0, 14], [0, 1], clamp) }}>
      <Img src={staticFile('assets/logo_512.png')} style={{ width: 300 * u, height: 300 * u, borderRadius: 68 * u, transform: `scale(${0.5 + pop * 0.5}) rotate(${(1 - pop) * -12}deg)`, opacity: Math.min(1, pop * 1.4), boxShadow: `0 ${30 * u}px ${90 * u}px rgba(0,0,0,0.6), 0 0 ${150 * u}px rgba(224,48,63,0.55)` }} />
      <div style={{ display: 'flex', marginTop: 40 * u }}>
        {name.map((g, i) => {
          const t = spring({ frame: f - 20 - i * 3, fps: 60, config: { damping: 14 } });
          return <span key={i} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 160 * u, color: C.white, whiteSpace: 'pre', display: 'inline-block', opacity: t, transform: `translateY(${(1 - t) * 60 * u}px)`, textShadow: `0 0 ${60 * u}px rgba(224,48,63,0.5)` }}>{g}</span>;
        })}
      </div>
      <div style={{ marginTop: 40 * u, opacity: date, transform: `scale(${0.8 + date * 0.2})`, textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 86 * u, color: C.greenBright, textShadow: `0 0 ${60 * u}px rgba(47,209,142,0.5)` }}>৩০ অক্টোবর থেকে</div>
        <div style={{ display: 'flex', gap: 24 * u, justifyContent: 'center', marginTop: 18 * u }}>
          {(['win', 'linux'] as const).map((k) => (
            <div key={k} style={{ display: 'flex', alignItems: 'center', gap: 14 * u, padding: `${12 * u}px ${28 * u}px`, borderRadius: 99, background: C.greenBright, color: '#03170F', fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 40 * u }}><OsIcon kind={k} size={42 * u} />{k === 'win' ? 'Windows' : 'Linux'}</div>
          ))}
        </div>
        <Sub size={34} style={{ marginTop: 16 * u }}>From 30 October on Windows and Linux</Sub>
      </div>
      <div style={{ marginTop: 34 * u, opacity: later, textAlign: 'center' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 50 * u, color: C.mustard }}>macOS, Android, iPhone ও iPad শীঘ্রই</div>
        <Sub size={30} style={{ marginTop: 8 * u }}>macOS, Android, iPhone and iPad coming soon</Sub>
      </div>
      <div style={{ marginTop: 56 * u }}><AddressPill kind="web" text="www.ekushenext.com" at={200} /></div>
      <Burst at={6} y={0.3} count={44} seed="we" spread={0.7} />
      <Burst at={60} y={0.52} count={30} seed="wd" spread={0.5} colors={[C.greenBright, '#ffffff', C.mustard]} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── the film ───────────────────────── */
const CHAPTERS = [Ch1, Ch2, Ch3, Ch4, Ch5, Ch6, Ch7, Ch8];

export const WhyDifferent: React.FC = () => {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Backdrop sun={interpolate(f, [0, 300, 3600, 3900], [0.55, 0.35, 0.35, 0.8], clamp)} field={0.85} warmth={0.6} />
      <Petals count={16} opacity={0.35} />
      <Sequence from={W.open} durationInFrames={W.ch}><Opening /></Sequence>
      {CHAPTERS.map((Ch, i) => (
        <Sequence key={i} from={W.ch + i * W.len} durationInFrames={W.len}><Ch /></Sequence>
      ))}
      <Sequence from={W.end} durationInFrames={WHY_FRAMES - W.end}><Ending /></Sequence>
      <Audio src={staticFile('music/why_different.wav')} />
    </AbsoluteFill>
  );
};
