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
import { ScanlineBeam } from '../../components/ScanlineBeam';
import { ShimmerSweep } from '../../components/ShimmerSweep';

const FEATURES = [
  {
    icon: '🔬',
    title: 'শব্দ এক্স-রে ও ২-ক্লিক যুক্তবর্ণ',
    desc: 'যেকোনো শব্দের বিশদ ডিকম্পোজিশন ও ২ ক্লিকেই যুক্তবর্ণ তৈরি!',
  },
  {
    icon: '⚡',
    title: 'ইনলাইন মিক্স-ইংলিশ ও শব্দ ফ্লিপ',
    desc: '/office লিখলেই ইংরেজি, আর Ctrl+Backspace চাপলেই শব্দ রূপান্তর!',
  },
  {
    icon: '🔢',
    title: 'স্বয়ংক্রিয় সংখ্যা সিঙ্ক (Smart Numbers)',
    desc: 'ইংরেজি মোডে 0-9 এবং একুশে নেক্সট বাংলা মোডে ০-৯ স্বয়ংক্রিয়!',
  },
  {
    icon: '💬',
    title: 'কুইক টেক্সট স্নিপেটস',
    desc: ';salam ➔ আসসালামু আলাইকুম, ;tk ➔ ৳, ;bd ➔ গণপ্রজাতন্ত্রী বাংলাদেশ।',
  },
];

