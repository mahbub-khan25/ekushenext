import React from 'react';
import {
  AbsoluteFill,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from 'remotion';
import { BackgroundParticles } from '../../components/BackgroundParticles';
import { ShaheedMinarGraphic } from '../../components/ShaheedMinarGraphic';
import { FloatingLetters } from '../../components/FloatingLetters';
import { ShimmerSweep } from '../../components/ShimmerSweep';

export const Scene1HeritageVertical: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);

  // Slow cinematic camera push-in
  const cameraZoom = interpolate(f, [0, 240], [1, 1.05], {
    extrapolateRight: 'clamp',
  });

  // Underline expansion
  const lineProgress = interpolate(f, [50, 95], [0, 100], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const titleSpring = spring({
    frame: f - 10,
    fps: 30,
    config: { damping: 12, stiffness: 80 },
  });

  const subtitleSpring = spring({
    frame: f - 35,
    fps: 30,
    config: { damping: 14, stiffness: 70 },
  });

  const quoteSpring = spring({
    frame: f - 75,
    fps: 30,
    config: { damping: 14, stiffness: 70 },
  });

  const fadeOut = interpolate(f, [205, 238], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  return (
    <AbsoluteFill
      style={{
        backgroundColor: '#050811',
        backgroundImage:
          'radial-gradient(circle at 50% 45%, rgba(225, 29, 72, 0.28) 0%, rgba(6, 78, 59, 0.22) 50%, #030712 100%)',
        color: '#ffffff',
        fontFamily: "'Kalpurush', 'HindSiliguri', sans-serif",
        opacity: fadeOut,
        overflow: 'hidden',
      }}
    >
      <BackgroundParticles theme="crimson" />
      <FloatingLetters opacity={0.65} isVertical={true} />

      {/* Main Content Container with Cinematic Push */}
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
        {/* Subtle Year / Date Pill */}
        <div
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 8,
            padding: '8px 24px',
            borderRadius: 999,
            background: 'rgba(225, 29, 72, 0.18)',
            border: '1.5px solid rgba(251, 113, 133, 0.5)',
            color: '#fecdd3',
            fontSize: 22,
            fontWeight: 700,
            letterSpacing: 2,
            marginBottom: 35,
            transform: `scale(${titleSpring})`,
            opacity: titleSpring,
          }}
        >
          <span>🌸</span>
          <span>১৯৫২ — অমর একুশে — ২০২৬</span>
          <span>🌸</span>
        </div>

        {/* Central Shaheed Minar Graphic */}
        <div style={{ marginBottom: 40, transform: 'scale(1.15)' }}>
          <ShaheedMinarGraphic scale={1.1} glowIntensity={1.3} />
        </div>

        {/* Main Title: রক্তের অক্ষরে কেনা আমাদের বর্ণমালা */}
        <div
          style={{
            transform: `scale(${subtitleSpring}) translateY(${(1 - subtitleSpring) * 20}px)`,
            opacity: subtitleSpring,
            maxWidth: 960,
          }}
        >
          <h1
            style={{
              fontSize: 64,
              fontWeight: 900,
              lineHeight: 1.25,
              margin: '0 0 16px 0',
              background:
                'linear-gradient(135deg, #ffffff 30%, #fecdd3 70%, #fda4af 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              textShadow: '0 0 45px rgba(225, 29, 72, 0.45)',
              letterSpacing: 1,
            }}
          >
            রক্তের অক্ষরে কেনা
            <br />
            আমাদের বর্ণমালা
          </h1>

          {/* Golden Animated Accent Underline */}
          <div
            style={{
              width: 380,
              height: 4,
              margin: '0 auto 26px auto',
              background: 'rgba(255, 255, 255, 0.1)',
              borderRadius: 2,
              overflow: 'hidden',
              position: 'relative',
            }}
          >
            <div
              style={{
                width: `${lineProgress}%`,
                height: '100%',
                background:
                  'linear-gradient(90deg, #e11d48, #f59e0b, #10b981)',
                boxShadow: '0 0 20px rgba(245, 158, 11, 0.9)',
                margin: '0 auto',
                borderRadius: 2,
              }}
            />
          </div>
        </div>

        {/* Historic Subtitle as shown in screenshot */}
        <div
          style={{
            transform: `translateY(${(1 - quoteSpring) * 20}px)`,
            opacity: quoteSpring,
            width: '100%',
            maxWidth: 960,
            textAlign: 'center',
          }}
        >
          <p
            style={{
              fontSize: 30,
              lineHeight: 1.65,
              color: '#f8fafc',
              margin: 0,
              fontWeight: 600,
              textShadow: '0 2px 14px rgba(0,0,0,0.8)',
            }}
          >
            ভাষার জন্য জীবন দেওয়ার অনন্য ইতিহাস কেবল বাংলাদেশিদেরই রয়েছে।
            <br />
            <span
              style={{
                color: '#fb7185',
                fontWeight: 700,
                textShadow: '0 0 20px rgba(225, 29, 72, 0.5)',
              }}
            >
              শহীদের আত্মত্যাগে সমৃদ্ধ এই বাংলা ভাষা আজ বিশ্ব দরবারে মর্যাদার প্রতীক।
            </span>
          </p>
        </div>
      </div>
    </AbsoluteFill>
  );
};
