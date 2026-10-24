import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals, ShaheedMinar } from '../v2/parts';
import { C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { AddressPill, Burst, Streaks, Sub, Sweep, graphemes } from './common';

/** "One keyboard, everywhere": the last teaser before the beta. Windows, macOS, Linux and now
 * Android, all on the one Ekushe engine. 33 s at 60 fps, 2K, 16:9 and 9:16.
 * music/everywhere.py is composed to the same marks. */
export const E = {
  open: 0,
  desk: 240,
  phone: 600,
  themes: 840,
  extras: 1060,
  engine: 1290,
  date: 1590,
  end: 1760,
} as const;
export const EVERYWHERE_FRAMES = 2000;

const Big: React.FC<{ children: React.ReactNode; size: number; style?: React.CSSProperties }> = ({ children, size, style }) => {
  const { u } = useLayout();
  return <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: size * u, lineHeight: 1.18, color: C.white, textAlign: 'center', ...style }}>{children}</div>;
};
const W = (f: number, a: number, b: number, d = 16) =>
  Math.min(interpolate(f, [a, a + d], [0, 1], clamp), interpolate(f, [b - d, b], [1, 0], clamp));

/* ───────────────────────── icons ───────────────────────── */
type Os = 'win' | 'mac' | 'linux' | 'android';
const OsIcon: React.FC<{ kind: Os; size: number }> = ({ kind, size }) => (
  <svg width={size} height={size} viewBox="0 0 24 24">
    {kind === 'win' && <path fill="currentColor" d="M3 5.6l7.4-1v7H3zm8.4-1.2L21 3v8.6h-9.6zM3 12.6h7.4v7l-7.4-1zm8.4 0H21V21l-9.6-1.4z" />}
    {kind === 'mac' && <path fill="currentColor" d="M16.4 12.6c0-2.4 2-3.6 2.1-3.7-1.2-1.7-3-1.9-3.6-2-1.5-.2-3 .9-3.8.9-.8 0-2-.9-3.3-.9-1.7 0-3.3 1-4.2 2.5-1.8 3.1-.5 7.7 1.3 10.2.9 1.2 1.9 2.6 3.2 2.6 1.3-.1 1.8-.8 3.3-.8s2 .8 3.3.8c1.4 0 2.3-1.2 3.1-2.5 1-1.4 1.4-2.8 1.4-2.9 0 0-2.8-1.1-2.8-4.2zM14 5.3c.7-.9 1.2-2 1-3.3-1 0-2.3.7-3 1.6-.6.7-1.2 1.9-1 3.1 1.1.1 2.3-.6 3-1.4z" />}
    {kind === 'linux' && <path fill="currentColor" d="M12 2c-2.2 0-3.4 1.9-3.4 4.3 0 1.4.3 2.3-.6 3.7-1 1.6-2.5 3.6-2.5 6.1 0 .6.1 1.1.2 1.6-.9.4-1.7.9-1.7 1.7 0 1.1 1.7 1.3 3.2 1.7 1.2.3 2 .9 3 .9.8 0 1.4-.4 1.8-.9.4.5 1 .9 1.8.9 1 0 1.8-.6 3-.9 1.5-.4 3.2-.6 3.2-1.7 0-.8-.8-1.3-1.7-1.7.1-.5.2-1 .2-1.6 0-2.5-1.5-4.5-2.5-6.1-.9-1.4-.6-2.3-.6-3.7C15.4 3.9 14.2 2 12 2z" />}
    {kind === 'android' && <path fill="currentColor" d="M7.2 8.6h9.6c.5 0 .9.4.9.9v7.2c0 .5-.4.9-.9.9h-.9v2.5a1.1 1.1 0 01-2.2 0v-2.5h-3.4v2.5a1.1 1.1 0 01-2.2 0v-2.5h-.9a.9.9 0 01-.9-.9V9.5c0-.5.4-.9.9-.9zM4.4 9a1.1 1.1 0 011.1 1.1v5a1.1 1.1 0 01-2.2 0v-5C3.3 9.5 3.8 9 4.4 9zm15.2 0a1.1 1.1 0 011.1 1.1v5a1.1 1.1 0 01-2.2 0v-5c0-.6.5-1.1 1.1-1.1zM15.6 3.1l-.9 1.5A5.3 5.3 0 0117.6 7.8H6.4a5.3 5.3 0 012.9-3.2l-.9-1.5a.3.3 0 01.5-.3l.9 1.6a5.8 5.8 0 014.4 0l.9-1.6a.3.3 0 01.5.3zM9.6 5.7a.6.6 0 100 1.2.6.6 0 000-1.2zm4.8 0a.6.6 0 100 1.2.6.6 0 000-1.2z" />}
  </svg>
);
const OS_NAME: Record<Os, string> = { win: 'Windows', mac: 'macOS', linux: 'Linux', android: 'Android' };

