#!/usr/bin/env python3
"""Score for Teaser 5, "আপনার লেখা, শুধু আপনার" and the first full look (22 s).

  0–5.5 s    calm and close: rain-like noise, soft piano-ish plucks, a snap as each connection is cut,
             the lock clicks shut
  5.5 s      the promise: a warm swell
  7.2–16.7 s the full look: a steady, proud 92 bpm groove, a page-turn swish for every Studio page,
             a shimmer when it turns light
  16.7 s     the platforms pop in, then the shared ending

python3 music/teaser5.py
"""
import os
import sys

os.environ['TEASER_DUR'] = '22.2'
sys.path.insert(0, os.path.dirname(__file__))
from tlib import *  # noqa: E402,F403

# these must match src/teasers/Teaser5.tsx
CUT_AT = [60, 95, 130, 165, 200, 235]
TITLE, LOOK, PLATFORMS, END = 330 / FPS, 430 / FPS, 1000 / FPS, 1140 / FPS
PAGES = [(430 + 40 + i * 88) / FPS for i in range(6)]
THEME = (430 + 40 + 3 * 88 + 40) / FPS


def snap(gain=0.3):
    t = t_arr(0.12)
    x = highpass(rng.normal(0, 1, len(t)), 1800) * np.exp(-t * 60)
    return (x + np.sin(2 * np.pi * 1400 * t) * np.exp(-t * 90) * 0.6) * gain


def rain(d, gain=0.04):
    return highpass(lowpass(rng.normal(0, 1, int(d * SR)), 5000), 1500) * gain


# 0–5.5: calm, with cuts
L_ = rain(5.6)
place(L_, 0.0, pan=-0.3)
place(rain(5.6), 0.0, pan=0.3)
place(pad([45, 52, 57], 5.6, bright=900, gain=0.17), 0.0)
for i, m in enumerate([69, 72, 76, 74, 72, 69, 67, 69]):
    place(pluck_c(m, 1.6, 0.3), 0.3 + i * 0.66, gain=0.32, pan=-0.2 + 0.05 * i)
for at in CUT_AT:
    place(snap(0.32), at / FPS, pan=rng.uniform(-0.6, 0.6))
place(click(0.3), 255 / FPS)   # the lock shuts
place(bell(76, 1.6, 0.10), 258 / FPS)
place(riser(1.2, 0.2), TITLE - 1.2)

# 5.5: the promise
place(impact(0.6), TITLE + 4 / FPS)
place(pad([57, 61, 64, 69], 1.8, bright=2600, gain=0.18), TITLE)
for at, m, d in [(TITLE + 0.2, 76, 0.4), (TITLE + 0.6, 81, 0.9)]:
    place(flute(m, d, 0.12), at)
place(whoosh(0.7, 0.2), LOOK - 0.35)

# 7.2–16.7: the full look
beat = 60 / 92
prog = [57, 53, 48, 55]  # A F C G, proud
for b in range(int((PLATFORMS - LOOK) / beat)):
    at = LOOK + b * beat
    root = prog[(b // 4) % 4]
    place(kick(0.5 if b % 2 == 0 else 0.3), at)
    if b % 2 == 1:
        place(snare(0.17), at)
    place(shaker(0.05), at + beat / 2, pan=0.3)
    for j, m in enumerate([root + 12, root + 16, root + 19]):
        place(pluck_c(m, 0.9, 0.5), at + j * beat / 3, gain=0.22, pan=-0.3 + j * 0.3)
    if b % 4 == 0:
        place(pad([root, root + 4, root + 7], beat * 4, bright=2200, gain=0.12), at)
        place(pad([root - 12], beat * 4, bright=300, gain=0.13), at)
for at in PAGES[1:]:
    place(whoosh(0.32, 0.13), at - 0.05)
sparkle(THEME)
for at, m, d in [(LOOK + 1.2, 76, 0.6), (LOOK + 1.9, 79, 0.6), (LOOK + 2.6, 81, 1.2), (LOOK + 5.2, 84, 0.6), (LOOK + 5.9, 83, 0.6), (LOOK + 6.6, 81, 1.4)]:
    place(flute(m, d, 0.10), at)

# 16.7: platforms
for i in range(3):
    place(timpani(45 + i * 4, 0.35), PLATFORMS + (6 + i * 9) / FPS)
    place(bell(81 + i * 4, 1.2, 0.09), PLATFORMS + (6 + i * 9) / FPS)
place(pad([45, 57, 61, 64], 2.3, bright=2800, gain=0.16), PLATFORMS)
place(riser(1.0, 0.2), END - 1.0)

end_card(END)
finish('public/music/teaser5.wav')
