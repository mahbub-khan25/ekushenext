import React from 'react';
import {
  AbsoluteFill,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
  Easing,
} from 'remotion';
import { BackgroundParticles } from '../components/BackgroundParticles';
import { ShimmerSweep } from '../components/ShimmerSweep';

const WORDS_DATA = [
  { bn: 'সৃষ্টি', en: 'Creation', root: 'স + ৃ + ষ্ + ট + ি', color: '#38bdf8' },
  { bn: 'স্মৃতি', en: 'Memory', root: 'স + ম + ৃ + ত + ি', color: '#a78bfa' },
  { bn: 'স্বাধীনতা', en: 'Freedom', root: 'স + ব + া + ধ + ী + ন + ত + া', color: '#34d399' },
  { bn: 'বৃষ্টি', en: 'Rain', root: 'ব + ৃ + ষ্ + ট + ি', color: '#38bdf8' },
  { bn: 'বিজ্ঞান', en: 'Science', root: 'ব + ি + জ + ঞ + া + ন', color: '#fbbf24' },
];

// Continuous mathematical focus level in [0, 1] for each card (silky smooth transitions)
const getCardFocus = (idx: number, currentF: number): number => {
  const windows = [
    { inStart: 46, inEnd: 60, outStart: 84, outEnd: 98 },
    { inStart: 84, inEnd: 98, outStart: 118, outEnd: 132 },
    { inStart: 118, inEnd: 132, outStart: 152, outEnd: 166 },
    { inStart: 152, inEnd: 166, outStart: 186, outEnd: 200 },
    { inStart: 186, inEnd: 200, outStart: 235, outEnd: 255 },
  ];

  const w = windows[idx];
  if (!w) return 0;

  const inVal = interpolate(currentF, [w.inStart, w.inEnd], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.inOut(Easing.quad),
  });

  const outVal = interpolate(currentF, [w.outStart, w.outEnd], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.inOut(Easing.quad),
  });

  return Math.min(inVal, outVal);
};

