import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';

export const ScanlineBeam: React.FC<{
  color?: string;
  speed?: number;
}> = ({ color = '#38bdf8', speed = 0.05 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);
  const pos = ((Math.sin(f * speed) + 1) / 2) * 100; // 0 to 100%

  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        pointerEvents: 'none',
        overflow: 'hidden',
        borderRadius: 'inherit',
      }}
    >
      <div
        style={{
          position: 'absolute',
          left: 0,
          right: 0,
          top: `${pos}%`,
          height: 3,
          backgroundColor: color,
          boxShadow: `0 0 15px 3px ${color}, 0 0 30px 6px ${color}88`,
          opacity: 0.85,
        }}
      />
    </div>
  );
};
