import { useVideoConfig } from 'remotion';

/** Palette: the flag's green field and red sun, krishnachura, mustard, night. */
export const C = {
  night: '#03120C',
  forest: '#06281D',
  green: '#0B6B4A',
  greenBright: '#2FD18E',
  red: '#E0303F',
  coral: '#F26B4F',
  mustard: '#E9B949',
  mist: '#EEF4F1',
  ink: '#0E241B',
  white: '#FFFFFF',
};

export const FONT_BN = "'Hind Siliguri', 'Kalpurush', sans-serif";
export const FONT_DISPLAY = "'Kalpurush', 'Hind Siliguri', serif";
export const FONT_LATIN = "'Inter Tight', 'Hind Siliguri', sans-serif";

/** Scene timeline in frames (60 fps). The music is composed to the same marks. */
export const T = {
  open: [0, 480],
  martyrs: [480, 1080],
  pride: [1080, 1380],
  problem: [1380, 1920],
  reveal: [1920, 2100],
  demo: [2100, 2520],
  features: [2520, 2940],
  speed: [2940, 3240],
  outro: [3240, 3600],
} as const;
export const TOTAL = 3600;

/** Layout helpers: `u` is one unit at 1440 px on the short side; `V` is the vertical format. */
export const useLayout = () => {
  const { width, height } = useVideoConfig();
  const V = height > width;
  const u = Math.min(width, height) / 1440;
  return { width, height, V, u };
};

export const bn = (n: number | string) => String(n).replace(/[0-9]/g, (d) => '০১২৩৪৫৬৭৮৯'[+d]);
