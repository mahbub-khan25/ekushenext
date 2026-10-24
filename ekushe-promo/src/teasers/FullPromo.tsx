import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals, ShaheedMinar } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { AddressPill, Burst, Streaks, Sub, Sweep, TypeOn, graphemes } from './common';
import { Bijoy, Converge, Dictionary, Logo, NextWord, Phonetic, Private, Prologue, Speed, Studio } from './Reveal';
import { Phone } from './Everywhere';

/** The full promo: the remembrance, the name, nine features, then every device (Windows and Linux
 * from 30 October, with the Ubuntu PPA and Fedora COPR; macOS, Android, iPhone and iPad later),
 * one engine, when and where, and the website. 112 s at 60 fps, 2K, 16:9 and 9:16.
 * music/full_promo.py is composed to the same marks. */
export const P = {
  prologue: 0,
  logo: 420,
  features: 660,
  devices: 2010,
  win: 2190,
  linux: 2790,
  mac: 3630,
  android: 4170,
  ios: 4860,
  engine: 5460,
  avail: 5820,
  end: 6300,
} as const;
export const FEAT_LEN = 150;
export const FULL_PROMO_FRAMES = 6720;
const MONO = "'DejaVu Sans Mono', 'Menlo', 'Consolas', monospace";

const Big: React.FC<{ children: React.ReactNode; size: number; style?: React.CSSProperties }> = ({ children, size, style }) => {
  const { u } = useLayout();
  return <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: size * u, lineHeight: 1.18, color: C.white, textAlign: 'center', ...style }}>{children}</div>;
};

/* ───────────────────────── icons ───────────────────────── */
type Os = 'win' | 'mac' | 'linux' | 'android' | 'ios';
export const OsIcon: React.FC<{ kind: Os; size: number }> = ({ kind, size }) => (
  <svg width={size} height={size} viewBox="0 0 24 24">
    {kind === 'win' && <path fill="currentColor" d="M3 5.6l7.4-1v7H3zm8.4-1.2L21 3v8.6h-9.6zM3 12.6h7.4v7l-7.4-1zm8.4 0H21V21l-9.6-1.4z" />}
    {(kind === 'mac') && <path fill="currentColor" d="M16.4 12.6c0-2.4 2-3.6 2.1-3.7-1.2-1.7-3-1.9-3.6-2-1.5-.2-3 .9-3.8.9-.8 0-2-.9-3.3-.9-1.7 0-3.3 1-4.2 2.5-1.8 3.1-.5 7.7 1.3 10.2.9 1.2 1.9 2.6 3.2 2.6 1.3-.1 1.8-.8 3.3-.8s2 .8 3.3.8c1.4 0 2.3-1.2 3.1-2.5 1-1.4 1.4-2.8 1.4-2.9 0 0-2.8-1.1-2.8-4.2zM14 5.3c.7-.9 1.2-2 1-3.3-1 0-2.3.7-3 1.6-.6.7-1.2 1.9-1 3.1 1.1.1 2.3-.6 3-1.4z" />}
    {kind === 'ios' && <><rect x="6.5" y="2.5" width="11" height="19" rx="2.6" fill="none" stroke="currentColor" strokeWidth="1.7" /><rect x="10" y="4.3" width="4" height="1.2" rx="0.6" fill="currentColor" /></>}
    {kind === 'linux' && <path fill="currentColor" d="M12 2c-2.2 0-3.4 1.9-3.4 4.3 0 1.4.3 2.3-.6 3.7-1 1.6-2.5 3.6-2.5 6.1 0 .6.1 1.1.2 1.6-.9.4-1.7.9-1.7 1.7 0 1.1 1.7 1.3 3.2 1.7 1.2.3 2 .9 3 .9.8 0 1.4-.4 1.8-.9.4.5 1 .9 1.8.9 1 0 1.8-.6 3-.9 1.5-.4 3.2-.6 3.2-1.7 0-.8-.8-1.3-1.7-1.7.1-.5.2-1 .2-1.6 0-2.5-1.5-4.5-2.5-6.1-.9-1.4-.6-2.3-.6-3.7C15.4 3.9 14.2 2 12 2z" />}
    {kind === 'android' && <path fill="currentColor" d="M7.2 8.6h9.6c.5 0 .9.4.9.9v7.2c0 .5-.4.9-.9.9h-.9v2.5a1.1 1.1 0 01-2.2 0v-2.5h-3.4v2.5a1.1 1.1 0 01-2.2 0v-2.5h-.9a.9.9 0 01-.9-.9V9.5c0-.5.4-.9.9-.9zM4.4 9a1.1 1.1 0 011.1 1.1v5a1.1 1.1 0 01-2.2 0v-5C3.3 9.5 3.8 9 4.4 9zm15.2 0a1.1 1.1 0 011.1 1.1v5a1.1 1.1 0 01-2.2 0v-5c0-.6.5-1.1 1.1-1.1zM15.6 3.1l-.9 1.5A5.3 5.3 0 0117.6 7.8H6.4a5.3 5.3 0 012.9-3.2l-.9-1.5a.3.3 0 01.5-.3l.9 1.6a5.8 5.8 0 014.4 0l.9-1.6a.3.3 0 01.5.3zM9.6 5.7a.6.6 0 100 1.2.6.6 0 000-1.2zm4.8 0a.6.6 0 100 1.2.6.6 0 000-1.2z" />}
  </svg>
);
const OS_NAME: Record<Os, string> = { win: 'Windows', mac: 'macOS', linux: 'Linux', android: 'Android', ios: 'iPhone · iPad' };

/* ───────────────────────── features (nine, 2.5 s each) ───────────────────────── */
const chip = (u: number, on = false): React.CSSProperties => ({ fontFamily: FONT_BN, fontWeight: 600, color: on ? '#03170F' : C.white, background: on ? C.greenBright : 'rgba(255,255,255,0.08)', border: `${2 * u}px solid ${on ? C.greenBright : 'rgba(255,255,255,0.2)'}`, borderRadius: 24 * u, padding: `${10 * u}px ${30 * u}px`, boxShadow: on ? `0 0 ${50 * u}px rgba(47,209,142,0.5)` : 'none' });

/** Typed shortcuts that become whole phrases */
const Shortcuts: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const rows: [string, string, number][] = [[';innalillah', 'ইন্না লিল্লাহি ওয়া ইন্না ইলাইহি রাজিউন', 8], [';brb', 'এখুনি আসছি', 62]];
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: (V ? 70 : 54) * u, alignItems: 'center' }}>
      {rows.map(([k, phrase, at]) => {
        const typed = at + 6 + k.length * 3;
        const grow = spring({ frame: f - typed - 4, fps: 60, config: { damping: 13 } });
        const g = graphemes(phrase);
        const shown = g.slice(0, Math.round(g.length * ease(f, typed + 4, typed + 34))).join('');
        return (
          <div key={k} style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 22 : 34) * u, opacity: ease(f, at, at + 10) }}>
            <div style={{ ...chip(u), fontFamily: FONT_LATIN, fontSize: 52 * u, color: C.mustard }}><TypeOn text={k} start={at + 4} every={3} style={{}} caret={C.mustard} /></div>
            <span style={{ fontSize: 56 * u, color: C.greenBright, opacity: grow, transform: `translateX(${(1 - grow) * -20 * u}px) rotate(${V ? 90 : 0}deg)`, display: 'inline-block' }}>→</span>
            <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 76 : 80) * u, color: C.white, minWidth: (V ? 0 : 900) * u, textAlign: V ? 'center' : 'left', textShadow: `0 0 ${40 * u}px rgba(47,209,142,${0.5 * grow})`, maxWidth: (V ? 1260 : 1500) * u }}>{shown}</div>
          </div>
        );
      })}
    </div>
  );
};

const FEATURES: { title: React.ReactNode; sub: string; body: React.ReactNode }[] = [
  { title: 'যেভাবে বলেন, সেভাবে লিখুন', sub: 'Type the way you speak', body: <Phonetic /> },
  { title: 'যুক্তবর্ণ মুখস্থ করতে হবে না', sub: 'No conjunct rules to memorise', body: <Converge /> },
  { title: 'প্রমিত বানান সবার আগে', sub: 'The Bangla Academy standard spelling comes first', body: <Dictionary /> },
  { title: 'পরের শব্দটাও বুঝে নেয়', sub: 'It suggests the next word', body: <NextWord /> },
  { title: 'ছোট্ট শর্টকাট, পুরো বাক্য', sub: 'Short codes for the phrases you write every day', body: <Shortcuts /> },
  { title: 'চোখের পলকের চেয়েও দ্রুত', sub: 'Faster than the blink of an eye', body: <Speed /> },
  { title: 'বিজয় থেকে ইউনিকোড, এক চাপে', sub: 'Old Bijoy text, fixed in one keystroke', body: <Bijoy /> },
  { title: 'একুশে স্টুডিও', sub: 'Writing pad, word X-ray, conjunct guide, spell check', body: <Studio /> },
  { title: 'আপনার লেখা, শুধু আপনার', sub: 'Nothing you type leaves your device', body: <Private /> },
];
const Feature: React.FC<{ i: number; title: React.ReactNode; sub: string; children: React.ReactNode }> = ({ i, title, sub, children }) => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const t = spring({ frame: f, fps: 60, config: { damping: 15, stiffness: 120 } });
  const out = ease(f, FEAT_LEN - 16, FEAT_LEN);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out, transform: `translateX(${-out * 120 * u}px)`, filter: `blur(${out * 10 * u}px)` }}>
      <div style={{ position: 'absolute', top: (V ? 330 : 90) * u, display: 'flex', alignItems: 'center', gap: 16 * u, opacity: t }}>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 30 * u, color: C.greenBright, letterSpacing: '0.2em' }}>{String(i + 1).padStart(2, '0')} / {String(FEATURES.length).padStart(2, '0')}</span>
        <div style={{ width: 200 * u, height: 4 * u, borderRadius: 9, background: 'rgba(255,255,255,0.15)', overflow: 'hidden' }}>
          <div style={{ width: `${((i + ease(f, 0, FEAT_LEN)) / FEATURES.length) * 100}%`, height: '100%', background: C.greenBright }} />
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

