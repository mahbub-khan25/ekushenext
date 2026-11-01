"""Corrector v2: OCR channel rules + fuzzy fallback + word-pair context."""
import math, re, sys, pickle, os, glob
from rapidfuzz import process, distance
sys.path.insert(0, os.path.dirname(__file__))
from correct import char_fix, load_lex, load_rules, BN

P = dict(margin=1.5, chan=1.5, ctx=0.6, fuzzy=0.0)

class C2:
    def __init__(self, rules, freq, lm, **kw):
        self.p = dict(P, **kw)
        self.rules, self.freq = rules, freq
        self.uni, self.bi = lm
        self.N = sum(self.uni.values())
        self.ins = [(i, lp) for i, lp in rules.get("", []) if lp > -6]
        self.words = [w for w, f in freq.items() if f >= 0.05]
        self.cache = {}

    def lf(self, w):
        return math.log(self.freq.get(w, 0) + 1e-3)

    def pair(self, a, b):
        if not a or not b: return 0.0
        c = self.bi.get((a, b), 0)
        if c: return math.log(c / self.uni[a])
        return math.log(0.4 * (self.uni.get(b, 0) + 0.5) / self.N)

    def variants(self, w):
        out = {}
        for i in range(len(w)):
            for n in (1, 2, 3):
                o = w[i:i+n]
                if len(o) < n: break
                for intended, lp in self.rules.get(o, ()):
                    v = w[:i] + intended + w[i+n:]
                    if v != w and out.get(v, -99) < lp: out[v] = lp
        for i in range(len(w) + 1):
            for intended, lp in self.ins:
                v = w[:i] + intended + w[i:]
                if out.get(v, -99) < lp: out[v] = lp
        return out

    def cands(self, w):
        if w in self.cache: return self.cache[w]
        known = w in self.freq
        c = {}
        one = self.variants(w)
        for v, lp in one.items():
            if v in self.freq: c[v] = max(c.get(v, -99), lp)
        if not known:
            for v, lp in sorted(one.items(), key=lambda x: -x[1])[:120]:
                for v2, lp2 in self.variants(v).items():
                    if v2 in self.freq: c[v2] = max(c.get(v2, -99), lp + lp2)
            if not c and len(w) >= 3 and self.p["fuzzy"] > 0:
                lim = 1 if len(w) <= 4 else 2
                for v, d, _ in process.extract(w, self.words, scorer=distance.Levenshtein.distance,
                                               score_cutoff=lim, limit=15):
                    c[v] = max(c.get(v, -99), -self.p["fuzzy"] * d)
        self.cache[w] = c
        return c

    def correct_tokens(self, toks):
        out = []
        for i, w in enumerate(toks):
            prev = out[-1] if out else None
            nxt = toks[i+1] if i + 1 < len(toks) else None
            cs = self.cands(w)
            if not cs: out.append(w); continue
            known = w in self.freq
            def score(x, chan):
                return self.lf(x) + self.p["chan"] * chan + self.p["ctx"] * (self.pair(prev, x) + self.pair(x, nxt))
            keep = score(w, 0.0) if known else None
            best = max(cs, key=lambda x: score(x, cs[x]))
            bs = score(best, cs[best])
            if keep is None or bs > keep + self.p["margin"]:
                out.append(best)
            else:
                out.append(w)
        return out

    def correct(self, text):
        text = char_fix(text)
        parts = BN.split(text); words = BN.findall(text)
        fixed = self.correct_tokens(words)
        res = [parts[0]]
        for w, p in zip(fixed, parts[1:]): res += [w, p]
        return "".join(res)

def load(**kw):
    return C2(load_rules(os.path.join(os.path.dirname(__file__), "..", "rules.tsv")), load_lex(),
              pickle.load(open(os.path.join(os.path.dirname(__file__), "..", "lm.pkl"), "rb")), **kw)

if __name__ == "__main__":
    # batch: correct2.py SRC_RUN DST_RUN [k=v ...]
    src, dst = sys.argv[1], sys.argv[2]
    kw = {k: float(v) for k, v in (a.split("=") for a in sys.argv[3:])}
    c = load(**kw); os.makedirs(dst, exist_ok=True)
    for f in glob.glob(f"{src}/*.txt"):
        if f.endswith(".B.txt"): continue
        open(f"{dst}/{os.path.basename(f)}", "w").write(c.correct(open(f).read()))