/* ───────────────────────── 0–4 s: one keyboard ───────────────────────── */
const Open: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const a = spring({ frame: f - 20, fps: 60, config: { damping: 14 } });
  const b = spring({ frame: f - 78, fps: 60, config: { damping: 12 } });
  const out = ease(f, 222, 240);
  const minar = ease(f, 0, 200);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', opacity: 1 - out, transform: `scale(${1 + out * 0.12})`, filter: `blur(${out * 12 * u}px)` }}>
      <div style={{ position: 'absolute', bottom: (V ? 300 : -60) * u, opacity: 0.5 * minar }}><ShaheedMinar size={(V ? 950 : 760) * u} rise={minar} glow={0.8} /></div>
      <AbsoluteFill style={{ background: 'radial-gradient(ellipse at 50% 45%, rgba(0,0,0,0) 0%, rgba(1,8,5,0.72) 75%)' }} />
      <div style={{ textAlign: 'center' }}>
        <Big size={V ? 150 : 150} style={{ opacity: a, transform: `translateY(${(1 - a) * 50 * u}px)` }}>এক কিবোর্ড।</Big>
        <Big size={V ? 170 : 170} style={{ color: C.greenBright, opacity: b, transform: `scale(${0.7 + b * 0.3})`, textShadow: `0 0 ${80 * u}px rgba(47,209,142,0.55)` }}>সব জায়গায়।</Big>
        <Sub size={42} style={{ marginTop: 18 * u, opacity: ease(f, 110, 140) }}>One keyboard. Everywhere.</Sub>
      </div>
      <Burst at={80} y={V ? 0.53 : 0.56} count={36} seed="op" spread={0.6} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 4–10 s: on the computer ───────────────────────── */
const DESK_TEXT = 'যেভাবে বলেন, সেভাবে লিখুন';
const DESK_ROMAN = ['jebhabe', 'bolen,', 'shebhabe', 'likhun'];
const DESK_BN = ['যেভাবে', 'বলেন,', 'সেভাবে', 'লিখুন'];
const Desk: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width } = useLayout();
  const enter = spring({ frame: f, fps: 60, config: { damping: 17, stiffness: 90 } });
  const out = ease(f, 340, 360);
  // one word every 50 frames: Roman letters typing, then the Bangla word placed
  const start = 40, per = 50;
  const wi = Math.min(DESK_ROMAN.length - 1, Math.max(0, Math.floor((f - start) / per)));
  const local = f - start - wi * per;
  const typing = f >= start && local < 34 && f < start + DESK_ROMAN.length * per;
  const shown = DESK_BN.slice(0, f < start ? 0 : wi + (local >= 34 ? 1 : 0)).join(' ');
  const roman = typing ? DESK_ROMAN[wi].slice(0, Math.max(1, Math.ceil((local / 30) * DESK_ROMAN[wi].length))) : '';
  const lw = V ? width * 0.9 : 1220 * u;
  const lh = lw * 0.6;
  const osTags: Os[] = ['win', 'mac', 'linux'];
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: 1 - out, transform: `translateY(${-out * 80 * u}px) scale(${1 - out * 0.06})` }}>
      <div style={{ textAlign: 'center', marginBottom: (V ? 70 : 40) * u, opacity: enter }}>
        <Big size={V ? 100 : 88}>আপনার কম্পিউটারে</Big>
        <Sub size={38} style={{ marginTop: 6 * u }}>On your computer</Sub>
      </div>
      <div style={{ perspective: 2800 * u }}>
        <div style={{ width: lw, height: lh, borderRadius: 26 * u, background: '#0C1713', border: `${10 * u}px solid #1B2622`, boxShadow: `0 ${40 * u}px ${120 * u}px -${30 * u}px rgba(0,0,0,0.9), 0 0 ${120 * u}px rgba(47,209,142,0.18)`, transform: `rotateX(${(1 - enter) * 30 + 4}deg) translateY(${(1 - enter) * 160 * u}px)`, overflow: 'hidden', position: 'relative' }}>
          <div style={{ height: 54 * u, background: '#16221D', display: 'flex', alignItems: 'center', gap: 12 * u, padding: `0 ${22 * u}px` }}>
            {['#FF5F57', '#FEBC2E', '#28C840'].map((c) => <div key={c} style={{ width: 18 * u, height: 18 * u, borderRadius: 99, background: c }} />)}
            <span style={{ marginLeft: 20 * u, fontFamily: FONT_LATIN, fontSize: 24 * u, color: 'rgba(234,243,238,0.55)' }}>চিঠি.docx</span>
          </div>
          <div style={{ padding: `${(V ? 60 : 70) * u}px ${(V ? 50 : 90) * u}px`, position: 'relative' }}>
            <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 70 : 86) * u, color: C.white, lineHeight: 1.5 }}>
              {shown}{shown && typing ? ' ' : ''}
              {typing && <span style={{ textDecoration: 'underline', textDecorationColor: C.greenBright, textUnderlineOffset: 10 * u }}>{DESK_BN[wi]}</span>}
              <span style={{ display: 'inline-block', width: 5 * u, height: (V ? 70 : 86) * u, background: C.greenBright, marginLeft: 6 * u, verticalAlign: '-0.12em', opacity: Math.floor(f / 18) % 2 ? 0.2 : 1 }} />
            </div>
            {typing && (
              <div style={{ position: 'absolute', left: (V ? 50 : 90) * u + Math.min(lw * 0.45, graphemes(shown).length * (V ? 30 : 38) * u), top: (V ? 170 : 200) * u, minWidth: 360 * u, borderRadius: 18 * u, background: 'rgba(20,34,28,0.96)', border: `${2 * u}px solid rgba(47,209,142,0.45)`, boxShadow: `0 ${20 * u}px ${60 * u}px rgba(0,0,0,0.6)`, padding: `${14 * u}px ${20 * u}px`, opacity: ease(local, 0, 6) }}>
                <div style={{ fontFamily: FONT_LATIN, fontSize: 26 * u, color: 'rgba(234,243,238,0.6)', marginBottom: 8 * u }}>{roman}</div>
                {[DESK_BN[wi], DESK_BN[wi] + 'ই'].map((c, i) => (
                  <div key={i} style={{ display: 'flex', gap: 16 * u, alignItems: 'center', padding: `${6 * u}px ${12 * u}px`, borderRadius: 12 * u, background: i === 0 ? 'rgba(47,209,142,0.22)' : 'transparent', fontFamily: FONT_BN, fontSize: 40 * u, color: C.white }}>
                    <b style={{ fontFamily: FONT_LATIN, fontSize: 24 * u, color: C.greenBright }}>{i + 1}</b>{c}
                  </div>
                ))}
              </div>
            )}
          </div>
          <Sweep at={start + DESK_ROMAN.length * per} dur={40} strength={0.14} />
        </div>
        <div style={{ width: lw * 1.08, height: 26 * u, marginLeft: -lw * 0.04, borderRadius: `0 0 ${30 * u}px ${30 * u}px`, background: 'linear-gradient(#2A3631, #121A17)', transform: `translateY(${(1 - enter) * 160 * u}px)` }} />
      </div>
      <div style={{ display: 'flex', gap: 26 * u, marginTop: (V ? 70 : 46) * u }}>
        {osTags.map((k, i) => {
          const t = spring({ frame: f - 220 - i * 10, fps: 60, config: { damping: 12 } });
          return <div key={k} style={{ display: 'flex', alignItems: 'center', gap: 14 * u, padding: `${12 * u}px ${26 * u}px`, borderRadius: 999, background: 'rgba(255,255,255,0.08)', border: `${2 * u}px solid rgba(255,255,255,0.2)`, color: C.white, opacity: t, transform: `translateY(${(1 - t) * 30 * u}px)` }}><OsIcon kind={k} size={40 * u} /><span style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 34 * u }}>{OS_NAME[k]}</span></div>;
        })}
      </div>
    </AbsoluteFill>
  );
};