/* ───────────────────────── typing in a document, with each system's suggestion box ───────────────────────── */
type Word = [string, string, string[]];
type Popup = 'win' | 'mac' | 'linux';
/** Words typed one after another: the Roman letters appear, the system's own suggestion box follows
 * the word, then the Bangla word is placed. `breakAfter` starts a new line after that word. */
const TypingDoc: React.FC<{ words: Word[]; start: number; per: number; size: number; color: string; popup: Popup; light?: boolean; breakAfter?: number[]; caret?: string }> = ({ words, start, per, size, color, popup, light, breakAfter = [], caret = C.greenBright }) => {
  const f = useCurrentFrame();
  const typeLen = Math.round(per * 0.68);
  const wi = Math.min(words.length - 1, Math.max(0, Math.floor((f - start) / per)));
  const local = f - start - wi * per;
  const typing = f >= start && local < typeLen && f < start + words.length * per;
  const done = f < start ? 0 : Math.min(words.length, wi + (local >= typeLen ? 1 : 0));
  const roman = typing ? words[wi][0].slice(0, Math.max(1, Math.ceil((local / (typeLen - 6)) * words[wi][0].length))) : '';
  const frac = typing ? Math.min(1, local / (typeLen - 6)) : 1;
  const pre = typing ? graphemes(words[wi][1]).slice(0, Math.max(1, Math.round(frac * graphemes(words[wi][1]).length))).join('') : '';
  const cands = typing ? (frac >= 1 ? [words[wi][1], ...words[wi][2]] : [pre, ...words[wi][2].slice(0, 1)]) : [];
  const parts: React.ReactNode[] = [];
  for (let i = 0; i < done; i++) {
    parts.push(<span key={i}>{words[i][1]}{breakAfter.includes(i) ? <br /> : ' '}</span>);
  }
  const s = size;
  const box = (() => {
    if (!typing) return null;
    const o = Math.min(1, local / 5);
    if (popup === 'win') {
      return (
        <div style={{ position: 'absolute', left: -12, top: '108%', minWidth: s * 4.6, padding: `${s * 0.16}px ${s * 0.18}px`, borderRadius: s * 0.22, background: light ? 'rgba(250,252,251,0.92)' : 'rgba(28,40,35,0.9)', border: `1px solid ${light ? 'rgba(0,0,0,0.08)' : 'rgba(255,255,255,0.12)'}`, boxShadow: `0 ${s * 0.3}px ${s * 0.9}px rgba(0,0,0,0.45)`, backdropFilter: 'blur(20px)', opacity: o, zIndex: 5 }}>
          <div style={{ fontFamily: FONT_LATIN, fontSize: s * 0.34, color: light ? '#5A6B64' : 'rgba(234,243,238,0.6)', margin: `0 0 ${s * 0.08}px ${s * 0.14}px` }}>{roman}</div>
          {cands.slice(0, 3).map((c, i) => (
            <div key={i} style={{ display: 'flex', alignItems: 'center', gap: s * 0.22, padding: `${s * 0.06}px ${s * 0.16}px`, borderRadius: s * 0.14, background: i === 0 ? (light ? 'rgba(11,107,74,0.14)' : 'rgba(47,209,142,0.22)') : 'transparent', fontFamily: FONT_BN, fontWeight: 500, fontSize: s * 0.58, color: light ? '#13221B' : '#F2F7F4', lineHeight: 1.35, whiteSpace: 'nowrap' }}>
              <b style={{ fontFamily: FONT_LATIN, fontSize: s * 0.3, color: light ? C.green : C.greenBright, width: s * 0.3 }}>{i + 1}</b>{c}
            </div>
          ))}
        </div>
      );
    }
    if (popup === 'mac') {
      return (
        <div style={{ position: 'absolute', left: -8, top: '110%', display: 'flex', alignItems: 'center', gap: s * 0.08, padding: `${s * 0.08}px ${s * 0.1}px`, borderRadius: s * 0.18, background: 'rgba(246,246,246,0.94)', border: '1px solid rgba(0,0,0,0.12)', boxShadow: `0 ${s * 0.25}px ${s * 0.7}px rgba(0,0,0,0.3)`, opacity: o, zIndex: 5, whiteSpace: 'nowrap' }}>
          {cands.slice(0, 4).map((c, i) => (
            <div key={i} style={{ display: 'flex', alignItems: 'baseline', gap: s * 0.12, padding: `${s * 0.04}px ${s * 0.18}px`, borderRadius: s * 0.12, background: i === 0 ? '#2C7BF6' : 'transparent', color: i === 0 ? '#fff' : '#1D1D1F', fontFamily: FONT_BN, fontWeight: 500, fontSize: s * 0.5 }}>
              <span style={{ fontFamily: FONT_LATIN, fontSize: s * 0.28, opacity: 0.75 }}>{i + 1}</span>{c}
            </div>
          ))}
          <span style={{ fontFamily: FONT_LATIN, fontSize: s * 0.3, color: '#86868B', padding: `0 ${s * 0.12}px` }}>›</span>
        </div>
      );
    }
    return (
      <div style={{ position: 'absolute', left: -6, top: '108%', minWidth: s * 3.8, padding: `${s * 0.08}px 0`, borderRadius: s * 0.08, background: '#2B2B2B', border: '1px solid #4A4A4A', boxShadow: `0 ${s * 0.2}px ${s * 0.6}px rgba(0,0,0,0.5)`, opacity: o, zIndex: 5 }}>
        <div style={{ fontFamily: FONT_LATIN, fontSize: s * 0.32, color: '#BDBDBD', padding: `0 ${s * 0.2}px ${s * 0.06}px`, borderBottom: '1px solid #3C3C3C' }}>{roman}</div>
        {cands.slice(0, 3).map((c, i) => (
          <div key={i} style={{ display: 'flex', alignItems: 'center', gap: s * 0.2, padding: `${s * 0.03}px ${s * 0.2}px`, background: i === 0 ? '#E95420' : 'transparent', fontFamily: FONT_BN, fontSize: s * 0.5, color: '#F5F5F5', lineHeight: 1.4, whiteSpace: 'nowrap' }}>
            <span style={{ fontFamily: FONT_LATIN, fontSize: s * 0.28, opacity: 0.8 }}>{i + 1}.</span>{c}
          </div>
        ))}
      </div>
    );
  })();
  return (
    <div style={{ fontFamily: FONT_BN, fontWeight: 500, fontSize: s, color, lineHeight: 1.55 }}>
      {parts}
      {typing && (
        <span style={{ position: 'relative', textDecoration: 'underline', textDecorationColor: caret, textUnderlineOffset: s * 0.14, textDecorationThickness: s * 0.05 }}>
          {pre}
          {box}
        </span>
      )}
      <span style={{ display: 'inline-block', width: Math.max(2, s * 0.05), height: s * 1.05, background: caret, marginLeft: s * 0.04, verticalAlign: '-0.2em', opacity: Math.floor(f / 18) % 2 ? 0.15 : 1 }} />
    </div>
  );
};

/** A design-size screen (1600 × 1000) scaled into the video */
const Screen: React.FC<{ w: number; enter: number; tilt?: number; children: React.ReactNode; glow?: string }> = ({ w, enter, tilt = 0, children, glow = 'rgba(47,209,142,0.22)' }) => {
  const { u } = useLayout();
  const k = w / 1600;
  return (
    <div style={{ width: w, height: 1000 * k, perspective: 3000 * u }}>
      <div style={{ width: w, height: 1000 * k, borderRadius: 22 * u, overflow: 'hidden', position: 'relative', transform: `rotateX(${(1 - enter) * 26 + 2}deg) rotateY(${tilt}deg) translateY(${(1 - enter) * 200 * u}px) scale(${0.92 + enter * 0.08})`, opacity: Math.min(1, enter * 1.5), boxShadow: `0 0 0 ${3 * u}px rgba(255,255,255,0.12), 0 ${50 * u}px ${140 * u}px -${30 * u}px rgba(0,0,0,0.95), 0 0 ${140 * u}px ${glow}` }}>
        <div style={{ position: 'absolute', left: 0, top: 0, width: 1600, height: 1000, transform: `scale(${k})`, transformOrigin: '0 0' }}>{children}</div>
      </div>
    </div>
  );
};