export const Scene2WordsAndPain: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);

  const cameraZoom = interpolate(f, [0, 268], [1, 1.04], {
    extrapolateRight: 'clamp',
  });

  const titleIn = spring({
    frame: f - 10,
    fps: 30,
    config: { damping: 13, stiffness: 80 },
  });

  const painIn = spring({
    frame: f - 80,
    fps: 30,
    config: { damping: 14, stiffness: 75 },
  });

  const fadeOut = interpolate(f, [240, 268], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  // Gliding spotlight position across the cards (-440 to +440px)
  const spotlightX = interpolate(
    f,
    [53, 91, 125, 159, 193],
    [-440, -220, 0, 220, 440],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
      easing: Easing.inOut(Easing.quad),
    }
  );

  const spotlightAuraOpacity = interpolate(
    f,
    [44, 58, 235, 255],
    [0, 0.7, 0.7, 0],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );

  // Warning pulse
  const warningGlow = 15 + Math.sin(f * 0.15) * 10;
  const warningWobble = Math.sin(f * 0.25) * 5;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: '#030712',
        backgroundImage:
          'radial-gradient(circle at 50% 50%, rgba(99, 102, 241, 0.15) 0%, rgba(15, 23, 42, 0.8) 50%, #020617 100%)',
        color: '#ffffff',
        fontFamily: "'Kalpurush', 'HindSiliguri', sans-serif",
        opacity: fadeOut,
        overflow: 'hidden',
      }}
    >
      <BackgroundParticles theme="mixed" />

      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '0 80px',
          zIndex: 10,
          transform: `scale(${cameraZoom})`,
        }}
      >
        {/* Section Header with Shimmer */}
        <div
          style={{
            transform: `scale(${titleIn}) translateY(${(1 - titleIn) * 20}px)`,
            opacity: titleIn,
            textAlign: 'center',
            marginBottom: 20,
          }}
        >
          <div
            style={{
              position: 'relative',
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              padding: '6px 20px',
              borderRadius: 999,
              background: 'rgba(56, 189, 248, 0.12)',
              border: '1px solid rgba(56, 189, 248, 0.35)',
              color: '#7dd3fc',
              fontSize: 18,
              fontWeight: 700,
              marginBottom: 12,
              overflow: 'hidden',
            }}
          >
            <ShimmerSweep interval={70} duration={22} color="rgba(125, 211, 252, 0.4)" />
            <span>✨</span>
            <span>ভাষার নান্দনিকতা ও যুক্তাক্ষরের চ্যালেঞ্জ</span>
            <span>✨</span>
          </div>

          <h2
            style={{
              fontSize: 48,
              fontWeight: 900,
              margin: 0,
              lineHeight: 1.2,
              background:
                'linear-gradient(135deg, #ffffff 40%, #bae6fd 80%, #7dd3fc 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              letterSpacing: 0.5,
            }}
          >
            বাংলা শব্দের সৌন্দর্য অনন্য, কিন্তু কম্পিউটারে টাইপিং?
          </h2>
        </div>

        {/* 5 Beautiful Word Cards in a Row with Cascading Pop-In & Smooth Gliding Focus */}
        <div
          style={{
            position: 'relative',
            display: 'flex',
            gap: 16,
            marginBottom: 22,
            width: '100%',
            maxWidth: 1120,
            justifyContent: 'center',
          }}
        >
          {/* Gliding Ambient Spotlight Aura behind the cards */}
          <div
            style={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              width: 280,
              height: 280,
              borderRadius: '50%',
              background:
                'radial-gradient(circle, rgba(168, 85, 247, 0.35) 0%, rgba(56, 189, 248, 0.22) 40%, transparent 70%)',
              transform: `translate(calc(-50% + ${spotlightX}px), -50%)`,
              filter: 'blur(36px)',
              opacity: spotlightAuraOpacity,
              pointerEvents: 'none',
              zIndex: 0,
            }}
          />

          {WORDS_DATA.map((w, idx) => {
            const cardSpring = spring({
              frame: f - (25 + idx * 7),
              fps: 30,
              config: { damping: 13, stiffness: 85 },
            });

            // Smooth continuous focus weight in [0, 1]
            const focusLevel = getCardFocus(idx, f);

            // Continuous breathing scale and lift
            const breathe = Math.sin(f * 0.12) * 0.012;
            const focusScale = 1 + focusLevel * (0.05 + breathe);
            const focusLift = -focusLevel * 8;

            return (
              <div
                key={w.bn}
                style={{
                  flex: 1,
                  maxWidth: 210,
                  transform: `scale(${cardSpring * focusScale}) translateY(${(1 - cardSpring) * 30 + focusLift}px)`,
                  opacity: cardSpring,
                  padding: '16px 14px',
                  borderRadius: 18,
                  background: 'rgba(15, 23, 42, 0.75)',
                  border: '1.5px solid rgba(255, 255, 255, 0.12)',
                  backdropFilter: 'blur(14px)',
                  boxShadow: `0 ${10 + focusLevel * 10}px ${25 + focusLevel * 15}px rgba(0,0,0,${0.4 + focusLevel * 0.25}), 0 0 ${focusLevel * 35}px ${w.color}66`,
                  textAlign: 'center',
                  position: 'relative',
                  overflow: 'hidden',
                  zIndex: focusLevel > 0.1 ? 2 : 1,
                }}
              >
                {/* Active Focus Gradient Fill Overlay (smooth fade) */}
                <div
                  style={{
                    position: 'absolute',
                    inset: 0,
                    borderRadius: 18,
                    background:
                      'linear-gradient(145deg, rgba(30, 41, 59, 0.95), rgba(15, 23, 42, 0.95))',
                    opacity: focusLevel,
                    pointerEvents: 'none',
                  }}
                />

                {/* Active Focus Glowing Border Overlay (smooth fade) */}
                <div
                  style={{
                    position: 'absolute',
                    inset: 0,
                    borderRadius: 18,
                    border: `2.5px solid ${w.color}`,
                    boxShadow: `inset 0 0 16px ${w.color}40`,
                    opacity: focusLevel,
                    pointerEvents: 'none',
                  }}
                />

                {/* Shimmer sweep on focused card */}
                {focusLevel > 0.05 && (
                  <div
                    style={{
                      position: 'absolute',
                      inset: 0,
                      opacity: focusLevel,
                      pointerEvents: 'none',
                    }}
                  >
                    <ShimmerSweep interval={40} duration={18} color="rgba(255, 255, 255, 0.45)" />
                  </div>
                )}

                {/* Word with smooth text glow and color crossfade */}
                <div
                  style={{
                    position: 'relative',
                    fontSize: 40,
                    fontWeight: 900,
                    color: w.color,
                    marginBottom: 4,
                    textShadow: `0 0 ${focusLevel * 22}px ${w.color}`,
                  }}
                >
                  <span>{w.bn}</span>
                  <span
                    style={{
                      position: 'absolute',
                      inset: 0,
                      color: '#ffffff',
                      opacity: focusLevel,
                      textShadow: `0 0 20px ${w.color}`,
                    }}
                  >
                    {w.bn}
                  </span>
                </div>

                <div
                  style={{
                    position: 'relative',
                    fontSize: 14,
                    color: '#94a3b8',
                    fontWeight: 600,
                    marginBottom: 8,
                  }}
                >
                  {w.en}
                </div>

                <div
                  style={{
                    position: 'relative',
                    fontSize: 12,
                    color: '#cbd5e1',
                    padding: '3px 6px',
                    borderRadius: 6,
                    background: `rgba(56, 189, 248, ${0.05 + focusLevel * 0.18})`,
                    border: `1px solid rgba(255, 255, 255, ${0.05 + focusLevel * 0.2})`,
                    fontWeight: 600,
                  }}
                >
                  {w.root}
                </div>
              </div>
            );
          })}
        </div>

        {/* Dedicated Golden illuminated Tribute Plaque to Pioneers */}
        <div
          style={{
            transform: `scale(${painIn}) translateY(${(1 - painIn) * 30 + warningWobble * 0.2}px)`,
            opacity: painIn,
            width: '100%',
            maxWidth: 1040,
            padding: '16px 28px',
            borderRadius: 20,
            background:
              'linear-gradient(135deg, rgba(245, 158, 11, 0.12), rgba(15, 23, 42, 0.85) 50%, rgba(225, 29, 72, 0.12))',
            border: '2px solid rgba(251, 191, 36, 0.65)',
            boxShadow:
              '0 15px 40px rgba(0,0,0,0.6), 0 0 35px rgba(245, 158, 11, 0.25)',
            textAlign: 'center',
            position: 'relative',
            overflow: 'hidden',
          }}
        >
          <ShimmerSweep interval={90} duration={26} color="rgba(251, 191, 36, 0.35)" />

          {/* Tribute Header */}
          <div
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              fontSize: 19,
              fontWeight: 800,
              color: '#fde047',
              letterSpacing: 0.5,
              marginBottom: 10,
              textShadow: '0 0 15px rgba(250, 204, 21, 0.5)',
            }}
          >
            <span>👑</span>
            <span>কৃতজ্ঞতা ও সশ্রদ্ধ স্মরণ — বাংলা কম্পিউটিংয়ের অগ্রদূতদের</span>
            <span>👑</span>
          </div>

          {/* 3 Prominent Pioneer Badges */}
          <div
            style={{
              display: 'flex',
              justifyContent: 'center',
              gap: 16,
              margin: '6px 0 12px 0',
            }}
          >
            {/* Avro Keyboard Badge */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '6px 16px',
                borderRadius: 12,
                background: 'rgba(245, 158, 11, 0.22)',
                border: '1.5px solid rgba(251, 191, 36, 0.75)',
                boxShadow: '0 0 20px rgba(245, 158, 11, 0.4)',
              }}
            >
              <span style={{ fontSize: 20 }}>⌨️</span>
              <span style={{ fontSize: 18, fontWeight: 800, color: '#fef08a' }}>Avro Keyboard</span>
            </div>

            {/* OpenBangla Keyboard Badge */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '6px 16px',
                borderRadius: 12,
                background: 'rgba(56, 189, 248, 0.22)',
                border: '1.5px solid rgba(56, 189, 248, 0.75)',
                boxShadow: '0 0 20px rgba(56, 189, 248, 0.4)',
              }}
            >
              <span style={{ fontSize: 20 }}>🌐</span>
              <span style={{ fontSize: 18, fontWeight: 800, color: '#bae6fd' }}>OpenBangla Keyboard</span>
            </div>

            {/* Lekho Badge */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '6px 16px',
                borderRadius: 12,
                background: 'rgba(52, 211, 153, 0.22)',
                border: '1.5px solid rgba(52, 211, 153, 0.75)',
                boxShadow: '0 0 20px rgba(52, 211, 153, 0.4)',
              }}
            >
              <span style={{ fontSize: 20 }}>✍️</span>
              <span style={{ fontSize: 18, fontWeight: 800, color: '#a7f3d0' }}>Lekho</span>
            </div>
          </div>

          {/* Narrative Line */}
          <div style={{ fontSize: 17, color: '#cbd5e1', lineHeight: 1.5, maxWidth: 960, margin: '0 auto' }}>
            কম্পিউটারে বাংলা লেখাকে সহজ ও সর্বজনীন করার ঐতিহাসিক অগ্রদূত হিসেবে তাঁদের অবদান চিরস্মরণীয়।
            <br />
            <b style={{ color: '#ffffff', fontWeight: 700 }}>
              তাঁদেরই দীর্ঘদিনের নিঃস্বার্থ পথচলা ও অনুপ্রেরণায় আধুনিক প্রজন্মের জন্য উন্মুক্তভাবে তৈরি 'একুশে নেক্সট' (Ekushe Next)।
            </b>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
