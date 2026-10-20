import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';

interface Particle {
  id: number;
  x: number;
  y: number;
  size: number;
  speed: number;
  color: string;
  opacity: number;
}

export const BackgroundParticles: React.FC<{ theme?: 'crimson' | 'emerald' | 'mixed' }> = ({
  theme = 'mixed',
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const f = frame * (30 / fps);

  // Deterministic particles
  const particles: Particle[] = React.useMemo(() => {
    const list: Particle[] = [];
    const colors =
      theme === 'crimson'
        ? ['#e11d48', '#f43f5e', '#fb7185', '#fda4af', '#f59e0b']
        : theme === 'emerald'
        ? ['#10b981', '#059669', '#34d399', '#6ee7b7', '#38bdf8']
        : ['#e11d48', '#10b981', '#f59e0b', '#38bdf8', '#fbbf24'];

    for (let i = 0; i < 40; i++) {
      list.push({
        id: i,
        x: (i * 47) % 1920,
        y: (i * 83) % 1080,
        size: 3 + ((i * 13) % 12),
        speed: 0.4 + ((i * 7) % 10) * 0.15,
        color: colors[i % colors.length],
        opacity: 0.15 + ((i * 17) % 45) / 100,
      });
    }
    return list;
  }, [theme]);

  const totalH = 1250;

  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        overflow: 'hidden',
        pointerEvents: 'none',
      }}
    >
      {particles.map((p) => {
        // Continuous smooth movement without modulo pop
        const rawY = (p.y - f * p.speed * 1.5) % totalH;
        const finalY = (rawY < 0 ? rawY + totalH : rawY) - 50;
        const waveX = Math.sin((f + p.id * 20) * 0.03) * 25;

        return (
          <div
            key={p.id}
            style={{
              position: 'absolute',
              left: p.x + waveX,
              top: finalY,
              width: p.size,
              height: p.size,
              borderRadius: '50%',
              backgroundColor: p.color,
              opacity: p.opacity,
              filter: `blur(${p.size > 8 ? 3 : 1}px)`,
              boxShadow: `0 0 ${p.size * 2}px ${p.color}`,
            }}
          />
        );
      })}
    </div>
  );
};
