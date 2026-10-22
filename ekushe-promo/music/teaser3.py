#!/usr/bin/env python3
"""Score for Teaser 3, "যেভাবে বলেন, সেভাবে লিখুন" (22 s). Bright and flowing: a key click on every
letter, a shimmer as each Roman word turns into Bangla, a hit for the headline, plucks under the
first look at the popup, a bell as each feature comes round the ring, then the shared ending.

python3 music/teaser3.py
"""
import os
import sys

os.environ['TEASER_DUR'] = '22.2'
sys.path.insert(0, os.path.dirname(__file__))
from tlib import *  # noqa: E402,F403

# these must match src/teasers/Teaser3.tsx
WORD_AT = [10, 70, 130]
WORDS = ['amar', 'bhasha', 'bangla']
HEAD = 200 / FPS
POPUP = 330 / FPS
HERO_KEYS = [(360 + 7 * k) / FPS for k in range(41)]
RING = 700 / FPS
TAG = 1000 / FPS
END = 1140 / FPS

# 0–3.3 s: the words. A warm pad, a click per letter, a shimmer on each morph
place(pad([57, 64, 69], 3.6, bright=1800, gain=0.15), 0.0)
place(pad([45], 3.6, bright=300, gain=0.16), 0.0)
for w, at in zip(WORDS, WORD_AT):
    for i in range(len(w)):
        place(click(0.14), (at + i * 4) / FPS, pan=-0.3 + i * 0.1)
    m = (at + 38) / FPS
    place(whoosh(0.3, 0.12), m - 0.1)
    sparkle(m, 0.07)
    place(pluck_c(69 + WORD_AT.index(at) * 4, 1.4, 0.6), m, gain=0.4)

# 3.3 s: the headline
place(riser(0.9, 0.25), HEAD - 0.9)
place(impact(0.6), HEAD + 4 / FPS)
place(pad([57, 61, 64, 69, 73], 2.2, bright=3200, gain=0.18), HEAD)
for at, m, d in [(HEAD + 0.2, 76, 0.4), (HEAD + 0.6, 81, 0.4), (HEAD + 1.0, 85, 1.0)]:
    place(flute(m, d, 0.12), at)

# 5.5–11.7 s: the first look. 100 bpm plucks and a soft beat, a click on every real keystroke
beat = 0.6
prog = [57, 57, 53, 53, 50, 50, 52, 52, 57, 57]  # A F D E
for b in range(int((RING - POPUP) / beat)):
    at = POPUP + b * beat
    root = prog[b % len(prog)]
    place(kick(0.4 if b % 2 == 0 else 0.25), at)
    place(shaker(0.06), at + beat / 2, pan=0.3)
    for j, m in enumerate([root + 12, root + 16, root + 19, root + 24]):
        place(pluck_c(m, 1.0, 0.5), at + j * beat / 4, gain=0.24, pan=-0.3 + j * 0.2)
for b, root in enumerate([57, 53, 50, 52, 57]):
    place(pad([root, root + 4, root + 7], 1.3, bright=2200, gain=0.11), POPUP + b * 1.2)
for at in HERO_KEYS:
    place(click(0.12), at, pan=rng.uniform(-0.3, 0.3))

# 11.7–16.7 s: the ring. Drifting pads, a bell as each feature reaches the front
place(pad([62, 66, 69, 74], 5.0, bright=2400, gain=0.14), RING)
place(pad([50], 5.0, bright=300, gain=0.15), RING)
for i in range(6):
    place(bell([81, 83, 85, 88, 85, 90][i], 1.6, 0.10), RING + 0.3 + i * 0.79, pan=-0.5 + i * 0.2)
for b in range(int((TAG - RING) / 0.5)):
    place(shaker(0.05), RING + b * 0.5 + 0.25, pan=-0.2)

# 16.7 s: the tagline
place(whoosh(0.7, 0.2), TAG - 0.4)
place(impact(0.55), TAG + 6 / FPS)
place(timpani(45, 0.6), TAG + 6 / FPS)
place(pad([45, 57, 61, 64], 2.3, bright=2600, gain=0.17), TAG)
sparkle(TAG + 0.2)
place(riser(1.0, 0.2), END - 1.0)

end_card(END)
finish('public/music/teaser3.wav')
