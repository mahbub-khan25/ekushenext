#!/usr/bin/env python3
"""Score for the website film "এক বার্তা, সব পর্দা" (promo-beta5), 47.7 s. Original music, from the
films' shared instruments (compose.py / tlib.py), so it sounds related to the launch film.

  0–30 s   a warm, quiet groove (F C G Am) under the four devices: a soft click on every typed letter,
           a whoosh out of each blue chip and a darker one into the next
  30–41 s  Dhaka: the groove thins to a pad while Ma reads; a bell on each arriving line, the theme
           on the flute as she answers, the lift as the title lands
  41–48 s  the end card: a soft hit, the theme, sparkles

python3 music/beta5_film.py  →  ../promo-beta5/assets/music.wav
"""
import json
import os
import sys

HERE = os.path.dirname(__file__)
os.environ['TEASER_DUR'] = '48.2'
sys.path.insert(0, HERE)
from tlib import *  # noqa: E402,F403

FILM = os.path.join(HERE, '..', '..', 'promo-beta5')
_js = open(os.path.join(FILM, 'assets', 'trace.js'), encoding='utf-8').read()
TR = json.loads(_js.split('= ', 1)[1].strip().rstrip(';'))  # the film's typed lines
s = lambda fr: fr / 30  # noqa: E731  (the film is 30 fps)
EVERY, GAP = 3, 6
SC = {'win': (0, 186), 'mac': (186, 219), 'phone': (405, 252), 'tab': (657, 243), 'mom': (900, 322), 'end': (1222, 210)}
beat = 60 / 92
prog = [53, 48, 55, 57]  # F C G Am


def typed(at, words, gain=0.045, pan=0.1):
    """A click on every letter, on the film's frames (3 a letter, 6 between words)"""
    t = at
    for k, w in enumerate(words):
        for i in range(len(w['r'])):
            place(click(gain), s(t + i * EVERY), pan=pan)
        t += len(w['r']) * EVERY + GAP
    return t - GAP  # the last letter


def groove(start, end, level=1.0):
    nb = int((end - start) / beat)
    for b in range(nb):
        at = start + b * beat
        root = prog[(b // 4) % 4]
        place(kick(0.32 * level if b % 2 == 0 else 0.2 * level), at)
        place(shaker(0.03 * level), at + beat / 2, pan=0.3)
        chord = [root + 12, root + 16 if root != 57 else root + 15, root + 19]
        for j, m in enumerate(chord):
            place(pluck_c(m, 0.9, 0.45), at + j * beat / 3, gain=0.12 * level, pan=-0.3 + j * 0.3)
        if b % 4 == 0:
            place(pad([root, chord[1] - 12, root + 7], beat * 4, bright=1800, gain=0.10), at)
            place(pad([root - 12], beat * 4, bright=300, gain=0.10), at)


THEME = [(0, 76, 0.6), (0.6, 77, 0.6), (1.2, 79, 1.2), (2.4, 76, 1.2), (4.8, 72, 0.6), (5.4, 74, 0.6), (6.0, 76, 1.6)]

# the four devices
groove(0.0, s(SC['mom'][0]) + 0.2, level=0.9)
for k, key in enumerate(('win', 'mac', 'phone', 'tab')):
    at, n = SC[key]
    if k:
        place(whoosh(0.4, 0.10, up=False), s(at) - 0.05)  # out of the chip
    typed(at - 2, TR[key])
    place(whoosh(0.5 if k % 2 else 0.42, 0.13, up=True), s(at + n - 21))  # into the next chip
    place(bell(84 + (k % 2) * 3, 0.9, 0.05), s(at + n - 21 - 60))  # the title lands
for t0, m, d in THEME:
    place(flute(m, d, 0.05), s(SC['mac'][0]) + 0.6 + t0)

# Dhaka: Ma reads and answers
M = SC['mom'][0]
place(whoosh(0.4, 0.09, up=False), s(M) - 0.05)
place(pad([45, 52, 57, 60], 11.0, bright=1500, gain=0.16), s(M))
for i in range(4):
    place(bell(79 + [0, 2, 4, 7][i], 1.0, 0.06), s(M + 40 + i * 12), pan=-0.3 + i * 0.2)
done = typed(M + 90, TR['mom'], gain=0.04, pan=-0.1)
sent = done + 3 + 12
place(bell(88, 1.6, 0.09), s(sent)); sparkle(s(sent + 2), gain=0.05)
for t0, m, d in THEME:
    place(flute(m - 5, d, 0.07), s(M + 90) + t0 * 0.9)
place(pad([53, 57, 60, 65], 3.6, bright=2600, gain=0.16), s(sent + 30))  # the title
place(whoosh(0.5, 0.14, up=True), s(M + 322 - 21))

# the end card
E = SC['end'][0]
end_card(s(E))
place(pad([53, 60, 65, 69], 4.0, bright=1600, gain=0.12), s(E + 90))

finish(os.path.join(FILM, 'assets', 'music.wav'))
