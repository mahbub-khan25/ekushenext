import React from 'react';
import {
  AbsoluteFill,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
  Easing,
} from 'remotion';
import { BackgroundParticles } from '../../components/BackgroundParticles';
import { ShimmerSweep } from '../../components/ShimmerSweep';

const WORDS_DATA = [
  { bn: 'সৃষ্টি', en: 'Creation', root: 'স + ৃ + ষ্ + ট + ি', color: '#38bdf8' },
  { bn: 'স্মৃতি', en: 'Memory', root: 'স + ম + ৃ + ত + ি', color: '#a78bfa' },
  { bn: 'স্বাধীনতা', en: 'Freedom', root: 'স + ব + া + ধ + ী + ন + ত + া', color: '#34d399' },
  { bn: 'বিজ্ঞান', en: 'Science', root: 'ব + ি + জ + ঞ + া + ন', color: '#fbbf24' },
];

// Continuous mathematical focus level in [0, 1] for each card (silky smooth transitions in 2x2 grid)
const getCardFocusVertical = (idx: number, currentF: number): number => {
  const windows = [
    { inStart: 46, inEnd: 60, outStart: 90, outEnd: 106 },   // Card 0: সৃষ্টি
    { inStart: 90, inEnd: 106, outStart: 136, outEnd: 152 }, // Card 1: স্মৃতি
    { inStart: 136, inEnd: 152, outStart: 182, outEnd: 198 }, // Card 2: স্বাধীনতা
    { inStart: 182, inEnd: 198, outStart: 235, outEnd: 255 }, // Card 3: বিজ্ঞান
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

export const Scene2WordsAndPainVertical: React.FC = () => {
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

  // Smoothly traveling spotlight across 2x2 grid (TL -> TR -> BL -> BR)
  const spotX = interpolate(
    f,
    [53, 98, 144, 190],
    [-220, 220, -220, 220],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
      easing: Easing.inOut(Easing.quad),
    }
  );

  const spotY = interpolate(
    f,
    [53, 98, 144, 190],
    [-110, -110, 110, 110],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
      easing: Easing.inOut(Easing.quad),
    }
  );

  const spotOpacity = interpolate(
    f,
    [44, 58, 235, 255],
    [0, 0.75, 0.75, 0],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );

  const warningWobble = Math.sin(f * 0.25) * 5;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: '#030712',
        backgroundImage:
          'radial-gradient(circle at 50% 50%, rgba(99, 102, 241, 0.18) 0%, rgba(15, 23, 42, 0.85) 50%, #020617 100%)',
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
          padding: '0 45px',
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
            marginBottom: 35,
          }}
        >
          <div
            style={{
              position: 'relative',
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              padding: '8px 24px',
              borderRadius: 999,
              background: 'rgba(56, 189, 248, 0.15)',
              border: '1.5px solid rgba(56, 189, 248, 0.45)',
              color: '#7dd3fc',
              fontSize: 22,
              fontWeight: 700,
              marginBottom: 18,
              overflow: 'hidden',
            }}
          >
            <ShimmerSweep interval={70} duration={22} color="rgba(125, 211, 252, 0.4)" />
            <span>✨</span>
            <span>ভাষার নান্দনিকতা ও যুক্তাক্ষর</span>
            <span>✨</span>
          </div>

          <h2
            style={{
              fontSize: 54,
              fontWeight: 900,
              margin: 0,
              lineHeight: 1.25,
              background:
                'linear-gradient(135deg, #ffffff 40%, #bae6fd 80%, #7dd3fc 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              letterSpacing: 0.5,
            }}
          >
            বাংলা শব্দের সৌন্দর্য অনন্য,
            <br />
            কিন্তু কম্পিউটারে টাইপিং?
          </h2>
        </div>

        {/* 2x2 Grid of Word Cards with Smooth Gliding Focus */}
        <div
          style={{
            position: 'relative',
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: 18,
            marginBottom: 35,
            width: '100%',
            maxWidth: 920,
          }}
        >
          {/* Gliding Ambient Spotlight Aura in 2x2 grid */}
          <div
            style={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              width: 320,
              height: 320,
              borderRadius: '50%',
              background:
                'radial-gradient(circle, rgba(168, 85, 247, 0.35) 0%, rgba(56, 189, 248, 0.22) 40%, transparent 70%)',
              transform: `translate(calc(-50% + ${spotX}px), calc(-50% + ${spotY}px))`,
              filter: 'blur(40px)',
              opacity: spotOpacity,
              pointerEvents: 'none',
              zIndex: 0,
            }}
          />

          {WORDS_DATA.map((w, idx) => {
            const cardSpring = spring({
              frame: f - (25 + idx * 8),
              fps: 30,
              config: { damping: 13, stiffness: 85 },
            });

            // Smooth continuous focus weight in [0, 1]
            const focusLevel = getCardFocusVertical(idx, f);

            // Continuous breathing scale and lift
            const breathe = Math.sin(f * 0.12) * 0.012;
            const focusScale = 1 + focusLevel * (0.04 + breathe);
            const focusLift = -focusLevel * 8;

            return (
              <div
                key={w.bn}
                style={{
                  transform: `scale(${cardSpring * focusScale}) translateY(${(1 - cardSpring) * 30 + focusLift}px)`,
                  opacity: cardSpring,
                  padding: '24px 20px',
                  borderRadius: 22,
                  background: 'rgba(15, 23, 42, 0.78)',
                  border: '2px solid rgba(255, 255, 255, 0.12)',
                  backdropFilter: 'blur(16px)',
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
                    borderRadius: 22,
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
                    borderRadius: 22,
                    border: `3px solid ${w.color}`,
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
                    fontSize: 50,
                    fontWeight: 900,
                    color: w.color,
                    marginBottom: 6,
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
                    fontSize: 18,
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
                    fontSize: 14,
                    color: '#cbd5e1',
                    padding: '4px 8px',
                    borderRadius: 8,
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

        {/* Dedicated Golden illuminated Tribute Plaque in Vertical */}
        <div
          style={{
            transform: `scale(${painIn}) translateY(${(1 - painIn) * 30 + warningWobble * 0.2}px)`,
            opacity: painIn,
            width: '100%',
            maxWidth: 920,
            padding: '24px 26px',
            borderRadius: 24,
            background:
              'linear-gradient(135deg, rgba(245, 158, 11, 0.15), rgba(15, 23, 42, 0.88) 50%, rgba(225, 29, 72, 0.15))',
            border: '2.5px solid rgba(251, 191, 36, 0.75)',
            boxShadow:
              '0 18px 45px rgba(0,0,0,0.65), 0 0 35px rgba(245, 158, 11, 0.3)',
            textAlign: 'center',
            position: 'relative',
            overflow: 'hidden',
          }}
        >
          <ShimmerSweep interval={90} duration={26} color="rgba(251, 191, 36, 0.4)" />

          <div
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              fontSize: 22,
              fontWeight: 800,
              color: '#fde047',
              letterSpacing: 0.5,
              marginBottom: 14,
              textShadow: '0 0 15px rgba(250, 204, 21, 0.5)',
            }}
          >
            <span>👑</span>
            <span>অগ্রদূতদের প্রতি গভীর শ্রদ্ধা</span>
            <span>👑</span>
          </div>

          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              gap: 10,
              margin: '8px 0 16px 0',
            }}
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: 10,
                padding: '8px 16px',
                borderRadius: 14,
                background: 'rgba(245, 158, 11, 0.22)',
                border: '1.5px solid rgba(251, 191, 36, 0.75)',
              }}
            >
              <span style={{ fontSize: 24 }}>⌨️</span>
              <span style={{ fontSize: 22, fontWeight: 800, color: '#fef08a' }}>Avro Keyboard</span>
            </div>

            <div style={{ display: 'flex', gap: 10 }}>
              <div
                style={{
                  flex: 1,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 8,
                  padding: '8px 12px',
                  borderRadius: 14,
                  background: 'rgba(56, 189, 248, 0.22)',
                  border: '1.5px solid rgba(56, 189, 248, 0.75)',
                }}
              >
                <span style={{ fontSize: 22 }}>🌐</span>
                <span style={{ fontSize: 18, fontWeight: 800, color: '#bae6fd' }}>OpenBangla</span>
              </div>

              <div
                style={{
                  flex: 1,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 8,
                  padding: '8px 12px',
                  borderRadius: 14,
                  background: 'rgba(52, 211, 153, 0.22)',
                  border: '1.5px solid rgba(52, 211, 153, 0.75)',
                }}
              >
                <span style={{ fontSize: 22 }}>✍️</span>
                <span style={{ fontSize: 18, fontWeight: 800, color: '#a7f3d0' }}>Lekho</span>
              </div>
            </div>
          </div>

          <div style={{ fontSize: 20, color: '#e2e8f0', lineHeight: 1.55 }}>
            বাংলা টাইপিংকে কম্পিউটারে সহজ করার জন্য তাঁদের অবদান চিরস্মরণীয়।
            <br />
            <b style={{ color: '#ffffff', fontWeight: 800 }}>
              তাঁদেরই পথচলায় অনুপ্রাণিত আধুনিক নতুন রূপ: 'একুশে নেক্সট' (Ekushe Next)।
            </b>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
