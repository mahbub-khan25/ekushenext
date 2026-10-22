import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals } from '../v2/parts';
import { bn, C, FONT_BN, FONT_LATIN, useLayout } from '../v2/theme';
import { Burst, END_LEN, EndCard, Sub, Sweep } from './common';
import META from './anthem_meta.json';

/** "আমার সোনার বাংলা", typed live with Ekushe Next in a real note app. Every picture of the app is a
 * real screen capture taken one keystroke at a time (public/anthem/). 60 fps. */

// Canvas of the captured screen area (screen x 530–1456, y 55–800 at 1.7593 px per screen px)
const CW = 1629, CH = 1290;

type Step = { k: string | null; img: string; d: number };
const dur = (k: string | null) => (k === 'Tab' ? 30 : k === 'Esc' ? 18 : k === 'Space' ? 15 : k === null ? 44 : 12);

export const A = { intro: 0, sw: 150, shift1: 262, shift2: 280, title: 430 } as const;
const TITLE: Step[] = META.title.map((s, i) => ({ k: s.k, img: s.img, d: i === 0 ? 0 : i === 1 ? 30 : 13 }));
TITLE[TITLE.length - 1].d = 50;
const LAPSE_EVERY = 5;
const LAST: Step[] = META.last.map((s) => ({ k: s.k, img: s.img, d: dur(s.k) }));
LAST[LAST.length - 1].d = 70;

const starts = (steps: Step[], at: number) => {
  const out: number[] = [];
  let t = at;
  for (const s of steps) { out.push(t); t += s.d; }
  return { out, end: t };
};
const TS = starts(TITLE.slice(1), A.title + 10);
export const LAPSE_AT = TS.end + 10;
const LAPSE_END = LAPSE_AT + META.lapse.length * LAPSE_EVERY;
export const LAST_AT = LAPSE_END + 30;
const LS = starts(LAST, LAST_AT + 20);
export const REVEAL_AT = LS.end + 10;
export const END_AT = REVEAL_AT + 330;
export const ANTHEM_FRAMES = END_AT + END_LEN;
/** Frames of every real keystroke shown (the music puts a soft click on each). */
export const KEY_FRAMES = [
  A.shift1, A.shift2,
  ...TS.out.filter((_, i) => TITLE[i + 1].k),
  ...LS.out.filter((_, i) => LAST[i].k),
];

/** Which captured image to show at global frame g, and the key pressed with it. */
const pick = (g: number): { img: string; key: string | null; keyAt: number; pop?: boolean; last?: boolean; px?: number[] } => {
  if (g < A.title + 10) return { img: TITLE[0].img, key: null, keyAt: 0 };
  if (g < LAPSE_AT) {
    let i = 0;
    while (i + 1 < TS.out.length && g >= TS.out[i + 1]) i++;
    return { img: TITLE[i + 1].img, key: TITLE[i + 1].k, keyAt: TS.out[i], pop: META.title[i + 1].pop, px: (META.title[i + 1] as { px?: number[] }).px };
  }
  if (g < LAST_AT + 20) {
    const i = Math.min(META.lapse.length - 1, Math.floor((g - LAPSE_AT) / LAPSE_EVERY));
    return { img: META.lapse[Math.max(0, i)].img, key: null, keyAt: 0 };
  }
  let i = 0;
  while (i + 1 < LS.out.length && g >= LS.out[i + 1]) i++;
  return { img: LAST[i].img, key: LAST[i].k, keyAt: LS.out[i], pop: META.last[i].pop, last: true, px: (META.last[i] as { px?: number[] }).px };
};

