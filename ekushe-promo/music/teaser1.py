#!/usr/bin/env python3
"""Score for Teaser 1, "The language they gave their lives for" (15 s). Uses the promo's
instruments (music/compose.py) and its flute motif, so the teasers and the film sound related.

  0–2.5 s    drone and a distant flute (A minor)
  2.5–11 s   Am F Dm E with plucked arpeggios; a soft bell as each martyr's name appears
  11.3–13 s  lift to C then G
  13.2–15 s  A major for the logo, long tail

python3 music/teaser1.py public/music/teaser1.wav
"""
import os
import sys

os.environ['TEASER_DUR'] = '20.2'
sys.path.insert(0, os.path.dirname(__file__))
from tlib import *  # noqa: E402,F403

NAME_AT = [(380 + i * 40) / FPS for i in range(5)]  # these must match src/teasers/Teaser1.tsx
KEY_AT = [(800 + 24 + i * 7) / FPS for i in range(28)]
LOGO = 1025 / FPS


A2, E3 = 45, 52
place(pad([A2, E3], 3.2, bright=450, gain=0.22), 0.0)
place(pad([69 + 12], 3.0, bright=2400, gain=0.035), 0.3, pan=0.3)
for at, m, d in [(0.6, 76, 0.7), (1.3, 74, 0.45), (1.75, 72, 1.3)]:
    place(flute(m, d, 0.10), at, pan=-0.2)

chords = [(2.5, [57, 60, 64]), (5.0, [53, 57, 60]), (7.5, [50, 53, 57]), (9.6, [52, 56, 59])]
for at, ns in chords:
    place(pad(ns, 2.8, bright=1500, gain=0.16), at)
    place(pad([ns[0] - 12], 2.8, bright=380, gain=0.17), at)
    for j, m in enumerate(ns + [ns[0] + 12, ns[1] + 12]):
        place(pluck_c(m, 1.8, 0.35), at + j * 0.36, gain=0.5, pan=-0.4 + j * 0.2)
for at, m, d in [(3.0, 69, 0.9), (3.9, 72, 0.5), (4.4, 76, 1.4), (5.9, 74, 0.5), (6.4, 72, 0.5), (6.9, 74, 1.3)]:
    place(flute(m, d, 0.14), at, pan=0.15)
for i, at in enumerate(NAME_AT):
    place(bell([81, 79, 76, 74, 76][i]), at, pan=-0.3 + i * 0.15)
place(flute(71, 0.7, 0.12), 10.0); place(flute(69, 1.2, 0.12), 10.7)

# the promise: C then G, a small timpani swell
place(pad([48, 55, 60, 64], 1.2, bright=2200, gain=0.17), 11.3)
place(pad([43, 50, 55, 59, 62], 1.3, bright=2600, gain=0.17), 12.4)
for i in range(10):
    place(timpani(43, 0.05 + i * 0.012), 12.3 + i * 0.09)
for at, m, d in [(11.4, 76, 0.45), (11.85, 79, 0.45), (12.3, 84, 0.9)]:
    place(flute(m, d, 0.13), at)

# the glimpse: quiet and suspended (F then G, no drums), a soft click on every real keystroke
place(pad([53, 57, 60, 65], 2.0, bright=1800, gain=0.13), 13.3)
place(pad([55, 59, 62, 67], 2.0, bright=2000, gain=0.13), 15.2)
place(pad([41], 3.9, bright=300, gain=0.12), 13.3)
for i, at in enumerate(KEY_AT):
    place(click(0.16), at, pan=-0.25 + 0.5 * (i % 2))
for j, m in enumerate([72, 76, 79, 76, 74, 79, 83, 79]):
    place(pluck_c(m, 1.2, 0.45), 13.4 + j * 0.45, gain=0.32, pan=0.3 - j * 0.08)
place(riser(1.4, 0.22), LOGO - 1.4)

# the ending: logo, website, email
end_card(LOGO)

finish('public/music/teaser1.wav')
