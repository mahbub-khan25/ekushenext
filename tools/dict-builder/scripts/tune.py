#!/usr/bin/env python3
"""Grid-search ranking weights on the tuning split of the pilot set; report the held-out split too."""
import sys, os, itertools
sys.path.insert(0, os.path.dirname(__file__))
import rank
from rank import Ranker
from eval_pilot import run

data, test = sys.argv[1], sys.argv[2]
rows = [l.rstrip("\n").split("\t") for l in open(test, encoding="utf-8") if l.strip() and not l.startswith("#")]
tests = [(t, a.split("|"), s) for t, a, s in rows]
tune = [x for i, x in enumerate(tests) if i % 3 != 2]
hold = [x for i, x in enumerate(tests) if i % 3 == 2]
r = Ranker(data)
results = []
for fw, ex, fv in itertools.product([0.35, 0.45, 0.55, 0.7], [0.3, 0.5, 0.8], [0.35, 0.6, 0.8]):
    r.params.update(freq_w=fw, exact=ex); rank.FINAL_VOWEL = fv
    st, _ = run(r, tune)
    results.append((st["all", "1"] + 0.5 * st["all", "3"], fw, ex, fv, st["all", "1"] / st["all", "n"], st["all", "3"] / st["all", "n"]))
results.sort(reverse=True)
for x in results[:5]:
    print("tune  score=%.1f freq_w=%.2f exact=%.2f final_vowel=%.2f top1=%.1f%% top3=%.1f%%" % (x[0], x[1], x[2], x[3], 100 * x[4], 100 * x[5]))
_, fw, ex, fv, *_ = results[0]
r.params.update(freq_w=fw, exact=ex); rank.FINAL_VOWEL = fv
st, _ = run(r, hold)
print("HOLDOUT n=%d top1=%.1f%% top3=%.1f%%" % (st["all", "n"], 100 * st["all", "1"] / st["all", "n"], 100 * st["all", "3"] / st["all", "n"]))