/** Each device scene: name, when it comes, what it does, and its screen */
const Device: React.FC<{ os: Os; soon: boolean; title: string; sub: string; points: [string, string, number][]; len: number; children: (enter: number, w: number) => React.ReactNode }> = ({ os, soon, title, sub, points, len, children }) => {
  const f = useCurrentFrame();
  const { u, V, width, height } = useLayout();
  const enter = spring({ frame: f - 8, fps: 60, config: { damping: 18, stiffness: 70 } });
  const head = spring({ frame: f, fps: 60, config: { damping: 14 } });
  const out = ease(f, len - 20, len);
  const sw = V ? 1340 * u : 1480 * u;
  const badge = spring({ frame: f - 30, fps: 60, config: { damping: 10 } });
  const accent = soon ? C.mustard : C.greenBright;
  const Badge = (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap: 14 * u, padding: `${10 * u}px ${28 * u}px`, borderRadius: 999, background: soon ? 'rgba(233,185,73,0.14)' : 'rgba(47,209,142,0.16)', border: `${2 * u}px solid ${soon ? 'rgba(233,185,73,0.7)' : 'rgba(47,209,142,0.7)'}`, boxShadow: `0 0 ${50 * u}px ${soon ? 'rgba(233,185,73,0.25)' : 'rgba(47,209,142,0.3)'}`, opacity: Math.min(1, badge * 1.3), transform: `scale(${0.7 + badge * 0.3})`, transformOrigin: V ? 'center' : 'left center' }}>
      <span style={{ width: 16 * u, height: 16 * u, borderRadius: 99, background: accent, boxShadow: `0 0 ${16 * u}px ${accent}` }} />
      <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 48 : 44) * u, color: accent }}>{soon ? 'শীঘ্রই আসছে' : '৩০ অক্টোবর থেকে'}</span>
      <span style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 28 * u, color: 'rgba(234,243,238,0.75)' }}>{soon ? 'Coming later' : 'From October 30'}</span>
    </div>
  );
  const Points = (
    <div style={{ display: 'flex', flexDirection: 'column', gap: (V ? 18 : 22) * u, marginTop: (V ? 0 : 50) * u, alignItems: V ? 'center' : 'flex-start' }}>
      {points.map(([bn, en, at]) => {
        const t = spring({ frame: f - at, fps: 60, config: { damping: 13 } });
        return (
          <div key={bn} style={{ display: 'flex', alignItems: 'center', gap: 18 * u, opacity: t * (1 - out), transform: `translateX(${(1 - t) * (V ? 0 : -50) * u}px) translateY(${(1 - t) * (V ? 30 : 0) * u}px)` }}>
            <span style={{ display: 'grid', placeItems: 'center', width: 46 * u, height: 46 * u, flex: 'none', borderRadius: 99, background: accent, color: '#03170F', fontSize: 28 * u, fontWeight: 800 }}>✓</span>
            <div style={{ textAlign: 'left' }}>
              <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 48 : 46) * u, color: C.white, lineHeight: 1.25 }}>{bn}</div>
              <div style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: (V ? 28 : 27) * u, color: 'rgba(234,243,238,0.65)' }}>{en}</div>
            </div>
          </div>
        );
      })}
    </div>
  );
  return (
    <AbsoluteFill style={{ opacity: 1 - out, transform: `scale(${1 - out * 0.05})` }}>
      <div style={{ position: 'absolute', left: V ? 0 : 130 * u, right: V ? 0 : undefined, top: V ? 170 * u : 0, bottom: V ? undefined : 0, width: V ? undefined : 860 * u, display: 'flex', flexDirection: 'column', justifyContent: 'center', alignItems: V ? 'center' : 'flex-start', textAlign: V ? 'center' : 'left' }}>
        <div style={{ opacity: head, transform: `translateY(${(1 - head) * 40 * u}px)` }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 18 * u, justifyContent: V ? 'center' : 'flex-start', color: C.white }}>
            <div style={{ display: 'grid', placeItems: 'center', width: 84 * u, height: 84 * u, borderRadius: 22 * u, background: 'rgba(255,255,255,0.1)', border: `${2 * u}px solid rgba(255,255,255,0.22)` }}><OsIcon kind={os} size={50 * u} /></div>
            <span style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 54 * u, letterSpacing: '-0.01em' }}>{OS_NAME[os]}</span>
          </div>
          <Big size={V ? 96 : 92} style={{ textAlign: V ? 'center' : 'left', marginTop: 24 * u }}>{title}</Big>
          <Sub size={36} style={{ textAlign: V ? 'center' : 'left', marginTop: 8 * u }}>{sub}</Sub>
          <div style={{ marginTop: 34 * u }}>{Badge}</div>
        </div>
        {!V && Points}
      </div>
      <div style={{ position: 'absolute', left: V ? (width - sw) / 2 : width - sw - 110 * u, top: V ? 860 * u : (height - sw * 0.625) / 2 }}>{children(enter, sw)}</div>
      {V && <div style={{ position: 'absolute', left: 0, right: 0, top: 860 * u + sw * 0.625 + 90 * u, display: 'flex', justifyContent: 'center' }}>{Points}</div>}
      <Burst at={30} x={V ? 0.5 : 0.18} y={V ? 0.2 : 0.42} count={18} seed={`b${os}`} spread={0.25} colors={[accent, '#ffffff', C.coral]} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── Windows ───────────────────────── */
