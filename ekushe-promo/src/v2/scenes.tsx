import React from 'react';
import { AbsoluteFill, Img, interpolate, random, spring, staticFile, useCurrentFrame, useVideoConfig } from 'remotion';
import trace from './demoTrace.json';
import { Backdrop, clamp, ease, Glass, Petals, RiseText, ShaheedMinar, window } from './parts';
import { bn, C, FONT_BN, FONT_DISPLAY, FONT_LATIN, useLayout } from './theme';

const ALPHABET = 'অ আ ই ঈ উ ঊ ঋ এ ঐ ও ঔ ক খ গ ঘ ঙ চ ছ জ ঝ ঞ ট ঠ ড ঢ ণ ত থ দ ধ ন প ফ ব ভ ম য র ল শ ষ স হ ড় ঢ় য় ৎ ং ঃ ঁ'.split(' ');

const Sub: React.FC<{ children: React.ReactNode; o?: number; size?: number; color?: string; style?: React.CSSProperties }> = ({ children, o = 1, size = 34, color = 'rgba(234,243,238,0.72)', style }) => {
  const { u } = useLayout();
  return <div style={{ fontFamily: FONT_LATIN, fontWeight: 500, fontSize: size * u, color, opacity: o, letterSpacing: '0.01em', textAlign: 'center', ...style }}>{children}</div>;
};

// ───────────────────────────── 1. Cold open: the alphabet gathers into "অ" ─────────────────────────────
export const SceneOpen: React.FC = () => {
  const f = useCurrentFrame();
  const { width, height, u, V } = useLayout();
  const gather = ease(f, 40, 190);
  const big = ease(f, 150, 230);
  const fadeAll = interpolate(f, [440, 480], [1, 0], clamp);
  return (
    <AbsoluteFill style={{ opacity: fadeAll }}>
      <Backdrop sun={interpolate(f, [0, 260], [0, 0.65], clamp)} field={interpolate(f, [0, 260], [0, 0.5], clamp)} />
      {ALPHABET.map((ch, i) => {
        const x0 = random(`ax${i}`) * width;
        const y0 = random(`ay${i}`) * height;
        const drift = Math.sin((f + i * 17) / 50) * 30 * u;
        const x = interpolate(gather, [0, 1], [x0 + drift, width / 2]);
        const y = interpolate(gather, [0, 1], [y0, height * 0.42]);
        const size = (40 + random(`as${i}`) * 70) * u;
        return (
          <div key={i} style={{ position: 'absolute', left: x, top: y, transform: 'translate(-50%,-50%)', fontFamily: FONT_DISPLAY, fontSize: size, color: C.mist, opacity: interpolate(f, [0, 40], [0, 0.55], clamp) * (1 - big), textShadow: `0 0 ${20 * u}px rgba(255,255,255,0.4)` }}>
            {ch}
          </div>
        );
      })}
      <div style={{ position: 'absolute', left: 0, right: 0, top: height * 0.42, transform: `translateY(-55%) scale(${0.6 + big * 0.4})`, textAlign: 'center', fontFamily: FONT_DISPLAY, fontSize: (V ? 520 : 560) * u, lineHeight: 1, color: C.white, opacity: big, textShadow: `0 0 ${80 * u}px rgba(224,48,63,0.7), 0 0 ${24 * u}px rgba(255,255,255,0.5)` }}>
        অ
      </div>
      <div style={{ position: 'absolute', left: 0, right: 0, top: height * (V ? 0.62 : 0.7) }}>
        <RiseText text="২১ ফেব্রুয়ারি, ১৯৫২" start={250} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 104 : 96) * u, color: C.white, textAlign: 'center' }} />
        <Sub o={ease(f, 300, 340)} style={{ marginTop: 18 * u }}>21 February 1952, Dhaka</Sub>
      </div>
    </AbsoluteFill>
  );
};

// ───────────────────────────── 2. The language martyrs ─────────────────────────────
const MARTYRS = ['সালাম', 'বরকত', 'রফিক', 'জব্বার', 'শফিউর'];

