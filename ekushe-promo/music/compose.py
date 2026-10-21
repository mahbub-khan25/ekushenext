#!/usr/bin/env python3
"""Original score for the Ekushe Next promo, synthesised from scratch (no samples, no licences).

60 s, 48 kHz stereo, composed to the video's scene marks (src/v2/theme.ts):
  0–8 s    open       low drone, distant flute motif (A minor)
  8–18 s   martyrs    solemn pad Am–F–Dm–E, plucked arpeggio, flute melody
  18–23 s  pride      swell to C and G, timpani roll
  23–32 s  problem    tense pulsing ostinato, ticking, a dissonant cluster
  32–35 s  reveal     riser, impact, the key turns to A major
  35–49 s  demo, features   bright 120 bpm plucks, drums, keyboard clicks on each typed key
  49–54 s  speed      faster arpeggio, riser
  54–60 s  outro      anthem cadence D–E–A, long tail

python3 music/compose.py public/music/ekushe-score.wav
"""
import json
import sys
import wave

import numpy as np
from scipy.signal import butter, fftconvolve, lfilter, sosfilt

SR = 48000
DUR = 60.0
N = int(SR * DUR)
rng = np.random.default_rng(21021952)
L = np.zeros(N)
R = np.zeros(N)


def midi(m):
    return 440.0 * 2 ** ((m - 69) / 12)


def t_arr(d):
    return np.arange(int(d * SR)) / SR


def env_adsr(n, a, d, s, r, sr=SR):
    a, d, r = int(a * sr), int(d * sr), int(r * sr)
    e = np.full(n, s, dtype=float)
    a = min(a, n)
    e[:a] = np.linspace(0, 1, a, endpoint=False)
    dd = min(d, max(0, n - a))
    e[a:a + dd] = np.linspace(1, s, dd, endpoint=False)
    r = min(r, n)
    if r:
        e[n - r:] *= np.linspace(1, 0, r)
    return e


def lowpass(x, fc, order=2):
    sos = butter(order, min(fc, SR * 0.45) / (SR / 2), btype='low', output='sos')
    return sosfilt(sos, x)


def highpass(x, fc, order=2):
    sos = butter(order, fc / (SR / 2), btype='high', output='sos')
    return sosfilt(sos, x)


def place(sig, at, gain=1.0, pan=0.0):
    """Add a mono signal at time `at` (seconds), panned -1..1."""
    i = int(at * SR)
    if i >= N:
        return
    sig = sig[: N - i]
    gl = np.cos((pan + 1) * np.pi / 4)
    gr = np.sin((pan + 1) * np.pi / 4)
    L[i:i + len(sig)] += sig * gain * gl
    R[i:i + len(sig)] += sig * gain * gr


# ───────────────────────── instruments ─────────────────────────

def saw(f, d, detune=0.0):
    t = t_arr(d)
    out = np.zeros_like(t)
    ff = f * (1 + detune)
    k = 1
    while k * ff < 9000 and k < 40:
        out += np.sin(2 * np.pi * k * ff * t + rng.uniform(0, 6.28)) / k
        k += 1
    return out * 0.5


def pad(notes, d, bright=1400, gain=0.12):
    sig = np.zeros(int(d * SR))
    for m in notes:
        for det in (-0.004, 0.0, 0.005):
            sig += saw(midi(m), d, det)
    sig = lowpass(sig, bright, 2)
    sig *= env_adsr(len(sig), min(1.8, d * 0.4), 0.5, 0.85, min(2.2, d * 0.45))
    return sig * gain / max(1, len(notes))


def pluck(f, d=1.6, bright=0.5):
    """Karplus–Strong string: a little like a dotara / guitar."""
    n = int(d * SR)
    p = max(2, int(SR / f))
    buf = rng.uniform(-1, 1, p)
    buf = lowpass(buf, 2000 + 6000 * bright, 1)
    out = np.zeros(n)
    idx = 0
    damp = 0.996
    for i in range(n):
        v = buf[idx]
        nxt = buf[(idx + 1) % p]
        buf[idx] = damp * 0.5 * (v + nxt)
        out[i] = v
        idx = (idx + 1) % p
    out *= env_adsr(n, 0.002, 0.05, 1.0, 0.25)
    return out * 0.35


