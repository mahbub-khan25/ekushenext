"""Shared instruments for the teaser scores. Set TEASER_DUR (seconds) before importing:

    import os; os.environ['TEASER_DUR'] = '20'
    from tlib import *
    ... place(...) ...
    finish('public/music/teaserN.wav')

Everything comes from compose.py (the film's score), so all the music sounds related.
"""
import os
import pathlib

_src = (pathlib.Path(__file__).parent / 'compose.py').read_text()
_head, _rest = _src.split('# ───────────────────────── score ─────────────────────────')
_mix = _rest.split('# ───────────────────────── mix: reverb, glue, master ─────────────────────────')[1]
exec(_head.replace('DUR = 60.0', 'DUR = %s' % float(os.environ.get('TEASER_DUR', '20'))))
FPS = 60


def bell(m, d=2.2, gain=0.12):
    t = t_arr(d)
    f = midi(m)
    tone = sum(a * np.sin(2 * np.pi * f * k * t) * np.exp(-t * dec) for k, a, dec in [(1, 1, 1.6), (2.76, 0.35, 3.0), (5.4, 0.12, 5.0)])
    return tone * env_adsr(len(t), 0.004, 0.1, 1.0, 0.5) * gain


def glitch(d=0.18, gain=0.16):
    n = int(d * SR)
    x = np.round(rng.normal(0, 1, n) * 3) / 3
    x = np.repeat(x[::24], 24)[:n]
    x *= (np.sin(2 * np.pi * 37 * t_arr(d)) > 0)
    return highpass(x, 600) * env_adsr(n, 0.002, 0.02, 0.8, 0.03) * gain


def whoosh(d=0.35, gain=0.18, up=True):
    n = int(d * SR)
    x = rng.normal(0, 1, n)
    out = np.zeros(n)
    cuts = np.linspace(600, 6000, 8) if up else np.linspace(6000, 600, 8)
    for i, fc in enumerate(cuts):
        seg = slice(i * n // 8, (i + 1) * n // 8)
        out[seg] = lowpass(x, fc)[seg]
    return out * np.sin(np.linspace(0, np.pi, n)) * gain


def sparkle(at, gain=0.09):
    for j, m in enumerate([88, 91, 95, 100]):
        place(bell(m, 1.0, gain), at + j * 0.045, pan=-0.4 + j * 0.27)


def end_card(at):
    """The shared ending (src/teasers/common.tsx EndCard): logo hit, then a sparkle as the website
    and the email type in. Frames 58 and 96 of the card."""
    place(impact(0.45), at)
    place(pad([45, 57, 61, 64, 69], 3.2, bright=3000, gain=0.20), at)
    place(bell(81, 2.0, 0.14), at + 0.05)
    place(flute(81, 1.4, 0.12), at + 0.15)
    place(flute(83, 0.5, 0.10), at + 1.6)
    place(flute(85, 1.2, 0.10), at + 2.1)
    for k in range(18):
        place(click(0.07), at + 68 / FPS + k * 2 / FPS, pan=0.3)
    for k in range(22):
        place(click(0.07), at + 106 / FPS + k * 2 / FPS, pan=-0.3)
    sparkle(at + 58 / FPS)
    sparkle(at + 96 / FPS)


def finish(out):
    g = dict(globals())
    exec(_mix.replace("lift = np.interp(np.arange(N) / SR, [0, 22.0, 23.5, 60], [1.7, 1.7, 1.0, 1.0])", "lift = np.ones(N)")
         .replace("out = sys.argv[1] if len(sys.argv) > 1 else 'public/music/ekushe-score.wav'", "out = %r" % out), g)
