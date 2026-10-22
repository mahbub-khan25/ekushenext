import React from 'react';
import { AbsoluteFill, Audio, Img, interpolate, Sequence, spring, staticFile, useCurrentFrame } from 'remotion';
import { Fonts } from '../v2/Fonts';
import { Backdrop, clamp, ease, Petals, RiseText, ShaheedMinar } from '../v2/parts';
import { C, FONT_BN, FONT_DISPLAY, FONT_LATIN, useLayout } from '../v2/theme';

/** Teaser 1 of 5: "The language they gave their lives for". 15 s at 60 fps. */
export const TEASER1_FRAMES = 900;
const NAMES = ['সালাম', 'বরকত', 'রফিক', 'জব্বার', 'শফিউর'];
export const NAME_AT = (i: number) => 380 + i * 40; // the score places a soft bell on each

const Sub: React.FC<{ children: React.ReactNode; o?: number; size?: number; style?: React.CSSProperties }> = ({ children, o = 1, size = 34, style }) => {
  const { u } = useLayout();
  return <div style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: size * u, color: 'rgba(234,243,238,0.72)', opacity: o, letterSpacing: '0.01em', textAlign: 'center', ...style }}>{children}</div>;
};
const fadeInOut = (f: number, a: number, b: number, d = 24) =>
  Math.min(interpolate(f, [a, a + d], [0, 1], clamp), interpolate(f, [b - d, b], [1, 0], clamp));

/** 0–2.8 s: the date, alone in the dark. */
const Date1952: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: fadeInOut(f, 0, 170, 30) }}>
      <RiseText text="২১ ফেব্রুয়ারি, ১৯৫২" start={20} stagger={10} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 120 : 110) * u, color: C.white, textAlign: 'center', textShadow: `0 0 ${50 * u}px rgba(224,48,63,0.55)` }} />
      <Sub o={ease(f, 60, 100)} style={{ marginTop: 18 * u }}>21 February 1952, Dhaka</Sub>
    </AbsoluteFill>
  );
};

/** 2.5–11.7 s: the Shaheed Minar rises with the sun; the names, one by one. */
const Martyrs: React.FC = () => {
  const f = useCurrentFrame(); // starts at frame 150
  const { u, V } = useLayout();
  const rise = ease(f, 0, 140);
  const minar = (V ? 1000 : 760) * u;
  return (
    <AbsoluteFill style={{ opacity: fadeInOut(f, 0, 550, 30), alignItems: 'center', justifyContent: 'center', flexDirection: V ? 'column' : 'row', gap: (V ? 80 : 130) * u }}>
      <div style={{ filter: `drop-shadow(0 ${30 * u}px ${60 * u}px rgba(0,0,0,0.6))`, transform: `scale(${1 + f * 0.00012})` }}>
        <ShaheedMinar size={minar} rise={rise} glow={ease(f, 20, 200)} />
      </div>
      <div style={{ width: (V ? 1240 : 1100) * u, display: 'flex', flexDirection: 'column', alignItems: V ? 'center' : 'flex-start' }}>
        <RiseText text="ভাষার জন্য যাঁরা প্রাণ দিলেন" start={60} style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 80 : 70) * u, color: C.mist, textAlign: V ? 'center' : 'left' }} />
        <Sub o={ease(f, 100, 140)} style={{ textAlign: V ? 'center' : 'left', marginTop: 10 * u }}>For those who gave their lives for our language</Sub>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: `${14 * u}px ${36 * u}px`, marginTop: 50 * u, justifyContent: V ? 'center' : 'flex-start' }}>
          {NAMES.map((n, i) => {
            const t = ease(f, NAME_AT(i) - 150, NAME_AT(i) - 150 + 36);
            return (
              <div key={n} style={{ fontFamily: FONT_DISPLAY, fontSize: (V ? 136 : 116) * u, lineHeight: 1.15, color: C.white, opacity: t, transform: `translateY(${(1 - t) * 30 * u}px)`, filter: `blur(${(1 - t) * 6 * u}px)`, textShadow: `0 0 ${40 * u}px rgba(224,48,63,${0.65 * t})` }}>
                {n}
              </div>
            );
          })}
        </div>
        <div style={{ fontFamily: FONT_BN, fontSize: 44 * u, color: 'rgba(234,243,238,0.7)', marginTop: 20 * u, opacity: ease(f, 420, 450) }}>এবং আরও অনেকে</div>
      </div>
    </AbsoluteFill>
  );
};

