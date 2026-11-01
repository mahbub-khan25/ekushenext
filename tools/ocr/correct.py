"""Prototype of dictionary post-correction for Bangla OCR.
learn:   correct.py learn DIR RUN rules.tsv        (DIR has *.gt.txt, RUN has OCR *.txt)
apply:   correct.py apply rules.tsv in.txt out.txt
"""
import math, re, sys, unicodedata, glob, os
from collections import Counter, defaultdict
from rapidfuzz.distance import Levenshtein

LEX = os.environ.get("EKUSHE_LEXICON", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "data", "dictionary", "lexicon.tsv"))
BN = re.compile(r"[ঀ-৿‌‍]+")

def norm(s):
    s = unicodedata.normalize("NFC", s)
    return s.replace("‌", "").replace("‍", "")

def char_fix(s):
    """Glyph look-alikes Tesseract emits that are never what was printed in Bangla text."""
    s = norm(s)
    s = s.replace("৷", "।")                       # ৷ (currency sign) for the dari
    s = re.sub(r"(?<=[ঀ-৿\s])\|(?=\s|$)", "।", s)  # | after Bangla = dari
    s = s.replace("আ্য", "অ্য")                        # অ্যা read as আ্যা
    s = re.sub(r"([ক-হ])্্", r"\1্", s)  # doubled hasanta
    s = re.sub(r"(?<=[ঀ-৿]) +(?=[।?!,;:)])", "", s)  # no space before a dari or comma
    # inside numbers, letters that look like digits are digits (এ/৫, ও/৩ ...)
    look = {"এ": "৫", "ও": "৩", "8": "৪", "o": "০", "O": "০"}
    def digits(m):
        t = m.group(0)
        nd = sum(c in "০১২৩৪৫৬৭৮৯" for c in t)
        if nd >= 1 and nd >= sum(c in look for c in t):
            return "".join(look.get(c, c) for c in t)
        return t
    s = re.sub(r"(?<![ঀ-ৗ])[০-৯এও8oO]+(?:[.,:/-][০-৯এও8oO]+)*(?![ঀ-ৗ])", digits, s)
    return s

def load_lex():
    freq = {}
    for line in open(LEX, encoding="utf-8"):
        f = line.rstrip("\n").split("\t")
        if f[0] == "word" or len(f) < 4 or f[3] == "typo":
            continue
        freq[norm(f[0])] = max(float(f[1]), freq.get(norm(f[0]), 0))
    return freq

def align_edits(a, b, ctx=1):
    """Character edits turning intended a into observed b, as (intended, observed) substrings
    with one character of context, merged when adjacent."""
    ops = Levenshtein.opcodes(a, b)
    out = []
    for op in ops:
        if op.tag == "equal":
            continue
        i1, i2, j1, j2 = op.src_start, op.src_end, op.dest_start, op.dest_end
        out.append((a[i1:i2], b[j1:j2]))
        # with left context, so insertions/deletions are anchored
        if i1 > 0:
            out.append((a[i1-1:i2], b[j1-1:j2] if j1 > 0 else a[i1-1] + b[j1:j2]))
    return out

def learn(gtdir, run, outp):
    cnt = Counter(); total = Counter()
    pairs = 0
    for hypf in glob.glob(f"{run}/*.txt"):
        if hypf.endswith(".B.txt"):
            continue
        base = os.path.basename(hypf)[:-4].rsplit("_", 1)[0]
        gt = char_fix(open(f"{gtdir}/{base}.gt.txt").read()).split()
        hy = char_fix(open(hypf).read()).split()
        for op in Levenshtein.opcodes(gt, hy):
            if op.tag == "equal":
                for w in gt[op.src_start:op.src_end]:
                    for n in (1, 2, 3):
                        for i in range(len(w) - n + 1): total[w[i:i+n]] += 1
            if op.tag == "replace" and op.src_end - op.src_start == op.dest_end - op.dest_start:
                for g, h in zip(gt[op.src_start:op.src_end], hy[op.dest_start:op.dest_end]):
                    if Levenshtein.distance(g, h) > 3:
                        continue
                    pairs += 1
                    for n in (1, 2, 3):
                        for i in range(len(g) - n + 1): total[g[i:i+n]] += 1
                    for e in align_edits(g, h):
                        if len(e[0]) <= 3 and len(e[1]) <= 3:
                            cnt[e] += 1
    with open(outp, "w") as f:
        for (i, o), c in cnt.most_common():
            if c < 2:
                continue
            p = c / max(total[i], c) if i else c / 100000
            f.write(f"{i}\t{o}\t{c}\t{math.log(p):.3f}\n")
    print("word pairs", pairs, "rules", sum(1 for v in cnt.values() if v >= 2))

def load_rules(p):
    rules = defaultdict(list)  # observed -> [(intended, logp)]
    for line in open(p, encoding="utf-8"):
        i, o, c, lp = line.rstrip("\n").split("\t")
        rules[o].append((i, float(lp)))
    return rules

class Corrector:
    def __init__(self, rules, freq):
        self.rules, self.freq = rules, freq
        self.obs = sorted((o for o in rules if o), key=len, reverse=True)
        self.ins = [(i, lp) for i, lp in rules.get("", []) if lp > -6]   # chars OCR tends to drop

    def variants(self, w):
        out = {}
        L = len(w)
        for i in range(L):
            for n in (1, 2, 3):
                o = w[i:i+n]
                if len(o) < n:
                    break
                for intended, lp in self.rules.get(o, ()):
                    v = w[:i] + intended + w[i+n:]
                    if v != w and (v not in out or out[v] < lp):
                        out[v] = lp
        for i in range(L + 1):
            for intended, lp in self.ins:
                v = w[:i] + intended + w[i:]
                if v not in out or out[v] < lp:
                    out[v] = lp
        return out

    def correct_word(self, w):
        f = self.freq.get(w)
        keep = math.log(f + 1e-3) if f else None
        cands = {}
        one = self.variants(w)
        for v, lp in one.items():
            if v in self.freq:
                cands[v] = max(cands.get(v, -99), lp)
        if not cands and keep is None:
            for v, lp in sorted(one.items(), key=lambda x: -x[1])[:150]:
                for v2, lp2 in self.variants(v).items():
                    if v2 in self.freq:
                        cands[v2] = max(cands.get(v2, -99), lp + lp2)
        if not cands:
            return w
        best = max(cands, key=lambda c: math.log(self.freq[c] + 1e-3) + 1.0 * cands[c])
        bs = math.log(self.freq[best] + 1e-3) + 1.0 * cands[best]
        if keep is None:
            return best
        # a known word stays unless the correction is much more likely
        return best if bs > keep + 4.0 else w

    def correct(self, text):
        text = char_fix(text)
        return BN.sub(lambda m: self.correct_word(m.group(0)), text)

if __name__ == "__main__":
    if sys.argv[1] == "learn":
        learn(sys.argv[2], sys.argv[3], sys.argv[4])
    else:
        c = Corrector(load_rules(sys.argv[2]), load_lex())
        open(sys.argv[4], "w").write(c.correct(open(sys.argv[3]).read()))
