#!/usr/bin/env python3
"""Score the ranker on a typed→expected test file. Usage: eval_pilot.py DATA_DIR TEST.tsv [--fails] [key=value ...]"""
import sys, os, collections
sys.path.insert(0, os.path.dirname(__file__))
from rank import Ranker

def run(r, tests):
    stats = collections.Counter(); fails = []
    for typed, acc, style in tests:
        res = r.rank(typed, 3)
        top1 = bool(res) and res[0] in acc
        top3 = any(x in acc for x in res)
        for st in (style, "all"):
            stats[st, "n"] += 1; stats[st, "1"] += top1; stats[st, "3"] += top3
        if not top1:
            fails.append((typed, acc[0], res))
    return stats, fails

def main():
    data, test = sys.argv[1], sys.argv[2]
    tests = []
    for line in open(test, encoding="utf-8"):
        if line.startswith("#") or not line.strip(): continue
        t, a, st = line.rstrip("\n").split("\t")
        tests.append((t, a.split("|"), st))
    r = Ranker(data)
    for kv in sys.argv[3:]:
        if "=" in kv:
            k, v = kv.split("="); r.params[k] = float(v)
    stats, fails = run(r, tests)
    for st in ("all", "both", "avro", "casual"):
        n = stats[st, "n"]
        if n: print(f"{st:7s} n={n:3d}  top1={stats[st,'1']/n:6.1%}  top3={stats[st,'3']/n:6.1%}")
    if "--fails" in sys.argv:
        for t, a, res in fails: print(f"  {t:16s} want {a:12s} got {' '.join(res)}")

if __name__ == "__main__":
    main()