_PLUCK_CACHE = {}


def pluck_c(m, d=1.6, bright=0.5):
    key = (m, round(d, 2), round(bright, 2))
    if key not in _PLUCK_CACHE:
        _PLUCK_CACHE[key] = pluck(midi(m), d, bright)
    return _PLUCK_CACHE[key]


def flute(m, d, gain=0.16):
    t = t_arr(d)
    f = midi(m)
    vib = 1 + 0.006 * np.sin(2 * np.pi * 5.2 * t) * np.clip(t / 0.4, 0, 1)
    ph = 2 * np.pi * np.cumsum(f * vib) / SR
    tone = np.sin(ph) + 0.18 * np.sin(2 * ph) + 0.05 * np.sin(3 * ph)
    breath = highpass(lowpass(rng.normal(0, 1, len(t)), 3500), 900) * 0.05
    e = env_adsr(len(t), 0.09, 0.15, 0.8, min(0.35, d * 0.4))
    return (tone + breath) * e * gain


def kick(gain=0.9):
    t = t_arr(0.45)
    f = 45 + 95 * np.exp(-t * 28)
    ph = 2 * np.pi * np.cumsum(f) / SR
    return np.sin(ph) * np.exp(-t * 7.5) * gain


def shaker(gain=0.12):
    t = t_arr(0.09)
    return highpass(rng.normal(0, 1, len(t)), 6000) * np.exp(-t * 55) * gain


def snare(gain=0.3):
    t = t_arr(0.25)
    body = np.sin(2 * np.pi * 190 * t) * np.exp(-t * 30)
    noise = highpass(rng.normal(0, 1, len(t)), 1800) * np.exp(-t * 18)
    return (body * 0.5 + noise) * gain


def timpani(m, gain=0.5):
    t = t_arr(1.6)
    f = midi(m)
    s = np.sin(2 * np.pi * f * t) + 0.4 * np.sin(2 * np.pi * f * 1.5 * t) * np.exp(-t * 6)
    hit = lowpass(rng.normal(0, 1, len(t)), 500) * np.exp(-t * 25) * 0.6
    return (s * np.exp(-t * 2.2) + hit) * gain


def impact(gain=1.0):
    t = t_arr(3.5)
    boom = np.sin(2 * np.pi * np.cumsum(32 + 60 * np.exp(-t * 6)) / SR) * np.exp(-t * 1.6)
    crash = highpass(rng.normal(0, 1, len(t)), 3000) * np.exp(-t * 2.5) * 0.25
    return (boom + crash) * gain


def riser(d, gain=0.25):
    t = t_arr(d)
    noise = rng.normal(0, 1, len(t))
    out = np.zeros_like(noise)
    # sweep a band-pass upwards in chunks
    chunk = 2048
    for i in range(0, len(t), chunk):
        fc = 300 + (8000 - 300) * (i / len(t)) ** 2
        b, a = butter(2, [fc * 0.7 / (SR / 2), min(fc * 1.3, SR * 0.45) / (SR / 2)], btype='band')
        out[i:i + chunk] = lfilter(b, a, noise[i:i + chunk])
    return out * np.linspace(0, 1, len(t)) ** 2 * gain


def click(gain=0.18):
    t = t_arr(0.035)
    tick = highpass(rng.normal(0, 1, len(t)), 2500) * np.exp(-t * 180)
    thock = np.sin(2 * np.pi * 900 * t) * np.exp(-t * 260) * 0.5
    return (tick + thock) * gain


def tick(gain=0.08):
    t = t_arr(0.02)
    return np.sin(2 * np.pi * 3200 * t) * np.exp(-t * 300) * gain


# ───────────────────────── score ─────────────────────────
A2, E3, A3 = 45, 52, 57

# 0–8: drone and a distant motif
place(pad([A2, E3], 9.0, bright=500, gain=0.20), 0.0)
place(pad([A3 + 12], 8.0, bright=2500, gain=0.04), 0.5, pan=0.3)
for at, m, d in [(2.2, 76, 0.8), (3.0, 74, 0.5), (3.5, 72, 1.6), (5.6, 71, 0.6), (6.2, 69, 1.8)]:
    place(flute(m, d, 0.11), at, pan=-0.2)

