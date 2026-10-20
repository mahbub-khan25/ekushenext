import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';

export const ShimmerSweep: React.FC<{
  interval?: number;
  duration?: number;
  color?: string;
  angle?: number;
}> = ({
  interval = 75,
  duration = 25,
  color = 'rgba(255, 255, 255, 0.28)',
  angle = 115,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);
  const cycle = f % interval;

  if (cycle > duration) {
    return null;
  }

  const progress = cycle / duration; // 0 to 1
  const leftPercent = -40 + progress * 180; // sweeps from -40% to 140%

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
          top: '-50%',
          bottom: '-50%',
          left: `${leftPercent}%`,
          width: '35%',
          transform: `rotate(${angle}deg)`,
          background: `linear-gradient(90deg, transparent 0%, ${color} 50%, transparent 100%)`,
          filter: 'blur(4px)',
        }}
      />
    </div>
  );
};
