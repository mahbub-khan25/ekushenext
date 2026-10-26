#!/usr/bin/env python3
"""Score for the launch-day reveal (src/teasers/LaunchReveal.tsx), 84 s. Original music.

  0–8 s     the dark: a low drone, a slow heartbeat, a few far bells
  8–16 s    the sun rises: a long riser and a timpani roll, then a wide warm chord as the Minar lights
  16–27 s   "30 October" typed and turned to Bangla, "it's finally here", the name
  27–66 s   the hopeful groove (F C G Am) under the computers and phones: a click on every letter,
            a soft bell each time a word is picked, a chime when a line is done
  66–76 s   the countdown on timpani, the boom, fireworks, the groove at full lift
  76–84 s   the ending: a hit, the theme, sparkles as the website types in

python3 music/launch_reveal.py
"""
import json
import os
import sys

HERE = os.path.dirname(__file__)
CFG = json.load(open(os.path.join(HERE, '..', 'src', 'teasers', 'launch_lines.json'), encoding='utf-8'))
os.environ['TEASER_DUR'] = str(CFG['R']['total'] / 60 + 0.4)
sys.path.insert(0, HERE)
from tlib import *  # noqa: E402,F403

RR = CFG['R']

s = lambda fr: fr / FPS  # noqa: E731
beat = 60 / 100
prog = [53, 48, 55, 57]  # F C G Am


def heartbeat(gain=0.5):
    t = t_arr(0.5)
    one = np.sin(2 * np.pi * (48 + 30 * np.exp(-t * 30)) * t) * np.exp(-t * 14)
    return lowpass(one, 400) * gain