const WIN_WORDS: Word[] = [
  ['priyo', 'প্রিয়', ['প্রিয়ো']], ['ammu,', 'আম্মু,', ['আম্মা,']],
  ['aj', 'আজ', ['আজই']], ['offise', 'অফিসে', ['অফিসের']], ['amar', 'আমার', ['আমরা']], ['prothom', 'প্রথম', ['প্রথমে']], ['din', 'দিন', ['দিনে']], ['chilo.', 'ছিল।', ['ছিলো।']],
  ['shobai', 'সবাই', ['সবাইকে']], ['khub', 'খুব', ['খুবই']], ['bhalo!', 'ভালো!', ['ভাল!']],
];
const WinScreen: React.FC = () => {
  const f = useCurrentFrame();
  // double Shift switches to Bangla at ~1 s
  const sw = interpolate(f, [50, 62], [0, 1], clamp);
  const bangla = f > 58;
  const kc = (at: number) => interpolate(f, [at, at + 4, at + 12], [0, 1, 0], clamp);
  const taskIcon = (bg: string, child: React.ReactNode, key: string, active = false) => (
    <div key={key} style={{ position: 'relative', width: 46, height: 46, borderRadius: 8, display: 'grid', placeItems: 'center', background: active ? 'rgba(255,255,255,0.08)' : 'transparent' }}>
      <div style={{ width: 30, height: 30, borderRadius: 7, background: bg, display: 'grid', placeItems: 'center' }}>{child}</div>
      {active && <div style={{ position: 'absolute', bottom: 2, width: 16, height: 3, borderRadius: 3, background: '#60CDFF' }} />}
    </div>
  );
  return (
    <div style={{ width: 1600, height: 1000, position: 'relative', overflow: 'hidden', background: 'radial-gradient(120% 90% at 30% 20%, #1E5BA8 0%, #0D2B5C 45%, #061532 100%)' }}>
      {/* Windows 11 bloom */}
      <div style={{ position: 'absolute', left: 520, top: 120, width: 900, height: 700, borderRadius: '50%', background: 'radial-gradient(circle, rgba(110,190,255,0.55) 0%, rgba(60,120,230,0.25) 40%, transparent 70%)', filter: 'blur(40px)' }} />
      {/* Notepad */}
      <div style={{ position: 'absolute', left: 170, top: 80, width: 1260, height: 760, borderRadius: 12, overflow: 'visible', background: '#202020', boxShadow: '0 30px 80px rgba(0,0,0,0.55), 0 0 0 1px rgba(255,255,255,0.08)' }}>
        <div style={{ height: 48, display: 'flex', alignItems: 'center', background: '#1C1C1C', borderRadius: '12px 12px 0 0', padding: '0 0 0 14px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, height: 38, marginTop: 10, padding: '0 16px', background: '#2B2B2B', borderRadius: '8px 8px 0 0', fontFamily: FONT_BN, fontSize: 17, color: '#E6E6E6' }}>
            <div style={{ width: 16, height: 18, borderRadius: 3, background: 'linear-gradient(#6BC2FF, #2B86E5)' }} />চিঠি.txt<span style={{ marginLeft: 24, color: '#9A9A9A', fontFamily: FONT_LATIN }}>✕</span>
          </div>
          <span style={{ marginLeft: 12, color: '#9A9A9A', fontFamily: FONT_LATIN, fontSize: 22 }}>+</span>
          <div style={{ marginLeft: 'auto', display: 'flex', fontFamily: FONT_LATIN, color: '#D0D0D0', fontSize: 18 }}>
            {['—', '▢', '✕'].map((c) => <div key={c} style={{ width: 62, textAlign: 'center' }}>{c}</div>)}
          </div>
        </div>
        <div style={{ height: 40, display: 'flex', alignItems: 'center', gap: 28, padding: '0 22px', background: '#2B2B2B', fontFamily: FONT_LATIN, fontSize: 16, color: '#D6D6D6', borderBottom: '1px solid #383838' }}>
          <span>File</span><span>Edit</span><span>View</span>
        </div>
        <div style={{ padding: '44px 56px', height: 610 }}>
          <TypingDoc words={WIN_WORDS} start={78} per={36} size={56} color="#F1F1F1" popup="win" breakAfter={[1, 7]} caret="#60CDFF" />
        </div>
        <div style={{ position: 'absolute', left: 0, right: 0, bottom: 0, height: 34, display: 'flex', alignItems: 'center', gap: 30, padding: '0 22px', background: '#2B2B2B', borderRadius: '0 0 12px 12px', fontFamily: FONT_LATIN, fontSize: 14, color: '#A8A8A8' }}>
          <span>Ln 3, Col 18</span><span style={{ marginLeft: 'auto' }}>100%</span><span>Windows (CRLF)</span><span>UTF-8</span>
        </div>
      </div>
      {/* double Shift */}
      <div style={{ position: 'absolute', left: 640, top: 410, display: 'flex', gap: 20, opacity: interpolate(f, [30, 40, 70, 84], [0, 1, 1, 0], clamp) }}>
        {[36, 48].map((at) => (
          <div key={at} style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 40, color: '#fff', padding: '18px 34px', borderRadius: 16, background: 'linear-gradient(#2E3A45, #1D252D)', boxShadow: `inset 0 0 0 2px rgba(255,255,255,0.25), 0 ${8 - kc(at) * 6}px 0 rgba(0,0,0,0.5), 0 0 ${kc(at) * 60}px rgba(96,205,255,0.9)`, transform: `translateY(${kc(at) * 6}px)` }}>⇧ Shift</div>
        ))}
      </div>
      {/* taskbar */}
      <div style={{ position: 'absolute', left: 0, right: 0, bottom: 0, height: 60, background: 'rgba(32,32,32,0.86)', borderTop: '1px solid rgba(255,255,255,0.08)', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6 }}>
        {taskIcon('transparent', <svg width="24" height="24" viewBox="0 0 24 24"><path fill="#4CC2FF" d="M3 3h8.5v8.5H3zm9.5 0H21v8.5h-8.5zM3 12.5h8.5V21H3zm9.5 0H21V21h-8.5z" /></svg>, 's')}
        {taskIcon('transparent', <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#E6E6E6" strokeWidth="2"><circle cx="10.5" cy="10.5" r="6.5" /><path d="M15.5 15.5L21 21" /></svg>, 'q')}
        {taskIcon('linear-gradient(#FFD35C,#F0A30A)', null, 'e')}
        {taskIcon('linear-gradient(135deg,#36D0A0,#1683D8)', null, 'b')}
        {taskIcon('linear-gradient(#6BC2FF,#2B86E5)', null, 'n', true)}
        <div style={{ position: 'absolute', right: 16, display: 'flex', alignItems: 'center', gap: 16, fontFamily: FONT_LATIN, color: '#E6E6E6', fontSize: 15 }}>
          <span>⌃</span>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 10px', borderRadius: 6, background: bangla ? 'rgba(47,209,142,0.18)' : 'rgba(255,255,255,0.06)', boxShadow: `0 0 ${sw * (1 - ease(f, 70, 140)) * 30}px rgba(47,209,142,0.9)` }}>
            <Img src={staticFile('assets/logo_64.png')} style={{ width: 22, height: 22, borderRadius: 5 }} />
            <span style={{ fontFamily: bangla ? FONT_BN : FONT_LATIN, fontWeight: 700, fontSize: bangla ? 17 : 14 }}>{bangla ? 'বাং' : 'ENG'}</span>
          </div>
          <span>📶</span><span>🔊</span>
          <div style={{ textAlign: 'right', lineHeight: 1.25, fontSize: 13 }}><div>10:30 AM</div><div>10/30/2026</div></div>
        </div>
      </div>
    </div>
  );
};

/* ───────────────────────── Linux ───────────────────────── */
type Line = { at: number; text: string; kind: 'cmd' | 'out' | 'ok' | 'note' };
const LX: Line[] = [
  { at: 20, text: 'sudo add-apt-repository ppa:cicadama/ekushe', kind: 'cmd' },
  { at: 118, text: 'Repository: ppa.launchpadcontent.net/cicadama/ekushe', kind: 'out' },
  { at: 126, text: 'Adding repository.', kind: 'out' },
  { at: 140, text: 'sudo apt install ekushe', kind: 'cmd' },
  { at: 196, text: 'Setting up ekushe (0.1.0~beta.14-1~noble1) ...', kind: 'out' },
  { at: 206, text: '✓ Ekushe Next is installed. Updates come with your system updates.', kind: 'ok' },
  { at: 236, text: '# Fedora', kind: 'note' },
  { at: 248, text: 'sudo dnf copr enable cicadama/ekushe', kind: 'cmd' },
  { at: 330, text: 'Repository successfully enabled.', kind: 'out' },
  { at: 342, text: 'sudo dnf install ekushe', kind: 'cmd' },
  { at: 398, text: 'Installed: ekushe-0.1.0~beta.14-1.fc44.x86_64', kind: 'out' },
  { at: 406, text: '✓ Complete!', kind: 'ok' },
];
const LINUX_WORDS: Word[] = [['muktir', 'মুক্তির', ['মুক্তি']], ['gan', 'গান', ['গানে']], ['gai', 'গাই,', ['গায়']], ['bhashar', 'ভাষার', ['ভাষা']], ['gan.', 'গান।', ['গানে।']]];
const LinuxScreen: React.FC = () => {
  const f = useCurrentFrame();
  const ed = spring({ frame: f - 470, fps: 60, config: { damping: 18, stiffness: 80 } });
  const termShift = ease(f, 470, 520);
  return (
    <div style={{ width: 1600, height: 1000, position: 'relative', overflow: 'hidden', background: 'radial-gradient(130% 100% at 80% 100%, #E95420 0%, #A0303F 32%, #4A1A47 62%, #1E0E24 100%)' }}>
      <div style={{ position: 'absolute', left: -200, top: 500, width: 1200, height: 900, borderRadius: '50%', background: 'radial-gradient(circle, rgba(255,140,80,0.35), transparent 65%)', filter: 'blur(30px)' }} />
      {/* GNOME top bar */}
      <div style={{ position: 'absolute', left: 0, right: 0, top: 0, height: 36, background: '#0B0B0B', display: 'flex', alignItems: 'center', padding: '0 16px', fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 16, color: '#F2F2F2' }}>
        <div style={{ display: 'flex', gap: 6 }}>{[0, 1, 2].map((i) => <div key={i} style={{ width: i === 0 ? 26 : 8, height: 8, borderRadius: 9, background: i === 0 ? '#fff' : '#888' }} />)}</div>
        <div style={{ position: 'absolute', left: 0, right: 0, textAlign: 'center', pointerEvents: 'none' }}>Oct 30  10:30</div>
        <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 14 }}>
          <span style={{ fontFamily: f > 520 ? FONT_BN : FONT_LATIN, fontWeight: 700, padding: '1px 10px', borderRadius: 99, background: f > 520 ? 'rgba(233,84,32,0.85)' : 'transparent' }}>{f > 520 ? 'বাং' : 'en'} ▾</span>
          <span>📶</span><span>🔊</span><span>🔋</span>
        </div>
      </div>
      {/* Terminal */}
      <div style={{ position: 'absolute', left: 70, top: 66, width: 1460, height: 850, borderRadius: 14, background: '#1E1E1E', boxShadow: '0 30px 90px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.08)', overflow: 'hidden', transform: `translate(${-termShift * 40}px, ${-termShift * 20}px) scale(${1 - termShift * 0.06})`, filter: `brightness(${1 - termShift * 0.35})` }}>
        <div style={{ height: 46, background: '#2D2D2D', display: 'flex', alignItems: 'center', justifyContent: 'center', position: 'relative', fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 16, color: '#E6E6E6' }}>
          khan@ubuntu: ~
          <div style={{ position: 'absolute', right: 14, display: 'flex', gap: 10 }}>{['#5A5A5A', '#5A5A5A', '#E95420'].map((c, i) => <div key={i} style={{ width: 18, height: 18, borderRadius: 99, background: c }} />)}</div>
        </div>
        <div style={{ padding: '22px 30px', fontFamily: MONO, fontSize: 31, lineHeight: 1.5, color: '#E8E8E8' }}>
          {LX.filter((l) => f >= l.at).map((l, i) => {
            if (l.kind === 'cmd') {
              const n = Math.min(l.text.length, Math.floor((f - l.at) / 1.6));
              const live = n < l.text.length;
              return (
                <div key={i} style={{ whiteSpace: 'pre' }}>
                  <span style={{ color: '#8AE234', fontWeight: 700 }}>khan@ubuntu</span><span style={{ color: '#fff' }}>:</span><span style={{ color: '#729FCF', fontWeight: 700 }}>~</span><span style={{ color: '#fff' }}>$ </span>
                  <span>{l.text.slice(0, n)}</span>
                  {live && <span style={{ display: 'inline-block', width: 17, height: 36, background: '#E8E8E8', verticalAlign: '-7px' }} />}
                </div>
              );
            }
            if (l.kind === 'note') return <div key={i} style={{ color: '#9A9A9A', marginTop: 14 }}>{l.text}</div>;
            return <div key={i} style={{ color: l.kind === 'ok' ? '#8AE234' : '#BFBFBF', fontWeight: l.kind === 'ok' ? 700 : 400, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', opacity: ease(f, l.at, l.at + 6) }}>{l.text}</div>;
          })}
          {f > 420 && <div style={{ whiteSpace: 'pre' }}><span style={{ color: '#8AE234', fontWeight: 700 }}>khan@ubuntu</span>:<span style={{ color: '#729FCF', fontWeight: 700 }}>~</span>$ <span style={{ display: 'inline-block', width: 17, height: 36, background: '#E8E8E8', verticalAlign: '-7px', opacity: Math.floor(f / 18) % 2 ? 0 : 1 }} /></div>}
        </div>
      </div>
      {/* Text editor, typing through IBus */}
      <div style={{ position: 'absolute', left: 520, top: 230, width: 1000, height: 600, borderRadius: 14, background: '#FAFAFA', boxShadow: '0 40px 100px rgba(0,0,0,0.6)', opacity: Math.min(1, ed * 1.4), transform: `translateY(${(1 - ed) * 260}px) scale(${0.9 + ed * 0.1})` }}>
        <div style={{ height: 50, background: '#EBEBEB', borderRadius: '14px 14px 0 0', display: 'flex', alignItems: 'center', padding: '0 18px', fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 16, color: '#303030', position: 'relative' }}>
          <span style={{ padding: '6px 14px', borderRadius: 8, background: '#DADADA', fontFamily: FONT_BN }}>গান.txt</span>
          <div style={{ position: 'absolute', left: 0, right: 0, textAlign: 'center', pointerEvents: 'none' }}>Text Editor</div>
          <div style={{ marginLeft: 'auto', display: 'flex', gap: 10 }}>{['#CFCFCF', '#CFCFCF', '#E95420'].map((c, i) => <div key={i} style={{ width: 18, height: 18, borderRadius: 99, background: c }} />)}</div>
        </div>
        <div style={{ padding: '40px 50px' }}>
          <TypingDoc words={LINUX_WORDS} start={540} per={44} size={60} color="#1E1E1E" popup="linux" light caret="#E95420" />
        </div>
      </div>
    </div>
  );
};

/* ───────────────────────── macOS ───────────────────────── */
const MAC_WORDS: Word[] = [['ekushe', 'একুশে', ['একুশ']], ['amar', 'আমার', ['আমরা']], ['ohonkar,', 'অহংকার,', ['অহঙ্কার,']], ['amar', 'আমার', ['আমরা']], ['bhasha', 'ভাষা', ['ভাষায়']], ['amar', 'আমার', ['আমরা']], ['porichoy.', 'পরিচয়।', ['পরিচয়']]];
const MacScreen: React.FC = () => {
  const f = useCurrentFrame();
  const menu = interpolate(f, [18, 30, 92, 104], [0, 1, 1, 0], clamp);
  const picked = f > 64;
  const dock = ['linear-gradient(#5AC8FA,#1E7CF2)', 'linear-gradient(#FFFFFF,#E5E5EA)', 'linear-gradient(#34C759,#0B8F3A)', 'linear-gradient(#FF9F0A,#FF6A00)', 'linear-gradient(#FF375F,#C9184A)', 'linear-gradient(#8E8E93,#3A3A3C)'];
  return (
    <div style={{ width: 1600, height: 1000, position: 'relative', overflow: 'hidden', background: 'linear-gradient(160deg, #0F3D2E 0%, #1F6B4E 35%, #C2453F 75%, #E9B949 100%)' }}>
      <div style={{ position: 'absolute', inset: 0, background: 'radial-gradient(80% 60% at 30% 30%, rgba(255,255,255,0.18), transparent 60%)' }} />
      {/* menu bar */}
      <div style={{ position: 'absolute', left: 0, right: 0, top: 0, height: 34, background: 'rgba(30,30,30,0.35)', backdropFilter: 'blur(20px)', display: 'flex', alignItems: 'center', gap: 24, padding: '0 20px', fontFamily: FONT_LATIN, fontSize: 16, color: '#fff' }}>
        <svg width="16" height="18" viewBox="0 0 24 24"><path fill="#fff" d="M16.4 12.6c0-2.4 2-3.6 2.1-3.7-1.2-1.7-3-1.9-3.6-2-1.5-.2-3 .9-3.8.9-.8 0-2-.9-3.3-.9-1.7 0-3.3 1-4.2 2.5-1.8 3.1-.5 7.7 1.3 10.2.9 1.2 1.9 2.6 3.2 2.6 1.3-.1 1.8-.8 3.3-.8s2 .8 3.3.8c1.4 0 2.3-1.2 3.1-2.5 1-1.4 1.4-2.8 1.4-2.9 0 0-2.8-1.1-2.8-4.2zM14 5.3c.7-.9 1.2-2 1-3.3-1 0-2.3.7-3 1.6-.6.7-1.2 1.9-1 3.1 1.1.1 2.3-.6 3-1.4z" /></svg>
        <b>TextEdit</b><span>File</span><span>Edit</span><span>Format</span><span>View</span><span>Window</span><span>Help</span>
        <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 18 }}>
          <span style={{ display: 'grid', placeItems: 'center', minWidth: 30, height: 24, padding: '0 6px', borderRadius: 6, background: menu > 0.5 ? 'rgba(255,255,255,0.25)' : 'transparent', fontFamily: picked ? FONT_BN : FONT_LATIN, fontWeight: 700, fontSize: picked ? 17 : 14 }}>{picked ? 'অ' : 'ABC'}</span>
          <span>📶</span><span>🔋</span><span>Fri 30 Oct  10:30</span>
        </div>
      </div>
      {/* input menu */}
      <div style={{ position: 'absolute', right: 250, top: 40, width: 330, borderRadius: 12, background: 'rgba(240,240,240,0.92)', boxShadow: '0 20px 60px rgba(0,0,0,0.4)', padding: 6, opacity: menu, transform: `translateY(${(1 - menu) * -10}px)`, fontFamily: FONT_LATIN, fontSize: 17, color: '#1D1D1F', zIndex: 6 }}>
        {[['ABC', !picked], ['Ekushe Bangla', picked]].map(([n, on], i) => (
          <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '8px 12px', borderRadius: 7, background: i === 1 && f > 50 && f < 66 ? '#2C7BF6' : 'transparent', color: i === 1 && f > 50 && f < 66 ? '#fff' : '#1D1D1F' }}>
            <span style={{ width: 16 }}>{on ? '✓' : ''}</span>
            {i === 1 && <Img src={staticFile('assets/logo_64.png')} style={{ width: 22, height: 22, borderRadius: 5 }} />}{n as string}
          </div>
        ))}
        <div style={{ height: 1, background: 'rgba(0,0,0,0.12)', margin: '6px 8px' }} />
        <div style={{ padding: '6px 12px 6px 40px', color: '#6E6E73' }}>Ekushe Studio…</div>
      </div>
      {/* TextEdit */}
      <div style={{ position: 'absolute', left: 230, top: 110, width: 1140, height: 660, borderRadius: 14, background: '#FFFFFF', boxShadow: '0 40px 100px rgba(0,0,0,0.45), 0 0 0 1px rgba(0,0,0,0.12)' }}>
        <div style={{ height: 52, borderBottom: '1px solid #E5E5E5', display: 'flex', alignItems: 'center', padding: '0 18px', position: 'relative', fontFamily: FONT_LATIN, fontSize: 16, color: '#3A3A3C' }}>
          <div style={{ display: 'flex', gap: 9 }}>{['#FF5F57', '#FEBC2E', '#28C840'].map((c) => <div key={c} style={{ width: 14, height: 14, borderRadius: 99, background: c }} />)}</div>
          <div style={{ position: 'absolute', left: 0, right: 0, textAlign: 'center', fontFamily: FONT_BN, fontWeight: 600 }}>একুশে.rtf</div>
        </div>
        <div style={{ padding: '50px 64px' }}>
          <TypingDoc words={MAC_WORDS} start={110} per={40} size={62} color="#1D1D1F" popup="mac" light breakAfter={[2]} caret="#2C7BF6" />
        </div>
      </div>
      {/* Dock */}
      <div style={{ position: 'absolute', left: '50%', bottom: 14, transform: 'translateX(-50%)', display: 'flex', gap: 12, padding: 10, borderRadius: 22, background: 'rgba(255,255,255,0.22)', border: '1px solid rgba(255,255,255,0.3)', backdropFilter: 'blur(20px)' }}>
        {dock.map((bg, i) => <div key={i} style={{ width: 62, height: 62, borderRadius: 15, background: bg, boxShadow: '0 4px 10px rgba(0,0,0,0.25)' }} />)}
        <Img src={staticFile('assets/logo_128.png')} style={{ width: 62, height: 62, borderRadius: 15 }} />
      </div>
    </div>
  );
};