/* ───────────────────────── 10–21.5 s: and now, the phone ───────────────────────── */
// Key centres in the keyboard picture (1080 × 708), from the keyboard's own report on the test phone
const KEY_X0 = [58, 111, 218], KEY_STEP = 107.3, ROW_Y = [190, 332, 473], SPACE = [647, 615];
const ROWS = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm'];
const keyAt = (ch: string): [number, number] => {
  if (ch === ' ') return [SPACE[0], SPACE[1]];
  for (let r = 0; r < 3; r++) { const i = ROWS[r].indexOf(ch); if (i >= 0) return [KEY_X0[r] + i * KEY_STEP, ROW_Y[r]]; }
  return [540, 400];
};
// What the strip shows while "amar sonar bangla" is typed
const PHRASE = 'amar sonar bangla ';
const WORDS: [string, string, string[]][] = [
  ['amar', 'আমার', ['আমার', 'আমরা', 'আমারও']],
  ['sonar', 'সোনার', ['সোনার', 'সোনা', 'সোনারও']],
  ['bangla', 'বাংলা', ['বাংলা', 'বাঙলা', 'বাংলাদেশ']],
];
const NEXT: Record<number, string[]> = { 0: ['সোনার', 'মনে', 'কাছে'], 1: ['বাংলা', 'তরী', 'হরিণ'], 2: ['আমি', 'তোমায়', 'ভালোবাসি'] };
const TYPE_START = 70, EVERY = 7;

const THEMES: [string, string, string][] = [
  ['ekushe', 'একুশে', 'Ekushe'], ['default', 'একুশে ডিফল্ট', 'Ekushe Default'], ['smooth', 'একুশে মসৃণ', 'Ekushe Smooth'],
  ['krishnachura', 'কৃষ্ণচূড়া', 'Krishnachura'], ['padma', 'পদ্মা', 'Padma'], ['shapla', 'শাপলা', 'Shapla'],
  ['kashful', 'কাশফুল', 'Kashful'], ['night', 'রাত', 'Night'], ['photo', 'নিজের ছবি', 'Your photo'],
];
const THEME_EVERY = 24;

