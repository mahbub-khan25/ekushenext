#!/usr/bin/env python3
"""Reference ranker for Ekushe's Roman → Bangla lookup (Phase 5 ports this to Rust).

score = alignment cost (typed text vs the word's best romanization)
        - FREQ_W * log10(ppm)            more common words rank higher
        + class penalty                  variant / colloquial / sadhu rank below the standard form
Lookup: skeleton(typed) bucket; if it is small, also buckets one skeleton edit away (typo tolerance).
"""
import math, os, sys, collections
sys.path.insert(0, os.path.dirname(__file__))
from romanize import skeleton

FREQ_W = 0.55
FINAL_VOWEL = 0.8
CLASS_PENALTY = {"standard": 0.0, "variant": 0.9, "colloquial": 0.7, "sadhu": 1.2, "vulgar": 1.5}
VOWELS = set("aeiou")

def wdist(a, b):
    """Weighted edit distance tuned for Banglish: vowel swaps, h, y/w and doubled letters are cheap."""
    la, lb = len(a), len(b)
    prev = [j * 1.0 for j in range(lb + 1)]
    for i in range(1, la + 1):
        cur = [i * 1.0] + [0.0] * lb
        ca = a[i-1]
        for j in range(1, lb + 1):
            cb = b[j-1]
            if ca == cb:
                sub = 0.0
            elif ca in VOWELS and cb in VOWELS:
                sub = FINAL_VOWEL if (i == la and j == lb) else 0.35   # the last vowel carries meaning (করো/করে)
            elif {ca, cb} in ({"s", "c"}, {"j", "z"}, {"b", "v"}, {"j", "y"}, {"w", "b"}, {"f", "p"}, {"y", "i"}, {"y", "e"}):
                sub = 0.4
            else:
                sub = 1.0
            ins_b = 0.25 if cb in "hyw^:`" or cb in VOWELS or (j > 1 and b[j-2] == cb) else 1.0
            del_a = 0.25 if ca in "hyw^:`" or ca in VOWELS or (i > 1 and a[i-2] == ca) else 1.0
            cur[j] = min(prev[j-1] + sub, prev[j] + del_a, cur[j-1] + ins_b)
        prev = cur
    return prev[lb]

class Ranker:
    def __init__(self, data_dir):
        self.roms, self.meta = {}, {}
        with open(os.path.join(data_dir, "romanizations.tsv"), encoding="utf-8") as f:
            f.readline()
            for line in f:
                w, ppm, cls, pref, rs = line.rstrip("\n").split("\t")
                self.meta[w] = (float(ppm), cls, pref)
                self.roms[w] = [(float(c), r) for c, r in (x.split(":", 1) for x in rs.split("|"))]
        self.index = {}
        with open(os.path.join(data_dir, "roman-index.tsv"), encoding="utf-8") as f:
            f.readline()
            for line in f:
                k, ws = line.rstrip("\n").split("\t")
                self.index[k] = ws.split(" ")
        self.deletes = collections.defaultdict(list)
        for k in self.index:
            for i in range(len(k)):
                self.deletes[k[:i] + k[i+1:]].append(k)

    def candidates(self, typed, limit=400):
        sk = skeleton(typed)
        keys = [sk]
        if len(self.index.get(sk, ())) < 20 and len(sk) >= 3:
            near = set(self.deletes.get(sk, ()))                      # typed one consonant too few
            for i in range(len(sk)):
                d = sk[:i] + sk[i+1:]
                near.update([d] if d in self.index else [])          # one too many
                near.update(self.deletes.get(d, ()))                  # one substituted
            keys += sorted(near)
        out = []
        for k in keys:
            out.extend(self.index.get(k, ())[:limit])
        return list(dict.fromkeys(out)), sk

    params = {"freq_w": FREQ_W, "exact": 0.8, "case": 0.3, "lift": 0.25}   # tuned on eval/pilot.tsv

    def align(self, typed, w):
        tl = typed.lower()
        p = self.params
        has_case = typed != tl
        def one(c, r):
            rl = r.lower()
            bonus = 0.0
            if rl == tl:
                bonus += p["exact"] * (1.0 if c <= 0.35 else 0.4)
                if has_case and r == typed:
                    bonus += p["case"]
            return c + wdist(tl, rl) - bonus
        return min(one(c, r) for c, r in self.roms[w])

    def rank(self, typed, n=5):
        cands, sk = self.candidates(typed)
        p = self.params
        scores = {}
        for w in cands:
            ppm, cls, pref = self.meta[w]
            if cls == "vulgar" and not any(r.lower() == typed.lower() and c <= 0.1 for c, r in self.roms[w]):
                continue                                   # only on an exact spelling
            a = self.align(typed, w)
            s = a - p["freq_w"] * math.log10(ppm + 1e-3) + CLASS_PENALTY.get(cls, 2.0)
            scores[w] = min(scores.get(w, 1e9), s)
            # a non-standard form that matches well also brings its standard form (করসি → করছি)
            if pref and pref in self.meta:
                pp = self.meta[pref][0]
                s2 = a + p["lift"] - p["freq_w"] * math.log10(pp + 1e-3)
                scores[pref] = min(scores.get(pref, 1e9), s2)
        return [w for w, _ in sorted(scores.items(), key=lambda kv: kv[1])[:n]]

if __name__ == "__main__":
    r = Ranker(sys.argv[1])
    for t in sys.argv[2:]:
        print(t, "→", " ".join(r.rank(t)))