/* ───────────────────────── iPhone and iPad ───────────────────────── */
const IOS_LOOKS: [string, string, boolean][] = [
  ['ekushe-typing', 'একুশে', false], ['default', 'একুশে ডিফল্ট', true], ['smooth', 'একুশে মসৃণ', true], ['krishnachura', 'কৃষ্ণচূড়া', true],
  ['padma', 'পদ্মা', false], ['shapla', 'শাপলা', true], ['night', 'রাত', false], ['default-dark', 'ডিফল্ট, রাতে', false], ['english', 'English', false], ['emoji', 'ইমোজি', false],
];
const IOS_EVERY = 44;
const IPhone: React.FC<{ w: number; look: number; prev: number; wipe: number; t: number }> = ({ w, look, prev, wipe, t }) => {
  const { u } = useLayout();
  const kbH = w * 0.94 * (798 / 1170);
  const light = IOS_LOOKS[look][2];
  const bg = light ? '#F2F2F7' : '#0B1411';
  const fg = light ? '#111' : '#EEF4F1';
  return (
    <div style={{ width: w, height: w * 2.05, borderRadius: w * 0.17, padding: w * 0.03, background: 'linear-gradient(150deg, #E6E8EA, #8C9196 30%, #2B2F33 60%, #A9AEB3)', boxShadow: `0 ${60 * u}px ${140 * u}px -${40 * u}px rgba(0,0,0,0.95), 0 0 ${160 * u}px rgba(47,209,142,0.22)`, position: 'relative' }}>
      <div style={{ width: '100%', height: '100%', borderRadius: w * 0.145, overflow: 'hidden', background: bg, position: 'relative', display: 'flex', flexDirection: 'column' }}>
        <div style={{ position: 'absolute', top: w * 0.03, left: '50%', width: w * 0.3, height: w * 0.085, borderRadius: 99, background: '#000', transform: 'translateX(-50%)', zIndex: 3 }} />
        <div style={{ display: 'flex', justifyContent: 'space-between', padding: `${w * 0.045}px ${w * 0.09}px 0`, fontFamily: FONT_LATIN, fontWeight: 700, fontSize: w * 0.04, color: fg }}><span>10:30</span><span>●●● 🔋</span></div>
        <div style={{ textAlign: 'center', marginTop: w * 0.07, fontFamily: FONT_BN, fontWeight: 600, fontSize: w * 0.042, color: fg }}>
          <div style={{ width: w * 0.12, height: w * 0.12, borderRadius: 99, margin: '0 auto', background: 'linear-gradient(135deg, #0B6B4A, #2FD18E)', display: 'grid', placeItems: 'center', color: '#fff', fontWeight: 700 }}>র</div>
          রাফি
        </div>
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'flex-end', gap: w * 0.02, padding: `${w * 0.04}px` }}>
          <div style={{ alignSelf: 'flex-start', maxWidth: '80%', padding: `${w * 0.02}px ${w * 0.035}px`, borderRadius: w * 0.05, background: light ? '#E5E5EA' : '#26332D', color: fg, fontFamily: FONT_BN, fontSize: w * 0.045, opacity: ease(t, 10, 22) }}>কাল কখন আসবি?</div>
          <div style={{ alignSelf: 'flex-end', maxWidth: '80%', padding: `${w * 0.02}px ${w * 0.035}px`, borderRadius: w * 0.05, background: '#0B8F5A', color: '#fff', fontFamily: FONT_BN, fontSize: w * 0.045, opacity: ease(t, 26, 38) }}>সকাল দশটায়, ইনশাআল্লাহ 🙂</div>
          <div style={{ alignSelf: 'stretch', marginTop: w * 0.02, padding: `${w * 0.02}px ${w * 0.035}px`, borderRadius: 99, border: `1px solid ${light ? '#C7C7CC' : '#3A4A43'}`, fontFamily: FONT_BN, fontSize: w * 0.045, color: fg }}>
            <span style={{ textDecoration: 'underline', textUnderlineOffset: 4 }}>আমার</span><span style={{ display: 'inline-block', width: 2, height: w * 0.05, background: '#2C7BF6', marginLeft: 2, verticalAlign: '-0.15em' }} />
          </div>
        </div>
        <div style={{ position: 'relative', margin: `0 ${w * 0.03}px ${w * 0.05}px`, height: kbH, borderRadius: w * 0.03, overflow: 'hidden', flexShrink: 0 }}>
          <Img src={staticFile(`ios/${IOS_LOOKS[prev][0]}.jpg`)} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%' }} />
          <Img src={staticFile(`ios/${IOS_LOOKS[look][0]}.jpg`)} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', clipPath: `inset(0 ${(1 - wipe) * 100}% 0 0)` }} />
          {wipe > 0 && wipe < 1 && <div style={{ position: 'absolute', top: 0, bottom: 0, left: `${wipe * 100}%`, width: 5 * u, background: 'rgba(255,255,255,0.9)', boxShadow: `0 0 ${30 * u}px #fff` }} />}
        </div>
      </div>
    </div>
  );
};
const IosScene: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width, height } = useLayout();
  const len = P.engine - P.ios;
  const rise = spring({ frame: f - 6, fps: 60, config: { damping: 16, stiffness: 70 } });
  const back = spring({ frame: f - 30, fps: 60, config: { damping: 16, stiffness: 70 } });
  const head = spring({ frame: f, fps: 60, config: { damping: 14 } });
  const out = ease(f, len - 20, len);
  const tf = f - 90;
  const li = tf < 0 ? 0 : Math.min(IOS_LOOKS.length - 1, 1 + Math.floor(tf / IOS_EVERY));
  const prev = Math.max(0, li - 1);
  const swapAt = 90 + (li - 1) * IOS_EVERY;
  const wipe = li === 0 ? 1 : ease(f, swapAt, swapAt + 12);
  const pw = (V ? 600 : 520) * u;
  const points: [string, string, number][] = [
    ['সব থিম, আইফোনেও', 'Every theme on iPhone too', 70],
    ['ইংরেজি শব্দ পূরণ আর বানান', 'English completion and spelling, built on iOS', 130],
    ['"ফুল অ্যাক্সেস" লাগে না', "No Full Access: your typing can't leave the phone", 190],
  ];
  const badge = spring({ frame: f - 30, fps: 60, config: { damping: 10 } });
  return (
    <AbsoluteFill style={{ opacity: 1 - out, transform: `scale(${1 - out * 0.05})` }}>
      <AbsoluteFill style={{ background: `radial-gradient(circle at ${V ? 50 : 66}% 58%, rgba(233,185,73,${0.18 * rise}) 0%, transparent 45%)` }} />
      <div style={{ position: 'absolute', left: V ? 0 : 130 * u, right: V ? 0 : undefined, top: V ? 150 * u : 0, bottom: V ? undefined : 0, width: V ? undefined : 900 * u, display: 'flex', flexDirection: 'column', justifyContent: 'center', alignItems: V ? 'center' : 'flex-start', textAlign: V ? 'center' : 'left' }}>
        <div style={{ opacity: head, transform: `translateY(${(1 - head) * 40 * u}px)` }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 18 * u, justifyContent: V ? 'center' : 'flex-start', color: C.white }}>
            <div style={{ display: 'grid', placeItems: 'center', width: 84 * u, height: 84 * u, borderRadius: 22 * u, background: 'rgba(255,255,255,0.1)', border: `${2 * u}px solid rgba(255,255,255,0.22)` }}><OsIcon kind="ios" size={50 * u} /></div>
            <span style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 54 * u }}>iPhone · iPad</span>
          </div>
          <Big size={V ? 96 : 92} style={{ textAlign: V ? 'center' : 'left', marginTop: 24 * u }}>আইফোন আর আইপ্যাডেও</Big>
          <Sub size={36} style={{ textAlign: V ? 'center' : 'left', marginTop: 8 * u }}>On iPhone and iPad, with the same engine</Sub>
          <div style={{ marginTop: 34 * u, display: 'inline-flex', alignItems: 'center', gap: 14 * u, padding: `${10 * u}px ${28 * u}px`, borderRadius: 999, background: 'rgba(233,185,73,0.14)', border: `${2 * u}px solid rgba(233,185,73,0.7)`, opacity: Math.min(1, badge * 1.3), transform: `scale(${0.7 + badge * 0.3})`, transformOrigin: V ? 'center' : 'left center' }}>
            <span style={{ width: 16 * u, height: 16 * u, borderRadius: 99, background: C.mustard }} />
            <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 48 : 44) * u, color: C.mustard }}>শীঘ্রই আসছে</span>
            <span style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 28 * u, color: 'rgba(234,243,238,0.75)' }}>Coming later</span>
          </div>
        </div>
        {!V && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 22 * u, marginTop: 50 * u }}>
            {points.map(([bnT, en, at]) => {
              const t = spring({ frame: f - at, fps: 60, config: { damping: 13 } });
              return (
                <div key={bnT} style={{ display: 'flex', alignItems: 'center', gap: 18 * u, opacity: t, transform: `translateX(${(1 - t) * -50 * u}px)` }}>
                  <span style={{ display: 'grid', placeItems: 'center', width: 46 * u, height: 46 * u, borderRadius: 99, background: C.mustard, color: '#03170F', fontSize: 28 * u, fontWeight: 800 }}>✓</span>
                  <div><div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 46 * u, color: C.white }}>{bnT}</div><div style={{ fontFamily: FONT_LATIN, fontSize: 27 * u, color: 'rgba(234,243,238,0.65)' }}>{en}</div></div>
                </div>
              );
            })}
            <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 54 * u, color: C.mustard, marginTop: 20 * u, opacity: ease(f, 90, 104) }}>{IOS_LOOKS[li][1]}</div>
          </div>
        )}
      </div>
      {/* a second iPhone behind, in a light look, for depth */}
      <div style={{ position: 'absolute', left: V ? width / 2 + 40 * u : width - pw - 120 * u - pw * 0.62, top: V ? 830 * u : (height - pw * 2.05) / 2 + 60 * u, transform: `translateY(${(1 - back) * height * 0.7}px) rotate(${-8 + (1 - back) * 6}deg) scale(0.86)`, opacity: 0.75, filter: 'brightness(0.8)' }}>
        <IPhone w={pw} look={2} prev={2} wipe={1} t={f} />
      </div>
      <div style={{ position: 'absolute', left: V ? width / 2 - pw * 0.85 : width - pw - 160 * u, top: V ? 780 * u : (height - pw * 2.05) / 2, transform: `translateY(${(1 - rise) * height * 0.8}px) rotate(${(1 - rise) * 8 + (V ? -2 : 3)}deg)` }}>
        <IPhone w={pw} look={li} prev={prev} wipe={wipe} t={f} />
      </div>
      {V && <div style={{ position: 'absolute', left: 0, right: 0, top: 2080 * u, textAlign: 'center', fontFamily: FONT_BN, fontWeight: 700, fontSize: 64 * u, color: C.mustard, opacity: ease(f, 90, 104) }}>{IOS_LOOKS[li][1]}</div>}
      {V && (
        <div style={{ position: 'absolute', left: 0, right: 0, top: 2190 * u, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 14 * u }}>
          {points.map(([bnT, , at]) => {
            const t = spring({ frame: f - at, fps: 60, config: { damping: 13 } });
            return <div key={bnT} style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 46 * u, color: C.white, opacity: t }}>✓ {bnT}</div>;
          })}
        </div>
      )}
      <Burst at={8} x={V ? 0.5 : 0.7} y={0.55} count={26} seed="ios" spread={0.5} colors={[C.mustard, '#ffffff', C.greenBright]} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── chapter: every device ───────────────────────── */
