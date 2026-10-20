import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';

export const GlowRings: React.FC<{
  color?: string;
  size?: number;
  ringCount?: number;
}> = ({ color = '#e11d48', size = 200, ringCount = 3 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);

  return (
    <div
      style={{
        position: 'absolute',
        width: size,
        height: size,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        pointerEvents: 'none',
      }}
    >
      {Array.from({ length: ringCount }).map((_, i) => {
        const cycleSpeed = 60;
        const offset = (f + i * (cycleSpeed / ringCount)) % cycleSpeed;
        const progress = offset / cycleSpeed; // 0 to 1
        const scale = 0.9 + progress * 0.9;
        const opacity = (1 - progress) * 0.7;

        return (
          <div
            key={i}
            style={{
              position: 'absolute',
              width: size,
              height: size,
              borderRadius: '50%',
              border: `2px solid ${color}`,
              transform: `scale(${scale})`,
              opacity,
              boxShadow: `0 0 25px ${color}`,
            }}
          />
        );
      })}
    </div>
  );
};
