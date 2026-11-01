"""Run an OCR command on every eval image and report character/word error rates by condition.
usage: eval_ocr.py NAME 'cmd with {img} and {out}'   (cmd writes {out}.txt)
Outputs are cached in runs/NAME/."""
import glob, os, re, subprocess, sys, unicodedata
from concurrent.futures import ThreadPoolExecutor
from rapidfuzz.distance import Levenshtein

name, cmd = sys.argv[1], sys.argv[2]
only = sys.argv[3] if len(sys.argv) > 3 else ""
os.makedirs(f"runs/{name}", exist_ok=True)

def norm(s):
    s = unicodedata.normalize("NFC", s)
    s = s.replace("‌", "").replace("‍", "")
    return re.sub(r"\s+", " ", s).strip()

D = os.environ.get("DIR", "eval")
imgs = sorted(i for i in glob.glob(f"{D}/*_*_*.png") if only in i)
def run(img):
    out = f"runs/{name}/" + os.path.basename(img)[:-4]
    if not os.path.exists(out + ".txt"):
        subprocess.run(cmd.format(img=img, out=out), shell=True, capture_output=True)
    return img, out

with ThreadPoolExecutor(2) as ex:
    results = list(ex.map(run, imgs))

stats = {}
for img, out in results:
    base = img.rsplit("_", 1)[0]; cond = img.rsplit("_", 1)[1][:-4]
    font = os.path.basename(base).rsplit("_", 1)[0]
    gt = norm(open(base + ".gt.txt").read()); hyp = norm(open(out + ".txt").read()) if os.path.exists(out + ".txt") else ""
    ce = Levenshtein.distance(gt, hyp); cn = len(gt)
    gw, hw = gt.split(), hyp.split()
    we = Levenshtein.distance(gw, hw); wn = len(gw)
    for k in (cond, "ALL", f"{cond}:{font}"):
        s = stats.setdefault(k, [0, 0, 0, 0]); s[0] += ce; s[1] += cn; s[2] += we; s[3] += wn
def show(k):
    ce, cn, we, wn = stats[k]; return f"CER {100*ce/cn:5.2f}%  WER {100*we/wn:5.2f}%"
print(f"== {name}")
for k in ["clean", "scan", "photo", "ALL"]:
    if k in stats: print(f"  {k:6} {show(k)}")
if os.environ.get("FONTS"):
    for k in sorted(x for x in stats if ":" in x): print(f"  {k:28} {show(k)}")