// Continuous mathematical focus level for feature items
const getFeatureFocusVertical = (idx: number, currentF: number): number => {
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

export const Scene4ModernFeaturesVertical: React.FC = () => {
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

  const activeIdx = Math.min(3, Math.floor(f / 65));
  const floatY = Math.sin(f * 0.05) * 6;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: '#030712',
        backgroundImage:
          'radial-gradient(circle at 50% 50%, rgba(56, 189, 248, 0.18) 0%, rgba(6, 78, 59, 0.22) 50%, #020617 100%)',
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
          padding: '0 40px',
          zIndex: 10,
          transform: `scale(${cameraZoom})`,
        }}
      >
        {/* Header */}
        <div
          style={{
            transform: `scale(${titleIn}) translateY(${(1 - titleIn) * 20}px)`,
            opacity: titleIn,
            textAlign: 'center',
            marginBottom: 28,
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
              fontSize: 20,
              fontWeight: 800,
              marginBottom: 14,
              overflow: 'hidden',
            }}
          >
            <ShimmerSweep interval={65} duration={20} color="rgba(125, 211, 252, 0.4)" />
            <span>⚡ একুশে নেক্সট স্টুডিওর অনন্য সুবিধাসমূহ ⚡</span>
          </div>

          <h2
            style={{
              fontSize: 50,
              fontWeight: 900,
              margin: 0,
              lineHeight: 1.25,
              background: 'linear-gradient(135deg, #ffffff 40%, #a7f3d0 80%, #34d399 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
            }}
          >
            দৈনন্দিন বাংলা লেখাকে
            <br />
            আনন্দময় করার প্রযুক্তি
          </h2>
        </div>

        {/* 🌟 3D Perspective App Window in PURE LIGHT MODE 🌟 */}
        <div
          style={{
            transform: `scale(${screenshotIn}) translateY(${floatY}px)`,
            opacity: screenshotIn,
            width: '100%',
            maxWidth: 960,
            borderRadius: 24,
            overflow: 'hidden',
            background: '#ffffff',
            border: '2px solid rgba(226, 232, 240, 0.95)',
            boxShadow: '0 25px 60px rgba(0,0,0,0.65), 0 0 45px rgba(56, 189, 248, 0.25)',
            marginBottom: 26,
          }}
        >
          {/* Light Mode Header */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '12px 20px',
              background: '#f8fafc',
              borderBottom: '1px solid #e2e8f0',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#ef4444' }} />
              <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#f59e0b' }} />
              <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#10b981' }} />
              <span style={{ fontSize: 14, fontWeight: 800, color: '#334155', marginLeft: 8 }}>
                একুশে নেক্সট স্টুডিও ও ফ্লোটিং পপ-আপ
              </span>
            </div>
            <span
              style={{
                fontSize: 12,
                padding: '3px 10px',
                borderRadius: 999,
                background: '#fef3c7',
                color: '#b45309',
                fontWeight: 800,
              }}
            >
              ☀️ Light Mode
            </span>
          </div>

          {/* Dynamic Light Mode Feature Content */}
          <div
            style={{
              position: 'relative',
              padding: '28px 24px',
              background: '#ffffff',
              minHeight: 250,
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
            }}
          >
            <ScanlineBeam color="#10b981" speed={0.05} />

            {activeIdx === 0 && (
              <div style={{ textAlign: 'center' }}>
                <div style={{ fontSize: 17, fontWeight: 800, color: '#0e7452', marginBottom: 12 }}>
                  🔬 শব্দ এক্স-রে ও যুক্তবর্ণ বিশ্লিষ্টকরণ:
                </div>
                <div style={{ display: 'flex', justifyContent: 'center', gap: 10, marginBottom: 16 }}>
                  {['স্বা', 'ধী', 'ন', 'তা'].map((ch, i) => (
                    <div
                      key={i}
                      style={{
                        padding: '12px 18px',
                        borderRadius: 12,
                        background: '#ecfdf5',
                        border: '1.5px solid #a7f3d0',
                        fontSize: 32,
                        fontWeight: 900,
                        color: '#065f46',
                      }}
                    >
                      {ch}
                    </div>
                  ))}
                </div>
                <div style={{ fontSize: 16, color: '#334155', fontWeight: 700 }}>
                  বিশ্লেষণ: <b style={{ color: '#0e7452' }}>স্ব = স + ব</b> (২-ক্লিকেই যুক্তবর্ণ!)
                </div>
              </div>
            )}

            {activeIdx === 1 && (
              <div>
                <div style={{ fontSize: 17, fontWeight: 800, color: '#0e7452', marginBottom: 12 }}>
                  ⚡ ইনলাইন মিক্স-ইংলিশ:
                </div>
                <div
                  style={{
                    padding: '16px 20px',
                    borderRadius: 14,
                    background: '#f8fafc',
                    border: '1.5px solid #e2e8f0',
                    fontSize: 26,
                    color: '#0f172a',
                    fontWeight: 800,
                    marginBottom: 12,
                  }}
                >
                  আগামীকালের <span style={{ color: '#0284c7', background: '#e0f2fe', padding: '2px 8px', borderRadius: 6 }}>/office</span> মিটিং
                </div>
                <div style={{ fontSize: 15, color: '#475569', fontWeight: 700 }}>
                  কীবোর্ড না পাল্টেই ইংরেজি! শব্দ রূপান্তরে <b>Ctrl+Backspace</b>।
                </div>
              </div>
            )}

            {activeIdx === 2 && (
              <div style={{ textAlign: 'center' }}>
                <div style={{ fontSize: 17, fontWeight: 800, color: '#0e7452', marginBottom: 12 }}>
                  🔢 স্বয়ংক্রিয় সংখ্যা মোড সিঙ্ক:
                </div>
                <div style={{ display: 'flex', justifyContent: 'center', gap: 18, marginBottom: 16 }}>
                  <div style={{ padding: '12px 24px', borderRadius: 14, background: '#f1f5f9', border: '1px solid #cbd5e1' }}>
                    <div style={{ fontSize: 13, color: '#64748b', fontWeight: 800 }}>English Mode</div>
                    <div style={{ fontSize: 32, fontWeight: 900, color: '#0f172a' }}>2026</div>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', fontSize: 26, color: '#0e7452' }}>➔</div>
                  <div style={{ padding: '12px 24px', borderRadius: 14, background: '#ecfdf5', border: '1.5px solid #10b981' }}>
                    <div style={{ fontSize: 13, color: '#047857', fontWeight: 800 }}>বাংলা মোড</div>
                    <div style={{ fontSize: 32, fontWeight: 900, color: '#0e7452' }}>২০২৬</div>
                  </div>
                </div>
                <div style={{ fontSize: 16, color: '#475569', fontWeight: 600 }}>
                  বাংলা মোডে ০-৯ সংখ্যা স্বয়ংক্রিয়ভাবে টাইপ হয়!
                </div>
              </div>
            )}

            {activeIdx === 3 && (
              <div>
                <div style={{ fontSize: 17, fontWeight: 800, color: '#0e7452', marginBottom: 10 }}>
                  💬 কুইক টেক্সট স্নিপেটস:
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 14px', borderRadius: 10, background: '#f8fafc', border: '1px solid #e2e8f0' }}>
                    <span style={{ fontWeight: 800, color: '#0284c7', fontSize: 17 }}>;salam</span>
                    <span style={{ fontWeight: 700, color: '#0f172a', fontSize: 17 }}>➔ আসসালামু আলাইকুম</span>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 14px', borderRadius: 10, background: '#f8fafc', border: '1px solid #e2e8f0' }}>
                    <span style={{ fontWeight: 800, color: '#0284c7', fontSize: 17 }}>;tk</span>
                    <span style={{ fontWeight: 700, color: '#0f172a', fontSize: 17 }}>➔ ৳ (টাকা চিহ্ন)</span>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Feature Pills */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: 10,
            width: '100%',
            maxWidth: 960,
          }}
        >
          {FEATURES.map((feat, idx) => {
            const focusLevel = getFeatureFocusVertical(idx, f);

            return (
              <div
                key={feat.title}
                style={{
                  position: 'relative',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 14,
                  padding: '12px 18px',
                  borderRadius: 16,
                  background: 'rgba(15, 23, 42, 0.65)',
                  border: '1.5px solid rgba(255, 255, 255, 0.08)',
                  overflow: 'hidden',
                  boxShadow: `0 ${2 + focusLevel * 4}px ${10 + focusLevel * 15}px rgba(0,0,0,${0.3 + focusLevel * 0.2}), 0 0 ${focusLevel * 20}px rgba(16, 185, 129, 0.3)`,
                  transform: `scale(${1 + focusLevel * 0.015})`,
                }}
              >
                {/* Smooth Gradient Fill Overlay */}
                <div
                  style={{
                    position: 'absolute',
                    inset: 0,
                    borderRadius: 16,
                    background:
                      'linear-gradient(135deg, rgba(16, 185, 129, 0.3), rgba(15, 23, 42, 0.95))',
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
                    boxShadow: 'inset 0 0 10px rgba(52, 211, 153, 0.2)',
                    opacity: focusLevel,
                    pointerEvents: 'none',
                  }}
                />

                <span style={{ fontSize: 26, position: 'relative' }}>{feat.icon}</span>
                <div style={{ flex: 1, position: 'relative' }}>
                  <div
                    style={{
                      position: 'relative',
                      fontSize: 18,
                      fontWeight: 800,
                      color: '#ffffff',
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
                  <div style={{ fontSize: 14, color: '#94a3b8' }}>{feat.desc}</div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
};