export const SceneMartyrs: React.FC = () => {
  const f = useCurrentFrame();
  const { height, u, V } = useLayout();
  const rise = ease(f, 0, 110);
  const fade = interpolate(f, [560, 600], [1, 0], clamp);
  const minar = (V ? 1050 : 800) * u;
  return (
    <AbsoluteFill style={{ opacity: Math.min(fade, interpolate(f, [0, 20], [0, 1], clamp)) }}>
      <Backdrop sun={0.55} field={0.7} />
      <Petals count={34} from={0} opacity={ease(f, 60, 160)} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: V ? 'column' : 'row', gap: (V ? 70 : 120) * u }}>
        <div style={{ filter: `drop-shadow(0 ${30 * u}px ${60 * u}px rgba(0,0,0,0.6))` }}>
          <ShaheedMinar size={minar} rise={rise} />
        </div>
        <div style={{ width: (V ? 1200 : 1100) * u, display: 'flex', flexDirection: 'column', alignItems: V ? 'center' : 'flex-start' }}>
          <RiseText text="ভাষার জন্য যাঁরা প্রাণ দিলেন" start={90} style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 72 : 66) * u, color: C.mist, textAlign: V ? 'center' : 'left' }} />
          <Sub o={ease(f, 120, 150)} style={{ textAlign: V ? 'center' : 'left', marginTop: 8 * u }}>They gave their lives for our language</Sub>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: `${18 * u}px ${34 * u}px`, marginTop: 44 * u, justifyContent: V ? 'center' : 'flex-start' }}>
            {MARTYRS.map((n, i) => {
              const t = ease(f, 170 + i * 42, 210 + i * 42);
              return (
                <div key={n} style={{ fontFamily: FONT_DISPLAY, fontSize: (V ? 130 : 112) * u, lineHeight: 1.15, color: C.white, opacity: t, transform: `translateY(${(1 - t) * 30 * u}px)`, textShadow: `0 0 ${36 * u}px rgba(224,48,63,${0.6 * t})` }}>
                  {n}
                </div>
              );
            })}
          </div>
          <div style={{ fontFamily: FONT_BN, fontSize: 44 * u, color: 'rgba(234,243,238,0.75)', marginTop: 18 * u, opacity: ease(f, 390, 420) }}>এবং আরও অনেকে · and others</div>
          <div style={{ marginTop: 46 * u, opacity: ease(f, 450, 490), transform: `translateY(${(1 - ease(f, 450, 490)) * 20 * u}px)` }}>
            <Glass style={{ padding: `${24 * u}px ${34 * u}px` }}>
              <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 46 : 44) * u, color: C.mist, textAlign: V ? 'center' : 'left' }}>১৯৯৯ সালে ইউনেস্কো ২১ ফেব্রুয়ারিকে ঘোষণা করে আন্তর্জাতিক মাতৃভাষা দিবস</div>
              <Sub size={28} style={{ textAlign: V ? 'center' : 'left', marginTop: 6 * u }}>In 1999 UNESCO declared 21 February International Mother Language Day</Sub>
            </Glass>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// ───────────────────────────── 3. Pride: 28 crore speakers ─────────────────────────────
export const ScenePride: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const n = Math.round(interpolate(ease(f, 20, 150), [0, 1], [0, 28]));
  const o = Math.min(interpolate(f, [0, 20], [0, 1], clamp), interpolate(f, [270, 300], [1, 0], clamp));
  return (
    <AbsoluteFill style={{ opacity: o }}>
      <Backdrop sun={0.8} field={0.6} warmth={0.6} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 330 : 300) * u, lineHeight: 1, color: C.white, textShadow: `0 0 ${60 * u}px rgba(224,48,63,0.55)` }}>{bn(n)} কোটি</div>
        <RiseText text="মানুষের মুখের ভাষা বাংলা" start={110} style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 76 * u, color: C.mist, marginTop: 20 * u }} />
        <RiseText text="বিশ্বের ষষ্ঠ সর্বাধিক প্রচলিত মাতৃভাষা" start={150} style={{ fontFamily: FONT_BN, fontWeight: 400, fontSize: 52 * u, color: C.mustard, marginTop: 18 * u, maxWidth: (V ? 1250 : 2200) * u }} />
        <Sub o={ease(f, 190, 220)} style={{ marginTop: 22 * u }}>About 284 million speakers. The world's sixth most spoken native language.</Sub>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// ───────────────────────────── 4. The problem ─────────────────────────────