/** The camera: which part of the captured canvas fills the viewport. */
type Rect = [number, number, number, number];
const lerpR = (a: Rect, b: Rect, t: number): Rect => a.map((v, i) => v + (b[i] - v) * t) as Rect;
const WHOLE: Rect = [0, 0, CW, 1270];
const TITLE_R = (V: boolean): Rect => (V ? [-30, 30, 640, 735] : [-30, 40, 1000, 561]);
const LAST_R = (V: boolean): Rect => (V ? [-30, 160, 640, 735] : [-30, 300, 1000, 561]);
const camera = (g: number, V: boolean): Rect => {
  const lapseLine = (i: number) => META.lapse[Math.max(0, Math.min(META.lapse.length - 1, i))].y;
  if (g < LAPSE_AT) return TITLE_R(V);
  if (g < LAST_AT) {
    const i = Math.floor((g - LAPSE_AT) / LAPSE_EVERY);
    // follow the line being typed, smoothed over a few states
    let y = 0;
    for (let k = -8; k <= 8; k++) y += lapseLine(i + k);
    y /= 17;
    const [, ty, w, h] = TITLE_R(V);
    const top = Math.max(ty, y + 150 - h);
    return [-30, top, w, h];
  }
  const prev = camera(LAST_AT - 1, V);
  if (g < REVEAL_AT) return lerpR(prev, LAST_R(V), ease(g, LAST_AT - 10, LAST_AT + 40));
  return lerpR(LAST_R(V), WHOLE, ease(g, REVEAL_AT, REVEAL_AT + 60));
};

/** A key on the keyboard, pressed at frame `at`. */
const Keycap: React.FC<{ label: string; at: number; now: number; size?: number; dim?: number }> = ({ label, at, now, size = 1, dim = 1 }) => {
  const f = now;
  const { u } = useLayout();
  const t = f - at;
  const pop = spring({ frame: t, fps: 60, config: { damping: 12, stiffness: 220 } });
  const press = t >= 0 && t < 6 ? 1 : interpolate(t, [6, 14], [1, 0], clamp);
  const wide = label.length > 1;
  const h = 104 * u * size;
  return (
    <div style={{ height: h, minWidth: h, padding: wide ? `0 ${30 * u * size}px` : 0, display: 'grid', placeItems: 'center', borderRadius: 22 * u * size, fontFamily: FONT_LATIN, fontWeight: 700, fontSize: (wide ? 40 : 54) * u * size, color: C.white, background: `linear-gradient(180deg, rgba(20,70,52,${0.95}), rgba(4,26,18,0.95))`, border: `${2 * u}px solid rgba(47,209,142,${0.35 + press * 0.6})`, boxShadow: `0 ${(8 - press * 6) * u}px 0 rgba(0,0,0,0.45), 0 0 ${50 * u * press}px rgba(47,209,142,${0.7 * press})`, transform: `translateY(${press * 6 * u}px) scale(${0.6 + pop * 0.4})`, opacity: Math.min(1, pop * 1.4) * dim }}>
      {label}
    </div>
  );
};

/** The last few keys pressed, newest on the right. */
const KeyTrail: React.FC<{ g: number; o: number }> = ({ g, o }) => {
  const { u } = useLayout();
  const keys: { k: string; at: number }[] = [];
  TS.out.forEach((at, i) => { const k = TITLE[i + 1].k; if (k && at <= g && g < LAPSE_AT) keys.push({ k, at }); });
  LS.out.forEach((at, i) => { const k = LAST[i].k; if (k && at <= g && g < REVEAL_AT) keys.push({ k, at }); });
  const show = keys.slice(-5);
  return (
    <div style={{ display: 'flex', gap: 16 * u, alignItems: 'center', opacity: o }}>
      {show.map((s, i) => <Keycap key={s.at} label={s.k} at={s.at} now={g} dim={0.35 + 0.65 * ((i + 1) / show.length) ** 2} />)}
    </div>
  );
};

/** The real Ekushe Next suggestion popup from the same capture, lifted out and enlarged. */
const PopupCard: React.FC<{ g: number; vw: number; img: string; last?: boolean; on: boolean; px?: number[] }> = ({ g, vw, img, last, on, px }) => {
  const { u, V } = useLayout();
  const [cy, ch] = last ? [968, 150] : [1145, 124];
  const [x0, x1] = px ?? [1000, 1629];
  const k = (V ? 1.62 : 1.55) * u; // enlargement of the real popup
  const W = Math.min((x1 - x0) * k, V ? vw - 60 * u : vw * 0.62);
  const o = on && g < REVEAL_AT ? 1 : 0;
  return (
    <div style={{ position: 'absolute', right: 30 * u, bottom: (V ? 170 : 30) * u, width: W, height: ch * k, overflow: 'hidden', borderRadius: 20 * u, background: '#F2F2F0', opacity: o, boxShadow: `0 0 0 ${3 * u}px rgba(47,209,142,0.75), 0 ${24 * u}px ${70 * u}px rgba(0,0,0,0.45)` }}>
      <Img src={staticFile(`anthem/${img}`)} style={{ position: 'absolute', right: -(1629 - x1) * k, top: -cy * k, width: CW * k, height: CW * k * (1311 / 1629), maxWidth: 'none' }} />
    </div>
  );
};

