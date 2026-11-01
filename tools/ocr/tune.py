import sys, os, glob, itertools, re, unicodedata
sys.path.insert(0, 'scripts')
import correct2
from rapidfuzz.distance import Levenshtein
def nofz_patch():
    orig = correct2.C2.cands
    def c(self, w):
        if w in self.cache: return self.cache[w]
        cs = {}; one = self.variants(w)
        for v, lp in one.items():
            if v in self.freq: cs[v] = max(cs.get(v, -99), lp)
        if w not in self.freq:
            for v, lp in sorted(one.items(), key=lambda x: -x[1])[:120]:
                for v2, lp2 in self.variants(v).items():
                    if v2 in self.freq: cs[v2] = max(cs.get(v2, -99), lp + lp2)
        self.cache[w] = cs; return cs
    correct2.C2.cands = c
nofz_patch()
def norm(s):
    s = unicodedata.normalize("NFC", s).replace("‌", "").replace("‍", ""); return re.sub(r"\s+", " ", s).strip()
D, RUN = sys.argv[1], sys.argv[2]
files = [f for f in sorted(glob.glob(f"{RUN}/*.txt")) if not f.endswith(".B.txt")]
data = []
for f in files:
    b = os.path.basename(f)[:-4].rsplit("_", 1)[0]
    data.append((open(f).read(), norm(open(f"{D}/{b}.gt.txt").read()).split()))
base = correct2.load()
def wer(**kw):
    base.p = dict(correct2.P, **kw)
    e = n = 0
    for t, g in data:
        h = norm(base.correct(t)).split(); e += Levenshtein.distance(g, h); n += len(g)
    return 100 * e / n
print("raw", 100*sum(Levenshtein.distance(g, norm(t).split()) for t,g in data)/sum(len(g) for _,g in data))
for m, ch, cx in itertools.product([0.5, 1.5, 3.0], [0.7, 1.0, 1.5], [0.3, 0.6, 1.0]):
    print(f"margin={m} chan={ch} ctx={cx} WER {wer(margin=m, chan=ch, ctx=cx):.2f}", flush=True)