const PAINS = [
  { big: 'Avwg evsjvq Mvb MvB', latin: true, bn: 'বিজয়ে লেখা ফাইল অন্য কম্পিউটারে ভেঙে যায়', en: 'Old Bijoy text turns to gibberish on other computers' },
  { big: 'k + Sh = ? kkh? kSh?', latin: true, bn: 'যুক্তবর্ণের কঠিন শর্টকাট মুখস্থ', en: 'Memorise tricky shortcuts for every conjunct' },
  { big: 'বাংলাদেস', latin: false, wrong: true, bn: 'ভুল বানান, কেউ ধরিয়ে দেয় না', en: 'Spelling mistakes nobody points out' },
  { big: 'Shift Shift Shift', latin: true, bn: 'বড় হাত-ছোট হাতের ঝামেলা', en: 'Capital letters change the meaning' },
];

export const SceneProblem: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width } = useLayout();
  const o = Math.min(interpolate(f, [0, 20], [0, 1], clamp), interpolate(f, [500, 540], [1, 0], clamp));
  const shatter = ease(f, 470, 530);
  return (
    <AbsoluteFill style={{ opacity: o }}>
      <Backdrop sun={0.25} field={0.35} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', padding: 80 * u }}>
        <RiseText text="কিন্তু বাংলা লেখা আজও কঠিন" start={10} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 96 : 88) * u, color: C.white }} />
        <Sub o={ease(f, 40, 70)} style={{ marginTop: 6 * u }}>But typing Bangla is still hard</Sub>
        <div style={{ display: 'grid', gridTemplateColumns: V ? '1fr' : '1fr 1fr', gap: 34 * u, marginTop: 60 * u, width: V ? 1200 * u : Math.min(width - 200 * u, 2100 * u) }}>
          {PAINS.map((p, i) => {
            const start = 80 + i * 75;
            const t = spring({ frame: f - start, fps: 60, config: { damping: 14, stiffness: 120 } });
            const jitter = f > start && f < start + 30 ? (random(`j${i}${f}`) - 0.5) * 14 * u : 0;
            const dir = i % 2 ? 1 : -1;
            return (
              <div key={i} style={{ opacity: t * (1 - shatter), transform: `translate(${jitter + dir * shatter * 260 * u}px, ${(1 - t) * 60 * u + shatter * (i - 1.5) * 120 * u}px) rotate(${shatter * dir * 8}deg)` }}>
                <Glass style={{ padding: `${30 * u}px ${36 * u}px`, borderColor: 'rgba(224,48,63,0.45)' }}>
                  <div style={{ fontFamily: p.latin ? FONT_LATIN : FONT_BN, fontWeight: p.latin ? 600 : 700, fontSize: 58 * u, color: p.latin ? '#FFB4BA' : C.white, textDecoration: p.wrong ? 'underline wavy #FF4D5B' : 'none', textUnderlineOffset: 14 * u, letterSpacing: p.latin ? '0.02em' : 0 }}>{p.big}</div>
                  <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 40 * u, color: C.mist, marginTop: 14 * u }}>{p.bn}</div>
                  <Sub size={26} style={{ textAlign: 'left', marginTop: 4 * u }}>{p.en}</Sub>
                </Glass>
              </div>
            );
          })}
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// ───────────────────────────── 5. Reveal ─────────────────────────────
export const SceneReveal: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const flash = interpolate(f, [0, 10, 50], [0, 1, 0], clamp);
  const s = spring({ frame: f - 6, fps: 60, config: { damping: 12, stiffness: 90 } });
  const o = interpolate(f, [160, 180], [1, 0], clamp);
  return (
    <AbsoluteFill style={{ opacity: o }}>
      <Backdrop sun={1} field={1} warmth={0.4} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column' }}>
        <Img src={staticFile('assets/logo_512.png')} style={{ width: (V ? 360 : 300) * u, height: (V ? 360 : 300) * u, borderRadius: 70 * u, transform: `scale(${0.4 + s * 0.6})`, opacity: s, boxShadow: `0 ${40 * u}px ${120 * u}px rgba(0,0,0,0.6), 0 0 ${120 * u}px rgba(224,48,63,0.4)` }} />
        <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 170 : 160) * u, color: C.white, marginTop: 40 * u, opacity: ease(f, 30, 70), transform: `translateY(${(1 - ease(f, 30, 70)) * 30 * u}px)`, lineHeight: 1.1 }}>একুশে নেক্সট</div>
        <div style={{ fontFamily: FONT_LATIN, fontWeight: 300, fontSize: 64 * u, color: C.mist, letterSpacing: '0.32em', opacity: ease(f, 50, 90), marginTop: 6 * u }}>EKUSHE NEXT</div>
        <RiseText text="বাংলা লেখার নতুন মান" start={80} style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 58 * u, color: C.mustard, marginTop: 34 * u }} />
      </AbsoluteFill>
      <AbsoluteFill style={{ background: 'white', opacity: flash }} />
    </AbsoluteFill>
  );
};

