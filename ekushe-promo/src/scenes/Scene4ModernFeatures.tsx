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
import { ScanlineBeam } from '../components/ScanlineBeam';
import { ShimmerSweep } from '../components/ShimmerSweep';

const FEATURES = [
  {
    icon: '🔬',
    title: 'শব্দ এক্স-রে ও ২-ক্লিক যুক্তবর্ণ নির্মাতা',
    desc: 'যেকোনো শব্দ পেস্ট করলেই কীভাবে গঠিত তা দেখায়, আর ২টি ব্যঞ্জনে ক্লিক করেই যুক্তবর্ণ!',
  },
  {
    icon: '⚡',
    title: 'ইনলাইন মিক্স-ইংলিশ ও শব্দ ফ্লিপ',
    desc: '/office লিখলেই ইংরেজি, আর শেষ শব্দ বাংলা ⇄ ইংরেজি পাল্টাতে Ctrl+Backspace!',
  },
  {
    icon: '🔢',
    title: 'স্বয়ংক্রিয় সংখ্যা সিঙ্ক (Smart Numbers)',
    desc: 'কিবোর্ড অনুযায়ী ইংরেজি মোডে 0-9 এবং বাংলা মোডে ০-৯ স্বয়ংক্রিয়ভাবে কাজ করে।',
  },
  {
    icon: '💬',
    title: 'কুইক টেক্সট স্নিপেটস (Personal Snippets)',
    desc: ';salam ➔ আসসালামু আলাইকুম, ;tk ➔ ৳, ;bd ➔ গণপ্রজাতন্ত্রী বাংলাদেশ।',
  },
];