const Devices: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const a = spring({ frame: f - 6, fps: 60, config: { damping: 14 } });
  const out = ease(f, 160, 180);
  const os: Os[] = ['win', 'linux', 'mac', 'android', 'ios'];
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out, transform: `scale(${1 + out * 0.1})`, filter: `blur(${out * 10 * u}px)` }}>
      <Big size={V ? 120 : 120} style={{ opacity: a, transform: `translateY(${(1 - a) * 50 * u}px)` }}>এক কিবোর্ড, <span style={{ color: C.greenBright }}>প্রতিটি ডিভাইসে</span></Big>
      <Sub size={40} style={{ marginTop: 12 * u, opacity: ease(f, 20, 40) }}>One keyboard, on every device</Sub>
      <div style={{ display: 'flex', gap: (V ? 30 : 60) * u, marginTop: 70 * u, flexWrap: 'wrap', justifyContent: 'center', maxWidth: (V ? 1200 : 2200) * u }}>
        {os.map((k, i) => {
          const t = spring({ frame: f - 40 - i * 10, fps: 60, config: { damping: 11 } });
          const s = (V ? 170 : 160) * u;
          return (
            <div key={k} style={{ textAlign: 'center', color: C.white, opacity: t, transform: `translateY(${(1 - t) * 60 * u}px) scale(${0.6 + t * 0.4})` }}>
              <div style={{ width: s, height: s, borderRadius: s * 0.26, display: 'grid', placeItems: 'center', background: 'rgba(255,255,255,0.08)', border: `${2 * u}px solid rgba(255,255,255,0.22)` }}><OsIcon kind={k} size={s * 0.52} /></div>
              <div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 34 * u, marginTop: 14 * u }}>{OS_NAME[k]}</div>
            </div>
          );
        })}
      </div>
      <Burst at={40} y={V ? 0.6 : 0.66} count={30} seed="dev" spread={0.6} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── one engine ───────────────────────── */