/** The captured screen, framed like a window, with the camera applied. */
const Screen: React.FC<{ g: number; vw: number; vh: number }> = ({ g, vw, vh }) => {
  const { u, V } = useLayout();
  const p = pick(g);
  const [x, y, w, h] = camera(g, V);
  const s = Math.min(vw / w, vh / h);
  const ox = (vw - w * s) / 2 - x * s, oy = (vh - h * s) / 2 - y * s;
  const reveal = g >= REVEAL_AT;
  const full = ease(g, REVEAL_AT + 40, REVEAL_AT + 90);
  const img = reveal ? LAST[LAST.length - 1].img : p.img;
  const lapse = g >= LAPSE_AT && g < LAST_AT;
  const li = Math.max(0, Math.min(META.lapse.length - 1, Math.floor((g - LAPSE_AT) / LAPSE_EVERY)));
  const line = Math.max(1, Math.min(10, Math.round((META.lapse[li].y - 279) / 31.5) + 1));
  return (
    <div style={{ position: 'relative', width: vw, height: vh, borderRadius: 28 * u, overflow: 'hidden', background: '#F2F2F0', boxShadow: `0 0 0 ${2 * u}px rgba(255,255,255,0.16), 0 ${50 * u}px ${130 * u}px -${30 * u}px rgba(0,0,0,0.9), 0 0 ${140 * u}px -${30 * u}px rgba(47,209,142,0.35)` }}>
      <Img src={staticFile(`anthem/${img}`)} style={{ position: 'absolute', left: ox, top: oy, width: CW * s, height: CW * s * (1311 / 1629), maxWidth: 'none' }} />
      {reveal && (
        // the whole app at the end: the real full-screen capture
        <Img src={staticFile('anthem/final_full.jpg')} style={{ position: 'absolute', left: 0, top: 0, width: vw, height: vh, objectFit: 'contain', background: '#F2F2F0', opacity: full }} />
      )}
      {g >= A.title && g < REVEAL_AT && <PopupCard g={g} vw={vw} img={p.img} last={p.last} on={!!p.pop} px={p.px} />}
      <div style={{ position: 'absolute', left: 34 * u, bottom: 34 * u, display: 'flex', gap: 20 * u, alignItems: 'center' }}>
        {g < A.title + 10 && g >= A.shift1 - 10 && (
          <div style={{ display: 'flex', gap: 24 * u, opacity: interpolate(g, [A.title - 10, A.title + 10], [1, 0], clamp) }}>
            <Keycap label="Shift" at={A.shift1} now={g} size={1.15} />
            <Keycap label="Shift" at={A.shift2} now={g} size={1.15} />
          </div>
        )}
        {g >= A.title + 10 && !lapse && <KeyTrail g={g} o={interpolate(g, [REVEAL_AT - 10, REVEAL_AT + 10], [1, 0], clamp)} />}
        {lapse && (
          <div style={{ display: 'flex', gap: 20 * u, opacity: Math.min(ease(g, LAPSE_AT, LAPSE_AT + 20), interpolate(g, [LAST_AT - 15, LAST_AT], [1, 0], clamp)) }}>
            <Chip o={1}><span>টাইম-ল্যাপস</span><span style={{ fontFamily: FONT_LATIN, fontSize: 30 * u, opacity: 0.7 }}>Time-lapse</span></Chip>
            <Chip o={1}><span>লাইন {bn(line)} / ১০</span></Chip>
          </div>
        )}
      </div>
      <ModeBadge g={g} />
    </div>
  );
};

const Caption: React.FC<{ bnText: string; en: string; at: number; until: number; big?: boolean }> = ({ bnText, en, at, until, big }) => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const o = Math.min(ease(f, at, at + 18), interpolate(f, [until - 14, until], [1, 0], clamp));
  if (o <= 0) return null;
  return (
    <div style={{ textAlign: 'center', opacity: o, transform: `translateY(${(1 - o) * 20 * u}px)` }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (big ? (V ? 84 : 74) : V ? 70 : 60) * u, color: C.white, lineHeight: 1.2 }}>{bnText}</div>
      <Sub size={V ? 36 : 32} style={{ marginTop: 4 * u }}>{en}</Sub>
    </div>
  );
};

