#!/usr/bin/env python3
"""Score for "One keyboard, everywhere" (src/teasers/Everywhere.tsx), 33.3 s. Original music.

  0–4 s      "এক কিবোর্ড। সব জায়গায়।": heartbeat, a low pad, the opening flute theme, a bright hit
  4–10 s     on the computer: soft plucks as each word lands, the groove starts (100 bpm, F C G Am)
  10 s       the phone rises: a riser into a warm hit; key taps tick as "amar sonar bangla" is typed
  14–17.7 s  themes: a bell for every new look
  17.7–21.5  English, emoji, 3D keys: a swish on each
  21.5 s     one engine: a deep hit, a big open chord, the groove at full strength, the theme on top
  26.5 s     the date: the groove drops, timpani and a flare
  29.3 s     the ending card: hit, sparkles as the website and the email type in

python3 music/everywhere.py
"""
import os
import sys

os.environ['TEASER_DUR'] = '33.4'
sys.path.insert(0, os.path.dirname(__file__))
from tlib import *  # noqa: E402,F403

# must match src/teasers/Everywhere.tsx
DESK, PHONE, THEMES, EXTRAS, ENGINE, DATE, END = [x / FPS for x in (240, 600, 840, 1060, 1290, 1590, 1760)]


def heartbeat(gain=0.5):
    t = t_arr(0.5)
    one = np.sin(2 * np.pi * (48 + 30 * np.exp(-t * 30)) * t) * np.exp(-t * 14)
    return lowpass(one, 400) * gain


# ── 0–4: one keyboard, everywhere
place(pad([45, 52, 57, 60], 4.2, bright=800, gain=0.2), 0.0)
place(pad([33], 4.2, bright=200, gain=0.16), 0.0)
for k in range(4):
    place(heartbeat(0.4), 0.2 + k * 0.7)
    place(heartbeat(0.24), 0.44 + k * 0.7)
place(flute(69, 0.8, 0.11), 0.35)
place(flute(72, 0.5, 0.11), 1.0)
place(impact(0.6), 80 / FPS)
place(pad([53, 57, 60, 64, 69], 2.8, bright=2600, gain=0.2), 80 / FPS)
place(bell(81, 1.8, 0.12), 80 / FPS + 0.05)
for at, m, d in [(1.9, 76, 0.4), (2.3, 79, 0.4), (2.7, 81, 1.0)]:
    place(flute(m, d, 0.12), at)
place(whoosh(0.5, 0.18), DESK - 0.25)