def groove(start, end, level=1.0, arp=3, pad_gain=0.11):
    nb = int((end - start) / beat)
    for b in range(nb):
        at = start + b * beat
        root = prog[(b // 4) % 4]
        place(kick((0.5 if b % 2 == 0 else 0.32) * level), at)
        if b % 2 == 1:
            place(snare(0.14 * level), at)
        place(shaker(0.045 * level), at + beat / 2, pan=0.3)
        chord = [root + 12, root + 16 if root != 57 else root + 15, root + 19, root + 24][:arp]
        for j, m in enumerate(chord):
            place(pluck_c(m, 0.8, 0.5), at + j * beat / len(chord), gain=0.15 * level, pan=-0.35 + j * 0.25)
        if b % 4 == 0:
            place(pad([root, chord[1] - 12, root + 7], beat * 4, bright=2200, gain=pad_gain), at)
            place(pad([root - 12], beat * 4, bright=300, gain=0.12), at)


MELODY = [(0, 76, 0.6), (0.6, 77, 0.6), (1.2, 79, 1.2), (2.4, 76, 1.2), (4.8, 72, 0.6), (5.4, 74, 0.6), (6.0, 76, 1.2), (7.2, 74, 1.2)]


def melody(at, gain=0.07, up=0):
    for t0, m, d in MELODY:
        place(flute(m + up, d, gain), at + t0)


def typed(start_frame, tokens, gain=0.055, pan=0.12):
    """A click on every letter and a bell on every picked word, on the same frames as the film."""
    t = start_frame
    for k, (roman, _) in enumerate(tokens):
        for i in range(len(roman)):
            place(click(gain), s(t + i * CFG['every']), pan=pan)
        e = t + len(roman) * CFG['every']
        place(bell(81 + (k % 3) * 2, 0.6, 0.035), s(e), pan=-pan)
        t = e + CFG['gap']
    return t - 4  # when the last word is committed


# ── 0–8 s: the dark
place(pad([33, 40, 45], s(RR['sun']) + 1.0, bright=380, gain=0.2), 0.0)
place(pad([57, 64], s(RR['sun']) + 1.0, bright=900, gain=0.06), 1.5)
for k in range(7):
    place(heartbeat(0.3), 0.6 + k * 1.1)
    place(heartbeat(0.18), 0.84 + k * 1.1)
place(bell(76, 3.0, 0.05), s(110), pan=-0.4)
place(bell(81, 3.0, 0.045), s(300), pan=0.4)

# ── 8–16 s: the sun rises and the Minar lights
place(riser(s(RR['date'] - RR['sun']), 0.22), s(RR['sun']))
for k in range(28):
    place(timpani(33, 0.06 + k * 0.006), s(RR['sun'] + 120) + k * 0.12)
place(pad([45, 52, 57, 60, 64], 4.0, bright=1500, gain=0.14), s(RR['sun'] + 160))
place(impact(0.75), s(RR['date'] - 40))
place(pad([41, 53, 57, 60, 65, 69], 5.0, bright=2800, gain=0.22), s(RR['date'] - 40))
place(bell(84, 2.6, 0.1), s(RR['date'] - 30))
melody(s(RR['date'] - 20), gain=0.06)

# ── 16–22 s: 30 October
D = RR['date']
for i in range(10):
    place(click(0.06), s(D + 20 + i * 6), pan=0.1)
place(bell(81, 1.8, 0.11), s(D + 110)); sparkle(s(D + 112), gain=0.07)
place(timpani(41, 0.35), s(D + 110))
for i in range(16):
    place(click(0.04), s(D + 170 + i * 4), pan=-0.1)
place(bell(84, 1.6, 0.09), s(D + 224)); sparkle(s(D + 226), gain=0.06)

# ── 22–27 s: the name
N = RR['name']
place(riser(1.0, 0.18), s(N) - 1.0)
place(impact(0.65), s(N + 8))
place(pad([53, 57, 60, 64, 69], 3.4, bright=2600, gain=0.2), s(N + 8))
place(bell(81, 2.0, 0.1), s(N + 14))
for k, fr in enumerate((110, 122, 134)):
    place(bell(84 + [0, 4, 7][k], 0.8, 0.06), s(N + fr), pan=-0.4 + k * 0.4)

# ── 27–66 s: the computers and the phones, over the groove
groove(s(RR['win']), s(RR['party']) - 0.05, level=0.85)
for key in ('win', 'mac', 'linux'):
    at = RR[key]
    place(whoosh(0.32, 0.12), s(at) - 0.1)
    if key == 'mac':
        place(bell(88, 0.9, 0.07), s(at + 40)); sparkle(s(at + 42), gain=0.05)  # "new"
    if key == 'linux':
        place(click(0.08), s(at + CFG[key]['start'] - 44)); place(click(0.08), s(at + CFG[key]['start'] - 38))  # double Shift
    done = typed(at + CFG[key]['start'], CFG[key]['tokens'])
    place(bell(84, 1.6, 0.09), s(done)); sparkle(s(done + 2), gain=0.06)
melody(s(RR['mac']) + 0.4, gain=0.05)
melody(s(RR['linux']) + 0.4, gain=0.05, up=2)
P = RR['phones']
place(whoosh(0.32, 0.12), s(P) - 0.1)
for k, key in enumerate(('android', 'ios')):
    done = typed(P + CFG['phone'][key], CFG['phone']['tokens'], gain=0.045, pan=-0.3 if k == 0 else 0.3)
    place(whoosh(0.2, 0.07), s(done + 16))
    place(bell(79 + k * 5, 1.0, 0.06), s(done + 20))
place(bell(88, 1.0, 0.07), s(P + 60)); place(bell(91, 1.0, 0.05), s(P + 140))  # "coming soon"

# ── 66–76 s: countdown, boom, fireworks
Q = RR['party']
for k, fr in enumerate((0, 60, 120)):
    place(timpani(36 + k * 2, 0.55), s(Q + fr))
    place(bell(72 + k * 2, 0.8, 0.07), s(Q + fr))
place(riser(1.0, 0.25), s(Q + 180) - 1.0)
place(impact(1.0), s(Q + 180))
place(pad([53, 57, 60, 64, 69, 72], 4.0, bright=3200, gain=0.22), s(Q + 180))
for fr in (180, 200, 214, 262, 290, 330, 372, 396, 440):
    place(whoosh(0.4, 0.05), s(Q + fr - 34))
    place(snare(0.12), s(Q + fr)); sparkle(s(Q + fr), gain=0.06)
for k in range(4):
    place(bell(84 + [0, 2, 4, 7][k], 0.7, 0.07), s(Q + 268 + k * 14), pan=-0.4 + k * 0.27)  # the key caps flip
groove(s(Q + 180), s(RR['end']) - 0.05, level=1.1, arp=4, pad_gain=0.13)
melody(s(Q + 200), gain=0.08, up=0)

# ── 76–84 s: download today
E = RR['end']
place(impact(0.7), s(E + 6))
place(pad([53, 57, 60, 64, 69, 72], 4.0, bright=3000, gain=0.2), s(E + 6))
place(pad([41, 53], 4.0, bright=300, gain=0.15), s(E + 6))
melody(s(E) + 0.3, gain=0.09)
place(bell(84, 2.0, 0.1), s(E + 70)); sparkle(s(E + 72), gain=0.07)
for k in range(18):
    place(click(0.05), s(E + 210 + k * 2), pan=0.3)  # the website types in
sparkle(s(E + 250))
place(pad([53, 60, 65, 69], 3.4, bright=1600, gain=0.14), s(E + 300))

finish('public/music/launch_reveal.wav')