# 8–18: martyrs — Am F Dm E, two seconds... then held
chords = [(8.0, [57, 60, 64]), (10.5, [53, 57, 60]), (13.0, [50, 53, 57]), (15.5, [52, 56, 59])]
for at, ns in chords:
    place(pad(ns, 2.9, bright=1500, gain=0.16), at)
    place(pad([ns[0] - 12], 2.9, bright=400, gain=0.16), at)
    for j, m in enumerate(ns + [ns[0] + 12, ns[1] + 12, ns[2]]):
        place(pluck_c(m, 1.8, 0.35), at + j * 0.36, gain=0.55, pan=-0.4 + j * 0.16)
melody = [(8.6, 69, 0.9), (9.5, 72, 0.5), (10.0, 76, 1.4), (11.6, 74, 0.5), (12.1, 72, 0.5), (12.6, 74, 1.6),
          (14.4, 77, 0.6), (15.0, 76, 0.6), (15.6, 74, 0.6), (16.2, 71, 0.6), (16.8, 69, 1.4)]
for at, m, d in melody:
    place(flute(m, d, 0.15), at, pan=0.15)

# 18–23: pride — C then G, timpani roll swell
place(pad([48, 55, 60, 64], 2.6, bright=2200, gain=0.17), 18.0)
place(pad([43, 50, 55, 59, 62], 2.6, bright=2600, gain=0.17), 20.4)
for i in range(18):
    place(timpani(43, 0.06 + i * 0.012), 20.3 + i * 0.13)
place(timpani(45, 0.8), 18.0)
for at, m, d in [(18.2, 76, 0.5), (18.7, 79, 0.5), (19.2, 84, 1.6), (20.9, 83, 0.6), (21.5, 79, 1.2)]:
    place(flute(m, d, 0.14), at)

# 23–32: problem — pulsing ostinato, ticks, dissonance
for i in range(int((32 - 23) / 0.25)):
    at = 23 + i * 0.25
    m = [45, 45, 46, 45][i % 4]
    place(lowpass(saw(midi(m), 0.22), 700) * env_adsr(int(0.22 * SR), 0.005, 0.05, 0.6, 0.08) * 0.18, at)
    place(tick(0.05 if i % 2 else 0.08), at, pan=0.5 if i % 2 else -0.5)
place(pad([57, 58, 63], 9.0, bright=900, gain=0.10), 23.0)
for i in range(9):
    place(kick(0.5), 23 + i * 1.0)
place(pad([57, 58, 59, 60], 3.0, bright=2000, gain=0.06), 29.0)

# 31–32: riser, 32: impact, key changes to A major
place(riser(1.9, 0.35), 30.1)
place(impact(0.7), 32.0)
place(pad([57, 61, 64, 69], 3.0, bright=3000, gain=0.20), 32.0)
place(pad([33, 45], 3.0, bright=300, gain=0.25), 32.0)

# 35–49: bright 120 bpm (beat = 0.5 s) A – F#m – D – E
prog = [[57, 61, 64], [54, 57, 61], [50, 54, 57], [52, 56, 59]]
beat = 0.5
for bar in range(7):
    at0 = 35.0 + bar * 2.0
    ns = prog[bar % 4]
    place(pad(ns + [ns[0] + 12], 2.05, bright=3200, gain=0.11), at0)
    place(pad([ns[0] - 12], 2.05, bright=500, gain=0.14), at0)
    arp = [ns[0] + 12, ns[1] + 12, ns[2] + 12, ns[1] + 12]
    for s in range(8):
        place(pluck_c(arp[s % 4], 0.9, 0.7), at0 + s * 0.25, gain=0.42, pan=-0.35 if s % 2 else 0.35)
    for b in range(4):
        place(kick(0.75 if b % 2 == 0 else 0.55), at0 + b * beat)
        place(shaker(0.10), at0 + b * beat + 0.25, pan=0.4)
        if bar >= 3 and b % 2 == 1:
            place(snare(0.26), at0 + b * beat)
lead = [(42.0, 69, 0.5), (42.5, 73, 0.5), (43.0, 76, 1.0), (44.0, 78, 0.5), (44.5, 76, 0.5), (45.0, 74, 1.0),
        (46.0, 73, 0.5), (46.5, 74, 0.5), (47.0, 76, 1.5)]