const Engine: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width, height } = useLayout();
  const len = P.avail - P.engine;
  const core = spring({ frame: f - 8, fps: 60, config: { damping: 11, mass: 0.9 } });
  const out = ease(f, len - 18, len);
  const cx = width / 2, cy = height * (V ? 0.5 : 0.55);
  const os: Os[] = ['win', 'linux', 'mac', 'android', 'ios'];
  const pos = os.map((_, i) => {
    const a = -Math.PI / 2 + (i * 2 * Math.PI) / 5;
    return V ? [cx + Math.cos(a) * 470 * u, cy + Math.sin(a) * 520 * u] : [cx + Math.cos(a) * 640 * u, cy + Math.sin(a) * 330 * u];
  });
  const pulse = (f % 60) / 60;
  const head = spring({ frame: f - 60, fps: 60, config: { damping: 14 } });
  return (
    <AbsoluteFill style={{ opacity: 1 - out, filter: `blur(${out * 12 * u}px)` }}>
      <svg width={width} height={height} style={{ position: 'absolute', inset: 0 }}>
        <defs><radialGradient id="fcore"><stop offset="0" stopColor="#FFFFFF" /><stop offset="0.25" stopColor="#5BE3AA" /><stop offset="0.7" stopColor="#0B6B4A" /><stop offset="1" stopColor="#0B6B4A" stopOpacity="0" /></radialGradient></defs>
        {pos.map(([x, y], i) => {
          const t = ease(f, 26 + i * 8, 60 + i * 8);
          const q = (pulse + i * 0.2) % 1;
          return (
            <g key={i}>
              <line x1={cx} y1={cy} x2={cx + (x - cx) * t} y2={cy + (y - cy) * t} stroke="rgba(47,209,142,0.55)" strokeWidth={4 * u} strokeDasharray={`${14 * u} ${10 * u}`} strokeDashoffset={-f * 2 * u} />
              {t >= 1 && <circle cx={cx + (x - cx) * q} cy={cy + (y - cy) * q} r={9 * u} fill="#E9F9F1" opacity={0.9} />}
            </g>
          );
        })}
        <circle cx={cx} cy={cy} r={(150 + Math.sin(f / 9) * 6) * u * core} fill="url(#fcore)" opacity={0.95} />
        {[0, 30].map((d) => { const r = ((f + d) % 60) / 60; return <circle key={d} cx={cx} cy={cy} r={(120 + r * 220) * u * core} fill="none" stroke={`rgba(47,209,142,${0.5 * (1 - r)})`} strokeWidth={3 * u} />; })}
      </svg>
      <div style={{ position: 'absolute', left: cx - 120 * u, top: cy - 120 * u, width: 240 * u, height: 240 * u, display: 'grid', placeItems: 'center', transform: `scale(${core})` }}>
        <Img src={staticFile('assets/logo_512.png')} style={{ width: 150 * u, height: 150 * u, borderRadius: 38 * u, boxShadow: `0 0 ${80 * u}px rgba(255,255,255,0.6)` }} />
      </div>
      {pos.map(([x, y], i) => {
        const t = spring({ frame: f - 40 - i * 9, fps: 60, config: { damping: 11 } });
        const s = (V ? 170 : 150) * u;
        const now = os[i] === 'win' || os[i] === 'linux';
        return (
          <div key={i} style={{ position: 'absolute', left: x - s / 2 - 40 * u, top: y - s / 2, width: s + 80 * u, textAlign: 'center', color: C.white, opacity: t, transform: `scale(${0.6 + t * 0.4})` }}>
            <div style={{ width: s, height: s, margin: '0 auto', borderRadius: s * 0.26, display: 'grid', placeItems: 'center', background: now ? 'rgba(47,209,142,0.16)' : 'rgba(233,185,73,0.12)', border: `${2 * u}px solid ${now ? 'rgba(47,209,142,0.7)' : 'rgba(233,185,73,0.55)'}`, boxShadow: `0 0 ${60 * u * t}px ${now ? 'rgba(47,209,142,0.3)' : 'rgba(233,185,73,0.2)'}` }}><OsIcon kind={os[i]} size={s * 0.52} /></div>
            <div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 34 * u, marginTop: 12 * u }}>{OS_NAME[os[i]]}</div>
          </div>
        );
      })}
      <div style={{ position: 'absolute', left: 0, right: 0, top: (V ? 180 : 80) * u, textAlign: 'center', opacity: head, transform: `translateY(${(1 - head) * 40 * u}px)`, padding: `0 ${60 * u}px` }}>
        <Big size={V ? 100 : 92}>সব সংস্করণের প্রাণ <span style={{ color: C.greenBright }}>একটাই ইঞ্জিন</span></Big>
        <Sub size={38} style={{ marginTop: 10 * u }}>One Ekushe engine at the heart of every version</Sub>
      </div>
      <div style={{ position: 'absolute', left: 0, right: 0, bottom: (V ? 200 : 70) * u, display: 'flex', justifyContent: 'center', gap: 20 * u, flexWrap: 'wrap', padding: `0 ${60 * u}px` }}>
        {['একই অভিধান', 'একই গতি', 'একই নির্ভুল বানান', 'সম্পূর্ণ ফ্রি'].map((w, i) => {
          const t = spring({ frame: f - 150 - i * 12, fps: 60, config: { damping: 12 } });
          const on = i === 3;
          return <div key={w} style={{ ...chip(u, on), fontSize: (V ? 50 : 46) * u, borderRadius: 999, opacity: t, transform: `translateY(${(1 - t) * 30 * u}px)` }}>{w}</div>;
        })}
      </div>
      <Burst at={8} x={0.5} y={V ? 0.5 : 0.55} count={44} seed="feng" spread={0.7} />
      <Sweep at={210} dur={50} strength={0.12} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── when and where ───────────────────────── */
const Avail: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const len = P.end - P.avail;
  const out = ease(f, len - 20, len);
  const head = spring({ frame: f - 4, fps: 60, config: { damping: 14 } });
  const c1 = spring({ frame: f - 30, fps: 60, config: { damping: 15, stiffness: 90 } });
  const c2 = spring({ frame: f - 150, fps: 60, config: { damping: 15, stiffness: 90 } });
  const card = (on: boolean, t: number, children: React.ReactNode) => (
    <div style={{ width: (V ? 1300 : 1080) * u, padding: `${(V ? 44 : 50) * u}px ${(V ? 50 : 56) * u}px`, borderRadius: 40 * u, background: on ? 'linear-gradient(160deg, rgba(47,209,142,0.2), rgba(11,107,74,0.18))' : 'linear-gradient(160deg, rgba(233,185,73,0.16), rgba(120,80,20,0.12))', border: `${3 * u}px solid ${on ? 'rgba(47,209,142,0.7)' : 'rgba(233,185,73,0.6)'}`, boxShadow: `0 0 ${100 * u}px ${on ? 'rgba(47,209,142,0.25)' : 'rgba(233,185,73,0.18)'}, inset 0 1px 0 rgba(255,255,255,0.2)`, backdropFilter: `blur(${16 * u}px)`, opacity: Math.min(1, t * 1.3), transform: `translateY(${(1 - t) * 120 * u}px) scale(${0.9 + t * 0.1})` }}>{children}</div>
  );
  const row = (os: Os, name: string, detail: string, at: number, accent: string, tags?: string[]) => {
    const t = spring({ frame: f - at, fps: 60, config: { damping: 13 } });
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 26 * u, marginTop: 30 * u, opacity: t, transform: `translateX(${(1 - t) * -40 * u}px)` }}>
        <div style={{ display: 'grid', placeItems: 'center', width: 92 * u, height: 92 * u, flex: 'none', borderRadius: 24 * u, background: 'rgba(255,255,255,0.1)', color: C.white }}><OsIcon kind={os} size={54 * u} /></div>
        <div style={{ flex: 1 }}>
          <div style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 46 * u, color: C.white }}>{name}</div>
          <div style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: 27 * u, color: 'rgba(234,243,238,0.7)' }}>{detail}</div>
          {tags && (
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12 * u, marginTop: 12 * u }}>
              {tags.map((tg, i) => {
                const s = spring({ frame: f - at - 20 - i * 8, fps: 60, config: { damping: 11 } });
                return <span key={tg} style={{ fontFamily: FONT_LATIN, fontWeight: 700, fontSize: 26 * u, color: '#03170F', background: accent, borderRadius: 999, padding: `${5 * u}px ${18 * u}px`, opacity: s, transform: `scale(${0.6 + s * 0.4})`, display: 'inline-block' }}>{tg}</span>;
              })}
            </div>
          )}
        </div>
      </div>
    );
  };
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out }}>
      <div style={{ position: 'absolute', top: (V ? 170 : 70) * u, textAlign: 'center', opacity: head, transform: `translateY(${(1 - head) * 40 * u}px)` }}>
        <Big size={V ? 104 : 88}>কবে, কোথায় পাবেন</Big>
        <Sub size={36} style={{ marginTop: 6 * u }}>When and where</Sub>
      </div>
      <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', gap: (V ? 50 : 60) * u, marginTop: (V ? 120 : 130) * u, alignItems: V ? 'center' : 'stretch' }}>
        {card(true, c1, (
          <>
            <div style={{ display: 'flex', alignItems: 'baseline', gap: 20 * u, flexWrap: 'wrap' }}>
              <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 70 * u, color: C.greenBright }}>৩০ অক্টোবর থেকে</span>
              <span style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 32 * u, color: 'rgba(234,243,238,0.75)' }}>From October 30</span>
            </div>
            {row('win', 'Windows 10 · 11', 'Installer, portable, Microsoft-signed', 60, C.greenBright, ['Setup', 'Portable'])}
            {row('linux', 'Linux', 'Ubuntu, Fedora, Debian, Mint and more', 90, C.greenBright, ['Ubuntu PPA', 'Fedora COPR', '.deb', '.rpm', 'AppImage'])}
          </>
        ))}
        {card(false, c2, (
          <>
            <div style={{ display: 'flex', alignItems: 'baseline', gap: 20 * u, flexWrap: 'wrap' }}>
              <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 70 * u, color: C.mustard }}>শীঘ্রই আসছে</span>
              <span style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 32 * u, color: 'rgba(234,243,238,0.75)' }}>Coming later</span>
            </div>
            {row('mac', 'macOS', 'Apple silicon and Intel', 180, C.mustard)}
            {row('android', 'Android', 'Phones and tablets', 200, C.mustard)}
            {row('ios', 'iPhone · iPad', 'The same engine, as an iOS keyboard', 220, C.mustard)}
          </>
        ))}
      </div>
      <Burst at={34} x={V ? 0.5 : 0.3} y={V ? 0.36 : 0.5} count={30} seed="av1" spread={0.4} colors={[C.greenBright, '#ffffff', C.coral]} />
      <Burst at={154} x={V ? 0.5 : 0.7} y={V ? 0.72 : 0.5} count={22} seed="av2" spread={0.35} colors={[C.mustard, '#ffffff']} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── where to find it ───────────────────────── */