// ───────────────────────────── 6. Live demo (real engine output) ─────────────────────────────
type Step = { key: string; roman?: string; preedit?: string; candidates?: string[]; committed?: string | null };
const STEPS = trace as Step[];
const KEY_EVERY = 13; // frames per keystroke
export const DEMO_KEYS_START = 50;

const KEY_ROWS = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm'];

export const SceneDemo: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width } = useLayout();
  const k = Math.floor((f - DEMO_KEYS_START) / KEY_EVERY);
  const idx = Math.min(STEPS.length - 1, k);
  let text = '';
  let state: Step = { key: '' };
  if (k >= 0) {
    for (let i = 0; i <= idx; i++) {
      const s = STEPS[i];
      if (s.committed) text += s.committed;
      state = s;
    }
  }
  const composing = k >= 0 && !state.committed && state.preedit;
  const pressed = k >= 0 && k < STEPS.length ? STEPS[k].key : '';
  const pressT = k >= 0 ? (f - DEMO_KEYS_START - k * KEY_EVERY) / KEY_EVERY : 1;
  const o = Math.min(interpolate(f, [0, 20], [0, 1], clamp), interpolate(f, [395, 420], [1, 0], clamp));
  const panelW = V ? 1340 * u : Math.min(width - 300 * u, 2000 * u);
  const keyW = (V ? 128 : 112) * u;
  return (
    <AbsoluteFill style={{ opacity: o }}>
      <Backdrop sun={0.7} field={0.9} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: (V ? 80 : 46) * u }}>
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 84 : 70) * u, color: C.white }}>যেভাবে বলেন, সেভাবে লিখুন</div>
          <Sub>Type it the way you say it</Sub>
        </div>
        <Glass strong style={{ width: panelW, padding: `${40 * u}px ${50 * u}px`, minHeight: (V ? 560 : 300) * u }}>
          <div style={{ display: 'flex', gap: 12 * u, marginBottom: 26 * u }}>
            {[C.red, C.mustard, C.greenBright].map((c) => <div key={c} style={{ width: 20 * u, height: 20 * u, borderRadius: '50%', background: c, opacity: 0.8 }} />)}
          </div>
          <div style={{ fontFamily: FONT_DISPLAY, fontSize: (V ? 128 : 112) * u, color: C.white, lineHeight: 1.35, minHeight: (V ? 330 : 150) * u }}>
            {text}
            {composing ? <span style={{ textDecoration: 'underline', textDecorationColor: C.greenBright, textUnderlineOffset: 16 * u, textDecorationThickness: 5 * u }}>{state.preedit}</span> : null}
            <span style={{ display: 'inline-block', width: 6 * u, height: 100 * u, background: C.greenBright, marginLeft: 6 * u, verticalAlign: 'middle', opacity: Math.floor(f / 30) % 2 ? 1 : 0.2 }} />
          </div>
          <div style={{ height: 120 * u, marginTop: 20 * u }}>
            {composing && state.candidates ? (
              <div style={{ display: 'inline-flex', flexDirection: 'column', gap: 12 * u, padding: `${16 * u}px ${18 * u}px`, borderRadius: 22 * u, background: 'rgba(255,255,255,0.12)', border: '1px solid rgba(255,255,255,0.25)' }}>
                <div style={{ fontFamily: FONT_LATIN, fontSize: 26 * u, color: 'rgba(234,243,238,0.7)', fontWeight: 600 }}>{state.roman}</div>
                <div style={{ display: 'flex', gap: 12 * u }}>
                  {state.candidates.slice(0, V ? 4 : 5).map((c, i) => (
                    <div key={i} style={{ fontFamily: i === 0 || /[a-z]/.test(c) ? (/[a-z]/.test(c) ? FONT_LATIN : FONT_DISPLAY) : FONT_DISPLAY, fontSize: (V ? 54 : 46) * u, padding: `${6 * u}px ${20 * u}px`, borderRadius: 14 * u, background: i === 0 ? C.green : 'rgba(255,255,255,0.1)', color: C.white, display: 'flex', alignItems: 'baseline', gap: 10 * u }}>
                      <span style={{ fontFamily: FONT_LATIN, fontSize: 22 * u, opacity: 0.7 }}>{i + 1}</span>
                      {c}
                    </div>
                  ))}
                </div>
              </div>
            ) : null}
          </div>
        </Glass>
        {/* Keyboard */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: (V ? 16 : 12) * u, alignItems: 'center', transform: `scale(${V ? 1 : 0.9})` }}>
          {KEY_ROWS.map((row) => (
            <div key={row} style={{ display: 'flex', gap: 12 * u }}>
              {row.split('').map((ch) => {
                const on = pressed === ch && pressT < 0.7;
                return (
                  <div key={ch} style={{ width: keyW, height: keyW, borderRadius: 20 * u, display: 'grid', placeItems: 'center', fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 42 * u, color: on ? C.ink : 'rgba(234,243,238,0.85)', background: on ? C.greenBright : 'rgba(255,255,255,0.07)', border: '1px solid rgba(255,255,255,0.16)', transform: `translateY(${on ? 4 * u : 0}px)`, boxShadow: on ? `0 0 ${40 * u}px ${C.greenBright}` : 'none' }}>
                    {ch}
                  </div>
                );
              })}
            </div>
          ))}
          <div style={{ width: keyW * 6, height: keyW * 0.8, borderRadius: 20 * u, background: pressed === ' ' && pressT < 0.7 ? C.greenBright : 'rgba(255,255,255,0.07)', border: '1px solid rgba(255,255,255,0.16)', display: 'grid', placeItems: 'center', fontFamily: FONT_LATIN, fontSize: 28 * u, color: pressed === ' ' && pressT < 0.7 ? C.ink : 'rgba(234,243,238,0.6)' }}>space</div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

