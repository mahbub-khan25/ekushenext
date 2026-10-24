#!/usr/bin/env python3
"""Score for the full promo (src/teasers/FullPromo.tsx), 112 s. Original music throughout.

  0–10 s      a message to Ma: soft heartbeat, key ticks, a bell as it turns into Bangla, her reply
  10–14 s     the name: riser, warm hit, open chord
  11–33.5 s   nine features: the hopeful 100 bpm groove (F C G Am), a swish on each
  33.5–36.5   "one keyboard, on every device": a lift, a bell for each platform
  36.5–60.5   Windows and Linux: the groove grows; a bright chime as Bangla switches on, key
              ticks in the terminal, a confident chord on each "installed"
  60.5–91     macOS, Android, iPhone/iPad: lighter, airy; bells for every new keyboard look
  91–97       one engine: deep hit, the full groove with the theme on top
  97–105      when and where: two warm chords, one per card
  105–112     the ending: hit, the theme once more, sparkles as the website and email type in

python3 music/full_promo.py
"""
import os
import sys

os.environ['TEASER_DUR'] = '115.2'
sys.path.insert(0, os.path.dirname(__file__))
from tlib import *  # noqa: E402,F403

# must match src/teasers/FullPromo.tsx
M = {k: v / FPS for k, v in dict(logo=600, features=840, devices=2190, win=2370, linux=2970, mac=3810,
                                  android=4350, ios=5040, engine=5640, avail=6000, end=6480).items()}
FEAT = 150 / FPS
beat = 60 / 100
prog = [53, 48, 55, 57]  # F C G Am


def heartbeat(gain=0.5):
    t = t_arr(0.5)
    one = np.sin(2 * np.pi * (48 + 30 * np.exp(-t * 30)) * t) * np.exp(-t * 14)
    return lowpass(one, 400) * gain


