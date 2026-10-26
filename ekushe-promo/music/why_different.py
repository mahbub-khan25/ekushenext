#!/usr/bin/env python3
"""Score for "what Ekushe Next does differently" (src/teasers/WhyDifferent.tsx), 70 s. Original music.

  0–6 s     opening: soft pad and heartbeat, key ticks for "amar bhasha", a bell as it turns into
            Bangla, then a warm hit for the name
  6–62 s    eight differences, 7 s each: the hopeful 100 bpm groove (F C G Am), a swish on each new
            one, key ticks where letters are typed, a bell where a word is chosen
  62–70 s   ending: hit, the theme, a chime for "30 October", sparkles as the website types in

python3 music/why_different.py
"""
import os
import sys

os.environ['TEASER_DUR'] = '70.4'
sys.path.insert(0, os.path.dirname(__file__))
from tlib import *  # noqa: E402,F403

# must match WhyDifferent.tsx (W)
CH, LEN, END = 360 / FPS, 420 / FPS, 3720 / FPS
beat = 60 / 100
prog = [53, 48, 55, 57]  # F C G Am


def heartbeat(gain=0.5):
    t = t_arr(0.5)
    one = np.sin(2 * np.pi * (48 + 30 * np.exp(-t * 30)) * t) * np.exp(-t * 14)
    return lowpass(one, 400) * gain


def groove(start, end, level=1.0, arp=3, pad_gain=0.11, snare_on=True):
    nb = int((end - start) / beat)
    for b in range(nb):
        at = start + b * beat
        root = prog[(b // 4) % 4]
        place(kick((0.5 if b % 2 == 0 else 0.32) * level), at)
        if snare_on and b % 2 == 1:
            place(snare(0.14 * level), at)
        place(shaker(0.045 * level), at + beat / 2, pan=0.3)
        chord = [root + 12, root + 16 if root != 57 else root + 15, root + 19, root + 24][:arp]
        for j, m in enumerate(chord):
            place(pluck_c(m, 0.8, 0.5), at + j * beat / len(chord), gain=0.17 * level, pan=-0.35 + j * 0.25)
        if b % 4 == 0:
            place(pad([root, chord[1] - 12, root + 7], beat * 4, bright=2200, gain=pad_gain), at)
            place(pad([root - 12], beat * 4, bright=300, gain=0.12), at)


MELODY = [(0, 76, 0.6), (0.6, 77, 0.6), (1.2, 79, 1.2), (2.4, 76, 1.2), (4.8, 72, 0.6), (5.4, 74, 0.6), (6.0, 76, 1.2), (7.2, 74, 1.2)]


def melody(at, gain=0.07, up=0):
    for t0, m, d in MELODY:
        place(flute(m + up, d, gain), at + t0)


def keys(at_frame, n, every):
    for k in range(n):
        place(click(0.06), at_frame / FPS + k * every / FPS, pan=0.12)


def chosen(at, up=0):
    place(bell(81 + up, 1.2, 0.08), at)
    sparkle(at + 0.02, gain=0.05)


# ── 0–6 s: opening
place(pad([45, 52, 57, 60], 6.2, bright=700, gain=0.18), 0.0)
place(pad([33], 6.2, bright=200, gain=0.14), 0.0)
for k in range(3):
    place(heartbeat(0.32), 0.25 + k * 0.85)
    place(heartbeat(0.2), 0.49 + k * 0.85)
keys(40, 11, 5)  # "amar bhasha"
place(bell(81, 1.8, 0.1), 112 / FPS)
sparkle(116 / FPS, gain=0.06)
place(riser(1.2, 0.2), 192 / FPS - 1.2)
place(impact(0.6), 192 / FPS)
place(pad([53, 57, 60, 64, 69], 3.0, bright=2600, gain=0.2), 192 / FPS)
place(bell(84, 2.0, 0.1), 200 / FPS)
melody(210 / FPS, gain=0.08)

# ── 6–62 s: the eight differences, the groove under all of them
groove(CH, END - 0.05, level=0.9)
for i in range(8):
    c = CH + i * LEN
    place(whoosh(0.32, 0.12), c - 0.12)
    place(bell(84 if i % 2 else 81, 0.9, 0.05), c + 0.1, pan=0.2)
    if i in (3, 7):
        melody(c + 0.6, gain=0.06, up=0 if i == 3 else 2)
c = lambda i, fr: CH + i * LEN + fr / FPS  # noqa: E731
# 1: porikkha, sompurno
keys(c(0, 40) * FPS, 8, 6); chosen(c(0, 118))
keys(c(0, 160) * FPS, 8, 6); chosen(c(0, 238), up=2)
# 2: kemon ase, then the chat
keys(c(1, 40) * FPS, 9, 6); chosen(c(1, 114))
place(whoosh(0.25, 0.08), c(1, 198)); place(bell(79, 1.0, 0.06), c(1, 252))
# 3: marks
keys(c(2, 40) * FPS, 5, 6); chosen(c(2, 80))
keys(c(2, 130) * FPS, 4, 6); chosen(c(2, 164), up=2)
# 4: conjunct hints, then স্থ
keys(c(3, 40) * FPS, 2, 14)
for k in range(6):
    place(bell(88 + (k % 3) * 2, 0.5, 0.03), c(3, 70 + k * 8), pan=-0.4 + k * 0.16)
keys(c(3, 216) * FPS, 2, 14); chosen(c(3, 230), up=3)
# 5: fix a wrong word, then it's learned
place(click(0.09), c(4, 150)); chosen(c(4, 150)); chosen(c(4, 250), up=4)
# 6: amar, Space, Backspace
keys(c(5, 40) * FPS, 4, 7); place(click(0.08), c(5, 80)); place(click(0.1), c(5, 170))
place(whoosh(0.3, 0.1, up=False), c(5, 166)); chosen(c(5, 172), up=-2)
# 7: the lock, the password
place(timpani(41, 0.3), c(6, 20)); place(bell(76, 1.6, 0.07), c(6, 24))
keys(c(6, 150) * FPS, 7, 6)
# 8: one engine, the devices light up
place(impact(0.4), c(7, 20))
for k in range(5):
    place(bell(81 + [0, 4, 7, 9, 12][k], 0.9, 0.06), c(7, 90 + k * 14), pan=-0.5 + k * 0.25)

# ── 62–70 s: ending
E = END
place(impact(0.7), E + 6 / FPS)
place(pad([53, 57, 60, 64, 69, 72], 4.0, bright=3000, gain=0.22), E + 6 / FPS)
place(pad([41, 53], 4.0, bright=300, gain=0.15), E + 6 / FPS)
place(bell(81, 2.0, 0.12), E + 0.1)
melody(E + 0.4, gain=0.09, up=0)
place(bell(84, 2.2, 0.12), E + 60 / FPS)  # 30 October
sparkle(E + 62 / FPS, gain=0.07)
place(pad([48, 55, 60, 64, 67], 3.6, bright=2400, gain=0.16), E + 120 / FPS)
for k in range(18):
    place(click(0.06), E + 210 / FPS + k * 2 / FPS, pan=0.3)  # the website types in
sparkle(E + 250 / FPS)
place(pad([53, 60, 65, 69], 3.0, bright=1600, gain=0.14), E + 5.0)

finish('public/music/why_different.wav')
