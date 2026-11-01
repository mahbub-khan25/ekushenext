import re, sys, unicodedata, pickle
from collections import Counter
import pyarrow.parquet as pq
ev = set(open("eval-paras.txt").read().split("\n"))
BN = re.compile(r"[ঀ-৿‌‍]+")
uni, bi = Counter(), Counter()
t = pq.read_table("wiki-bn.parquet", columns=["text"]).column("text").to_pylist()
for art in t:
    for p in art.split("\n"):
        p = re.sub(r"\s+", " ", unicodedata.normalize("NFC", p.strip()))
        if p in ev: continue
        w = [x.replace("‌", "").replace("‍", "") for x in BN.findall(p)]
        uni.update(w); bi.update(zip(w, w[1:]))
bi = Counter({k: v for k, v in bi.items() if v >= 2})
pickle.dump((uni, bi), open("lm.pkl", "wb"))
print(len(uni), len(bi), sum(uni.values()))