const End: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const pop = spring({ frame: f - 4, fps: 60, config: { damping: 11, mass: 0.8 } });
  const ring = ease(f, 4, 50);
  const m = ease(f, 0, 160);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [0, 12], [0, 1], clamp) }}>
      <div style={{ position: 'absolute', bottom: (V ? -120 : -420) * u, opacity: 0.22 * m }}><ShaheedMinar size={(V ? 1000 : 820) * u} rise={m} glow={0.8} /></div>
      <AbsoluteFill style={{ background: 'radial-gradient(ellipse at 50% 45%, rgba(0,0,0,0) 0%, rgba(1,8,5,0.7) 80%)' }} />
      <div style={{ position: 'relative' }}>
        <div style={{ position: 'absolute', left: '50%', top: '50%', width: 300 * u, height: 300 * u, borderRadius: 999, border: `${3 * u}px solid rgba(255,255,255,${0.5 * (1 - ring)})`, transform: `translate(-50%,-50%) scale(${0.8 + ring * 1.6})` }} />
        <Img src={staticFile('assets/logo_512.png')} style={{ width: (V ? 250 : 210) * u, height: (V ? 250 : 210) * u, borderRadius: 52 * u, transform: `scale(${0.6 + pop * 0.4})`, opacity: Math.min(1, pop * 1.5), boxShadow: `0 ${30 * u}px ${80 * u}px rgba(0,0,0,0.55), 0 0 ${120 * u}px rgba(224,48,63,0.5)` }} />
      </div>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 140 : 120) * u, color: C.white, marginTop: 22 * u, lineHeight: 1.1, opacity: ease(f, 10, 30), textShadow: `0 0 ${50 * u}px rgba(224,48,63,0.45)` }}>একুশে নেক্সট</div>
      <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 56 : 50) * u, color: 'rgba(234,243,238,0.9)', marginTop: 6 * u, opacity: ease(f, 20, 40) }}>যেভাবে বলেন, সেভাবে লিখুন</div>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 18 * u, marginTop: 22 * u, opacity: ease(f, 30, 50), flexWrap: 'wrap', justifyContent: 'center' }}>
        <span style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 64 : 58) * u, color: C.mustard }}>বেটা উদ্বোধন ৩০ অক্টোবর ২০২৬</span>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: 34 * u, color: 'rgba(234,243,238,0.75)' }}>Windows · Linux · Free</span>
      </div>
      <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 26 : 30) * u, marginTop: (V ? 56 : 46) * u }}>
        <AddressPill kind="web" text="www.ekushenext.com" at={70} />
        <AddressPill kind="mail" text="support@ekushenext.com" at={110} />
      </div>
      <Burst at={4} y={V ? 0.34 : 0.28} count={40} seed="fend" spread={0.65} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── the film ───────────────────────── */
const WIN_POINTS: [string, string, number][] = [
  ['দুবার Shift, বাংলা চালু', 'Double-tap Shift to switch', 60],
  ['Word, Chrome, Facebook, সব জায়গায়', 'Works in every app', 120],
  ['Microsoft-এর মাধ্যমে সাইন করা', 'Signed through Microsoft', 180],
];
const LINUX_POINTS: [string, string, number][] = [
  ['Ubuntu: ppa:cicadama/ekushe', 'apt install ekushe, updates with your system', 30],
  ['Fedora: copr cicadama/ekushe', 'dnf install ekushe, for Fedora 43–45', 250],
  ['IBus-এ নিজের কিবোর্ড', 'A native IBus keyboard, in GTK and Qt apps', 500],
];
const MAC_POINTS: [string, string, number][] = [
  ['ইনপুট মেনু থেকে বেছে নিন', 'Pick it from the input menu', 60],
  ['macOS-এর নিজের পরামর্শ বার', "Suggestions in macOS's own candidate bar", 140],
  ['Apple silicon আর Intel', 'Apple silicon and Intel Macs', 220],
];

export const FullPromo: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const sun = interpolate(f, [0, 380, 420, 660, 2010, 2190, P.android, P.ios, P.engine, P.avail, P.end, P.end + 60], [0.15, 0.5, 1, 0.7, 0.6, 0.45, 0.6, 0.5, 1.0, 0.7, 0.8, 1.1], clamp);
  const fade = interpolate(f, [0, 30, FULL_PROMO_FRAMES - 40, FULL_PROMO_FRAMES], [0, 1, 1, 0], clamp);
  const petals = interpolate(f, [0, 300, P.devices, P.devices + 60, P.engine, P.engine + 40, P.end, P.end + 60], [0.4, 0.7, 0.4, 0.2, 0.2, 0.5, 0.5, 0.9], clamp);
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Audio src={staticFile('music/full_promo.wav')} />
      <AbsoluteFill style={{ opacity: fade }}>
        <Backdrop sun={sun} field={interpolate(f, [0, 420], [0.3, 1], clamp)} warmth={f > P.avail ? 1 : 0.35} />
        <Petals count={22} opacity={petals} />
        <Streaks from={P.devices - 20} to={P.devices + 30} count={30} color="rgba(255,255,255,0.85)" seed="fp1" />
        <Streaks from={P.engine - 20} to={P.engine + 26} count={30} seed="fp2" />
        <Sequence from={P.prologue} durationInFrames={P.logo - P.prologue}><Prologue /></Sequence>
        <Sequence from={P.logo} durationInFrames={P.features - P.logo}><Logo /></Sequence>
        {FEATURES.map((s, i) => (
          <Sequence key={i} from={P.features + i * FEAT_LEN} durationInFrames={FEAT_LEN}>
            <Feature i={i} title={s.title} sub={s.sub}>{s.body}</Feature>
          </Sequence>
        ))}
        <Sequence from={P.devices} durationInFrames={P.win - P.devices}><Devices /></Sequence>
        <Sequence from={P.win} durationInFrames={P.linux - P.win}>
          <Device os="win" soon={false} title="উইন্ডোজে, যেকোনো অ্যাপে" sub="Windows 10 and 11, in every app" points={WIN_POINTS} len={P.linux - P.win}>
            {(e, w) => <Screen w={w} enter={e} glow="rgba(96,205,255,0.25)"><WinScreen /></Screen>}
          </Device>
        </Sequence>
        <Sequence from={P.linux} durationInFrames={P.mac - P.linux}>
          <Device os="linux" soon={false} title="লিনাক্সে, দুটি কমান্ডে" sub="Linux: two commands, then updates come by themselves" points={LINUX_POINTS} len={P.mac - P.linux}>
            {(e, w) => <Screen w={w} enter={e} glow="rgba(233,84,32,0.28)"><LinuxScreen /></Screen>}
          </Device>
        </Sequence>
        <Sequence from={P.mac} durationInFrames={P.android - P.mac}>
          <Device os="mac" soon title="ম্যাকেও, নিজের মতো" sub="On the Mac, the way a Mac keyboard should work" points={MAC_POINTS} len={P.android - P.mac}>
            {(e, w) => <Screen w={w} enter={e} glow="rgba(44,123,246,0.22)"><MacScreen /></Screen>}
          </Device>
        </Sequence>
        <Sequence from={P.android} durationInFrames={P.ios - P.android}><Phone soon /></Sequence>
        <Sequence from={P.ios} durationInFrames={P.engine - P.ios}><IosScene /></Sequence>
        <Sequence from={P.engine} durationInFrames={P.avail - P.engine}><Engine /></Sequence>
        <Sequence from={P.avail} durationInFrames={P.end - P.avail}><Avail /></Sequence>
        <Sequence from={P.end} durationInFrames={FULL_PROMO_FRAMES - P.end}><End /></Sequence>
        <AbsoluteFill style={{ pointerEvents: 'none', boxShadow: `inset 0 0 ${260 * u}px rgba(0,0,0,0.65)` }} />
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