/** `soon` turns the heading into the full promo's "coming later" one. */
export const Phone: React.FC<{ soon?: boolean }> = ({ soon }) => {
  const f = useCurrentFrame(); // 0 at E.phone
  const { u, V, width, height } = useLayout();
  const rise = spring({ frame: f - 6, fps: 60, config: { damping: 16, stiffness: 70 } });
  const out = ease(f, E.engine - E.phone - 22, E.engine - E.phone);
  const pw = V ? 790 * u : 590 * u;
  const ph = pw * 2.06;
  const kbH = pw * (708 / 1080);
  // typing
  const n = Math.max(0, Math.floor((f - TYPE_START) / EVERY));
  const typedRoman = PHRASE.slice(0, Math.min(PHRASE.length, n));
  let placed = '', word = '', wIndex = 0;
  {
    let rest = typedRoman;
    for (let i = 0; i < WORDS.length; i++) {
      const w = WORDS[i][0];
      if (rest.startsWith(w + ' ')) { placed += WORDS[i][1] + ' '; rest = rest.slice(w.length + 1); wIndex = i + 1; }
      else { word = rest; wIndex = i; break; }
    }
  }
  const composing = word.length > 0;
  const cands = composing ? (word === WORDS[wIndex][0] ? WORDS[wIndex][2] : [WORDS[wIndex][1].slice(0, Math.max(1, Math.round((word.length / WORDS[wIndex][0].length) * graphemes(WORDS[wIndex][1]).length))), WORDS[wIndex][2][1]]) : [];
  const preedit = composing ? graphemes(WORDS[wIndex][1]).slice(0, Math.max(1, Math.round((word.length / WORDS[wIndex][0].length) * graphemes(WORDS[wIndex][1]).length))).join('') : '';
  const preds = !composing && wIndex > 0 && typedRoman.length > 0 ? NEXT[wIndex - 1] : [];
  const lastKeyAt = TYPE_START + (n - 1) * EVERY;
  const pressing = n > 0 && n <= PHRASE.length && f - lastKeyAt < 6 ? PHRASE[n - 1] : null;
  // themes: the keyboard changes look, one after another
  const tf = f - (E.themes - E.phone);
  const ti = tf < 0 ? -1 : Math.min(THEMES.length - 1, Math.floor(tf / THEME_EVERY));
  const ef = f - (E.extras - E.phone);
  const extraImg = ef < 0 ? null : ef < 75 ? 'english' : ef < 150 ? 'emoji' : '3d';
  const kbSrc = extraImg ? `android/kb-${extraImg}.jpg` : ti >= 0 ? `android/kb-${THEMES[ti][0]}.jpg` : 'android/kb-ekushe-idle.jpg';
  const prevSrc = extraImg ? (ef < 75 ? `android/kb-${THEMES[THEMES.length - 1][0]}.jpg` : ef < 150 ? 'android/kb-english.jpg' : 'android/kb-emoji.jpg') : ti > 0 ? `android/kb-${THEMES[ti - 1][0]}.jpg` : 'android/kb-ekushe-idle.jpg';
  const swapAt = extraImg ? (ef < 75 ? 0 : ef < 150 ? 75 : 150) + (E.extras - E.phone) : ti >= 0 ? E.themes - E.phone + ti * THEME_EVERY : -999;
  const wipe = ease(f, swapAt, swapAt + 12);
  const showOverlayStrip = ti < 0 && !extraImg;
  const light = ti >= 0 && !extraImg && ['default', 'smooth', 'krishnachura', 'shapla', 'kashful'].includes(THEMES[ti][0]);
  const screenBg = light ? '#EDE7DF' : '#0E1613';
  const chatText = light ? '#13221B' : '#EAF3EE';
  // callouts beside the phone
  const callouts: [number, string, string][] = [
    [E.phone + 30, 'নতুন · Android', 'New'],
    [E.themes + 10, 'দেশের রঙে থিম', 'Themes in the colours of home'],
    [E.extras + 6, 'ইংরেজিও এখানেই', 'English too: teh → the'],
    [E.extras + 81, 'ইমোজি এক চাপে', 'Emoji in one tap'],
    [E.extras + 156, 'থ্রিডি কী, ইন্টারনেট ছাড়াই', '3D keys · no internet needed'],
  ];
  const G = f + E.phone;
  const active = callouts.filter(([at]) => G >= at).slice(-1)[0];
  const ci = active ? callouts.indexOf(active) : -1;
  const ct = active ? spring({ frame: G - active[0], fps: 60, config: { damping: 13 } }) : 0;
  const head = spring({ frame: f - 10, fps: 60, config: { damping: 14 } });
  const tilt = V ? 0 : interpolate(f, [0, 690], [-10, 8]);
  return (
    <AbsoluteFill style={{ opacity: 1 - out, transform: `scale(${1 - out * 0.08})` }}>
      <AbsoluteFill style={{ background: `radial-gradient(circle at ${V ? 50 : 64}% 55%, rgba(226,59,59,${0.22 * rise}) 0%, transparent 45%)` }} />
      {/* the words */}
      <div style={{ position: 'absolute', left: V ? 0 : 150 * u, right: V ? 0 : undefined, top: V ? 110 * u : 0, bottom: V ? undefined : 0, width: V ? undefined : 900 * u, display: 'flex', flexDirection: 'column', justifyContent: 'center', alignItems: V ? 'center' : 'flex-start', textAlign: V ? 'center' : 'left' }}>
        <div style={{ opacity: head, transform: `translateY(${(1 - head) * 40 * u}px)` }}>
          <div style={{ fontFamily: FONT_LATIN, fontWeight: 700, letterSpacing: '0.38em', fontSize: 30 * u, color: soon ? C.mustard : '#FF8A84' }}>{soon ? 'ANDROID · COMING LATER' : 'NEW · ANDROID'}</div>
          <Big size={V ? 120 : 130} style={{ textAlign: V ? 'center' : 'left', marginTop: 10 * u }}>{soon ? 'অ্যান্ড্রয়েড ফোনে' : 'এবার ফোনেও'}</Big>
          <Sub size={40} style={{ textAlign: V ? 'center' : 'left', marginTop: 4 * u }}>{soon ? 'On Android phones · শীঘ্রই আসছে' : 'Now on your phone'}</Sub>
        </div>
        {!V && active && ci > 0 && (
          <div key={ci} style={{ marginTop: 70 * u, opacity: ct, transform: `translateX(${(1 - ct) * -60 * u}px)` }}>
            <div style={{ display: 'inline-block', fontFamily: FONT_BN, fontWeight: 700, fontSize: 84 * u, color: C.white, padding: `${8 * u}px ${36 * u}px`, borderRadius: 26 * u, background: 'rgba(47,209,142,0.14)', border: `${2 * u}px solid rgba(47,209,142,0.5)`, boxShadow: `0 0 ${60 * u}px rgba(47,209,142,0.25)` }}>{active[1]}</div>
            <Sub size={38} style={{ textAlign: 'left', marginTop: 14 * u }}>{active[2]}</Sub>
            {ci === 1 && ti >= 0 && <div style={{ marginTop: 30 * u, fontFamily: FONT_BN, fontWeight: 600, fontSize: 60 * u, color: C.mustard }}>{THEMES[ti][1]} <span style={{ fontFamily: FONT_LATIN, fontSize: 34 * u, color: 'rgba(234,243,238,0.6)', fontWeight: 500 }}>{THEMES[ti][2]}</span></div>}
          </div>
        )}
      </div>
      {/* the phone */}
      <div style={{ position: 'absolute', left: V ? (width - pw) / 2 : width - pw - 380 * u, top: V ? 650 * u : (height - ph) / 2 + 20 * u, width: pw, height: ph, perspective: 3000 * u }}>
        <div style={{ width: '100%', height: '100%', borderRadius: pw * 0.13, padding: pw * 0.03, background: 'linear-gradient(150deg, #34403B, #0B0F0D 40%, #1F2925)', boxShadow: `0 0 0 ${2 * u}px rgba(255,255,255,0.1), 0 ${60 * u}px ${140 * u}px -${40 * u}px rgba(0,0,0,0.95), 0 0 ${180 * u}px rgba(47,209,142,${0.25 + 0.1 * Math.sin(f / 20)})`, transform: `translateY(${(1 - rise) * height * 0.8}px) rotateY(${tilt * (V ? 0 : 1)}deg) rotateZ(${(1 - rise) * 8}deg)`, position: 'relative' }}>
          <div style={{ position: 'absolute', top: pw * 0.055, left: '50%', width: pw * 0.24, height: pw * 0.06, borderRadius: 99, background: '#000', transform: 'translateX(-50%)', zIndex: 3 }} />
          <div style={{ width: '100%', height: '100%', borderRadius: pw * 0.1, overflow: 'hidden', background: screenBg, position: 'relative', display: 'flex', flexDirection: 'column' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: pw * 0.03, padding: `${pw * 0.13}px ${pw * 0.05}px ${pw * 0.03}px`, borderBottom: `1px solid ${light ? 'rgba(0,0,0,0.08)' : 'rgba(255,255,255,0.08)'}` }}>
              <div style={{ width: pw * 0.1, height: pw * 0.1, borderRadius: 99, display: 'grid', placeItems: 'center', background: 'linear-gradient(135deg, #E23B3B, #F26B4F)', color: '#fff', fontFamily: FONT_BN, fontWeight: 700, fontSize: pw * 0.05 }}>আ</div>
              <div><div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: pw * 0.048, color: chatText }}>আম্মু</div><div style={{ fontFamily: FONT_BN, fontSize: pw * 0.032, color: '#2FB37A' }}>অনলাইন</div></div>
            </div>
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'flex-end', gap: pw * 0.022, padding: `${pw * 0.04}px ${pw * 0.04}px ${pw * 0.03}px` }}>
              {[['them', 'বাবা, কেমন আছিস?'], ['me', 'ভালো আছি আম্মু 😊']].map(([who, t], i) => (
                <div key={i} style={{ alignSelf: who === 'me' ? 'flex-end' : 'flex-start', maxWidth: '82%', padding: `${pw * 0.022}px ${pw * 0.035}px`, borderRadius: pw * 0.045, background: who === 'me' ? (light ? '#D9FDD3' : '#0B6B4A') : (light ? '#FFFFFF' : '#202C27'), color: who === 'me' && light ? '#0E241B' : chatText, fontFamily: FONT_BN, fontWeight: 500, fontSize: pw * 0.045, opacity: ease(f, 20 + i * 14, 34 + i * 14) }}>{t}</div>
              ))}
              {(typedRoman.length > 0) && (
                <div style={{ alignSelf: 'flex-end', maxWidth: '86%', padding: `${pw * 0.022}px ${pw * 0.035}px`, borderRadius: pw * 0.045, background: light ? '#D9FDD3' : '#0B6B4A', color: light ? '#0E241B' : '#F2FBF6', fontFamily: FONT_BN, fontWeight: 600, fontSize: pw * 0.052, outline: `${2 * u}px dashed rgba(47,209,142,0.4)`, outlineOffset: 3 * u }}>
                  {placed}{composing && <span style={{ textDecoration: 'underline', textUnderlineOffset: 6 * u }}>{preedit}</span>}
                  <span style={{ display: 'inline-block', width: 3 * u, height: pw * 0.05, background: 'currentColor', marginLeft: 3 * u, verticalAlign: '-0.12em', opacity: Math.floor(f / 18) % 2 ? 0.2 : 1 }} />
                </div>
              )}
            </div>
            {/* the keyboard: the real one, from the test phone */}
            <div style={{ position: 'relative', width: '100%', height: kbH, flexShrink: 0 }}>
              <Img src={staticFile(prevSrc)} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%' }} />
              <Img src={staticFile(kbSrc)} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', clipPath: `inset(0 ${(1 - wipe) * 100}% 0 0)` }} />
              {wipe > 0 && wipe < 1 && <div style={{ position: 'absolute', top: 0, bottom: 0, left: `${wipe * 100}%`, width: 6 * u, background: 'rgba(255,255,255,0.9)', boxShadow: `0 0 ${30 * u}px #fff` }} />}
              {showOverlayStrip && typedRoman.length > 0 && (
                <div style={{ position: 'absolute', left: 0, right: 0, top: 0, height: kbH * (121 / 708), background: '#07372B', display: 'flex', alignItems: 'center', gap: pw * 0.016, padding: `0 ${pw * 0.02}px` }}>
                  {(composing ? cands : preds).map((c, i) => (
                    <div key={c + i} style={{ padding: `${pw * 0.01}px ${pw * 0.03}px`, borderRadius: pw * 0.02, background: composing && i === 0 ? '#E23B3B' : '#0E4D3A', color: '#F4FAF7', fontFamily: FONT_BN, fontWeight: composing && i === 0 ? 700 : 500, fontSize: pw * 0.04 }}>{c}</div>
                  ))}
                </div>
              )}
              {showOverlayStrip && pressing && (() => {
                const [x, y] = keyAt(pressing);
                const k = pw / 1080;
                const p = (f - lastKeyAt) / 6;
                return <div style={{ position: 'absolute', left: x * k, top: y * k, width: 110 * k, height: 110 * k, marginLeft: -55 * k, marginTop: -55 * k, borderRadius: 99, background: `rgba(255,255,255,${0.45 * (1 - p)})`, transform: `scale(${0.6 + p * 0.8})`, boxShadow: `0 0 ${40 * k}px rgba(47,209,142,0.8)` }} />;
              })()}
            </div>
          </div>
        </div>
      </div>
      {/* in the tall format, the callout sits under the heading */}
      {V && active && ci > 0 && (
        <div key={ci} style={{ position: 'absolute', top: 430 * u, left: 0, right: 0, textAlign: 'center', opacity: ct, transform: `translateY(${(1 - ct) * 30 * u}px)` }}>
          <div style={{ display: 'inline-block', fontFamily: FONT_BN, fontWeight: 700, fontSize: 72 * u, color: C.white, padding: `${6 * u}px ${32 * u}px`, borderRadius: 24 * u, background: 'rgba(47,209,142,0.14)', border: `${2 * u}px solid rgba(47,209,142,0.5)` }}>{active[1]}{ci === 1 && ti >= 0 ? ` · ${THEMES[ti][1]}` : ''}</div>
          <Sub size={34} style={{ marginTop: 10 * u }}>{active[2]}</Sub>
        </div>
      )}
      <Burst at={6} x={V ? 0.5 : 0.7} y={0.55} count={30} seed="ph" spread={0.55} />
      {[E.themes, E.extras].map((a) => <Burst key={a} at={a - E.phone} x={V ? 0.5 : 0.7} y={V ? 0.7 : 0.6} count={16} seed={`p${a}`} spread={0.3} />)}
    </AbsoluteFill>
  );
};