const Chip: React.FC<{ children: React.ReactNode; o: number }> = ({ children, o }) => {
  const { u } = useLayout();
  return <div style={{ display: 'flex', alignItems: 'center', gap: 14 * u, padding: `${12 * u}px ${28 * u}px`, borderRadius: 999, background: 'rgba(6,40,29,0.92)', border: `${2 * u}px solid rgba(47,209,142,0.6)`, opacity: o, fontFamily: FONT_BN, fontWeight: 600, fontSize: 40 * u, color: C.white }}>{children}</div>;
};

/** The real mode indicators of Ekushe Next: English, then Bangla after Shift Shift. */
const ModeBadge: React.FC<{ g: number }> = ({ g }) => {
  const { u } = useLayout();
  if (g < A.sw + 20 || g > A.title + 30) return null;
  const o = Math.min(ease(g, A.sw + 20, A.sw + 44), interpolate(g, [A.title, A.title + 30], [1, 0], clamp));
  const bnOn = g >= A.shift2 + 4;
  const t = spring({ frame: g - (A.shift2 + 4), fps: 60, config: { damping: 11, stiffness: 180 } });
  const H = 92 * u;
  return (
    <div style={{ position: 'absolute', right: 40 * u, bottom: 40 * u, opacity: o, display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: 10 * u }}>
      <Img src={staticFile(bnOn ? 'anthem/badge_bn_t.png' : 'anthem/badge_en_t.png')} style={{ height: H, borderRadius: 16 * u, boxShadow: `0 ${16 * u}px ${50 * u}px rgba(0,0,0,0.35)${bnOn ? `, 0 0 ${70 * u * (1 - t * 0.5)}px rgba(47,209,142,0.8)` : ''}`, transform: bnOn ? `scale(${0.75 + t * 0.25})` : 'none' }} />
      <div style={{ fontFamily: FONT_LATIN, fontSize: 26 * u, fontWeight: 600, color: 'rgba(14,36,27,0.65)' }}>{bnOn ? 'Ekushe Next: বাংলা' : 'Ekushe Next: English'}</div>
    </div>
  );
};

/** 0–2.5 s: the title. */
const Intro: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const t = spring({ frame: f - 6, fps: 60, config: { damping: 12, stiffness: 120 } });
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [130, 150], [1, 0], clamp) }}>
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 170 : 160) * u, color: C.white, lineHeight: 1.15, textAlign: 'center', transform: `scale(${1.4 - t * 0.4})`, opacity: Math.min(1, t * 1.4), textShadow: `0 0 ${70 * u}px rgba(224,48,63,0.55)` }}>আমার সোনার বাংলা</div>
      <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 60 : 54) * u, color: C.mustard, marginTop: 18 * u, opacity: ease(f, 30, 54) }}>একুশে নেক্সটে লাইভ লেখা</div>
      <Sub o={ease(f, 44, 70)} size={36} style={{ marginTop: 8 * u }}>Typed live with Ekushe Next · real screen capture</Sub>
    </AbsoluteFill>
  );
};