// Continuous mathematical focus level for feature items
const getFeatureFocus = (idx: number, currentF: number): number => {
  const windows = [
    { inStart: 0, inEnd: 20, outStart: 60, outEnd: 75 },
    { inStart: 60, inEnd: 75, outStart: 125, outEnd: 140 },
    { inStart: 125, inEnd: 140, outStart: 190, outEnd: 205 },
    { inStart: 190, inEnd: 205, outStart: 260, outEnd: 280 },
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

export const Scene4ModernFeatures: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);

  const cameraZoom = interpolate(f, [0, 298], [1, 1.04], {
    extrapolateRight: 'clamp',
  });

  const titleIn = spring({
    frame: f - 8,
    fps: 30,
    config: { damping: 13, stiffness: 80 },
  });

  const screenshotIn = spring({
    frame: f - 24,
    fps: 30,
    config: { damping: 14, stiffness: 70 },
  });

  const fadeOut = interpolate(f, [270, 298], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  // Cycle through feature highlight every 65 frames
  const activeIdx = Math.min(3, Math.floor(f / 65));

  // 3D Screenshot floating wobble at 60fps
  const rotY = -6 + Math.sin(f * 0.04) * 3;
  const rotX = Math.cos(f * 0.03) * 2;
  const floatY = Math.sin(f * 0.05) * 6;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: '#030712',
        backgroundImage:
          'radial-gradient(circle at 65% 50%, rgba(56, 189, 248, 0.15) 0%, rgba(6, 78, 59, 0.2) 45%, #020617 100%)',
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
          padding: '0 70px',
          zIndex: 10,
          transform: `scale(${cameraZoom})`,
        }}
      >
        {/* Header Badge with Shimmer */}
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
              marginBottom: 10,
              overflow: 'hidden',
            }}
          >
            <ShimmerSweep interval={65} duration={20} color="rgba(125, 211, 252, 0.4)" />
            <span>⚡</span>
            <span>একুশে নেক্সট স্টুডিও ও ফ্লোটিং পপ-আপের অনন্য বৈশিষ্ট্যসমূহ</span>
            <span>⚡</span>
          </div>

          <h2
            style={{
              fontSize: 46,
              fontWeight: 900,
              margin: 0,
              lineHeight: 1.2,
              background:
                'linear-gradient(135deg, #ffffff 40%, #a7f3d0 80%, #34d399 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              letterSpacing: 0.5,
            }}
          >
            দৈনন্দিন বাংলা লেখাকে আনন্দময় করার আধুনিক উদ্ভাবন
          </h2>
        </div>

        {/* 2-Column Showcase */}
        <div
          style={{
            display: 'flex',
            width: '100%',
            maxWidth: 1200,
            gap: 36,
            alignItems: 'center',
          }}
        >
          {/* Left Column: 4 Feature Cards */}
          <div
            style={{
              flex: 1,
              display: 'flex',
              flexDirection: 'column',
              gap: 12,
            }}
          >
            {FEATURES.map((feat, idx) => {
              const cardIn = spring({
                frame: f - (15 + idx * 8),
                fps: 30,
                config: { damping: 13, stiffness: 80 },
              });

              const focusLevel = getFeatureFocus(idx, f);
              const iconBounce = Math.sin(f * 0.2) * 3 * focusLevel;

              return (
                <div
                  key={feat.title}
                  style={{
                    transform: `scale(${cardIn * (1 + focusLevel * 0.02)}) translateX(${(1 - cardIn) * -30}px)`,
                    opacity: cardIn,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 16,
                    padding: '14px 20px',
                    borderRadius: 16,
                    background: 'rgba(15, 23, 42, 0.65)',
                    border: '1.5px solid rgba(255, 255, 255, 0.08)',
                    boxShadow: `0 ${4 + focusLevel * 6}px ${15 + focusLevel * 15}px rgba(0,0,0,${0.3 + focusLevel * 0.2}), 0 0 ${focusLevel * 25}px rgba(16, 185, 129, 0.35)`,
                    backdropFilter: 'blur(12px)',
                    position: 'relative',
                    overflow: 'hidden',
                  }}
                >
                  {/* Smooth Gradient Fill Overlay */}
                  <div
                    style={{
                      position: 'absolute',
                      inset: 0,
                      borderRadius: 16,
                      background:
                        'linear-gradient(135deg, rgba(16, 185, 129, 0.28), rgba(15, 23, 42, 0.95))',
                      opacity: focusLevel,
                      pointerEvents: 'none',
                    }}
                  />

                  {/* Smooth Glowing Border Overlay */}
                  <div
                    style={{
                      position: 'absolute',
                      inset: 0,
                      borderRadius: 16,
                      border: '2px solid rgba(52, 211, 153, 0.85)',
                      boxShadow: 'inset 0 0 12px rgba(52, 211, 153, 0.25)',
                      opacity: focusLevel,
                      pointerEvents: 'none',
                    }}
                  />

                  {focusLevel > 0.05 && (
                    <div
                      style={{
                        position: 'absolute',
                        inset: 0,
                        opacity: focusLevel,
                        pointerEvents: 'none',
                      }}
                    >
                      <ShimmerSweep interval={45} duration={18} color="rgba(167, 243, 208, 0.4)" />
                    </div>
                  )}

                  <div
                    style={{
                      position: 'relative',
                      fontSize: 30,
                      width: 50,
                      height: 50,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      borderRadius: 12,
                      background: `rgba(16, 185, 129, ${0.05 + focusLevel * 0.3})`,
                      transform: `translateY(${iconBounce}px) scale(${1 + focusLevel * 0.1})`,
                    }}
                  >
                    {feat.icon}
                  </div>
                  <div style={{ flex: 1, textAlign: 'left', position: 'relative' }}>
                    <div
                      style={{
                        position: 'relative',
                        fontSize: 20,
                        fontWeight: 700,
                        color: '#ffffff',
                        marginBottom: 4,
                      }}
                    >
                      <span>{feat.title}</span>
                      <span
                        style={{
                          position: 'absolute',
                          inset: 0,
                          color: '#a7f3d0',
                          opacity: focusLevel,
                        }}
                      >
                        {feat.title}
                      </span>
                    </div>
                    <div
                      style={{
                        fontSize: 15,
                        color: '#94a3b8',
                        lineHeight: 1.45,
                      }}
                    >
                      {feat.desc}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Right Column: 🌟 3D Perspective App Window in PURE LIGHT MODE 🌟 */}
          <div
            style={{
              flex: 1.1,
              position: 'relative',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              transform: `scale(${screenshotIn}) translateX(${(1 - screenshotIn) * 30}px) perspective(1000px) rotateY(${rotY}deg) rotateX(${rotX}deg) translateY(${floatY}px)`,
              opacity: screenshotIn,
            }}
          >
            <div
              style={{
                position: 'relative',
                width: '100%',
                borderRadius: 22,
                overflow: 'hidden',
                background: '#ffffff',
                border: '2px solid rgba(226, 232, 240, 0.95)',
                boxShadow:
                  '0 25px 60px rgba(0,0,0,0.6), 0 0 45px rgba(56, 189, 248, 0.25)',
              }}
            >
              {/* Light Mode Window Header */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '10px 18px',
                  background: '#f8fafc',
                  borderBottom: '1px solid #e2e8f0',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <div style={{ width: 11, height: 11, borderRadius: '50%', backgroundColor: '#ef4444' }} />
                  <div style={{ width: 11, height: 11, borderRadius: '50%', backgroundColor: '#f59e0b' }} />
                  <div style={{ width: 11, height: 11, borderRadius: '50%', backgroundColor: '#10b981' }} />
                  <span style={{ fontSize: 13, fontWeight: 700, color: '#475569', marginLeft: 8 }}>
                    একুশে নেক্সট স্টুডিও ও ফ্লোটিং পপ-আপ (Light Mode)
                  </span>
                </div>
                <span
                  style={{
                    fontSize: 11,
                    padding: '3px 8px',
                    borderRadius: 999,
                    background: '#fef3c7',
                    color: '#b45309',
                    fontWeight: 700,
                  }}
                >
                  ☀️ Light Mode
                </span>
              </div>

              {/* Dynamic Feature Live Demo Canvas in Light Mode */}
              <div
                style={{
                  position: 'relative',
                  padding: 24,
                  background: '#ffffff',
                  minHeight: 280,
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'center',
                }}
              >
                <ScanlineBeam color="#10b981" speed={0.05} />

                {/* Feature 0: Word X-Ray */}
                {activeIdx === 0 && (
                  <div style={{ textAlign: 'center' }}>
                    <div style={{ fontSize: 15, fontWeight: 800, color: '#0e7452', marginBottom: 10 }}>
                      🔬 শব্দ এক্স-রে ডিকম্পোজিশন ও ভিজ্যুয়াল বিল্ডার:
                    </div>
                    <div
                      style={{
                        display: 'flex',
                        justifyContent: 'center',
                        gap: 12,
                        marginBottom: 16,
                      }}
                    >
                      {['স্বা', 'ধী', 'ন', 'তা'].map((ch, i) => (
                        <div
                          key={i}
                          style={{
                            padding: '12px 18px',
                            borderRadius: 12,
                            background: '#ecfdf5',
                            border: '1.5px solid #a7f3d0',
                            fontSize: 28,
                            fontWeight: 900,
                            color: '#065f46',
                            boxShadow: '0 4px 15px rgba(16, 185, 129, 0.15)',
                          }}
                        >
                          {ch}
                        </div>
                      ))}
                    </div>
                    <div
                      style={{
                        padding: '10px 16px',
                        borderRadius: 10,
                        background: '#f8fafc',
                        border: '1px solid #e2e8f0',
                        fontSize: 15,
                        color: '#334155',
                        fontWeight: 700,
                      }}
                    >
                      যুক্তবর্ণ বিশ্লেষণ: <b style={{ color: '#0e7452' }}>স্ব = স + ্ + ব</b> (স্বাভাবিক উচ্চারণ)
                    </div>
                  </div>
                )}

                {/* Feature 1: Inline Mix-English */}
                {activeIdx === 1 && (
                  <div>
                    <div style={{ fontSize: 15, fontWeight: 800, color: '#0e7452', marginBottom: 12 }}>
                      ⚡ ইনলাইন মিক্স-ইংলিশ ও ইনস্ট্যান্ট শব্দ ফ্লিপ:
                    </div>
                    <div
                      style={{
                        padding: '16px 20px',
                        borderRadius: 12,
                        background: '#f8fafc',
                        border: '1.5px solid #e2e8f0',
                        fontSize: 24,
                        color: '#0f172a',
                        fontWeight: 700,
                        marginBottom: 14,
                      }}
                    >
                      আগামীকাল সকালে <span style={{ color: '#0284c7', background: '#e0f2fe', padding: '2px 8px', borderRadius: 6 }}>meeting</span> এ দেখা হবে
                    </div>
                    <div style={{ display: 'flex', gap: 10 }}>
                      <span style={{ fontSize: 13, padding: '4px 10px', borderRadius: 6, background: '#ecfdf5', color: '#047857', fontWeight: 700 }}>
                        প্রো-টিপ: /meeting লিখলেই ইংরেজি!
                      </span>
                      <span style={{ fontSize: 13, padding: '4px 10px', borderRadius: 6, background: '#fef3c7', color: '#b45309', fontWeight: 700 }}>
                        Ctrl+Backspace ➔ শব্দ ফ্লিপ
                      </span>
                    </div>
                  </div>
                )}

                {/* Feature 2: Smart Numbers */}
                {activeIdx === 2 && (
                  <div style={{ textAlign: 'center' }}>
                    <div style={{ fontSize: 15, fontWeight: 800, color: '#0e7452', marginBottom: 12 }}>
                      🔢 স্বয়ংক্রিয় সংখ্যা মোড সিঙ্ক:
                    </div>
                    <div style={{ display: 'flex', justifyContent: 'center', gap: 16, marginBottom: 16 }}>
                      <div style={{ padding: '12px 20px', borderRadius: 12, background: '#f1f5f9', border: '1px solid #cbd5e1' }}>
                        <div style={{ fontSize: 12, color: '#64748b', fontWeight: 700 }}>EN Mode</div>
                        <div style={{ fontSize: 28, fontWeight: 900, color: '#0f172a' }}>2026</div>
                      </div>
                      <div style={{ display: 'flex', alignItems: 'center', fontSize: 24, color: '#0e7452' }}>➔</div>
                      <div style={{ padding: '12px 20px', borderRadius: 12, background: '#ecfdf5', border: '1.5px solid #10b981' }}>
                        <div style={{ fontSize: 12, color: '#047857', fontWeight: 800 }}>বাংলা মোড</div>
                        <div style={{ fontSize: 28, fontWeight: 900, color: '#0e7452' }}>২০২৬</div>
                      </div>
                    </div>
                    <div style={{ fontSize: 14, color: '#475569', fontWeight: 600 }}>
                      একুশে নেক্সট বাংলা মোডে ০-৯ সংখ্যা স্বয়ংক্রিয়ভাবে টাইপ হয়!
                    </div>
                  </div>
                )}

                {/* Feature 3: Quick Snippets */}
                {activeIdx === 3 && (
                  <div>
                    <div style={{ fontSize: 15, fontWeight: 800, color: '#0e7452', marginBottom: 10 }}>
                      💬 কুইক টেক্সট স্নিপেটস (Quick Expansion):
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 14px', borderRadius: 8, background: '#f8fafc', border: '1px solid #e2e8f0' }}>
                        <span style={{ fontWeight: 800, color: '#0284c7' }}>;salam</span>
                        <span style={{ fontWeight: 700, color: '#0f172a' }}>➔ আসসালামু আলাইকুম</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 14px', borderRadius: 8, background: '#f8fafc', border: '1px solid #e2e8f0' }}>
                        <span style={{ fontWeight: 800, color: '#0284c7' }}>;tk</span>
                        <span style={{ fontWeight: 700, color: '#0f172a' }}>➔ ৳ (টাকা প্রতীক)</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 14px', borderRadius: 8, background: '#f8fafc', border: '1px solid #e2e8f0' }}>
                        <span style={{ fontWeight: 800, color: '#0284c7' }}>;bd</span>
                        <span style={{ fontWeight: 700, color: '#0f172a' }}>➔ গণপ্রজাতন্ত্রী বাংলাদেশ</span>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
