#!/usr/bin/env python3
"""Music for the last teaser (src/teasers/FinalTeaser.tsx): the scores of teasers 3, 4 and 5 joined
at the same marks, each running a moment past its join and fading under the next.

python3 music/final_teaser.py
"""
import wave
import numpy as np

FPS = 60
J4, J5 = 1140, 1140 + 1060     # must match FinalTeaser.tsx
TAIL, FADE_IN = 0.35, 0.03     # seconds


def read(p):
    w = wave.open(p)
    sr, ch = w.getframerate(), w.getnchannels()
    x = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(np.float64) / 32768
    return sr, x.reshape(-1, ch)


sr, t3 = read('public/music/teaser3.wav')
_, t4 = read('public/music/teaser4.wav')
_, t5 = read('public/music/teaser5.wav')
total = int((J5 / FPS) * sr) + len(t5)
out = np.zeros((total, t3.shape[1]))


def put(x, start_s, stop_s=None):
    a = int(start_s * sr)
    x = x.copy()
    n_in = int(FADE_IN * sr)
    x[:n_in] *= np.linspace(0, 1, n_in)[:, None]
    if stop_s is not None:
        keep = int((stop_s - start_s + TAIL) * sr)
        x = x[:keep]
        n_out = int(TAIL * 2 * sr)
        x[-n_out:] *= np.linspace(1, 0, n_out)[:, None] ** 2
    out[a:a + len(x)] += x[: total - a]


put(t3, 0, J4 / FPS)
put(t4, J4 / FPS, J5 / FPS)
put(t5, J5 / FPS)
peak = np.abs(out).max()
if peak > 0.97:
    out *= 0.97 / peak
w = wave.open('public/music/final_teaser.wav', 'wb')
w.setnchannels(out.shape[1]); w.setsampwidth(2); w.setframerate(sr)
w.writeframes((out * 32767).astype(np.int16).tobytes())
w.close()
print('wrote public/music/final_teaser.wav', round(total / sr, 2), 's')
