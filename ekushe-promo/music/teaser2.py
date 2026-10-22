#!/usr/bin/env python3
"""Score for Teaser 2, "Why is typing Bangla still this hard?" (20 s). Same instruments and flute
motif as the film and Teaser 1.

  0–2.5 s     a lone tick and a low drone (the cursor)
  2.5–9.5 s   tension: pulsing A–B♭ ostinato, ticks, glitch bursts, a whoosh between problems
  9.5 s       "আর না।": silence, then a hit
  11–17.7 s   release: bright A major, 120 bpm plucks and drums, a click on every real keystroke
  17.7 s      the logo: A major, bell and flute

python3 music/teaser2.py public/music/teaser2.wav
"""
import pathlib

src = (pathlib.Path(__file__).parent / 'compose.py').read_text()
head, rest = src.split('# ───────────────────────── score ─────────────────────────')
mix = rest.split('# ───────────────────────── mix: reverb, glue, master ─────────────────────────')[1]
exec(head.replace('DUR = 60.0', 'DUR = 20.0'))

FPS = 60  # the marks below must match src/teasers/Teaser2.tsx
P = [150 / FPS, 290 / FPS, 430 / FPS]
HIT = 570 / FPS
REVEAL = 660 / FPS
CONV_KEYS = [(700 + 4 * i) / FPS for i in range(20)]
PAD_KEYS = [(830 + 10 * i) / FPS for i in range(6)]
FIX = 992 / FPS
LOGO = 1060 / FPS


def bell(m, d=2.2, gain=0.12):
    t = t_arr(d)
    f = midi(m)
    tone = sum(a * np.sin(2 * np.pi * f * k * t) * np.exp(-t * dec) for k, a, dec in [(1, 1, 1.6), (2.76, 0.35, 3.0), (5.4, 0.12, 5.0)])
    return tone * env_adsr(len(t), 0.004, 0.1, 1.0, 0.5) * gain


def glitch(d=0.18, gain=0.16):
    n = int(d * SR)
    x = rng.normal(0, 1, n)
    x = np.round(x * 3) / 3                       # crushed
    hold = 24
    x = np.repeat(x[::hold], hold)[:n]            # sample-rate reduced
    x *= (np.sin(2 * np.pi * 37 * t_arr(d)) > 0)  # stutter
    return highpass(x, 600) * env_adsr(n, 0.002, 0.02, 0.8, 0.03) * gain


def whoosh(d=0.35, gain=0.18):
    n = int(d * SR)
    x = rng.normal(0, 1, n)
    out = np.zeros(n)
    for i, fc in enumerate(np.linspace(600, 6000, 8)):
        seg = slice(i * n // 8, (i + 1) * n // 8)
        out[seg] = lowpass(x, fc)[seg]
    return out * np.sin(np.linspace(0, np.pi, n)) * gain


# 0–2.5: the cursor
place(pad([33, 45], 2.6, bright=300, gain=0.2), 0.0)
for i in range(6):
    place(tick(0.07), 0.3 + i * 0.6, pan=0.4)

# 2.5–9.5: tension
for i in range(int((HIT - P[0]) / 0.25)):
    at = P[0] + i * 0.25
    m = [45, 45, 46, 45][i % 4]
    place(lowpass(saw(midi(m), 0.22), 700) * env_adsr(int(0.22 * SR), 0.005, 0.05, 0.6, 0.08) * 0.2, at)
    place(tick(0.05 if i % 2 else 0.08), at, pan=0.5 if i % 2 else -0.5)
    if i % 4 == 0:
        place(kick(0.45), at)
place(pad([45, 46, 52], HIT - P[0], bright=900, gain=0.10), P[0])
for k in range(10):  # Bijoy breaking apart
    place(glitch(0.08 + 0.06 * rng.random(), 0.14), P[0] + 0.15 + k * 0.21 + 0.05 * rng.random(), pan=rng.uniform(-0.7, 0.7))
for at in P[1:]:
    place(whoosh(), at - 0.2)
for k in range(5):  # the conjunct that keeps coming out wrong
    place(pluck_c(58 + [0, 1, 0, 3, 1][k], 0.5, 0.8), P[1] + 0.3 + k * 0.4, gain=0.35, pan=0.3)
place(pad([45, 46, 51, 52], 2.2, bright=1600, gain=0.07), P[2])  # the misspelling: a sour cluster
place(riser(1.6, 0.3), HIT - 1.7)

# 9.5: silence, then the hit
place(impact(0.9), HIT)
place(timpani(33, 0.9), HIT)
place(pad([45, 52, 57], 1.4, bright=1200, gain=0.16), HIT)

# 11–17.7: release
place(pad([57, 61, 64, 69], 6.8, bright=2800, gain=0.16), REVEAL)
place(pad([45], 6.8, bright=300, gain=0.16), REVEAL)
beat = 0.5
n_beats = int((LOGO - REVEAL) / beat)
prog = [57, 57, 54, 54, 50, 50, 52, 52]  # A A F#m F#m D D E E
for b in range(n_beats):
    at = REVEAL + b * beat
    root = prog[(b // 2) % len(prog)]
    place(kick(0.55 if b % 2 == 0 else 0.35), at)
    if b % 2 == 1:
        place(snare(0.18), at)
    place(shaker(0.07), at + beat / 2, pan=0.3)
    for j, m in enumerate([root + 12, root + 16, root + 19]):
        place(pluck_c(m, 0.9, 0.55), at + j * beat / 3, gain=0.28, pan=-0.3 + j * 0.3)
for at in CONV_KEYS + PAD_KEYS:
    place(click(0.15), at, pan=rng.uniform(-0.3, 0.3))
for at, m, d in [(REVEAL + 0.1, 76, 0.5), (REVEAL + 0.6, 81, 0.9), (REVEAL + 2.4, 83, 0.5), (REVEAL + 2.9, 85, 0.9)]:
    place(flute(m, d, 0.11), at)
place(bell(88, 1.4, 0.10), PAD_KEYS[-1] + 0.35)  # ক্ষমা lands
place(bell(88, 1.4, 0.10), FIX)                    # the spelling fixed
place(riser(1.2, 0.22), LOGO - 1.2)

# 17.7: the logo
place(impact(0.45), LOGO)
place(pad([45, 57, 61, 64, 69], 2.3, bright=3000, gain=0.20), LOGO)
place(bell(81, 2.0, 0.14), LOGO + 0.05)
place(flute(81, 1.5, 0.12), LOGO + 0.15)

exec(mix.replace("lift = np.interp(np.arange(N) / SR, [0, 22.0, 23.5, 60], [1.7, 1.7, 1.0, 1.0])", "lift = np.ones(N)")
        .replace("'public/music/ekushe-score.wav'", "'public/music/teaser2.wav'"))
