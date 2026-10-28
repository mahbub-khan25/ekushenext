# Launch reveal: Remotion → HyperFrames

The same 84 s film as `ekushe-promo/src/teasers/LaunchReveal.tsx` (9:16, 1440×2560, 60 fps), rebuilt
as a HyperFrames project. Render: `npm run render -- --fps 60` (or `npx hyperframes render --fps 60`).

## Layout

| File | What | Time |
| --- | --- | --- |
| `index.html` | the root: scene hosts and the music | 0–84 s |
| `compositions/scenery.html` | night sky, stars, the sun rising, the Shaheed Minar | 0–84 s (under everything) |
| `compositions/dark.html` | "অপেক্ষার প্রহর শেষে...।", "নতুন ভোর" | 0–16 s |
| `compositions/date.html` | 30 october → ৩০ অক্টোবর, "অবশেষে এসে গেছে!" | 16–22 s |
| `compositions/name.html` | logo, name, Windows · macOS · Linux | 22–27 s |
| `compositions/win.html`, `mac.html`, `linux.html` | the welcome typed on each computer | 27–56 s |
| `compositions/phones.html` | Android and iPhone, coming soon | 56–66 s |
| `compositions/party.html` | countdown, letter fireworks, key caps, wishes | 66–76 s |
| `compositions/ending.html` | download today, website | 76–84 s |
| `assets/ek.js` | shared helpers and the typed words (`EK.L`) | |
| `assets/desktop.js` | the Windows / macOS / Linux desktops | |

## How it was ported

- **Native (GSAP tweens)**: every entrance, exit, fade and blur. Remotion `spring()` calls became GSAP
  tweens whose ease is Remotion's own spring curve (`EK.sp(config)`), so the motion matches exactly
  rather than approximately with `back.out`. `ease()` (cubic out) became `power2.out`.
- **Driven from the timeline (one `tick` tween per scene)**: the procedural parts, which were
  per-frame maths in Remotion: the typing and suggestion bar, the key caps, the sky and stars, the
  sun and Minar, particle bursts, petals, letter fireworks, the countdown, the floating wishes.
  They use exact copies of Remotion's `random()` and `spring()`, so particles land where they did.
- **Copy** is plain text in each scene file and can be edited in the preview. The typed lines live in
  `assets/ek.js` (`EK.L`) because the music's key clicks are timed to them
  (`ekushe-promo/music/launch_reveal.py`); change both together.
- **Music**: the same `launch_reveal.wav`.

## Differences from the Remotion version

- Nothing visible was dropped. Side-by-side frames at 14 points across the film
  (`npx hyperframes snapshot --against`) score SSIM 0.92–0.95; the 5 s frame scores 0.74 only because
  the star twinkle and the dark sky's encoding noise differ; the text and Minar are the same.
- `check` warns about small text (24–36 px subtitles and phone status bars) and key caps that show
  for under 0.4 s. Both are as designed in the original and were kept.
