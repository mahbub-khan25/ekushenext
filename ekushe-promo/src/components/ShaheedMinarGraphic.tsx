import React from 'react';
import { interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';

export const ShaheedMinarGraphic: React.FC<{
  scale?: number;
  glowIntensity?: number;
  sunPulse?: boolean;
}> = ({ scale = 1, glowIntensity = 1, sunPulse = true }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);

  const entrance = spring({
    frame,
    fps,
    config: { damping: 14, stiffness: 70 },
  });

  const pulse = sunPulse ? 1 + Math.sin(f * 0.06) * 0.05 : 1;
  const sunGlow = 40 * glowIntensity * pulse;

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        transform: `scale(${scale * entrance})`,
        opacity: entrance,
      }}
    >
      <svg
        width="460"
        height="380"
        viewBox="0 0 460 380"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        style={{
          filter: `drop-shadow(0 0 ${20 * glowIntensity}px rgba(225, 29, 72, 0.4))`,
        }}
      >
        <defs>
          <radialGradient
            id="sunGrad"
            cx="50%"
            cy="50%"
            r="50%"
            fx="50%"
            fy="50%"
          >
            <stop offset="0%" stopColor="#ff4d6d" />
            <stop offset="70%" stopColor="#e11d48" />
            <stop offset="100%" stopColor="#9f1239" />
          </radialGradient>

          <linearGradient id="pillarGrad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#ffffff" />
            <stop offset="40%" stopColor="#e2e8f0" />
            <stop offset="100%" stopColor="#94a3b8" />
          </linearGradient>

          <linearGradient id="redBaseGrad" x1="0" y1="0" x2="1" y2="0">
            <stop offset="0%" stopColor="#991b1b" />
            <stop offset="50%" stopColor="#dc2626" />
            <stop offset="100%" stopColor="#991b1b" />
          </linearGradient>

          <filter id="sunBlur" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur stdDeviation="8" result="blur" />
            <feComposite in="SourceGraphic" in2="blur" operator="over" />
          </filter>
        </defs>

        {/* Radiant Solar Corona Wave Rings */}
        {[1, 2, 3].map((ringIdx) => {
          const wave = ((f * 1.2 + ringIdx * 30) % 90) / 90;
          const r = 78 + wave * 95;
          const op = (1 - wave) * 0.45 * glowIntensity;
          return (
            <circle
              key={ringIdx}
              cx="230"
              cy="175"
              r={r}
              stroke="#fb7185"
              strokeWidth="2"
              fill="none"
              opacity={op}
              style={{ filter: 'drop-shadow(0 0 10px #e11d48)' }}
            />
          );
        })}

        {/* The Sacred Rising Red Sun behind the Minar */}
        <circle
          cx="230"
          cy="175"
          r={78 * pulse}
          fill="url(#sunGrad)"
          filter="url(#sunBlur)"
          style={{
            filter: `drop-shadow(0 0 ${sunGlow}px rgba(244, 63, 94, 0.85))`,
          }}
        />

        {/* Outer Far Columns (Left and Right) */}
        <rect
          x="92"
          y="200"
          width="18"
          height="125"
          rx="3"
          fill="url(#pillarGrad)"
          opacity="0.9"
        />
        <rect
          x="350"
          y="200"
          width="18"
          height="125"
          rx="3"
          fill="url(#pillarGrad)"
          opacity="0.9"
        />

        {/* Intermediate Columns (Left and Right) */}
        <rect
          x="126"
          y="150"
          width="20"
          height="175"
          rx="3"
          fill="url(#pillarGrad)"
        />
        <rect
          x="314"
          y="150"
          width="20"
          height="175"
          rx="3"
          fill="url(#pillarGrad)"
        />

        {/* Center Main Arch Pillars */}
        <rect
          x="166"
          y="95"
          width="26"
          height="230"
          rx="4"
          fill="url(#pillarGrad)"
        />
        <rect
          x="268"
          y="95"
          width="26"
          height="230"
          rx="4"
          fill="url(#pillarGrad)"
        />

        {/* Slanted Connecting Header Beams */}
        <path
          d="M 126 150 L 166 120 L 166 136 L 126 166 Z"
          fill="url(#pillarGrad)"
          opacity="0.95"
        />
        <path
          d="M 334 150 L 294 120 L 294 136 L 334 166 Z"
          fill="url(#pillarGrad)"
          opacity="0.95"
        />
        <path
          d="M 166 95 L 230 45 L 294 95 L 278 106 L 230 68 L 182 106 Z"
          fill="url(#pillarGrad)"
        />

        {/* Modern Stepped Pedestal Base */}
        <rect
          x="65"
          y="325"
          width="330"
          height="18"
          rx="3"
          fill="url(#redBaseGrad)"
          stroke="#ef4444"
          strokeWidth="1.5"
        />
        <rect
          x="45"
          y="343"
          width="370"
          height="14"
          rx="2"
          fill="#1e293b"
          stroke="#475569"
          strokeWidth="1"
        />
        <rect
          x="25"
          y="357"
          width="410"
          height="12"
          rx="2"
          fill="#0f172a"
        />
      </svg>
    </div>
  );
};
