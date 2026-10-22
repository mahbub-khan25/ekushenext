#!/usr/bin/env python3
"""Score for the real-example film: "আমার সোনার বাংলা" typed live with Ekushe Next (about 38 s).
An original, gentle score from the teasers' instruments (it does not quote the anthem's melody).

  0–2.5 s     title: warm D major swell, a bell
  2.5–7.2 s   Shift Shift: suspended pad, a chime when Bangla turns on
  7.2–12.3 s  the title is typed: plucked arpeggios, a soft click on every real keystroke
  12.3–20.4 s time-lapse of the whole anthem: the music moves (light pulse, rising chords)
  20.9–29.6 s the last line, keystroke by keystroke; a tick on each Tab; a bell on the final ॥
  29.6–35.2 s the whole note: full chord, timpani swell, flute
  35.2 s–     the shared ending

Timing marks come from music/anthem_marks.json (computed from src/teasers/Anthem.tsx).
python3 music/anthem.py
"""
import json
import os
import sys

HERE = os.path.dirname(__file__)
M = json.load(open(os.path.join(HERE, 'anthem_marks.json')))
os.environ['TEASER_DUR'] = '%.2f' % (M['TOTAL'] / 60 + 0.6)
sys.path.insert(0, HERE)
from tlib import *  # noqa: E402,F403

s = lambda fr: fr / FPS  # noqa: E731
LAPSE, LAPSE_END, LAST, REVEAL, END = s(M['LAPSE_AT']), s(M['LAPSE_END']), s(M['LAST_AT']), s(M['REVEAL']), s(M['END'])
D2, A2, D3 = 38, 45, 50

# title
place(pad([D3, 57, 62, 66], 3.0, bright=2200, gain=0.17), 0.0)
place(pad([D2], 3.0, bright=300, gain=0.16), 0.0)
place(bell(78, 2.4, 0.13), 0.12)
for at, m, d in [(0.5, 74, 0.5), (1.0, 78, 0.5), (1.5, 81, 1.1)]:
    place(flute(m, d, 0.11), at, pan=0.15)

# Shift Shift: suspended, then Bangla turns on
place(pad([D3, 55, 57, 64], 4.9, bright=1300, gain=0.13), 2.5)
place(pad([D2 + 5], 4.9, bright=280, gain=0.13), 2.5)
place(riser(1.2, 0.10), s(280) - 1.2)
for j, m in enumerate([86, 90, 93]):
    place(bell(m, 1.4, 0.08), s(284) + j * 0.06, pan=-0.3 + j * 0.3)

# the title is typed
title_chords = [(7.2, [50, 54, 57]), (8.5, [47, 50, 54]), (9.8, [43, 47, 50]), (11.1, [45, 49, 52])]
for at, ns in title_chords:
    place(pad(ns + [ns[0] + 12], 1.4, bright=1500, gain=0.13), at)
    place(pad([ns[0] - 12], 1.4, bright=320, gain=0.14), at)
    for j, m in enumerate([ns[0] + 12, ns[1] + 12, ns[2] + 12, ns[1] + 24]):
        place(pluck_c(m, 1.2, 0.4), at + j * 0.32, gain=0.36, pan=-0.3 + j * 0.2)

# time-lapse: movement
lapse_chords = [[50, 54, 57], [45, 49, 52], [47, 50, 54], [43, 47, 50], [50, 54, 57], [45, 49, 52]]
step = (LAPSE_END - LAPSE) / len(lapse_chords)
for i, ns in enumerate(lapse_chords):
    at = LAPSE + i * step
    place(pad(ns + [ns[0] + 12], step + 0.1, bright=1800 + i * 150, gain=0.14), at)
    place(pad([ns[0] - 12], step + 0.1, bright=360, gain=0.15), at)
    for j in range(8):
        m = (ns + [ns[0] + 12, ns[1] + 12])[j % 5] + 12
        place(pluck_c(m, 0.7, 0.5), at + j * step / 8, gain=0.28, pan=-0.4 + (j % 4) * 0.27)
    for j in range(int(step / 0.25)):
        place(shaker(0.05 + 0.02 * (j % 2)), at + j * 0.25)
    if i % 2 == 0:
        place(kick(0.35), at)
for at, m, d in [(LAPSE + 1.0, 74, 0.8), (LAPSE + 2.0, 76, 0.6), (LAPSE + 2.7, 78, 1.2), (LAPSE + 4.6, 81, 0.6), (LAPSE + 5.3, 78, 0.6), (LAPSE + 6.0, 76, 1.4)]:
    place(flute(m, d, 0.10), at, pan=0.2)
place(whoosh(0.5, 0.12, up=False), LAPSE_END - 0.1)

# the last line: calm again
place(pad([47, 50, 54, 57], 4.4, bright=1200, gain=0.12), LAST)
place(pad([35], 4.4, bright=280, gain=0.13), LAST)
place(pad([43, 47, 50, 54], 4.4, bright=1200, gain=0.12), LAST + 4.4)
place(pad([31], 4.4, bright=280, gain=0.13), LAST + 4.4)
for t in M['tabs']:
    place(bell(83, 0.8, 0.06), s(t) + 0.02, pan=0.35)
place(bell(81, 2.6, 0.13), s(M['fin']))
place(flute(78, 1.4, 0.10), s(M['fin']) + 0.1)

# every real keystroke: a soft click
for i, k in enumerate(M['keys']):
    place(click(0.15), s(k), pan=-0.25 + 0.5 * (i % 2))

# the whole note
place(riser(1.0, 0.16), REVEAL - 1.0)
place(impact(0.30), REVEAL + 50 / FPS)
place(pad([D3, 57, 62, 66, 69], END - REVEAL, bright=2600, gain=0.17), REVEAL)
place(pad([D2], END - REVEAL, bright=320, gain=0.16), REVEAL)
for i in range(12):
    place(timpani(38, 0.04 + i * 0.01), REVEAL + 0.2 + i * 0.1)
for at, m, d in [(REVEAL + 1.0, 78, 0.6), (REVEAL + 1.6, 81, 0.6), (REVEAL + 2.2, 86, 1.2), (REVEAL + 3.6, 83, 0.6), (REVEAL + 4.2, 81, 1.0)]:
    place(flute(m, d, 0.12), at)
place(riser(1.0, 0.18), END - 1.0)

# the ending: logo, website, email
end_card(END)

finish(os.path.join(HERE, '..', 'public', 'music', 'anthem.wav'))
