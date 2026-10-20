#!/usr/bin/env python3
"""Split the merged list into tiers by frequency (per million tokens).
core: ppm >= 0.3 (~100k words, ranked suggestions)  extended: 0.03 <= ppm < 0.3 (~300k, recognition)"""
import sys, os
src, out = sys.argv[1], sys.argv[2]
lines = open(src, encoding="utf-8").read().split("\n")
hdr, rows = lines[0], [l for l in lines[1:] if l]
tiers = {"words-core.tsv": lambda p: p >= 0.3, "words-extended.tsv": lambda p: 0.03 <= p < 0.3}
for name, keep in tiers.items():
    sel = [l for l in rows if keep(float(l.split("\t")[1]))]
    with open(os.path.join(out, name), "w", encoding="utf-8") as f:
        f.write(hdr + "\n" + "\n".join(sel) + "\n")
    print(name, len(sel))