const Stage: React.FC = () => {
  const g = useCurrentFrame() + A.sw;
  const { u, V, width, height } = useLayout();
  const vw = V ? width - 80 * u : 1960 * u;
  const vh = V ? 1560 * u : 1100 * u;
  const enter = spring({ frame: g - A.sw, fps: 60, config: { damping: 18, stiffness: 80 } });
  const out = interpolate(g, [END_AT - 20, END_AT], [1, 0], clamp);
  const top = V ? 250 * u : 40 * u;
  const capH = V ? 230 * u : 150 * u;
  return (
    <AbsoluteFill style={{ opacity: out }}>
      <div style={{ position: 'absolute', top, left: 0, right: 0, height: capH, display: 'grid', placeItems: 'center' }}>
        <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center' }}>
          <Sequence from={0} durationInFrames={A.title - A.sw + 10} layout="none"><Caption bnText="Shift দু’বার চাপুন, বাংলা চালু" en="Tap Shift twice to switch to Bangla" at={20} until={A.title - A.sw + 10} /></Sequence>
        </AbsoluteFill>
        <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center' }}>
          <Sequence from={A.title - A.sw} durationInFrames={LAPSE_AT - A.title + 10} layout="none"><Caption bnText="যেমন উচ্চারণ, তেমন লিখুন" en="Type it the way it sounds" at={6} until={LAPSE_AT - A.title + 10} /></Sequence>
        </AbsoluteFill>
        <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center' }}>
          <Sequence from={LAPSE_AT - A.sw} durationInFrames={LAST_AT - LAPSE_AT + 10} layout="none"><Caption bnText="পুরো জাতীয় সংগীত, প্রতিটি শব্দ লাইভ" en="The whole national anthem, every word typed live" at={6} until={LAST_AT - LAPSE_AT + 10} /></Sequence>
        </AbsoluteFill>
        <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center' }}>
          <Sequence from={LAST_AT - A.sw} durationInFrames={REVEAL_AT - LAST_AT + 10} layout="none"><LastCaption /></Sequence>
        </AbsoluteFill>
        <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center' }}>
          <Sequence from={REVEAL_AT - A.sw} durationInFrames={END_AT - REVEAL_AT} layout="none"><Caption big bnText="সম্পূর্ণ ১০ লাইন, একটিও কপি-পেস্ট নয়" en="All ten lines, not a single copy-paste" at={30} until={END_AT - REVEAL_AT} /></Sequence>
        </AbsoluteFill>
      </div>
      <div style={{ position: 'absolute', top: top + capH + (V ? 30 : 10) * u, left: (width - vw) / 2, opacity: enter, transform: `translateY(${(1 - enter) * 300 * u}px) scale(${0.85 + enter * 0.15})` }}>
        <Screen g={g} vw={vw} vh={vh} />
        <Sweep at={REVEAL_AT - A.sw + 60} dur={50} strength={0.16} />
      </div>
      <Burst at={REVEAL_AT - A.sw + 50} y={V ? 0.45 : 0.5} count={40} seed="rev" spread={0.6} />
    </AbsoluteFill>
  );
};

/** Captions for the last line, changing with what the keys are doing. */
const LastCaption: React.FC = () => {
  const f = useCurrentFrame();
  const g = f + LAST_AT;
  const tabFrom = LS.out[LAST.findIndex((s) => s.k === 'Tab')];
  const tabTo = LS.out[LAST.length - 2];
  const until = REVEAL_AT - LAST_AT + 10;
  if (g < tabFrom - 10) return <Caption bnText="শেষ লাইন, প্রতিটি কী-চাপ" en="The last line, keystroke by keystroke" at={6} until={tabFrom - LAST_AT - 10} />;
  if (g < tabTo + 20) return <Sequence from={tabFrom - LAST_AT - 10} layout="none"><Caption bnText="Tab চেপে ঠিক শব্দটি বেছে নিন" en="Press Tab to pick the word you mean" at={0} until={tabTo - tabFrom + 30} /></Sequence>;
  return <Sequence from={tabTo - LAST_AT + 20} layout="none"><Caption bnText="মা, তোর বদনখানি মলিন হলে, ও মা, আমি নয়নজলে ভাসি॥" en="The tenth and final line" at={0} until={until - (tabTo - LAST_AT + 20)} /></Sequence>;
};

export const Anthem: React.FC = () => {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: C.night }}>
      <Fonts />
      <Backdrop sun={interpolate(f, [0, A.sw, REVEAL_AT, END_AT], [0.9, 0.45, 0.5, 1], clamp)} field={interpolate(f, [0, REVEAL_AT, ANTHEM_FRAMES], [0.6, 0.7, 1], clamp)} warmth={interpolate(f, [REVEAL_AT, ANTHEM_FRAMES], [0, 0.4], clamp)} />
      <Petals count={22} opacity={interpolate(f, [0, 60, A.sw, A.sw + 60, REVEAL_AT, REVEAL_AT + 60], [0, 0.8, 0.8, 0.25, 0.25, 0.8], clamp)} />
      <Sequence from={0} durationInFrames={A.sw}><Intro /></Sequence>
      <Burst at={8} y={0.45} count={36} seed="in" />
      <Sequence from={A.sw} durationInFrames={END_AT - A.sw}><Stage /></Sequence>
      <Sequence from={END_AT}><EndCard /></Sequence>
      <Audio src={staticFile('music/anthem.wav')} />
    </AbsoluteFill>
  );
};
