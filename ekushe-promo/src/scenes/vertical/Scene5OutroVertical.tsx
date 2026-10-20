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
import { BackgroundParticles } from '../../components/BackgroundParticles';
import { GlowRings } from '../../components/GlowRings';
import { ShimmerSweep } from '../../components/ShimmerSweep';

const PLATFORMS = [
  { name: 'Windows', icon: '🪟', status: 'প্রস্তুত', isLive: true },
  { name: 'macOS', icon: '🍎', status: 'আসছে', isLive: false },
  { name: 'Linux', icon: '🐧', status: 'আসছে', isLive: false },
  { name: 'iOS', icon: '📱', status: 'আসছে', isLive: false },
  { name: 'Android', icon: '🤖', status: 'আসছে', isLive: false },
];

export const Scene5OutroVertical: React.FC = () => {
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
          'radial-gradient(circle at 50% 45%, rgba(16, 185, 129, 0.28) 0%, rgba(225, 29, 72, 0.18) 50%, #020617 100%)',
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
          padding: '0 45px',
          textAlign: 'center',
          zIndex: 10,
          transform: `scale(${cameraZoom})`,
        }}
      >
        {/* App Logo */}
        <div
          style={{
            position: 'relative',
            width: 140,
            height: 140,
            borderRadius: 36,
            padding: 8,
            background:
              'linear-gradient(135deg, rgba(16, 185, 129, 0.55), rgba(225, 29, 72, 0.35))',
            border: '3px solid rgba(52, 211, 153, 0.85)',
            boxShadow:
              '0 0 50px rgba(16, 185, 129, 0.6), 0 0 90px rgba(225, 29, 72, 0.3)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            transform: `scale(${entrance}) translateY(${logoFloatY}px)`,
            opacity: entrance,
            marginBottom: 28,
          }}
        >
          <GlowRings color="#10b981" size={190} ringCount={3} />
          <Img
            src={staticFile('assets/logo_256.png')}
            style={{
              width: '100%',
              height: '100%',
              objectFit: 'contain',
            }}
          />
        </div>

        {/* Headline */}
        <div
          style={{
            transform: `scale(${entrance}) translateY(${(1 - entrance) * 20}px)`,
            opacity: entrance,
            marginBottom: 28,
          }}
        >
          <h1
            style={{
              fontSize: 66,
              fontWeight: 900,
              margin: '0 0 12px 0',
              lineHeight: 1.2,
              background: 'linear-gradient(135deg, #ffffff 30%, #a7f3d0 70%, #6ee7b7 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
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
              lineHeight: 1.45,
            }}
          >
            "ভাষার অহংকার হৃদয়ে,
            <br />
            আধুনিক প্রযুক্তির সহজ শক্তি কিবোর্ডে।"
          </p>
        </div>

        {/* CTA Button */}
        <div
          style={{
            position: 'relative',
            transform: `scale(${buttonSpring * buttonPulse}) translateY(${(1 - buttonSpring) * 20}px)`,
            opacity: buttonSpring,
            width: '100%',
            maxWidth: 880,
            marginBottom: 35,
          }}
        >
          <div
            style={{
              position: 'relative',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: 8,
              padding: '20px 24px',
              borderRadius: 24,
              background: 'linear-gradient(135deg, #059669 0%, #10b981 50%, #047857 100%)',
              border: '2.5px solid rgba(167, 243, 208, 0.8)',
              boxShadow: '0 18px 45px rgba(5, 150, 105, 0.6), 0 0 50px rgba(16, 185, 129, 0.45)',
              color: '#ffffff',
              overflow: 'hidden',
            }}
          >
            <ShimmerSweep interval={55} duration={20} color="rgba(255, 255, 255, 0.5)" />
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <span style={{ fontSize: 30, transform: `translateY(${arrowBounce}px)` }}>⬇️</span>
              <span style={{ fontSize: 26, fontWeight: 900 }}>Ekushe-Next-Setup.exe</span>
            </div>
            <span style={{ fontSize: 18, color: '#ecfdf5', fontWeight: 700 }}>
              আজই ডাউনলোড ও বিনামূল্যে ব্যবহার করুন
            </span>
          </div>
        </div>

        {/* Platform Capsules */}
        <div
          style={{
            display: 'flex',
            flexWrap: 'wrap',
            justifyContent: 'center',
            gap: 10,
            marginBottom: 30,
            maxWidth: 900,
          }}
        >
          {PLATFORMS.map((plat) => (
            <div
              key={plat.name}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '8px 16px',
                borderRadius: 14,
                background: plat.isLive ? 'rgba(16, 185, 129, 0.25)' : 'rgba(30, 41, 59, 0.6)',
                border: plat.isLive ? '1.5px solid rgba(52, 211, 153, 0.7)' : '1px solid rgba(255, 255, 255, 0.1)',
              }}
            >
              <span style={{ fontSize: 20 }}>{plat.icon}</span>
              <span style={{ fontSize: 16, fontWeight: 800, color: '#ffffff' }}>{plat.name}</span>
              <span style={{ fontSize: 13, fontWeight: 800, color: plat.isLive ? '#34d399' : '#94a3b8' }}>
                {plat.isLive ? '🟢 প্রস্তুত' : 'শীঘ্রই'}
              </span>
            </div>
          ))}
        </div>

        {/* Tribute Ribbon in Vertical */}
        <div
          style={{
            transform: `scale(${entrance})`,
            opacity: entrance,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: 8,
            padding: '12px 20px',
            borderRadius: 18,
            background: 'rgba(245, 158, 11, 0.12)',
            border: '1.5px solid rgba(251, 191, 36, 0.45)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 16, color: '#fef08a', fontWeight: 800 }}>
            <span>❤️ Inspired by Pioneers:</span>
          </div>
          <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', justifyContent: 'center' }}>
            <span style={{ fontSize: 13, padding: '3px 8px', borderRadius: 6, background: 'rgba(245, 158, 11, 0.25)', color: '#ffffff', fontWeight: 800 }}>
              ⌨️ Avro
            </span>
            <span style={{ fontSize: 13, padding: '3px 8px', borderRadius: 6, background: 'rgba(56, 189, 248, 0.25)', color: '#ffffff', fontWeight: 800 }}>
              🌐 OpenBangla
            </span>
            <span style={{ fontSize: 13, padding: '3px 8px', borderRadius: 6, background: 'rgba(52, 211, 153, 0.25)', color: '#ffffff', fontWeight: 800 }}>
              ✍️ Lekho
            </span>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
