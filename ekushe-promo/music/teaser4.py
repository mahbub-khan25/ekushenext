#!/usr/bin/env python3
"""Score for Teaser 4, "চোখের পলকের চেয়েও দ্রুত" (20.7 s).

  0–4 s      slow motion: a deep heartbeat, a stretched swell as the eye closes
  4–10 s     the rapid fire: driving 140 bpm pulse, a click per word and a tick per letter
  10 s       the eye snaps open: impact and a burst
  11.7–16 s  the numbers: a descending sweep as the count falls to 0.98 ms, then a chime
  16 s       the tagline, then the shared ending

python3 music/teaser4.py
"""
import json
import os
import pathlib
import sys

os.environ['TEASER_DUR'] = '20.7'
sys.path.insert(0, os.path.dirname(__file__))
from tlib import *  # noqa: E402,F403

# these must match src/teasers/Teaser4.tsx
RAPID, OPEN, RACE, TAG, END = 240 / FPS, 600 / FPS, 700 / FPS, 960 / FPS, 1060 / FPS
WORD_EVERY = 14
words = json.loads((pathlib.Path(__file__).parent.parent / 'src/teasers/t4words.json').read_text())


def heartbeat(gain=0.6):
    t = t_arr(0.5)
    f = 42 + 20 * np.exp(-t * 20)
    x = np.sin(2 * np.pi * np.cumsum(f) / SR) * np.exp(-t * 9)
    return lowpass(x, 200) * gain


# 0–4: slow motion
place(pad([33, 40, 45], 4.2, bright=500, gain=0.2), 0.0)
for at in (0.3, 1.3, 2.3, 3.3):
    place(heartbeat(0.7), at)
    place(heartbeat(0.45), at + 0.28)
place(riser(3.0, 0.18), 0.9)

# 4–10: the rapid fire
place(impact(0.45), RAPID)
beat = 60 / 140
for b in range(int((OPEN - RAPID) / (beat / 2))):
    at = RAPID + b * beat / 2
    if b % 2 == 0:
        place(kick(0.5), at)
    if b % 4 == 2:
        place(snare(0.2), at)
    place(shaker(0.05), at + beat / 4, pan=0.3)
    m = [57, 57, 64, 57, 62, 57, 64, 69][b % 8]
    place(lowpass(saw(midi(m - 12), 0.18), 900) * env_adsr(int(0.18 * SR), 0.003, 0.05, 0.5, 0.06) * 0.16, at)
place(pad([57, 61, 64], OPEN - RAPID, bright=2000, gain=0.10), RAPID)
for i, w in enumerate(words):
    at = RAPID + i * WORD_EVERY / FPS
    for j in range(len(w['roman'])):
        place(tick(0.035), at + j * 1.2 / FPS, pan=0.3)
    place(click(0.13), at + (len(w['roman']) * 1.2 + 2) / FPS)
    place(pluck_c(69 + [0, 4, 7, 12][i % 4], 0.6, 0.6), at + 4 / FPS, gain=0.18, pan=-0.3)
place(riser(1.2, 0.22), OPEN - 1.2)

# 10: open
place(impact(0.85), OPEN + 4 / FPS)
place(timpani(45, 0.7), OPEN + 4 / FPS)
place(pad([45, 57, 64, 69], 1.6, bright=2600, gain=0.17), OPEN)
sparkle(OPEN + 0.1)

# 11.7–16: the numbers
place(pad([50, 57, 62, 66], 4.3, bright=2200, gain=0.14), RACE)
place(pad([38], 4.3, bright=300, gain=0.15), RACE)
place(whoosh(1.9, 0.16, up=False), RACE + 0.6)
for k in range(12):
    place(pluck_c(81 - k, 0.5, 0.6), RACE + 0.66 + k * 0.15, gain=0.16, pan=0.2)
place(bell(88, 1.6, 0.13), RACE + 2.55)
sparkle(RACE + 2.55, 0.07)
for b in range(int((TAG - RACE) / 0.5)):
    place(shaker(0.05), RACE + b * 0.5 + 0.25)

# 16: tagline
place(whoosh(0.6, 0.2), TAG - 0.35)
place(impact(0.55), TAG + 4 / FPS)
place(pad([45, 57, 61, 64], 1.7, bright=2800, gain=0.17), TAG)
place(riser(0.9, 0.18), END - 0.9)

end_card(END)
finish('public/music/teaser4.wav')