# ── 4–21.5: the groove, the computer and then the phone
beat = 60 / 100
prog = [53, 48, 55, 57]  # F C G Am
t0 = DESK
nb = int((ENGINE - t0) / beat)
for b in range(nb):
    at = t0 + b * beat
    if at > PHONE - 0.6 and at < PHONE + 0.2:
        continue  # a breath before the phone rises
    root = prog[(b // 4) % 4]
    full = at >= PHONE
    place(kick(0.5 if b % 2 == 0 else 0.3), at)
    if b % 2 == 1:
        place(snare(0.15 if full else 0.1), at)
    if full:
        place(shaker(0.05), at + beat / 2, pan=0.3)
    chord = [root + 12, root + 16 if root != 57 else root + 15, root + 19]
    for j, m in enumerate(chord):
        place(pluck_c(m, 0.8, 0.5), at + j * beat / 3, gain=0.18, pan=-0.3 + j * 0.3)
    if b % 4 == 0:
        place(pad([root, chord[1] - 12, root + 7], beat * 4, bright=2000, gain=0.11), at)
        place(pad([root - 12], beat * 4, bright=300, gain=0.12), at)
# each Bangla word landing on the computer
for i in range(4):
    place(bell(79 + [0, 3, 5, 7][i], 0.9, 0.07), DESK + (40 + i * 50 + 34) / FPS, pan=0.2)
place(riser(1.0, 0.22), PHONE - 1.0)

# ── 10: the phone
place(impact(0.7), PHONE + 6 / FPS)
place(pad([53, 57, 60, 64, 69], 3.0, bright=2800, gain=0.18), PHONE + 6 / FPS)
for k in range(18):  # "amar sonar bangla " typed, one key every 7 frames
    place(click(0.09), PHONE + (70 + k * 7) / FPS, pan=rng.uniform(-0.3, 0.3))
for i in range(3):
    place(bell(81 + i * 2, 0.8, 0.06), PHONE + (70 + [4, 10, 17][i] * 7) / FPS)
# themes: a bell on each new look
for i in range(9):
    place(bell([76, 79, 81, 83, 84, 86, 88, 86, 84][i], 0.7, 0.07), THEMES + i * 24 / FPS, pan=-0.4 + i * 0.1)
    place(whoosh(0.18, 0.06), THEMES + i * 24 / FPS - 0.05)
for i, off in enumerate([0, 75, 150]):
    place(whoosh(0.3, 0.12), EXTRAS + off / FPS - 0.1)
    place(bell(84 + i * 2, 1.0, 0.08), EXTRAS + off / FPS + 0.05)
melody = [(0, 76, 0.6), (0.6, 77, 0.6), (1.2, 79, 1.2), (2.4, 76, 1.2), (4.8, 72, 0.6), (5.4, 74, 0.6), (6.0, 76, 1.2)]
for at, m, d in melody:
    place(flute(m, d, 0.07), PHONE + 2.4 + at)
place(riser(1.0, 0.2), ENGINE - 1.0)

# ── 21.5: one engine
place(impact(1.0), ENGINE + 8 / FPS)
place(timpani(41, 0.5), ENGINE + 8 / FPS)
place(pad([41, 53, 57, 60, 64, 69], 5.0, bright=3200, gain=0.22), ENGINE + 8 / FPS)
sparkle(ENGINE + 0.2)
nb2 = int((DATE - ENGINE - 0.3) / beat)
for b in range(nb2):
    at = ENGINE + 0.15 + b * beat
    root = prog[(b // 4) % 4]
    place(kick(0.6 if b % 2 == 0 else 0.4), at)
    place(shaker(0.06), at + beat / 2, pan=-0.3)
    if b % 2 == 1:
        place(snare(0.18), at)
    for j, m in enumerate([root + 12, root + 16 if root != 57 else root + 15, root + 19, root + 24]):
        place(pluck_c(m, 0.7, 0.6), at + j * beat / 4, gain=0.17, pan=-0.4 + j * 0.27)
for at, m, d in [(0.6, 81, 0.6), (1.2, 83, 0.6), (1.8, 84, 1.2), (3.0, 81, 0.6), (3.6, 79, 1.2)]:
    place(flute(m, d, 0.11), ENGINE + at)
for i in range(4):
    place(bell(84 + i * 2, 0.8, 0.07), ENGINE + (140 + i * 12) / FPS)
place(riser(0.6, 0.2), DATE + 24 / FPS - 0.6)

# ── 26.5: the date
place(timpani(45, 0.55), DATE + 24 / FPS)
place(impact(0.9), DATE + 24 / FPS)
place(pad([45, 57, 61, 64, 69, 73], 3.2, bright=3200, gain=0.24), DATE + 24 / FPS)
place(pad([33, 45], 3.2, bright=300, gain=0.17), DATE + 24 / FPS)
sparkle(DATE + 0.55)
for at, m, d in [(0.9, 81, 0.5), (1.4, 83, 0.5), (1.9, 85, 1.0)]:
    place(flute(m, d, 0.12), DATE + at)

# ── 29.3: the ending card
end_card(END)
place(pad([45, 57, 61, 64, 69], 3.8, bright=2000, gain=0.14), END + 1.0)

finish('public/music/everywhere.wav')
