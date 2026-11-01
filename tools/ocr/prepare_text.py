"""Pull clean Bangla paragraphs from a Wikipedia parquet file: eval pages and training lines."""
import random, re, sys, unicodedata
import pyarrow.parquet as pq

src, out_eval, out_train = sys.argv[1], sys.argv[2], sys.argv[3]
t = pq.read_table(src, columns=["text"]).column("text").to_pylist()
random.seed(21)
random.shuffle(t)
BN = re.compile(r"[ঀ-৿]")
paras = []
for art in t:
    for p in art.split("\n"):
        p = unicodedata.normalize("NFC", p.strip())
        if len(p) < 120 or len(p) > 900:
            continue
        bn = len(BN.findall(p))
        if bn / len(p) < 0.7:  # mostly Bangla prose
            continue
        if any(c in p for c in "|{}[]=<>\\_*#"):
            continue
        p = re.sub(r"\s+", " ", p)
        paras.append(p)
    if len(paras) > 60000:
        break
print("paragraphs", len(paras))
open(out_eval, "w").write("\n".join(paras[:400]) + "\n")      # held out for evaluation only
open(out_train, "w").write("\n".join(paras[400:]) + "\n")
