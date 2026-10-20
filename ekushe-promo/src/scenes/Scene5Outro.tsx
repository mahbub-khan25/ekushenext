import React from 'react';
import {
  AbsoluteFill,
  Img,
  interpolate,
  spring,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from 'remotion';
import { BackgroundParticles } from '../components/BackgroundParticles';
import { GlowRings } from '../components/GlowRings';
import { ShimmerSweep } from '../components/ShimmerSweep';

const PLATFORMS = [
  { name: 'Windows', icon: '🪟', status: 'প্রস্তুত', isLive: true },
  { name: 'macOS', icon: '🍎', status: 'আসছে', isLive: false },
  { name: 'Linux', icon: '🐧', status: 'আসছে', isLive: false },
  { name: 'iOS', icon: '📱', status: 'আসছে', isLive: false },
  { name: 'Android', icon: '🤖', status: 'আসছে', isLive: false },
];

export const Scene5Outro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);

  const cameraZoom = interpolate(f, [0, 240], [1, 1.05], {
    extrapolateRight: 'clamp',
  });

  const entrance = spring({
    frame: f - 10,
    fps: 30,
    config: { damping: 13, stiffness: 75 },
  });

  const buttonSpring = spring({
    frame: f - 35,
    fps: 30,
    config: { damping: 12, stiffness: 90 },
  });

  // Final subtle fade to black on the last 20 frames (215-238)
  const finalFade = interpolate(f, [215, 238], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const logoFloatY = Math.sin(f * 0.08) * 6;
  const buttonPulse = 1 + Math.sin(f * 0.1) * 0.03;
  const arrowBounce = Math.sin(f * 0.22) * 4;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: '#020617',
        backgroundImage:
          'radial-gradient(circle at 50% 45%, rgba(16, 185, 129, 0.25) 0%, rgba(225, 29, 72, 0.15) 50%, #020617 100%)',
        color: '#ffffff',
        fontFamily: "'Kalpurush', 'HindSiliguri', sans-serif",
        opacity: finalFade,
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
          textAlign: 'center',
          zIndex: 10,
          transform: `scale(${cameraZoom})`,
        }}
      >
        {/* Glorious Big App Icon & Pulsing Halo */}
        <div
          style={{
            position: 'relative',
            width: 120,
            height: 120,
            borderRadius: 32,
            padding: 8,
            background:
              'linear-gradient(135deg, rgba(16, 185, 129, 0.5), rgba(225, 29, 72, 0.3))',
            border: '2.5px solid rgba(52, 211, 153, 0.8)',
            boxShadow:
              '0 0 50px rgba(16, 185, 129, 0.6), 0 0 90px rgba(225, 29, 72, 0.3)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            transform: `scale(${entrance}) translateY(${logoFloatY}px)`,
            opacity: entrance,
            marginBottom: 20,
          }}
        >
          <GlowRings color="#10b981" size={170} ringCount={3} />
          <Img
            src={staticFile('assets/logo_256.png')}
            style={{
              width: '100%',
              height: '100%',
              objectFit: 'contain',
              filter: 'drop-shadow(0 6px 14px rgba(0,0,0,0.6))',
            }}
          />
        </div>

        {/* Grand Headline: একুশে নেক্সট — ভাষার অহংকার হৃদয়ে */}
        <div
          style={{
            transform: `scale(${entrance}) translateY(${(1 - entrance) * 20}px)`,
            opacity: entrance,
            marginBottom: 16,
          }}
        >
          <h1
            style={{
              fontSize: 66,
              fontWeight: 900,
              margin: '0 0 8px 0',
              lineHeight: 1.15,
              background:
                'linear-gradient(135deg, #ffffff 30%, #a7f3d0 70%, #6ee7b7 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              textShadow: '0 0 45px rgba(16, 185, 129, 0.5)',
              letterSpacing: 1,
            }}
          >
            একুশে নেক্সট · Ekushe Next
          </h1>

          <p
            style={{
              fontSize: 26,
              color: '#e2e8f0',
              margin: 0,
              fontWeight: 700,
              letterSpacing: 0.5,
            }}
          >
            "ভাষার অহংকার হৃদয়ে, আধুনিক প্রযুক্তির সহজ শক্তি কিবোর্ডে।"
          </p>
        </div>

        {/* Hero Call-to-Action (CTA) Download Button with Breathing Scale */}
        <div
          style={{
            position: 'relative',
            transform: `scale(${buttonSpring * buttonPulse}) translateY(${(1 - buttonSpring) * 20}px)`,
            opacity: buttonSpring,
            marginBottom: 22,
          }}
        >
          <div
            style={{
              position: 'relative',
              display: 'inline-flex',
              alignItems: 'center',
              gap: 16,
              padding: '16px 42px',
              borderRadius: 20,
              background:
                'linear-gradient(135deg, #059669 0%, #10b981 50%, #047857 100%)',
              border: '2px solid rgba(167, 243, 208, 0.7)',
              boxShadow:
                '0 15px 40px rgba(5, 150, 105, 0.55), 0 0 50px rgba(16, 185, 129, 0.45)',
              color: '#ffffff',
              fontSize: 24,
              fontWeight: 800,
              cursor: 'pointer',
              overflow: 'hidden',
            }}
          >
            <ShimmerSweep interval={55} duration={20} color="rgba(255, 255, 255, 0.5)" />
            <span style={{ fontSize: 28, transform: `translateY(${arrowBounce}px)` }}>⬇️</span>
            <span>Ekushe-Next-Setup.exe — আজই ডাউনলোড ও ব্যবহার করুন</span>
            <span
              style={{
                fontSize: 14,
                padding: '4px 10px',
                borderRadius: 999,
                background: 'rgba(255,255,255,0.25)',
                color: '#ffffff',
                fontWeight: 700,
              }}
            >
              100% Free & Open Source
            </span>
          </div>
        </div>

        {/* Platform Availability Capsules */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: 12,
            marginBottom: 20,
          }}
        >
          <span style={{ fontSize: 16, color: '#94a3b8', fontWeight: 600, marginRight: 4 }}>
            ✨ সকল প্ল্যাটফর্মেই একুশে নেক্সটের সহজ অভিজ্ঞতা:
          </span>
          {PLATFORMS.map((plat, idx) => {
            const pSpring = spring({
              frame: f - (45 + idx * 6),
              fps: 30,
              config: { damping: 13, stiffness: 85 },
            });

            return (
              <div
                key={plat.name}
                style={{
                  transform: `scale(${pSpring})`,
                  opacity: pSpring,
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  padding: '7px 16px',
                  borderRadius: 12,
                  background: plat.isLive
                    ? 'rgba(16, 185, 129, 0.22)'
                    : 'rgba(30, 41, 59, 0.55)',
                  border: plat.isLive
                    ? '1.5px solid rgba(52, 211, 153, 0.6)'
                    : '1px solid rgba(255, 255, 255, 0.08)',
                  boxShadow: plat.isLive
                    ? '0 0 15px rgba(16, 185, 129, 0.35)'
                    : 'none',
                }}
              >
                <span style={{ fontSize: 18 }}>{plat.icon}</span>
                <span style={{ fontSize: 15, fontWeight: 700, color: '#ffffff' }}>
                  {plat.name}
                </span>
                <span
                  style={{
                    fontSize: 12,
                    fontWeight: 800,
                    color: plat.isLive ? '#34d399' : '#64748b',
                  }}
                >
                  {plat.isLive ? '🟢 প্রস্তুত' : 'শীঘ্রই'}
                </span>
              </div>
            );
          })}
        </div>

        {/* Final Pioneer Tribute Ribbon */}
        <div
          style={{
            transform: `scale(${entrance})`,
            opacity: entrance,
            display: 'inline-flex',
            alignItems: 'center',
            gap: 12,
            padding: '8px 24px',
            borderRadius: 999,
            background: 'rgba(245, 158, 11, 0.1)',
            border: '1.5px solid rgba(251, 191, 36, 0.45)',
            boxShadow: '0 0 25px rgba(245, 158, 11, 0.2)',
            position: 'relative',
            overflow: 'hidden',
          }}
        >
          <ShimmerSweep interval={75} duration={22} color="rgba(251, 191, 36, 0.35)" />
          <span style={{ fontSize: 16 }}>❤️</span>
          <span style={{ fontSize: 15, color: '#fef08a', fontWeight: 700 }}>
            Inspired by:
          </span>
          <span style={{ fontSize: 14, padding: '2px 8px', borderRadius: 6, background: 'rgba(245, 158, 11, 0.2)', color: '#ffffff', fontWeight: 800 }}>
            ⌨️ Avro Keyboard
          </span>
          <span style={{ fontSize: 14, padding: '2px 8px', borderRadius: 6, background: 'rgba(56, 189, 248, 0.2)', color: '#ffffff', fontWeight: 800 }}>
            🌐 OpenBangla
          </span>
          <span style={{ fontSize: 14, padding: '2px 8px', borderRadius: 6, background: 'rgba(52, 211, 153, 0.2)', color: '#ffffff', fontWeight: 800 }}>
            ✍️ Lekho
          </span>
          <span style={{ fontSize: 14, color: '#cbd5e1' }}>
            — অগ্রদূতদের প্রতি গভীর কৃতজ্ঞতা
          </span>
        </div>
      </div>
    </AbsoluteFill>
  );
};