/* ───────────────────────── 21.5–26.5 s: one engine at the heart of every version ───────────────────────── */
const Engine: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width, height } = useLayout();
  const core = spring({ frame: f - 8, fps: 60, config: { damping: 11, mass: 0.9 } });
  const out = ease(f, 282, 300);
  const cx = width / 2, cy = height * (V ? 0.52 : 0.56);
  const R = (V ? 470 : 470) * u;
  const os: Os[] = ['win', 'mac', 'linux', 'android'];
  const pos = os.map((_, i) => {
    const a = V ? (-Math.PI / 2 + (i * Math.PI) / 2 + Math.PI / 4) : (Math.PI + (i * Math.PI) / 3);
    return V ? [cx + Math.cos(a) * R * 0.95, cy + Math.sin(a) * R * 1.15] : [cx + Math.cos(a) * R * 1.55, cy + Math.sin(a) * R * 0.62 - 40 * u];
  });
  const pulse = (f % 60) / 60;
  const head = spring({ frame: f - 60, fps: 60, config: { damping: 14 } });
  return (
    <AbsoluteFill style={{ opacity: 1 - out, filter: `blur(${out * 12 * u}px)` }}>
      <svg width={width} height={height} style={{ position: 'absolute', inset: 0 }}>
        <defs>
          <radialGradient id="ecore"><stop offset="0" stopColor="#FFFFFF" /><stop offset="0.25" stopColor="#5BE3AA" /><stop offset="0.7" stopColor="#0B6B4A" /><stop offset="1" stopColor="#0B6B4A" stopOpacity="0" /></radialGradient>
        </defs>
        {pos.map(([x, y], i) => {
          const t = ease(f, 26 + i * 10, 60 + i * 10);
          const px = cx + (x - cx) * ((pulse + i * 0.25) % 1), py = cy + (y - cy) * ((pulse + i * 0.25) % 1);
          return (
            <g key={i}>
              <line x1={cx} y1={cy} x2={cx + (x - cx) * t} y2={cy + (y - cy) * t} stroke="rgba(47,209,142,0.55)" strokeWidth={4 * u} strokeDasharray={`${14 * u} ${10 * u}`} strokeDashoffset={-f * 2 * u} />
              {t >= 1 && <circle cx={px} cy={py} r={9 * u} fill="#E9F9F1" opacity={0.9} />}
            </g>
          );
        })}
        <circle cx={cx} cy={cy} r={(150 + Math.sin(f / 9) * 6) * u * core} fill="url(#ecore)" opacity={0.95} />
        {[0, 30].map((d) => { const r = ((f + d) % 60) / 60; return <circle key={d} cx={cx} cy={cy} r={(120 + r * 220) * u * core} fill="none" stroke={`rgba(47,209,142,${0.5 * (1 - r)})`} strokeWidth={3 * u} />; })}
      </svg>
      <div style={{ position: 'absolute', left: cx - 120 * u, top: cy - 120 * u, width: 240 * u, height: 240 * u, display: 'grid', placeItems: 'center', transform: `scale(${core})` }}>
        <Img src={staticFile('assets/logo_512.png')} style={{ width: 150 * u, height: 150 * u, borderRadius: 38 * u, boxShadow: `0 0 ${80 * u}px rgba(255,255,255,0.6)` }} />
      </div>
      {pos.map(([x, y], i) => {
        const t = spring({ frame: f - 40 - i * 10, fps: 60, config: { damping: 11 } });
        const s = (V ? 190 : 170) * u;
        return (
          <div key={i} style={{ position: 'absolute', left: x - s / 2, top: y - s / 2, width: s, textAlign: 'center', color: C.white, opacity: t, transform: `scale(${0.6 + t * 0.4})` }}>
            <div style={{ width: s, height: s, borderRadius: s * 0.26, display: 'grid', placeItems: 'center', background: os[i] === 'android' ? 'rgba(226,59,59,0.22)' : 'rgba(255,255,255,0.08)', border: `${2 * u}px solid ${os[i] === 'android' ? 'rgba(255,138,132,0.7)' : 'rgba(255,255,255,0.22)'}`, boxShadow: `0 0 ${60 * u * t}px ${os[i] === 'android' ? 'rgba(226,59,59,0.35)' : 'rgba(47,209,142,0.25)'}`, backdropFilter: `blur(${10 * u}px)` }}><OsIcon kind={os[i]} size={s * 0.52} /></div>
            <div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 38 * u, marginTop: 14 * u }}>{OS_NAME[os[i]]}</div>
          </div>
        );
      })}
      <div style={{ position: 'absolute', left: 0, right: 0, top: (V ? 230 : 120) * u, textAlign: 'center', opacity: head, transform: `translateY(${(1 - head) * 40 * u}px)`, padding: `0 ${60 * u}px` }}>
        <Big size={V ? 100 : 96}>সব সংস্করণের প্রাণ <span style={{ color: C.greenBright }}>একটাই ইঞ্জিন</span></Big>
        <Sub size={38} style={{ marginTop: 10 * u }}>One Ekushe engine at the heart of every version</Sub>
      </div>
      <div style={{ position: 'absolute', left: 0, right: 0, bottom: (V ? 260 : 90) * u, display: 'flex', justifyContent: 'center', gap: 20 * u, flexWrap: 'wrap', padding: `0 ${60 * u}px` }}>
        {['একই অভিধান', 'একই গতি', 'একই নির্ভুল বানান', 'সম্পূর্ণ ফ্রি'].map((w, i) => {
          const t = spring({ frame: f - 140 - i * 12, fps: 60, config: { damping: 12 } });
          const on = i === 3;
          return <div key={w} style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 50 : 46) * u, color: on ? '#03170F' : C.white, background: on ? C.greenBright : 'rgba(255,255,255,0.08)', border: `${2 * u}px solid ${on ? C.greenBright : 'rgba(255,255,255,0.2)'}`, borderRadius: 999, padding: `${8 * u}px ${30 * u}px`, opacity: t, transform: `translateY(${(1 - t) * 30 * u}px)`, boxShadow: on ? `0 0 ${50 * u}px rgba(47,209,142,0.5)` : 'none' }}>{w}</div>;
        })}
      </div>
      <Burst at={8} x={0.5} y={V ? 0.52 : 0.56} count={44} seed="eng" spread={0.7} />
      <Sweep at={200} dur={50} strength={0.12} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 26.5–29.3 s: the date ───────────────────────── */