for at, m, d in lead:
    place(flute(m, d, 0.13), at)

# keyboard clicks on every typed key of the demo (scene starts at frame 2100; keys every 13 frames)
try:
    steps = json.load(open('src/v2/demoTrace.json'))
except FileNotFoundError:
    steps = [None] * 27
for i in range(len(steps)):
    at = (2100 + 50 + i * 13) / 60.0
    place(click(0.20 if i % 5 else 0.24), at, pan=rng.uniform(-0.3, 0.3))

# 49–54: speed — faster arpeggio and a riser
for i in range(int(5.0 / 0.125)):
    at = 49.0 + i * 0.125
    ns = [57, 61, 64, 69, 73, 76]
    place(pluck_c(ns[i % 6], 0.5, 0.9), at, gain=0.3, pan=np.sin(i) * 0.5)
place(pad([57, 64, 69, 73], 5.0, bright=4000, gain=0.12), 49.0)
for b in range(10):
    place(kick(0.6), 49.0 + b * 0.5)
    place(shaker(0.12), 49.25 + b * 0.5)
place(riser(1.6, 0.3), 52.4)

# 54–60: outro cadence D – E – A, long tail
place(impact(0.6), 54.0)
place(pad([50, 54, 57, 62], 2.0, bright=3000, gain=0.17), 54.0)
place(pad([52, 56, 59, 64], 2.0, bright=3000, gain=0.17), 56.0)
place(pad([45, 57, 61, 64, 69], 4.0, bright=2600, gain=0.2), 58.0 - 0.0)
for at, m, d in [(54.2, 74, 0.6), (54.8, 76, 0.6), (55.4, 78, 0.6), (56.0, 80, 1.2), (57.3, 81, 2.6)]:
    place(flute(m, d, 0.15), at)
for j, m in enumerate([57, 61, 64, 69, 73, 76]):
    place(pluck_c(m, 2.0, 0.5), 58.0 + j * 0.12, gain=0.4, pan=-0.5 + j * 0.2)
place(timpani(45, 0.6), 58.0)

# ───────────────────────── mix: reverb, glue, master ─────────────────────────
ir_len = int(2.8 * SR)
tt = np.arange(ir_len) / SR
ir_l = rng.normal(0, 1, ir_len) * np.exp(-tt * 2.6)
ir_r = rng.normal(0, 1, ir_len) * np.exp(-tt * 2.6)
ir_l, ir_r = lowpass(ir_l, 6000), lowpass(ir_r, 6000)
ir_l /= np.sqrt(np.sum(ir_l ** 2))
ir_r /= np.sqrt(np.sum(ir_r ** 2))
wet_l = fftconvolve(L, ir_l)[:N]
wet_r = fftconvolve(R, ir_r)[:N]
# lift the quiet opening (0–23 s) so it is not lost under the later drums
lift = np.interp(np.arange(N) / SR, [0, 22.0, 23.5, 60], [1.7, 1.7, 1.0, 1.0])
L *= lift
R *= lift
wet_l *= lift
wet_r *= lift
mixL = L * 0.82 + wet_l * 0.38
mixR = R * 0.82 + wet_r * 0.38

# fade the very end, gentle saturation, normalise to -1 dBFS
fade = np.ones(N)
fade[-int(1.2 * SR):] = np.linspace(1, 0, int(1.2 * SR)) ** 1.5
mixL *= fade
mixR *= fade
peak = max(np.max(np.abs(mixL)), np.max(np.abs(mixR)))
mixL = np.tanh(mixL / peak * 1.4) / np.tanh(1.4)
mixR = np.tanh(mixR / peak * 1.4) / np.tanh(1.4)
gain = 10 ** (-1 / 20) / max(np.max(np.abs(mixL)), np.max(np.abs(mixR)))
stereo = np.stack([mixL * gain, mixR * gain], axis=1)

out = sys.argv[1] if len(sys.argv) > 1 else 'public/music/ekushe-score.wav'
with wave.open(out, 'wb') as w:
    w.setnchannels(2)
    w.setsampwidth(2)
    w.setframerate(SR)
    w.writeframes((stereo * 32767).astype('<i2').tobytes())
print('wrote', out)