def groove(start, end, level=1.0, shaker_on=True, snare_on=True, arp=3, pad_gain=0.12, skip=()):
    """The F C G Am groove from the teasers, between two times."""
    nb = int((end - start) / beat)
    for b in range(nb):
        at = start + b * beat
        if any(a <= at < z for a, z in skip):
            continue
        root = prog[(b // 4) % 4]
        place(kick((0.55 if b % 2 == 0 else 0.35) * level), at)
        if snare_on and b % 2 == 1:
            place(snare(0.16 * level), at)
        if shaker_on:
            place(shaker(0.05 * level), at + beat / 2, pan=0.3)
        chord = [root + 12, root + 16 if root != 57 else root + 15, root + 19, root + 24][:arp]
        for j, m in enumerate(chord):
            place(pluck_c(m, 0.8, 0.5), at + j * beat / len(chord), gain=0.19 * level, pan=-0.35 + j * 0.25)
        if b % 4 == 0:
            place(pad([root, chord[1] - 12, root + 7], beat * 4, bright=2200, gain=pad_gain), at)
            place(pad([root - 12], beat * 4, bright=300, gain=0.13), at)


MELODY = [(0, 76, 0.6), (0.6, 77, 0.6), (1.2, 79, 1.2), (2.4, 76, 1.2), (4.8, 72, 0.6), (5.4, 74, 0.6), (6.0, 76, 1.2), (7.2, 74, 1.2)]


def melody(at, gain=0.08, up=0):
    for t0, m, d in MELODY:
        place(flute(m + up, d, gain), at + t0)


# ── 0–10: a message to Ma (OPEN in FullPromo.tsx: morph 200, send 250, dots 285, reply 340, line 430)
place(pad([45, 52, 57, 60], 10.4, bright=700, gain=0.19), 0.0)
place(pad([33], 10.4, bright=200, gain=0.15), 0.0)
for k in range(4):
    at = 0.3 + k * 0.85
    place(heartbeat(0.36), at)
    place(heartbeat(0.22), at + 0.24)
for i, m in enumerate([57, 64, 60, 64, 55, 62]):
    place(pluck_c(m, 1.4, 0.25), 0.6 + i * 0.8, gain=0.18, pan=-0.3 + 0.1 * i)
for k in range(25):  # the Roman letters typed, one every 4 frames
    place(click(0.05), (70 + k * 4) / FPS, pan=0.15)
place(bell(81, 1.8, 0.11), 200 / FPS)  # it turns into Bangla
sparkle(204 / FPS, gain=0.07)
place(whoosh(0.3, 0.12), 248 / FPS)  # sent
place(bell(76, 1.2, 0.08), 252 / FPS)
place(bell(79, 1.6, 0.09), 340 / FPS)  # her reply
for at, m, d in [(3.7, 69, 0.9), (4.6, 72, 0.6), (5.2, 71, 0.6), (5.8, 69, 1.4)]:
    place(flute(m, d, 0.1), at)
# "talk to your mother in your mother tongue"
place(pad([45, 52, 57, 60, 64, 69], 3.0, bright=1800, gain=0.2), 430 / FPS)
place(bell(84, 2.0, 0.1), 432 / FPS)
for at, m, d in [(7.4, 76, 0.7), (8.1, 74, 0.5), (8.6, 72, 0.6), (9.2, 69, 1.0)]:
    place(flute(m, d, 0.11), at)
place(riser(1.3, 0.24), M['logo'] + 30 / FPS - 1.3)

# ── 7: the name
L = M['logo'] + 30 / FPS
place(impact(0.75), L)
place(pad([53, 57, 60, 64, 69], 4.0, bright=2600, gain=0.22), L)
place(pad([41], 4.0, bright=250, gain=0.16), L)
place(bell(81, 2.0, 0.12), M['logo'] + 0.6)
for at, m, d in [(1.2, 76, 0.5), (1.7, 79, 0.5), (2.2, 81, 1.4)]:
    place(flute(m, d, 0.12), M['logo'] + at)
place(whoosh(0.5, 0.2), M['features'] - 0.25)

# ── 11–33.5: nine features
groove(M['features'], M['devices'] - 0.3)
for i in range(1, 9):
    place(whoosh(0.32, 0.13), M['features'] + i * FEAT - 0.12)
for i in range(9):
    place(bell(84 if i % 2 else 81, 0.9, 0.06), M['features'] + i * FEAT + 0.15, pan=0.2)
melody(M['features'] + 2.4)
melody(M['features'] + 12.0)
# the shortcut feature: a sparkle as each phrase appears
for off in (8 + 6 + 14 * 3 + 4, 62 + 6 + 4 * 3 + 4):  # ;alhamdulillah, ;brb
    sparkle(M['features'] + 4 * FEAT + off / FPS, gain=0.06)
place(riser(1.0, 0.18), M['devices'] - 1.0)

# ── 33.5: every device
D = M['devices']
place(impact(0.8), D + 6 / FPS)
place(pad([53, 57, 60, 64, 69, 72], 3.0, bright=3000, gain=0.22), D + 6 / FPS)
place(pad([41, 53], 3.0, bright=300, gain=0.15), D + 6 / FPS)
for i in range(5):
    place(bell([79, 81, 84, 86, 88][i], 0.9, 0.08), D + (40 + i * 10) / FPS, pan=-0.4 + i * 0.2)
place(whoosh(0.4, 0.16), M['win'] - 0.2)

# ── 36.5–60.5: Windows, then Linux
groove(M['win'] + 0.1, M['mac'] - 0.3, level=1.0, arp=4, pad_gain=0.13)
W = M['win']
place(click(0.2), W + 36 / FPS)
place(click(0.2), W + 48 / FPS)
place(bell(88, 1.2, 0.11), W + 58 / FPS)  # Bangla switches on
sparkle(W + 60 / FPS, gain=0.07)
for k in range(11):  # each Bangla word placed
    place(bell(79 + [0, 2, 4, 5, 7, 9, 7, 5, 4, 2, 0][k], 0.6, 0.045), W + (78 + k * 36 + 24) / FPS, pan=0.25)
melody(W + 1.8, gain=0.07)
place(whoosh(0.4, 0.15), M['linux'] - 0.2)
Lx = M['linux']
for at, n in [(20, 44), (140, 23), (248, 36), (342, 23)]:  # commands typed, a tick every 3 letters
    for k in range(0, n, 3):
        place(click(0.06), Lx + (at + k * 1.6) / FPS, pan=rng.uniform(-0.3, 0.3))
for at in (206, 406):  # "installed" and "Complete!"
    place(pad([60, 64, 67, 72], 1.6, bright=3000, gain=0.13), Lx + at / FPS)
    place(bell(84, 1.2, 0.09), Lx + at / FPS)
place(whoosh(0.5, 0.14), Lx + 460 / FPS)
for k in range(5):
    place(bell(81 + [0, 2, 4, 2, 5][k], 0.6, 0.05), Lx + (540 + k * 44 + 30) / FPS, pan=-0.2)
melody(Lx + 4.0, gain=0.07, up=0)
place(riser(0.9, 0.15), M['mac'] - 0.9)

# ── 60.5–91: macOS, Android, iPhone and iPad (coming later): lighter, airy
Mc = M['mac']
place(impact(0.45), Mc + 8 / FPS)
place(pad([57, 60, 64, 69, 72], 4.0, bright=3400, gain=0.17), Mc)
groove(Mc + 0.2, M['android'] - 0.4, level=0.7, snare_on=False, arp=3, pad_gain=0.1)
place(bell(86, 1.0, 0.08), Mc + 64 / FPS)  # Ekushe Bangla picked from the input menu
for k in range(7):
    place(bell(84 + [0, 3, 5, 3, 7, 5, 3][k], 0.6, 0.045), Mc + (110 + k * 40 + 27) / FPS, pan=0.3)
melody(Mc + 2.0, gain=0.065, up=5)
place(riser(1.0, 0.2), M['android'] - 1.0)

A = M['android']  # the phone scene from the teaser, same marks as music/everywhere.py
place(impact(0.7), A + 6 / FPS)
place(pad([53, 57, 60, 64, 69], 3.0, bright=2800, gain=0.18), A + 6 / FPS)
groove(A + 0.3, M['ios'] - 0.3, level=0.85, arp=3)
for k in range(18):
    place(click(0.09), A + (70 + k * 7) / FPS, pan=rng.uniform(-0.3, 0.3))
for i in range(9):
    at = A + (240 + i * 24) / FPS
    place(bell([76, 79, 81, 83, 84, 86, 88, 86, 84][i], 0.7, 0.07), at, pan=-0.4 + i * 0.1)
    place(whoosh(0.18, 0.06), at - 0.05)
for i, off in enumerate([460, 535, 610]):
    place(whoosh(0.3, 0.12), A + off / FPS - 0.1)
    place(bell(84 + i * 2, 1.0, 0.08), A + off / FPS + 0.05)
melody(A + 2.4, gain=0.07)
place(whoosh(0.5, 0.16), M['ios'] - 0.25)

I = M['ios']
place(impact(0.55), I + 6 / FPS)
place(pad([57, 61, 64, 69, 73], 4.0, bright=3400, gain=0.18), I + 6 / FPS)
groove(I + 0.3, M['engine'] - 0.6, level=0.75, snare_on=False, arp=4, pad_gain=0.1)
for i in range(9):
    place(bell([81, 84, 86, 88, 86, 84, 88, 91, 88][i], 0.7, 0.065), I + (90 + i * 44) / FPS, pan=-0.3 + i * 0.07)
melody(I + 1.6, gain=0.07, up=5)
place(riser(1.0, 0.22), M['engine'] - 1.0)

# ── 91–97: one engine
E = M['engine']
place(impact(1.0), E + 8 / FPS)
place(timpani(41, 0.5), E + 8 / FPS)
place(pad([41, 53, 57, 60, 64, 69], 5.4, bright=3200, gain=0.22), E + 8 / FPS)
sparkle(E + 0.2)
groove(E + 0.15, M['avail'] - 0.2, level=1.1, arp=4)
for at, m, d in [(0.6, 81, 0.6), (1.2, 83, 0.6), (1.8, 84, 1.2), (3.0, 81, 0.6), (3.6, 79, 1.2)]:
    place(flute(m, d, 0.11), E + at)
for i in range(4):
    place(bell(84 + i * 2, 0.8, 0.07), E + (150 + i * 12) / FPS)

# ── 97–105: when and where
V = M['avail']
place(impact(0.6), V + 34 / FPS)
place(pad([53, 57, 60, 64, 69], 3.4, bright=2600, gain=0.2), V + 34 / FPS)
place(pad([41], 3.4, bright=250, gain=0.15), V + 34 / FPS)
sparkle(V + 0.7, gain=0.07)
place(timpani(45, 0.4), V + 154 / FPS)
place(pad([45, 57, 61, 64, 69, 73], 4.4, bright=3200, gain=0.2), V + 154 / FPS)
place(pad([33, 45], 4.4, bright=300, gain=0.15), V + 154 / FPS)
for k in range(5):
    place(bell(81 + k * 2, 0.7, 0.05), V + (110 + k * 8) / FPS, pan=-0.2 + k * 0.1)
for k in range(3):
    place(bell(84 + k * 3, 0.7, 0.05), V + (180 + k * 20) / FPS, pan=0.2)
for at, m, d in [(3.0, 76, 0.6), (3.6, 79, 0.6), (4.2, 81, 1.6), (6.0, 79, 1.4)]:
    place(flute(m, d, 0.1), V + at)
for k in range(5):
    place(heartbeat(0.25), V + 4.0 + k * 0.85)
place(riser(0.9, 0.2), M['end'] - 0.9)

# ── 105–112: the ending
N = M['end']
place(impact(0.6), N)
place(pad([45, 57, 61, 64, 69], 7.0, bright=3000, gain=0.2), N)
place(pad([33, 45], 7.0, bright=250, gain=0.15), N)
place(bell(81, 2.0, 0.14), N + 0.05)
for at, m, d in [(0.3, 69, 0.9), (1.2, 72, 0.6), (1.8, 71, 0.6), (2.4, 76, 1.8), (4.4, 81, 2.4)]:
    place(flute(m, d, 0.12), N + at)
for k in range(18):
    place(click(0.07), N + 80 / FPS + k * 2 / FPS, pan=0.3)
for k in range(22):
    place(click(0.07), N + 120 / FPS + k * 2 / FPS, pan=-0.3)
sparkle(N + 70 / FPS + 0.6)
sparkle(N + 110 / FPS + 0.75)

finish('public/music/full_promo.wav')
