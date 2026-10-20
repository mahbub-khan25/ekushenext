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

export const Scene3IntroducingEkushe: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps); // normalized 30fps-scale time for 60fps buttery smoothness

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
  let secondaryCandidates: string[] = [];
  let candidateHint = '';
  let xrayWord = '';
  let xrayBreakdown: string[] = [];
  let isPoemComplete = false;

  if (f < 24) {
    currentKeys = [];
  } else if (f < 38) {
    const step = Math.min(4, Math.floor((f - 24) / 3.5));
    currentKeys = ['b', 'o', 'l', 'o'].slice(0, step + 1);
    line1Text = step >= 3 ? 'বল' : step >= 2 ? 'বল্' : step >= 1 ? 'ব' : 'ব';
    activeCandidate = '1. বল';
    secondaryCandidates = ['2. বলো', '3. বলও'];
    candidateHint = 'bolo ➔ বল';
  } else if (f < 54) {
    currentKeys = ['␣ Space'];
    line1Text = 'বল ';
    activeCandidate = '1. বল';
    secondaryCandidates = [];
  } else if (f < 76) {
    const step = Math.min(3, Math.floor((f - 54) / 5.5));
    currentKeys = ['b', 'i', 'r'].slice(0, step + 1);
    line1Text = 'বল ' + (step >= 2 ? 'বীর' : step >= 1 ? 'বি' : 'ব');
    activeCandidate = '1. বীর';
    secondaryCandidates = ['2. বির', '3. বীর্য'];
    candidateHint = 'bir ➔ বীর (Zero-Case i=ী)';
  } else if (f < 92) {
    currentKeys = ['␣ Space'];
    line1Text = 'বল বীর';
    activeCandidate = '1. বীর';
    secondaryCandidates = [];
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
    secondaryCandidates = ['2. বলো'];
    candidateHint = 'bolo ➔ বল';
  } else if (f < 146) {
    isLine2Active = true;
    line1Text = 'বল বীর';
    currentKeys = ['␣ Space'];
    line2Text = 'বল ';
  } else if (f < 192) {
    // typing unnto (উন্নত)
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
    secondaryCandidates = ['2. উন্নতো', '3. উন্নতি'];
    candidateHint = '⚡ unnto ➔ উন্নত (ন্ন = ন + ্ন + ন)';
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
    // typing momo (মম)
    isLine2Active = true;
    line1Text = 'বল বীর';
    const chars = ['m', 'o', 'm', 'o'];
    const step = Math.min(4, Math.floor((f - 206) / 4.5));
    currentKeys = chars.slice(0, step + 1);
    line2Text = 'বল উন্নত ' + (step >= 3 ? 'মম' : 'ম');
    activeCandidate = '1. মম';
    secondaryCandidates = ['2. মমো'];
    candidateHint = 'momo ➔ মম';
    xrayWord = 'উন্নত';
    xrayBreakdown = ['উ', 'ন্ন (ন+্+ন)', 'ত'];
  } else if (f < 238) {
    isLine2Active = true;
    line1Text = 'বল বীর';
    currentKeys = ['␣ Space'];
    line2Text = 'বল উন্নত মম ';
  } else if (f < 262) {
    // typing shir! (শির!)
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
    secondaryCandidates = ['2. শির'];
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
          'radial-gradient(circle at 50% 30%, rgba(16, 185, 129, 0.22) 0%, rgba(15, 23, 42, 0.85) 60%, #030712 100%)',
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
          padding: '0 60px',
          zIndex: 10,
          transform: `scale(${cameraZoom})`,
        }}
      >
        {/* Brand Header with Ekushe Next Logo */}
        <div
          style={{
            transform: `scale(${logoSpring}) translateY(${logoFloatY}px)`,
            opacity: logoSpring,
            display: 'flex',
            alignItems: 'center',
            gap: 18,
            marginBottom: 12,
          }}
        >
          <div
            style={{
              position: 'relative',
              width: 76,
              height: 76,
              borderRadius: 20,
              padding: 5,
              background:
                'linear-gradient(135deg, rgba(16, 185, 129, 0.45), rgba(5, 150, 105, 0.15))',
              border: '2px solid rgba(52, 211, 153, 0.7)',
              boxShadow: '0 0 30px rgba(16, 185, 129, 0.5)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <GlowRings color="#10b981" size={110} ringCount={2} />
            <Img
              src={staticFile('assets/logo_256.png')}
              style={{
                width: '100%',
                height: '100%',
                objectFit: 'contain',
                filter: 'drop-shadow(0 4px 10px rgba(0,0,0,0.5))',
              }}
            />
          </div>

          <div style={{ textAlign: 'left' }}>
            <h1
              style={{
                fontSize: 50,
                fontWeight: 900,
                margin: 0,
                lineHeight: 1.1,
                background:
                  'linear-gradient(135deg, #ffffff 40%, #6ee7b7 100%)',
                WebkitBackgroundClip: 'text',
                WebkitTextFillColor: 'transparent',
                letterSpacing: 1,
              }}
            >
              একুশে নেক্সট <span style={{ color: '#34d399', fontSize: 34 }}>· Ekushe Next</span>
            </h1>
            <p
              style={{
                fontSize: 18,
                color: '#94a3b8',
                margin: 0,
                fontWeight: 600,
              }}
            >
              The Next-Gen Bengali Writing Studio for Windows (Light Mode)
            </p>
          </div>
        </div>

        {/* Live Keystroke Monitor HUD */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 10,
            marginBottom: 12,
            padding: '6px 18px',
            borderRadius: 14,
            background: 'rgba(15, 23, 42, 0.75)',
            border: '1.5px solid rgba(52, 211, 153, 0.35)',
            boxShadow: '0 4px 20px rgba(0,0,0,0.4)',
          }}
        >
          <span style={{ fontSize: 14, color: '#6ee7b7', fontWeight: 700 }}>
            ⌨️ Live Keystrokes:
          </span>
          <div style={{ display: 'flex', gap: 6 }}>
            {currentKeys.length === 0 ? (
              <span style={{ fontSize: 13, color: '#64748b' }}>কীবোর্ড প্রস্তুত...</span>
            ) : (
              currentKeys.map((k, i) => (
                <span
                  key={i}
                  style={{
                    padding: '3px 8px',
                    borderRadius: 6,
                    background: 'linear-gradient(180deg, #1e293b, #0f172a)',
                    border: '1px solid #34d399',
                    color: '#ffffff',
                    fontSize: 14,
                    fontWeight: 800,
                    boxShadow: '0 0 10px rgba(52, 211, 153, 0.4)',
                  }}
                >
                  {k}
                </span>
              ))
            )}
          </div>
        </div>

        {/* 🌟 Authentic Ekushe Next Studio Window in PURE LIGHT MODE 🌟 */}
        <div
          style={{
            transform: `scale(${uiSpring}) translateY(${(1 - uiSpring) * 25 + windowFloatY}px)`,
            opacity: uiSpring,
            width: '100%',
            maxWidth: 1080,
            borderRadius: 22,
            background: '#ffffff',
            border: '1.5px solid rgba(226, 232, 240, 0.9)',
            boxShadow:
              '0 25px 60px rgba(0,0,0,0.55), 0 0 45px rgba(16, 185, 129, 0.25)',
            overflow: 'hidden',
          }}
        >
          {/* Light Mode Window Title Bar */}
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
              <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#ef4444' }} />
              <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#f59e0b' }} />
              <div style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#10b981' }} />
              <span
                style={{
                  fontSize: 13,
                  fontWeight: 700,
                  color: '#475569',
                  marginLeft: 10,
                }}
              >
                একুশে নেক্সট স্টুডিও — Ekushe Next Modern Bangla Writing Suite (Light Mode)
              </span>
            </div>

            {/* Right Status Toolbar Pills */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span
                style={{
                  fontSize: 12,
                  padding: '3px 10px',
                  borderRadius: 999,
                  background: '#fef3c7',
                  color: '#b45309',
                  fontWeight: 700,
                  border: '1px solid #fde68a',
                }}
              >
                ☀️ Light Mode
              </span>
              <span
                style={{
                  fontSize: 12,
                  padding: '3px 10px',
                  borderRadius: 999,
                  background: '#f1f5f9',
                  color: '#475569',
                  fontWeight: 700,
                  border: '1px solid #e2e8f0',
                }}
              >
                কালপুরুষ ফন্ট
              </span>
              <span
                style={{
                  fontSize: 12,
                  padding: '3px 12px',
                  borderRadius: 999,
                  background: '#0e7452',
                  color: '#ffffff',
                  fontWeight: 800,
                  boxShadow: '0 0 10px rgba(14, 116, 82, 0.4)',
                }}
              >
                বাং · বাংলা সক্রিয়
              </span>
            </div>
          </div>

          {/* Light Mode Studio Tab Bar */}
          <div
            style={{
              display: 'flex',
              padding: '0 16px',
              background: '#f1f5f9',
              borderBottom: '1px solid #e2e8f0',
              gap: 8,
            }}
          >
            <div
              style={{
                padding: '9px 18px',
                background: '#ffffff',
                borderTopLeftRadius: 10,
                borderTopRightRadius: 10,
                borderTop: '3px solid #0e7452',
                color: '#0e7452',
                fontWeight: 800,
                fontSize: 14,
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              ⌨️ স্মার্ট বাংলা রাইটিং প্যাড
            </div>
            <div
              style={{
                padding: '9px 14px',
                color: '#64748b',
                fontWeight: 600,
                fontSize: 13,
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              🔬 শব্দ এক্স-রে ও যুক্তবর্ণ নির্মাতা
            </div>
            <div
              style={{
                padding: '9px 14px',
                color: '#64748b',
                fontWeight: 600,
                fontSize: 13,
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              🔤 যুক্তবর্ণ সহায়িকা
            </div>
            <div
              style={{
                padding: '9px 14px',
                color: '#64748b',
                fontWeight: 600,
                fontSize: 13,
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              ✅ বানান পরীক্ষক
            </div>
          </div>

          {/* Writing Canvas & Live Demonstration */}
          <div
            style={{
              padding: '24px 32px',
              background: '#ffffff',
              minHeight: 270,
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
              position: 'relative',
            }}
          >
            {/* The Poetry Typing Area */}
            <div style={{ position: 'relative' }}>
              {/* Line 1 */}
              <div
                style={{
                  fontSize: 42,
                  fontWeight: 900,
                  color: '#0f172a',
                  lineHeight: 1.35,
                  minHeight: 56,
                  display: 'flex',
                  alignItems: 'center',
                }}
              >
                <span>{line1Text}</span>
                {!isLine2Active && (
                  <span
                    style={{
                      display: 'inline-block',
                      width: 4,
                      height: 42,
                      backgroundColor: '#0e7452',
                      marginLeft: 4,
                      opacity: cursorBlink ? 1 : 0,
                    }}
                  />
                )}
              </div>

              {/* Line 2 */}
              <div
                style={{
                  fontSize: 42,
                  fontWeight: 900,
                  color: isLine2Active ? '#0e7452' : '#94a3b8',
                  lineHeight: 1.35,
                  minHeight: 56,
                  display: 'flex',
                  alignItems: 'center',
                }}
              >
                <span>{line2Text}</span>
                {isLine2Active && (
                  <span
                    style={{
                      display: 'inline-block',
                      width: 4,
                      height: 42,
                      backgroundColor: '#0e7452',
                      marginLeft: 4,
                      opacity: cursorBlink ? 1 : 0,
                    }}
                  />
                )}
              </div>

              {/* Poet Attribution when completed or nearing complete */}
              {f >= 255 && (
                <div
                  style={{
                    marginTop: 12,
                    fontSize: 22,
                    fontWeight: 800,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 10,
                  }}
                >
                  <span style={{ fontSize: 24 }}>🖋️</span>
                  <span style={{ color: '#0f172a' }}>— কাজী নজরুল ইসলাম</span>
                  <span
                    style={{
                      fontSize: 14,
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

              {/* 🌟 Live Floating Candidate Bar in LIGHT MODE 🌟 */}
              {activeCandidate && !isPoemComplete && (
                <div
                  style={{
                    position: 'absolute',
                    left: isLine2Active ? 180 : 160,
                    top: isLine2Active ? 100 : 48,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 10,
                    padding: '6px 14px',
                    borderRadius: 12,
                    background: '#ffffff',
                    border: '2px solid #10b981',
                    boxShadow: '0 12px 30px rgba(0,0,0,0.18)',
                    zIndex: 20,
                  }}
                >
                  <span
                    style={{
                      padding: '4px 12px',
                      borderRadius: 8,
                      background: '#0e7452',
                      color: '#ffffff',
                      fontWeight: 800,
                      fontSize: 17,
                    }}
                  >
                    {activeCandidate}
                  </span>
                  {secondaryCandidates.map((c, i) => (
                    <span
                      key={i}
                      style={{
                        fontSize: 15,
                        color: '#64748b',
                        fontWeight: 600,
                      }}
                    >
                      {c}
                    </span>
                  ))}
                  {candidateHint && (
                    <span
                      style={{
                        marginLeft: 6,
                        fontSize: 13,
                        padding: '2px 8px',
                        borderRadius: 6,
                        background: '#ecfdf5',
                        color: '#047857',
                        fontWeight: 700,
                        border: '1px solid #a7f3d0',
                      }}
                    >
                      {candidateHint}
                    </span>
                  )}
                </div>
              )}
            </div>

            {/* Word X-Ray & Zero-Case Conjunct Proof Banner */}
            <div
              style={{
                marginTop: 16,
                padding: '12px 20px',
                borderRadius: 14,
                background: '#f8fafc',
                border: '1.5px solid #e2e8f0',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                <span style={{ fontSize: 20 }}>🔬</span>
                <div>
                  <div style={{ fontSize: 14, fontWeight: 800, color: '#0f172a' }}>
                    শব্দ এক্স-রে (Word X-Ray) ও জিরো-কেস ফোনেটিক:
                  </div>
                  <div style={{ fontSize: 13, color: '#64748b' }}>
                    {xrayWord ? (
                      <span>
                        <b style={{ color: '#0e7452' }}>"{xrayWord}"</b> গঠিত:{' '}
                        {xrayBreakdown.join(' + ')}
                      </span>
                    ) : (
                      <span>ছোট হাতের অক্ষরে টাইপ করলেই জটিল যুক্তবর্ণ স্বয়ংক্রিয়ভাবে গঠিত হয়</span>
                    )}
                  </div>
                </div>
              </div>

              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  padding: '4px 12px',
                  borderRadius: 999,
                  background: '#ecfdf5',
                  border: '1px solid #a7f3d0',
                  color: '#047857',
                  fontSize: 13,
                  fontWeight: 700,
                }}
              >
                <span>⚡</span>
                <span>Zero-Case Phonetic™</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