/** Frames (within the demo scene) at which a key is pressed — the music adds a soft click there. */
export const demoKeyFrames = () => STEPS.map((_, i) => DEMO_KEYS_START + i * KEY_EVERY);

// ───────────────────────────── 7. Features ─────────────────────────────
const FEATURES = [
  { title: 'নিজস্ব অভিধান', big: '৪,১৮,৭৪৩ শব্দ', bn: '২৭৪ কোটি শব্দের লেখা থেকে তৈরি; বাংলা একাডেমির প্রমিত বানান আগে', en: "Ekushe's own dictionary, built from 2.7 billion words of text. Standard spelling first." },
  { title: 'ছোট হাতেই সব', big: 'kkhoma → ক্ষমা', bn: 'Shift বা যুক্তবর্ণের শর্টকাট মুখস্থ লাগে না', en: 'No Shift, no conjunct shortcuts to memorise' },
  { title: 'বিজয় ⇄ ইউনিকোড', big: 'Avwg → আমি', bn: 'পুরনো বিজয় লেখা এক ক্লিকে ইউনিকোডে', en: 'Old Bijoy text to Unicode in one click' },
  { title: 'ভুল হলে সহজে ঠিক', big: 'বাংলাদেস → বাংলাদেশ', bn: 'বানান পরীক্ষা, আর Backspace চাপলেই আগের লেখা', en: 'Spell check, and Backspace brings back what you typed' },
];