/** 11.3–13.5 s: the promise. */
const Vow: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', padding: `0 ${80 * u}px`, opacity: fadeInOut(f, 0, 140, 22) }}>
      <RiseText text="তাঁদের ভাষা" start={6} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 150 : 130) * u, color: C.white, textAlign: 'center', lineHeight: 1.2 }} />
      <RiseText text="লেখা হবে আরও সহজ" start={22} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 110 : 100) * u, color: C.greenBright, textAlign: 'center', lineHeight: 1.25, textShadow: `0 0 ${40 * u}px rgba(47,209,142,0.35)` }} />
      <Sub o={ease(f, 40, 70)} style={{ marginTop: 26 * u }}>Their language, made easier to write</Sub>
    </AbsoluteFill>
  );
};

/** 13.2–15 s: the series ending (same in every teaser). */
export const ComingSoon: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const pop = spring({ frame: f - 4, fps: 60, config: { damping: 13, mass: 0.8 } });
  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', opacity: interpolate(f, [0, 16], [0, 1], clamp) }}>
      <Img src={staticFile('assets/logo_512.png')} style={{ width: (V ? 300 : 260) * u, height: (V ? 300 : 260) * u, borderRadius: 64 * u, transform: `scale(${0.7 + pop * 0.3})`, opacity: pop, boxShadow: `0 ${30 * u}px ${80 * u}px rgba(0,0,0,0.55), 0 0 ${120 * u}px rgba(224,48,63,0.45)` }} />
      <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 150 : 130) * u, color: C.white, marginTop: 34 * u, lineHeight: 1.1, opacity: ease(f, 10, 34), transform: `translateY(${(1 - ease(f, 10, 34)) * 24 * u}px)` }}>একুশে নেক্সট</div>
      <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 64 : 58) * u, color: C.mustard, marginTop: 18 * u, opacity: ease(f, 26, 50) }}>আসছে শীঘ্রই</div>
      <Sub o={ease(f, 34, 58)} size={36} style={{ marginTop: 6 * u }}>Coming soon</Sub>
      <div style={{ marginTop: 44 * u, padding: `${16 * u}px ${38 * u}px`, borderRadius: 999, background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.22)', fontFamily: FONT_LATIN, fontWeight: 600, fontSize: (V ? 48 : 44) * u, color: C.greenBright, opacity: ease(f, 44, 70) }}>ekushenext.com</div>
    </AbsoluteFill>
  );
};

export const Teaser1: React.FC = () => {
  const f = useCurrentFrame();
  const sun = interpolate(f, [0, 150, 300, 690, 800, 900], [0, 0.15, 0.7, 0.75, 1, 1], clamp);
  const field = interpolate(f, [0, 200, 700, 900], [0.05, 0.6, 0.75, 1], clamp);
  const end = interpolate(f, [870, 900], [1, 0.9], clamp);
  return (
    <AbsoluteFill style={{ background: C.night, opacity: end }}>
      <Fonts />
      <Backdrop sun={sun} field={field} warmth={interpolate(f, [690, 900], [0, 0.4], clamp)} />
      <Petals count={30} opacity={interpolate(f, [0, 120, 860, 900], [0.25, 0.8, 0.8, 0.6], clamp)} />
      <Sequence from={0} durationInFrames={170}><Date1952 /></Sequence>
      <Sequence from={150} durationInFrames={550}><Martyrs /></Sequence>
      <Sequence from={680} durationInFrames={140}><Vow /></Sequence>
      <Sequence from={795}><ComingSoon /></Sequence>
      <Audio src={staticFile('music/teaser1.wav')} />
    </AbsoluteFill>
  );
};
