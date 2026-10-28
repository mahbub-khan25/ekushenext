#!/usr/bin/env python3
"""Merge per-file word counts into one multi-source frequency list.

Usage: merge_counts.py --out words.tsv SOURCE=counts1.tsv[,counts2.tsv...] [SOURCE=...]

Keeps a word if it is seen in at least 2 sources (>= MIN_PER_SOURCE each) with
total count >= MIN_TOTAL and document count >= MIN_DOCS, or if a single source
has it >= SINGLE_MIN times in >= SINGLE_DOCS documents (flagged `single_source`).
The score is the mean frequency per million tokens across sources, so no single
domain (encyclopedia, web, news) dominates the ranking.
Also flags `variant_of=<word>` when a confusable-spelling neighbour is >= 20x more frequent.
"""
import sys, math, argparse, collections

MIN_TOTAL, MIN_PER_SOURCE, MIN_DOCS = 10, 2, 3
SINGLE_MIN, SINGLE_DOCS = 50, 10

CONFUSABLE_SETS = [
    "িী", "ুূ", "ইঈ", "উঊ", "শষস", "নণ", "জয", "ংঙ", "তৎ", "েৈ", "োৌ",
]
SUBS = collections.defaultdict(set)
for group in CONFUSABLE_SETS:
    for a in group:
        for b in group:
            if a != b:
                SUBS[a].add(b)
MULTI_SUBS = [("ড়", "র"), ("র", "ড়"), ("ঢ়", "র"), ("ড়", "ঢ়"), ("ঢ়", "ড়"),
              # য-ফলা with and without া sound the same: ব্যাবস্থা is a misspelling of ব্যবস্থা
              ("্যা", "্য"), ("্য", "্যা")]

def read_counts(path):
    total = 0
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if parts[0] == "#tokens":
                total = int(parts[1])
                continue
            yield parts[0], int(parts[1]), int(parts[2])
    read_counts.last_total = total

def neighbours(w):
    chars = list(w)
    for i, c in enumerate(chars):
        for alt in SUBS.get(c, ()):
            yield "".join(chars[:i] + [alt] + chars[i + 1:])
    for a, b in MULTI_SUBS:
        start = w.find(a)
        while start != -1:
            yield w[:start] + b + w[start + len(a):]
            start = w.find(a, start + 1)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("sources", nargs="+")
    a = ap.parse_args()

    names, totals = [], []
    counts = collections.defaultdict(lambda: [0, 0, 0, 0, 0, 0])  # up to 3 sources: c0,c1,c2,d0,d1,d2
    for si, spec in enumerate(a.sources):
        name, files = spec.split("=", 1)
        names.append(name)
        tot = 0
        for path in files.split(","):
            for w, c, d in read_counts(path):
                e = counts[w]
                e[si] += c
                e[3 + si] += d
            tot += read_counts.last_total
        totals.append(tot)
        print(f"{name}: {tot:,} tokens", file=sys.stderr)
    n = len(names)

    kept = {}
    for w, e in counts.items():
        cs, ds = e[:n], e[3:3 + n]
        total, docs = sum(cs), sum(ds)
        present = sum(1 for c in cs if c >= MIN_PER_SOURCE)
        flags = []
        if present >= 2 and total >= MIN_TOTAL and docs >= MIN_DOCS:
            pass
        elif any(c >= SINGLE_MIN and d >= SINGLE_DOCS for c, d in zip(cs, ds)):
            flags.append("single_source")
        else:
            continue
        ppm = sum(c / t * 1e6 for c, t in zip(cs, totals)) / n
        kept[w] = (ppm, total, docs, cs, flags)
    del counts

    for w, (ppm, total, docs, cs, flags) in kept.items():
        best = None
        for nb in neighbours(w):
            k = kept.get(nb)
            if k and k[0] >= 20 * ppm and (best is None or k[0] > kept[best][0]):
                best = nb
        if best:
            flags.append(f"variant_of={best}")

    rows = sorted(kept.items(), key=lambda kv: -kv[1][0])
    with open(a.out, "w", encoding="utf-8") as f:
        f.write("word\tppm\ttotal\tdocs\t" + "\t".join(names) + "\tflags\n")
        for w, (ppm, total, docs, cs, flags) in rows:
            f.write(f"{w}\t{ppm:.4f}\t{total}\t{docs}\t" + "\t".join(map(str, cs)) + "\t" + ",".join(flags) + "\n")
    print(f"kept {len(rows):,} words", file=sys.stderr)

if __name__ == "__main__":
    main()