export const SceneFeatures: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V, width } = useLayout();
  const o = Math.min(interpolate(f, [0, 20], [0, 1], clamp), interpolate(f, [395, 420], [1, 0], clamp));
  return (
    <AbsoluteFill style={{ opacity: o }}>
      <Backdrop sun={0.6} field={1} warmth={0.5} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column', padding: 70 * u }}>
        <RiseText text="যা আলাদা" start={0} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 84 * u, color: C.white }} />
        <Sub o={ease(f, 20, 45)}>What is different</Sub>
        <div style={{ display: 'grid', gridTemplateColumns: V ? '1fr' : '1fr 1fr', gap: (V ? 40 : 30) * u, marginTop: 50 * u, width: V ? 1300 * u : Math.min(width - 200 * u, 2150 * u) }}>
          {FEATURES.map((ft, i) => {
            const t = spring({ frame: f - (40 + i * 60), fps: 60, config: { damping: 16, stiffness: 110 } });
            return (
              <div key={i} style={{ opacity: t, transform: `translateY(${(1 - t) * 70 * u}px) scale(${0.94 + t * 0.06})` }}>
                <Glass strong style={{ padding: `${30 * u}px ${38 * u}px`, height: '100%' }}>
                  <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: 36 * u, color: C.greenBright }}>{ft.title}</div>
                  <div style={{ fontFamily: /[A-Za-z]/.test(ft.big) ? `${FONT_LATIN}` : FONT_BN, fontWeight: 700, fontSize: (V ? 76 : 70) * u, color: C.white, marginTop: 6 * u, lineHeight: 1.25 }}>
                    {ft.big}
                  </div>
                  <div style={{ fontFamily: FONT_BN, fontSize: 36 * u, color: C.mist, marginTop: 10 * u, lineHeight: 1.45 }}>{ft.bn}</div>
                  <Sub size={25} style={{ textAlign: 'left', marginTop: 6 * u }}>{ft.en}</Sub>
                </Glass>
              </div>
            );
          })}
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// ───────────────────────────── 8. Speed ─────────────────────────────
export const SceneSpeed: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const o = Math.min(interpolate(f, [0, 20], [0, 1], clamp), interpolate(f, [275, 300], [1, 0], clamp));
  const ms = interpolate(ease(f, 20, 110), [0, 1], [100, 0.98]);
  const barMax = (V ? 900 : 1500) * u;
  const grow = ease(f, 130, 200);
  return (
    <AbsoluteFill style={{ opacity: o }}>
      <Backdrop sun={0.5} field={0.8} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column' }}>
        <div style={{ fontFamily: FONT_BN, fontWeight: 600, fontSize: (V ? 72 : 60) * u, color: C.mist }}>প্রতিটি কী-চাপে সময় লাগে</div>
        <div style={{ fontFamily: FONT_LATIN, fontWeight: 800, fontSize: (V ? 300 : 280) * u, lineHeight: 1.05, color: C.white, letterSpacing: '-0.03em', textShadow: `0 0 ${70 * u}px rgba(47,209,142,0.5)` }}>
          {ms.toFixed(2)}<span style={{ fontSize: 110 * u, fontWeight: 600, marginLeft: 16 * u, letterSpacing: 0 }}>ms</span>
        </div>
        <Sub>average time per keystroke, measured over 1,414 keystrokes</Sub>
        <div style={{ marginTop: 60 * u, display: 'flex', flexDirection: 'column', gap: 26 * u, width: barMax + 380 * u }}>
          {[
            { label: 'একুশে নেক্সট', sub: '~1 ms', w: 0.01, c: C.greenBright },
            { label: 'চোখের পলক', sub: '~100 ms (a blink)', w: 1, c: 'rgba(255,255,255,0.35)' },
          ].map((b) => (
            <div key={b.label} style={{ display: 'flex', alignItems: 'center', gap: 26 * u }}>
              <div style={{ width: 340 * u, fontFamily: FONT_BN, fontWeight: 600, fontSize: 40 * u, color: C.white, textAlign: 'right' }}>{b.label}</div>
              <div style={{ height: 40 * u, width: Math.max(8 * u, barMax * b.w * grow), borderRadius: 20 * u, background: b.c, boxShadow: b.c === C.greenBright ? `0 0 ${30 * u}px ${C.greenBright}` : 'none' }} />
              <div style={{ fontFamily: FONT_LATIN, fontSize: 28 * u, color: 'rgba(234,243,238,0.7)', opacity: grow }}>{b.sub}</div>
            </div>
          ))}
        </div>
        <RiseText text="চোখের পলকের চেয়ে প্রায় ১০০ গুণ দ্রুত" start={210} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: (V ? 66 : 62) * u, color: C.mustard, marginTop: 56 * u }} />
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// ───────────────────────────── 9. Platforms and call to action ─────────────────────────────
const Monitor: React.FC<{ s: number; kind: 'win' | 'mac' | 'linux' }> = ({ s, kind }) => (
  <svg width={s} height={s} viewBox="0 0 100 100" fill="none" stroke={C.mist} strokeWidth="4" strokeLinecap="round" strokeLinejoin="round">
    {kind === 'linux' ? (
      <>
        <rect x="10" y="18" width="80" height="56" rx="8" />
        <path d="M24 38 L34 46 L24 54 M42 56 H58" />
      </>
    ) : kind === 'mac' ? (
      <>
        <rect x="18" y="16" width="64" height="46" rx="6" />
        <path d="M8 76 H92 L86 84 H14 Z" />
      </>
    ) : (
      <>
        <rect x="10" y="14" width="80" height="54" rx="6" />
        <path d="M40 68 L36 84 H64 L60 68" />
      </>
    )}
  </svg>
);

