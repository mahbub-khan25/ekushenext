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

export const Scene3IntroducingEkusheVertical: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);

  const cameraZoom = interpolate(f, [0, 298], [1, 1.04], {
    extrapolateRight: 'clamp',
  });

  const logoSpring = spring({
    frame: f - 8,
    fps: 30,
    config: { damping: 12, stiffness: 90 },
  });

  const uiSpring = spring({
    frame: f - 24,
    fps: 30,
    config: { damping: 14, stiffness: 75 },
  });

  const fadeOut = interpolate(f, [270, 298], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  // Typing state machine for "বল বীর \n বল উন্নত মম শির!"
  let currentKeys: string[] = [];
  let line1Text = '';
  let line2Text = '';
  let isLine2Active = false;
  let activeCandidate = '';
  let candidateHint = '';
  let xrayWord = '';
  let xrayBreakdown: string[] = [];
  let isPoemComplete = false;

  if (f < 24) {
    currentKeys = [];
  } else if (f < 38) {
    const step = Math.min(4, Math.floor((f - 24) / 3.5));
    currentKeys = ['b', 'o', 'l', 'o'].slice(0, step + 1);
    line1Text = step >= 3 ? 'বল' : step >= 2 ? 'বল্' : 'ব';
    activeCandidate = '1. বল';
    candidateHint = 'bolo ➔ বল';
  } else if (f < 54) {
    currentKeys = ['␣ Space'];
    line1Text = 'বল ';
    activeCandidate = '1. বল';
  } else if (f < 76) {
    const step = Math.min(3, Math.floor((f - 54) / 5.5));
    currentKeys = ['b', 'i', 'r'].slice(0, step + 1);
    line1Text = 'বল ' + (step >= 2 ? 'বীর' : step >= 1 ? 'বি' : 'ব');
    activeCandidate = '1. বীর';
    candidateHint = 'bir ➔ বীর';
  } else if (f < 92) {
    currentKeys = ['␣ Space'];
    line1Text = 'বল বীর';
    activeCandidate = '1. বীর';
  } else if (f < 108) {
    currentKeys = ['↵ Enter'];
    line1Text = 'বল বীর';
    isLine2Active = true;
  } else if (f < 132) {
    isLine2Active = true;
    line1Text = 'বল বীর';
    const step = Math.min(4, Math.floor((f - 108) / 4.8));
    currentKeys = ['b', 'o', 'l', 'o'].slice(0, step + 1);
    line2Text = step >= 3 ? 'বল' : step >= 2 ? 'বল্' : 'ব';
    activeCandidate = '1. বল';
    candidateHint = 'bolo ➔ বল';
  } else if (f < 146) {
    isLine2Active = true;
    line1Text = 'বল বীর';
    currentKeys = ['␣ Space'];
    line2Text = 'বল ';
  } else if (f < 192) {
    // unnto (উন্নত)
    isLine2Active = true;
    line1Text = 'বল বীর';
    const chars = ['u', 'n', 'n', 't', 'o'];
    const step = Math.min(5, Math.floor((f - 146) / 7));
    currentKeys = chars.slice(0, step + 1);
    line2Text =
      'বল ' +
      (step >= 4
        ? 'উন্নত'
        : step >= 3
        ? 'উন্নত্'
        : step >= 2
        ? 'উন্ন'
        : step >= 1
        ? 'উন'
        : 'উ');
    activeCandidate = '1. উন্নত';
    candidateHint = '⚡ unnto ➔ উন্নত (ন্ন = ন+্+ন)';
    xrayWord = 'উন্নত';
    xrayBreakdown = ['উ', 'ন্ন (ন+্+ন)', 'ত'];
  } else if (f < 206) {
    isLine2Active = true;
    line1Text = 'বল বীর';
    currentKeys = ['␣ Space'];
    line2Text = 'বল উন্নত ';
    xrayWord = 'উন্নত';
    xrayBreakdown = ['উ', 'ন্ন (ন+্+ন)', 'ত'];
  } else if (f < 228) {
    isLine2Active = true;
    line1Text = 'বল বীর';
    const chars = ['m', 'o', 'm', 'o'];
    const step = Math.min(4, Math.floor((f - 206) / 4.5));
    currentKeys = chars.slice(0, step + 1);
    line2Text = 'বল উন্নত ' + (step >= 3 ? 'মম' : 'ম');
    activeCandidate = '1. মম';
    candidateHint = 'momo ➔ মম';
    xrayWord = 'উন্নত';
    xrayBreakdown = ['উ', 'ন্ন (ন+্+ন)', 'ত'];
  } else if (f < 238) {
    isLine2Active = true;
    line1Text = 'বল বীর';
    currentKeys = ['␣ Space'];
    line2Text = 'বল উন্নত মম ';
  } else if (f < 262) {
    isLine2Active = true;
    line1Text = 'বল বীর';
    const chars = ['s', 'h', 'i', 'r', '!'];
    const step = Math.min(5, Math.floor((f - 238) / 4.0));
    currentKeys = chars.slice(0, step + 1);
    line2Text =
      'বল উন্নত মম ' +
      (step >= 4
        ? 'শির!'
        : step >= 3
        ? 'শির'
        : step >= 2
        ? 'শি'
        : 'শ');
    activeCandidate = '1. শির!';
    candidateHint = 'shir! ➔ শির!';
    xrayWord = 'উন্নত';
    xrayBreakdown = ['উ', 'ন্ন (ন+্+ন)', 'ত'];
  } else {
    isLine2Active = true;
    line1Text = 'বল বীর';
    line2Text = 'বল উন্নত মম শির!';
    isPoemComplete = true;
    currentKeys = ['🎉 সম্পূর্ণ'];
    xrayWord = 'উন্নত';
    xrayBreakdown = ['উ', 'ন্ন (ন+্+ন)', 'ত'];
  }

  const logoFloatY = Math.sin(f * 0.08) * 5;
  const windowFloatY = Math.sin(f * 0.04) * 5;
  const cursorBlink = Math.sin(f * 0.4) > 0;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: '#020617',
        backgroundImage:
          'radial-gradient(circle at 50% 30%, rgba(16, 185, 129, 0.25) 0%, rgba(15, 23, 42, 0.88) 60%, #030712 100%)',
        color: '#ffffff',
        fontFamily: "'Kalpurush', 'HindSiliguri', sans-serif",
        opacity: fadeOut,
        overflow: 'hidden',
      }}
    >
      <BackgroundParticles theme="emerald" />

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
        {/* Brand Header */}
        <div
          style={{
            transform: `scale(${logoSpring}) translateY(${logoFloatY}px)`,
            opacity: logoSpring,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            marginBottom: 20,
            textAlign: 'center',
          }}
        >
          <div
            style={{
              position: 'relative',
              width: 90,
              height: 90,
              borderRadius: 24,
              padding: 6,
              background:
                'linear-gradient(135deg, rgba(16, 185, 129, 0.45), rgba(5, 150, 105, 0.15))',
              border: '2px solid rgba(52, 211, 153, 0.7)',
              boxShadow: '0 0 35px rgba(16, 185, 129, 0.5)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              marginBottom: 12,
            }}
          >
            <GlowRings color="#10b981" size={130} ringCount={2} />
            <Img
              src={staticFile('assets/logo_256.png')}
              style={{
                width: '100%',
                height: '100%',
                objectFit: 'contain',
              }}
            />
          </div>

          <h1
            style={{
              fontSize: 54,
              fontWeight: 900,
              margin: '0 0 6px 0',
              lineHeight: 1.15,
              background: 'linear-gradient(135deg, #ffffff 40%, #6ee7b7 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
            }}
          >
            একুশে নেক্সট · Ekushe Next
          </h1>
          <p style={{ fontSize: 22, color: '#a7f3d0', margin: 0, fontWeight: 700 }}>
            স্মার্ট বাংলা রাইটিং স্টুডিও (Light Mode)
          </p>
        </div>

        {/* Live Keystroke Monitor HUD */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 10,
            marginBottom: 16,
            padding: '8px 22px',
            borderRadius: 16,
            background: 'rgba(15, 23, 42, 0.8)',
            border: '1.5px solid rgba(52, 211, 153, 0.45)',
          }}
        >
          <span style={{ fontSize: 16, color: '#6ee7b7', fontWeight: 800 }}>
            ⌨️ Live Keys:
          </span>
          <div style={{ display: 'flex', gap: 6 }}>
            {currentKeys.length === 0 ? (
              <span style={{ fontSize: 16, color: '#64748b' }}>কীবোর্ড প্রস্তুত...</span>
            ) : (
              currentKeys.map((k, i) => (
                <span
                  key={i}
                  style={{
                    padding: '4px 10px',
                    borderRadius: 8,
                    background: '#1e293b',
                    border: '1px solid #34d399',
                    color: '#ffffff',
                    fontSize: 16,
                    fontWeight: 800,
                  }}
                >
                  {k}
                </span>
              ))
            )}
          </div>
        </div>

        {/* 🌟 Authentic Studio Window in LIGHT MODE 🌟 */}
        <div
          style={{
            transform: `scale(${uiSpring}) translateY(${(1 - uiSpring) * 25 + windowFloatY}px)`,
            opacity: uiSpring,
            width: '100%',
            maxWidth: 960,
            borderRadius: 26,
            background: '#ffffff',
            border: '2px solid rgba(226, 232, 240, 0.95)',
            boxShadow:
              '0 30px 70px rgba(0,0,0,0.65), 0 0 50px rgba(16, 185, 129, 0.3)',
            overflow: 'hidden',
          }}
        >
          {/* Light Mode Window Header */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '14px 22px',
              background: '#f8fafc',
              borderBottom: '1px solid #e2e8f0',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div style={{ width: 14, height: 14, borderRadius: '50%', backgroundColor: '#ef4444' }} />
              <div style={{ width: 14, height: 14, borderRadius: '50%', backgroundColor: '#f59e0b' }} />
              <div style={{ width: 14, height: 14, borderRadius: '50%', backgroundColor: '#10b981' }} />
              <span
                style={{
                  fontSize: 15,
                  fontWeight: 800,
                  color: '#334155',
                  marginLeft: 10,
                }}
              >
                একুশে নেক্সট স্টুডিও
              </span>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span
                style={{
                  fontSize: 13,
                  padding: '4px 10px',
                  borderRadius: 999,
                  background: '#fef3c7',
                  color: '#b45309',
                  fontWeight: 700,
                }}
              >
                ☀️ Light Mode
              </span>
              <span
                style={{
                  fontSize: 13,
                  padding: '4px 14px',
                  borderRadius: 999,
                  background: '#0e7452',
                  color: '#ffffff',
                  fontWeight: 800,
                }}
              >
                বাং · বাংলা
              </span>
            </div>
          </div>

          {/* Typing Canvas */}
          <div
            style={{
              padding: '30px 28px',
              background: '#ffffff',
              minHeight: 330,
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
              position: 'relative',
            }}
          >
            <div>
              {/* Line 1 */}
              <div
                style={{
                  fontSize: 52,
                  fontWeight: 900,
                  color: '#0f172a',
                  lineHeight: 1.35,
                  minHeight: 70,
                  display: 'flex',
                  alignItems: 'center',
                }}
              >
                <span>{line1Text}</span>
                {!isLine2Active && (
                  <span
                    style={{
                      display: 'inline-block',
                      width: 5,
                      height: 52,
                      backgroundColor: '#0e7452',
                      marginLeft: 6,
                      opacity: cursorBlink ? 1 : 0,
                    }}
                  />
                )}
              </div>

              {/* Line 2 */}
              <div
                style={{
                  fontSize: 52,
                  fontWeight: 900,
                  color: isLine2Active ? '#0e7452' : '#94a3b8',
                  lineHeight: 1.35,
                  minHeight: 70,
                  display: 'flex',
                  alignItems: 'center',
                }}
              >
                <span>{line2Text}</span>
                {isLine2Active && (
                  <span
                    style={{
                      display: 'inline-block',
                      width: 5,
                      height: 52,
                      backgroundColor: '#0e7452',
                      marginLeft: 6,
                      opacity: cursorBlink ? 1 : 0,
                    }}
                  />
                )}
              </div>

              {/* Poet Attribution */}
              {f >= 255 && (
                <div
                  style={{
                    marginTop: 14,
                    fontSize: 24,
                    fontWeight: 800,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 10,
                  }}
                >
                  <span style={{ fontSize: 26 }}>🖋️</span>
                  <span style={{ color: '#0f172a' }}>— কাজী নজরুল ইসলাম</span>
                  <span
                    style={{
                      fontSize: 16,
                      padding: '3px 10px',
                      borderRadius: 999,
                      background: '#fee2e2',
                      color: '#b91c1c',
                      fontWeight: 800,
                    }}
                  >
                    বিদ্রোহী
                  </span>
                </div>
              )}

              {/* Candidate Bar in Light Mode */}
              {activeCandidate && !isPoemComplete && (
                <div
                  style={{
                    marginTop: 18,
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: 10,
                    padding: '8px 16px',
                    borderRadius: 14,
                    background: '#ffffff',
                    border: '2px solid #10b981',
                    boxShadow: '0 12px 30px rgba(0,0,0,0.18)',
                  }}
                >
                  <span
                    style={{
                      padding: '4px 14px',
                      borderRadius: 10,
                      background: '#0e7452',
                      color: '#ffffff',
                      fontWeight: 800,
                      fontSize: 20,
                    }}
                  >
                    {activeCandidate}
                  </span>
                  {candidateHint && (
                    <span
                      style={{
                        fontSize: 16,
                        padding: '4px 10px',
                        borderRadius: 8,
                        background: '#ecfdf5',
                        color: '#047857',
                        fontWeight: 800,
                      }}
                    >
                      {candidateHint}
                    </span>
                  )}
                </div>
              )}
            </div>

            {/* Word X-Ray Card */}
            <div
              style={{
                marginTop: 24,
                padding: '16px 20px',
                borderRadius: 16,
                background: '#f8fafc',
                border: '1.5px solid #e2e8f0',
              }}
            >
              <div style={{ fontSize: 17, fontWeight: 800, color: '#0f172a', marginBottom: 4 }}>
                🔬 শব্দ এক্স-রে (Word X-Ray):
              </div>
              <div style={{ fontSize: 16, color: '#475569' }}>
                {xrayWord ? (
                  <span>
                    <b style={{ color: '#0e7452' }}>"{xrayWord}"</b> গঠিত:{' '}
                    {xrayBreakdown.join(' + ')}
                  </span>
                ) : (
                  <span>ছোট হাতের অক্ষরে টাইপ করলেই নিখুঁত যুক্তবর্ণ গঠিত হয়</span>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Mobile Benefit Card below the Window */}
        <div
          style={{
            marginTop: 24,
            width: '100%',
            maxWidth: 960,
            padding: '16px 24px',
            borderRadius: 20,
            background: 'rgba(15, 23, 42, 0.78)',
            border: '1.5px solid rgba(52, 211, 153, 0.45)',
            boxShadow: '0 10px 30px rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            gap: 16,
          }}
        >
          <span style={{ fontSize: 32 }}>⚡</span>
          <div style={{ textAlign: 'left' }}>
            <div style={{ fontSize: 20, fontWeight: 800, color: '#6ee7b7' }}>
              Zero-Case Phonetic™ — Shift ছাড়াই সহজ টাইপিং
            </div>
            <div style={{ fontSize: 16, color: '#cbd5e1' }}>
              ছোট হাতের অক্ষরেই unnto ➔ উন্নত, srishti ➔ সৃষ্টি!
            </div>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
