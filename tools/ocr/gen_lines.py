"""Render training lines (tesstrain style) with realistic old-print / scan / photo damage.
gen_lines.py OUTDIR N SEED
Each line: OUT/l_SEED_i.png + .box (WordStr) + .gt.txt. Letters the stock Bangla model cannot
output are written in the box with stand-ins it already has (see SUB); Ekushe maps them back."""
import os, sys, random, re, glob, unicodedata
import numpy as np, cv2
from PIL import Image, ImageDraw, ImageFont
SUB = {"ঐ": "ৰ", "ঔ": "€", "ঋ": "£", "ঊ": "«", "৳": "»"}
OUT, N, SEED = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
random.seed(SEED); np.random.seed(SEED)
W = os.environ.get("OCRWORK", ".")
FONTS = [f for f in glob.glob(os.path.expanduser("~/.fonts/*.ttf")) if "mitra" not in f]
NO_KHANDA = ("akaash", "sagar")
def lines_from(path, k, minw=3, maxw=11):
    txt = open(path, encoding="utf-8").read()
    words = txt.split(); out = []
    for _ in range(k):
        i = random.randrange(0, max(1, len(words) - 20)); n = random.randint(minw, maxw)
        out.append(" ".join(words[i:i + n]))
    return out
sadhu = lines_from(f"{W}/lit/sadhu.txt", N * 4 // 10)
wiki = lines_from(f"{W}/train-paras.txt", N * 3 // 10)
modern = lines_from(f"{W}/lit/modern.txt", N * 2 // 10)
# rare letters: sentences built from dictionary words that contain them
lex = [l.split("\t")[0] for l in open(os.path.join(os.path.dirname(__file__), "../../data/dictionary/lexicon.tsv"), encoding="utf-8")][1:]
rare = [w for w in lex if any(c in w for c in SUB)]
common = [w for w in lex[:20000]]
special = []
for _ in range(N - len(sadhu) - len(wiki) - len(modern)):
    ws = random.sample(common, random.randint(2, 6)) + random.sample(rare, random.randint(1, 3))
    random.shuffle(ws)
    if random.random() < 0.3: ws.insert(random.randrange(len(ws)), "৳" + str(random.randint(5, 9999)).translate(str.maketrans("0123456789", "০১২৩৪৫৬৭৮৯")))
    special.append(" ".join(ws))
texts = sadhu + wiki + modern + special
random.shuffle(texts)

def render(text, font):
    size = random.randint(26, 48)
    F = ImageFont.truetype(font, size, layout_engine=ImageFont.Layout.RAQM)
    d = ImageDraw.Draw(Image.new("L", (10, 10)))
    x0, y0, x1, y1 = d.textbbox((0, 0), text, font=F)
    pad = random.randint(6, 20)
    im = Image.new("L", (x1 - x0 + 2 * pad, y1 - y0 + 2 * pad), 255)
    ImageDraw.Draw(im).text((pad - x0, pad - y0), text, font=F, fill=0)
    return np.array(im)

def damage(a):
    a = a.astype(np.float32)
    h, w = a.shape
    kind = random.random()
    # ink spread / thin strokes (old letterpress, photocopies)
    k = random.choice([0, 0, 1, 1, 2])
    if k:
        ker = np.ones((k + 1, k + 1), np.uint8)
        a = cv2.erode(a, ker) if random.random() < 0.6 else cv2.dilate(a, ker)
    if random.random() < 0.5:  # slight rotation / shear
        ang = random.uniform(-1.2, 1.2)
        M = cv2.getRotationMatrix2D((w / 2, h / 2), ang, 1)
        a = cv2.warpAffine(a, M, (w, h), borderValue=255)
    # paper: grey level, stains, uneven light
    paper = random.uniform(170, 255) if kind < 0.5 else 255
    ink = random.uniform(0, min(80, paper - 110))
    a = ink + (a / 255) * (paper - ink)
    if random.random() < 0.5:
        g = cv2.resize(np.random.rand(4, 8).astype(np.float32), (w, h), interpolation=cv2.INTER_CUBIC)
        a = a * (1 - random.uniform(0, 0.35) * g)
    if random.random() < 0.4:  # specks / dirt
        m = np.random.rand(h, w) < random.uniform(0.001, 0.01)
        a[m] = random.uniform(ink, paper)
    if random.random() < 0.7:
        a = cv2.GaussianBlur(a, (0, 0), random.uniform(0.3, 1.4))
    a = a + np.random.randn(h, w) * random.uniform(0, 14)
    if random.random() < 0.6:  # low resolution scan, then back up
        f = random.uniform(0.35, 0.8)
        a = cv2.resize(cv2.resize(a, None, fx=f, fy=f, interpolation=cv2.INTER_AREA), (w, h), interpolation=cv2.INTER_LINEAR)
    a = np.clip(a, 0, 255).astype(np.uint8)
    if random.random() < 0.35:  # bi-level archive scans (CCITT/JBIG2)
        _, a = cv2.threshold(a, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    elif random.random() < 0.5:
        _, enc = cv2.imencode(".jpg", a, [cv2.IMWRITE_JPEG_QUALITY, random.randint(25, 85)])
        a = cv2.imdecode(enc, 0)
    return a

OK = set(l.split(" ")[0] for l in open(f"{W}/ft2/unicharset.txt", encoding="utf-8").read().split("\n")[1:] if l) | set(SUB) | {" ", "ড়", "ঢ়", "য়"}
OK -= set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")
os.makedirs(OUT, exist_ok=True); n = 0
for i, t in enumerate(texts):
    t = unicodedata.normalize("NFC", re.sub(r"\s+", " ", t)).strip()
    t = t.replace("‌", "").replace("‍", "")
    if len(t) < 4 or any(c not in OK for c in t): continue
    font = random.choice(FONTS)
    if any(x in font.lower() for x in NO_KHANDA) and "ৎ" in t: font = random.choice([f for f in FONTS if not any(x in f.lower() for x in NO_KHANDA)])
    try: a = render(t, font)
    except Exception: continue
    a = damage(a)
    b = f"{OUT}/l_{SEED}_{i:05d}"
    cv2.imwrite(b + ".png", a)
    s = "".join(SUB.get(c, c) for c in t)
    open(b + ".gt.txt", "w").write(t + "\n")
    h, w = a.shape
    with open(b + ".box", "w") as f:
        f.write(f"WordStr 0 0 {w} {h} 0 #{s}\n\t 0 0 {w} {h} 0\n")
    n += 1
print("lines", n)
