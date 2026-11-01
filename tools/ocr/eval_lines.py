"""eval_lines.py DIR MODEL [--correct]: OCR each line image (psm 7), report CER / WER against .gt.txt"""
import glob, os, re, subprocess, sys, unicodedata, tempfile
from concurrent.futures import ThreadPoolExecutor
from rapidfuzz.distance import Levenshtein
d, model = sys.argv[1], sys.argv[2]
corr = "--correct" in sys.argv
def norm(s):
    s = unicodedata.normalize("NFC", s).replace("‌", "").replace("‍", "").replace("ৰ", "র")
    return re.sub(r"\s+", " ", s).strip()
out = fos.environ.get("OCRWORK", ".") + "/runs/lines_{os.path.basename(d.rstrip('/'))}_{model.replace('+','_')}"
os.makedirs(out, exist_ok=True)
imgs = sorted(glob.glob(f"{d}/*.png"))
def run(img):
    o = f"{out}/{os.path.basename(img)[:-4]}"
    if not os.path.exists(o + ".txt"):
        subprocess.run(["tesseract", img, o, "--tessdata-dir", os.environ.get("OCRWORK", ".") + "/td", "-l", model, "--psm", "7",
                        "-c", "tessedit_char_blacklist=ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"], capture_output=True)
    return img, o
with ThreadPoolExecutor(2) as ex: res = list(ex.map(run, imgs))
if corr:
    cd = out + "_c"
    subprocess.run([os.environ.get("OCR_CORRECT", "ocr_correct"), out, cd], capture_output=True)
T = [0, 0, 0, 0]
for img, o in res:
    gt = norm(open(img[:-4] + ".gt.txt").read())
    p = (o.replace(out, out + "_c") if corr else o) + ".txt"
    hy = norm(open(p).read()) if os.path.exists(p) else ""
    T[0] += Levenshtein.distance(gt, hy); T[1] += len(gt)
    T[2] += Levenshtein.distance(gt.split(), hy.split()); T[3] += len(gt.split())
print(f"{os.path.basename(d.rstrip('/')):12s} {model:22s} {'corrected' if corr else 'raw':9s} CER {100*T[0]/T[1]:5.2f}%  WER {100*T[2]/T[3]:5.2f}%  ({len(res)} lines)")
