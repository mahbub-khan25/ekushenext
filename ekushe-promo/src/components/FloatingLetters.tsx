import React from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';

const GLOWING_CHARS = [
  { char: 'অ', x: 120, y: 180, vx: 80, vy: 260, size: 64, delay: 0, color: '#f43f5e' },
  { char: 'আ', x: 1720, y: 220, vx: 940, vy: 340, size: 72, delay: 5, color: '#10b981' },
  { char: 'ক', x: 280, y: 780, vx: 120, vy: 720, size: 80, delay: 10, color: '#fbbf24' },
  { char: 'খ', x: 1600, y: 820, vx: 900, vy: 800, size: 76, delay: 15, color: '#38bdf8' },
  { char: 'ছ', x: 210, y: 460, vx: 90, vy: 520, size: 70, delay: 8, color: '#34d399' },
  { char: 'র', x: 1680, y: 510, vx: 920, vy: 590, size: 68, delay: 12, color: '#f43f5e' },
  { char: 'স্মৃতি', x: 380, y: 310, vx: 180, vy: 1120, size: 48, delay: 18, color: '#a78bfa' },
  { char: 'সৃষ্টি', x: 1480, y: 340, vx: 820, vy: 1180, size: 52, delay: 22, color: '#f59e0b' },
  { char: '২১', x: 960, y: 140, vx: 540, vy: 160, size: 56, delay: 2, color: '#e11d48' },
  { char: 'ং', x: 440, y: 880, vx: 160, vy: 1440, size: 44, delay: 25, color: '#6ee7b7' },
  { char: 'ঁ', x: 1400, y: 890, vx: 860, vy: 1480, size: 48, delay: 28, color: '#fda4af' },
  { char: 'জ্ঞ', x: 180, y: 920, vx: 140, vy: 1720, size: 50, delay: 14, color: '#38bdf8' },
  { char: 'ক্ষ', x: 1760, y: 940, vx: 900, vy: 1760, size: 54, delay: 16, color: '#f43f5e' },
];

export const FloatingLetters: React.FC<{
  opacity?: number;
  highlightWord?: string;
  isVertical?: boolean;
}> = ({ opacity = 0.85, highlightWord, isVertical }) => {
  const frame = useCurrentFrame();
  const { width, fps } = useVideoConfig();
  const f = frame * (30 / fps);
  const vertical = isVertical ?? width < 1200;

  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        pointerEvents: 'none',
        overflow: 'hidden',
        fontFamily: "'Kalpurush', 'HindSiliguri', sans-serif",
      }}
    >
      {GLOWING_CHARS.map((item, idx) => {
        const floatY = Math.sin((f + item.delay * 8) * 0.04) * 22;
        const floatX = Math.cos((f + item.delay * 8) * 0.035) * 16;
        const rotation = Math.sin((f + item.delay * 12) * 0.03) * 12;
        const pulse = 1 + Math.sin((f + item.delay * 10) * 0.06) * 0.1;
        const twinkle = 0.75 + Math.sin((f + item.delay * 15) * 0.08) * 0.25;

        const isHighlighted = highlightWord && item.char.includes(highlightWord);
        const itemOpacity = (isHighlighted ? 1 : opacity * 0.75) * twinkle;
        const scale = isHighlighted ? pulse * 1.3 : pulse;

        const posX = vertical ? item.vx + floatX : item.x + floatX;
        const posY = vertical ? item.vy + floatY : item.y + floatY;

        return (
          <div
            key={idx}
            style={{
              position: 'absolute',
              left: posX,
              top: posY,
              fontSize: item.size,
              fontWeight: 700,
              color: item.color,
              opacity: itemOpacity,
              transform: `scale(${scale}) rotate(${rotation}deg)`,
              filter: `drop-shadow(0 0 ${isHighlighted ? 25 : 12}px ${item.color})`,
              transition: 'opacity 0.2s ease',
              userSelect: 'none',
            }}
          >
            {item.char}
          </div>
        );
      })}
    </div>
  );
};