const DateCard: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const a = spring({ frame: f - 4, fps: 60, config: { damping: 14 } });
  const b = spring({ frame: f - 24, fps: 60, config: { damping: 9, stiffness: 160 } });
  const out = ease(f, 152, 170);
  const m = ease(f, 0, 120);
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', opacity: 1 - out }}>
      <div style={{ position: 'absolute', bottom: (V ? 300 : -30) * u, opacity: 0.75 * m }}><ShaheedMinar size={(V ? 1000 : 820) * u} rise={m} glow={1} /></div>
      <AbsoluteFill style={{ background: 'linear-gradient(180deg, rgba(1,8,5,0.7), rgba(1,8,5,0.1) 45%, rgba(1,8,5,0.6))' }} />
      <AbsoluteFill style={{ background: `radial-gradient(circle at 50% 45%, rgba(233,185,73,${0.3 * b * (1 - ease(f, 40, 150))}) 0%, transparent 50%)` }} />
      <div style={{ textAlign: 'center', marginTop: (V ? -260 : -120) * u }}>
        <div style={{ fontFamily: FONT_LATIN, fontWeight: 700, letterSpacing: '0.42em', fontSize: 30 * u, color: C.mustard, opacity: a }}>PUBLIC BETA</div>
        <Big size={V ? 90 : 84} style={{ fontWeight: 600, opacity: a, marginTop: 8 * u }}>বেটা উদ্বোধন</Big>
        <Big size={V ? 210 : 200} style={{ color: C.mustard, textShadow: `0 0 ${90 * u}px rgba(233,185,73,0.55)`, opacity: Math.min(1, b * 1.4), transform: `scale(${0.6 + b * 0.4})` }}>৩০ অক্টোবর</Big>
        <Sub size={42} style={{ marginTop: 6 * u, opacity: ease(f, 40, 60) }}>October 30, 2026 · Windows · macOS · Linux · Android</Sub>
      </div>
      <Burst at={24} y={V ? 0.42 : 0.46} count={60} seed="dt" spread={0.9} />
      <Sweep at={34} dur={50} strength={0.18} />
    </AbsoluteFill>
  );
};

