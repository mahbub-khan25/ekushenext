#!/usr/bin/env python3
"""Score for the launch-date reveal (src/teasers/Reveal.tsx), 50 s. Original music throughout.

  0–7 s      remembrance: a slow heartbeat, a low A minor pad, a lonely flute over soft plucks
  7 s        the name: a riser, then a warm hit and an open chord
  11–19 s    the date: reels tick faster and faster; a timpani as each one lands, then the flare
  19–41.5 s  nine features: a hopeful 100 bpm groove (F C G Am), a swish on every new feature
  41.5 s     "see you on 30 October": the groove falls away, strings and flute, the opening theme again
  45.5 s     the ending card: hit, sparkles as the website and the email type in

python3 music/reveal.py
"""
import os
import sys

os.environ['TEASER_DUR'] = '50'
sys.path.insert(0, os.path.dirname(__file__))
from tlib import *  # noqa: E402,F403

# these must match src/teasers/Reveal.tsx
LOGO, DATE, FEATURES, FINALE, END = 420 / FPS, 660 / FPS, 1140 / FPS, 2490 / FPS, 2730 / FPS
LOCKS = [800 / FPS, 862 / FPS, 924 / FPS]
FLARE = 944 / FPS
FEAT = 150 / FPS


def heartbeat(gain=0.5):
    t = t_arr(0.5)
    one = np.sin(2 * np.pi * (48 + 30 * np.exp(-t * 30)) * t) * np.exp(-t * 14)
    return lowpass(one, 400) * gain


# ── 0–7: remembrance
place(pad([45, 52, 57, 60], 7.4, bright=700, gain=0.20), 0.0)
place(pad([33], 7.4, bright=200, gain=0.16), 0.0)
for k in range(8):
    at = 0.3 + k * 0.85
    place(heartbeat(0.42), at)
    place(heartbeat(0.26), at + 0.24)
theme = [(1.0, 69, 0.9), (1.9, 72, 0.6), (2.5, 71, 0.6), (3.1, 69, 1.4), (4.6, 67, 0.6), (5.2, 69, 0.6), (5.8, 64, 1.4)]
for at, m, d in theme:
    place(flute(m, d, 0.11), at)
for i, m in enumerate([57, 64, 60, 64, 55, 62, 59, 62]):
    place(pluck_c(m, 1.4, 0.25), 0.6 + i * 0.8, gain=0.2, pan=-0.3 + 0.08 * i)
place(riser(1.3, 0.24), LOGO + 30 / FPS - 1.3)

# ── 7: the name
place(impact(0.75), LOGO + 30 / FPS)
place(pad([53, 57, 60, 64, 69], 4.0, bright=2600, gain=0.22), LOGO + 30 / FPS)
place(pad([41], 4.0, bright=250, gain=0.16), LOGO + 30 / FPS)
place(bell(81, 2.0, 0.12), LOGO + 0.6)
for at, m, d in [(LOGO + 1.2, 76, 0.5), (LOGO + 1.7, 79, 0.5), (LOGO + 2.2, 81, 1.4)]:
    place(flute(m, d, 0.12), at)
place(whoosh(0.6, 0.18), DATE - 0.3)

# ── 11–19: the date
place(pad([45, 52, 57], 4.6, bright=1200, gain=0.16), DATE)
place(pad([33], 4.6, bright=200, gain=0.18), DATE)
t = DATE + 40 / FPS
step = 0.035
while t < LOCKS[-1]:
    place(tick(0.10), t, pan=rng.uniform(-0.5, 0.5))
    landed = sum(t > L for L in LOCKS)
    step *= 1.0 + 0.012 * (1 + landed)
    t += step
for i, L in enumerate(LOCKS):
    place(timpani(45 + i * 3, 0.55), L)
    place(click(0.25), L)
    place(bell(76 + i * 3, 1.4, 0.10), L + 0.02)
place(riser(0.4, 0.22), FLARE - 0.4)
place(impact(1.0), FLARE)
place(pad([45, 57, 61, 64, 69, 73], 5.2, bright=3200, gain=0.25), FLARE)
place(pad([33, 45], 5.2, bright=300, gain=0.18), FLARE)
sparkle(FLARE + 0.15)
for at, m, d in [(FLARE + 0.6, 81, 0.6), (FLARE + 1.2, 83, 0.6), (FLARE + 1.8, 85, 1.6)]:
    place(flute(m, d, 0.13), at)
place(whoosh(0.5, 0.2), FEATURES - 0.25)

# ── 19–41.5: the features
beat = 60 / 100
prog = [53, 48, 55, 57]  # F C G Am
nb = int((FINALE - FEATURES) / beat)
for b in range(nb):
    at = FEATURES + b * beat
    root = prog[(b // 4) % 4]
    place(kick(0.55 if b % 2 == 0 else 0.35), at)
    if b % 2 == 1:
        place(snare(0.16), at)
    place(shaker(0.05), at + beat / 2, pan=0.3)
    chord = [root + 12, root + 16 if root != 57 else root + 15, root + 19]
    for j, m in enumerate(chord):
        place(pluck_c(m, 0.8, 0.5), at + j * beat / 3, gain=0.2, pan=-0.3 + j * 0.3)
    if b % 4 == 0:
        place(pad([root, chord[1] - 12, root + 7], beat * 4, bright=2200, gain=0.12), at)
        place(pad([root - 12], beat * 4, bright=300, gain=0.13), at)
for i in range(1, 9):
    place(whoosh(0.32, 0.13), FEATURES + i * FEAT - 0.12)
for i in range(9):
    place(bell(84 if i % 2 else 81, 0.9, 0.06), FEATURES + i * FEAT + 0.15, pan=0.2)
melody = [(0, 76, 0.6), (0.6, 77, 0.6), (1.2, 79, 1.2), (2.4, 76, 1.2), (4.8, 72, 0.6), (5.4, 74, 0.6), (6.0, 76, 1.2), (7.2, 74, 1.2)]
for rep in range(2):
    for at, m, d in melody:
        place(flute(m, d, 0.08), FEATURES + 2.4 + rep * 9.6 + at)
place(riser(1.0, 0.16), FINALE - 1.0)

# ── 41.5: see you on 30 October
place(impact(0.5), FINALE)
place(pad([45, 52, 57, 60, 64], 4.4, bright=1600, gain=0.24), FINALE)
place(pad([33], 4.4, bright=200, gain=0.17), FINALE)
for at, m, d in [(0.5, 69, 0.8), (1.3, 72, 0.6), (1.9, 71, 0.6), (2.5, 76, 1.6)]:
    place(flute(m, d, 0.13), FINALE + at)
for k in range(4):
    place(heartbeat(0.3), FINALE + 0.4 + k * 0.85)

# ── 45.5: the ending card
end_card(END)
place(pad([45, 57, 61, 64, 69], 4.2, bright=2000, gain=0.14), END + 1.0)

finish('public/music/reveal.wav')
