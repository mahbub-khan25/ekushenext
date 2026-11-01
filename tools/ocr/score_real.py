"""score.py OUTDIR  -> per-crop and total CER/WER against gt/*.gt.txt (English crop v5221en reported apart)"""
import glob,os,re,sys,unicodedata
from rapidfuzz.distance import Levenshtein
def norm(s):
    s=unicodedata.normalize("NFC",s).replace("‌","").replace("‍","")
    s=s.replace("’","'").replace("‘","'").replace("“",'"').replace("”",'"').replace("৷","।")
    s=re.sub(r"([।,;:?!.\"'()\-—])",r" \1 ",s)  # punctuation as own tokens
    return re.sub(r"\s+"," ",s).strip()
d=sys.argv[1]; T=[0,0,0,0]; rows=[]
for g in sorted(glob.glob("gt/*.gt.txt")):
    k=os.path.basename(g)[:-7]; h=f"{d}/{k}.txt"
    gt=norm(open(g).read()); hy=norm(open(h).read()) if os.path.exists(h) else ""
    # words only (no punctuation) for WER
    gw=[w for w in gt.split() if re.search(r"\w",w)]; hw=[w for w in hy.split() if re.search(r"\w",w)]
    ce=Levenshtein.distance(gt.replace(" ",""),hy.replace(" ","")); cn=len(gt.replace(" ",""))
    we=Levenshtein.distance(gw,hw); wn=len(gw)
    rows.append((k,100*ce/cn,100*we/wn,wn))
    if k!="v5221en": T[0]+=ce;T[1]+=cn;T[2]+=we;T[3]+=wn
for r in rows: print(f"  {r[0]:10s} CER {r[1]:5.1f}%  WER {r[2]:5.1f}%  ({r[3]} words)")
print(f"== {d}: Bangla crops CER {100*T[0]/T[1]:.2f}%  WER {100*T[2]/T[3]:.2f}%  words {T[3]}")