export const SceneOutro: React.FC = () => {
  const f = useCurrentFrame();
  const { u, V } = useLayout();
  const { durationInFrames } = useVideoConfig();
  void durationInFrames;
  const end = interpolate(f, [330, 360], [1, 0], clamp);
  return (
    <AbsoluteFill style={{ opacity: Math.min(interpolate(f, [0, 20], [0, 1], clamp), end) }}>
      <Backdrop sun={1} field={1} warmth={0.3} />
      <Petals count={18} opacity={0.6} />
      <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center', flexDirection: 'column' }}>
        <div style={{ display: 'flex', gap: 34 * u, flexDirection: 'row' }}>
          {([['Windows', 'win'], ['macOS', 'mac'], ['Linux', 'linux']] as const).map(([name, kind], i) => {
            const t = spring({ frame: f - 10 - i * 12, fps: 60, config: { damping: 14 } });
            return (
              <div key={name} style={{ opacity: t, transform: `translateY(${(1 - t) * 50 * u}px)` }}>
                <Glass strong style={{ width: (V ? 380 : 360) * u, padding: `${30 * u}px 0`, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12 * u }}>
                  <Monitor s={110 * u} kind={kind} />
                  <div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: 42 * u, color: C.white }}>{name}</div>
                </Glass>
              </div>
            );
          })}
        </div>
        <RiseText text="সম্পূর্ণ ফ্রি, ওপেন সোর্স" start={60} style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 70 * u, color: C.white, marginTop: 56 * u }} />
        <Sub o={ease(f, 80, 110)}>Free and open source. Windows, macOS and Linux.</Sub>
        <div style={{ marginTop: 50 * u, opacity: ease(f, 130, 160) }}>
          <Glass style={{ padding: `${18 * u}px ${40 * u}px`, borderRadius: 999 }}>
            <div style={{ fontFamily: FONT_LATIN, fontWeight: 600, fontSize: (V ? 46 : 44) * u, color: C.greenBright }}>github.com/mahbub-khan25/ekushenext</div>
          </Glass>
        </div>
        <div style={{ marginTop: 70 * u, display: 'flex', alignItems: 'center', gap: 28 * u, opacity: ease(f, 190, 230), flexDirection: V ? 'column' : 'row' }}>
          <Img src={staticFile('assets/logo_256.png')} style={{ width: 120 * u, height: 120 * u, borderRadius: 28 * u }} />
          <div style={{ textAlign: V ? 'center' : 'left' }}>
            <div style={{ fontFamily: FONT_BN, fontWeight: 700, fontSize: 76 * u, color: C.white, lineHeight: 1.15 }}>ভাষার মর্যাদা, আঙুলের ডগায়</div>
            <Sub style={{ textAlign: V ? 'center' : 'left' }}>Ekushe Next. The pride of our language, at your fingertips.</Sub>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