/* ───────────────────────── 29.3–33.3 s: where to find it ───────────────────────── */
const End: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const pop = spring({ frame: f - 4, fps: 60, config: { damping: 11, mass: 0.8 } });
  const ring = ease(f, 4, 50);
  const os: Os[] = ['win', 'mac', 'linux', 'android'];
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [0, 12], [0, 1], clamp) }}>
      <div style={{ position: 'relative' }}>
        <div style={{ position: 'absolute', left: '50%', top: '50%', width: 300 * u, height: 300 * u, borderRadius: 999, border: `${3 * u}px solid rgba(255,255,255,${0.5 * (1 - ring)})`, transform: `translate(-50%,-50%) scale(${0.8 + ring * 1.6})` }} />
        <Img src={staticFile('assets/logo_512.png')} style={{ width: (V ? 240 : 200) * u, height: (V ? 240 : 200) * u, borderRadius: 50 * u, transform: `scale(${0.6 + pop * 0.4})`, opacity: Math.min(1, pop * 1.5), boxShadow: `0 ${30 * u}px ${80 * u}px rgba(0,0,0,0.55), 0 0 ${120 * u}px rgba(224,48,63,0.5)` }} />
      </div>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 130 : 112) * u, color: C.white, marginTop: 22 * u, lineHeight: 1.1, opacity: ease(f, 10, 30) }}>একুশে নেক্সট</div>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 18 * u, marginTop: 14 * u, opacity: ease(f, 22, 42), flexWrap: 'wrap', justifyContent: 'center' }}>
        <span style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 58 : 52) * u, color: C.mustard }}>বেটা উদ্বোধন ৩০ অক্টোবর ২০২৬</span>
        <span style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: 34 * u, color: 'rgba(234,243,238,0.7)' }}>Beta launch October 30, 2026</span>
      </div>
      <div style={{ display: 'flex', gap: 34 * u, marginTop: 30 * u, color: 'rgba(234,243,238,0.85)' }}>
        {os.map((k, i) => {
          const t = spring({ frame: f - 30 - i * 6, fps: 60, config: { damping: 12 } });
          return <div key={k} style={{ display: 'flex', alignItems: 'center', gap: 10 * u, opacity: t, transform: `translateY(${(1 - t) * 20 * u}px)` }}><OsIcon kind={k} size={38 * u} /><span style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 30 * u }}>{OS_NAME[k]}</span></div>;
        })}
      </div>
      <div style={{ display: 'flex', flexDirection: V ? 'column' : 'row', alignItems: 'center', gap: (V ? 26 : 30) * u, marginTop: (V ? 56 : 46) * u }}>
        <AddressPill kind="web" text="www.ekushenext.com" at={58} />
        <AddressPill kind="mail" text="support@ekushenext.com" at={96} />
      </div>
      <Burst at={4} y={V ? 0.34 : 0.28} count={34} seed="evend" spread={0.6} />
    </AbsoluteFill>
  );
};

export const Everywhere: React.FC = () => {
  const f = useCurrentFrame();
  const { u } = useLayout();
  const sun = interpolate(f, [0, 200, 240, 600, 640, 1290, 1330, 1590, 1620], [0.25, 0.6, 0.9, 0.6, 1.0, 0.7, 1.1, 0.7, 1.2], clamp);
  const fade = interpolate(f, [0, 24, EVERYWHERE_FRAMES - 30, EVERYWHERE_FRAMES], [0, 1, 1, 0], clamp);
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Audio src={staticFile('music/everywhere.wav')} />
      <AbsoluteFill style={{ opacity: fade }}>
        <Backdrop sun={sun} field={interpolate(f, [0, 240], [0.4, 1], clamp)} warmth={f > E.date ? 1 : 0.4} />
        <Petals count={20} opacity={interpolate(f, [0, 200, E.phone, E.phone + 60, E.date, E.date + 40], [0.3, 0.6, 0.6, 0.25, 0.25, 0.8], clamp)} />
        <Streaks from={E.phone - 20} to={E.phone + 30} count={30} color="rgba(255,255,255,0.85)" seed="t1" />
        <Streaks from={E.engine - 20} to={E.engine + 26} count={30} seed="t2" />
        <Sequence from={E.open} durationInFrames={E.desk - E.open}><Open /></Sequence>
        <Sequence from={E.desk} durationInFrames={E.phone - E.desk}><Desk /></Sequence>
        <Sequence from={E.phone} durationInFrames={E.engine - E.phone}><Phone /></Sequence>
        <Sequence from={E.engine} durationInFrames={E.date - E.engine}><Engine /></Sequence>
        <Sequence from={E.date} durationInFrames={E.end - E.date}><DateCard /></Sequence>
        <Sequence from={E.end} durationInFrames={EVERYWHERE_FRAMES - E.end}><End /></Sequence>
        <AbsoluteFill style={{ pointerEvents: 'none', boxShadow: `inset 0 0 ${260 * u}px rgba(0,0,0,0.65)` }} />
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
