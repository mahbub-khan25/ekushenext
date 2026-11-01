"""Training lines from pages people sent from Studio (see fetch_samples.py): each line the reader
found on the page is cut out of the picture and labelled with the person's corrected text for it.

    python3 user_lines.py SAMPLES_DIR OUT_DIR

The corrected text is matched to the lines as read word by word, so it still works when the person
joined lines into paragraphs. A line is kept only when its correction is close enough to what was
read (at most 40% of the letters changed): otherwise the person rewrote it, or it is matched wrongly.
Writes <name>.png, <name>.gt.txt and <name>.box (WordStr) for lstm.train."""
import difflib, glob, json, os, re, sys, unicodedata
from PIL import Image

src, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
BN = re.compile(r"[ঀ-৿]")


def norm(s):
    s = unicodedata.normalize("NFC", s).replace("‌", "").replace("‍", "")
    return re.sub(r"\s+", " ", s).strip()


def lev(a, b):
    if len(a) < len(b):
        a, b = b, a
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


kept = total = 0
for meta in sorted(glob.glob(f"{src}/*/*/meta.json")):
    d = os.path.dirname(meta)
    corrected = open(f"{d}/corrected.txt", encoding="utf-8").read()
    pages = sorted(glob.glob(f"{d}/page-*.json"))
    # every line as read, in page order, with the words it holds
    lines = []
    for pj in pages:
        for ln in json.load(open(pj, encoding="utf-8")):
            t = norm(ln.get("text") or "")
            if t and ln.get("bbox"):
                lines.append((pj[:-5] + ".jpg", ln["bbox"], t.split()))
    read = [w for _, _, ws in lines for w in ws]
    fixed = norm(corrected).split()
    # map each word as read to a position in the corrected words
    pos = [None] * (len(read) + 1)
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, read, fixed, autojunk=False).get_opcodes():
        for k in range(i1, i2 + 1):
            pos[k] = j1 + round((k - i1) * (j2 - j1) / max(1, i2 - i1)) if pos[k] is None else pos[k]
    at, pics = 0, {}
    for jpg, bb, ws in lines:
        total += 1
        a, b = pos[at], pos[at + len(ws)]
        at += len(ws)
        if a is None or b is None or b <= a:
            continue
        gt = " ".join(fixed[a:b])
        got = " ".join(ws)
        if not BN.search(gt) or lev(gt, got) > 0.4 * max(len(gt), 1):
            continue
        im = pics.get(jpg) or pics.setdefault(jpg, Image.open(jpg).convert("L"))
        x0, y0, x1, y1 = (int(bb[k]) for k in ("x0", "y0", "x1", "y1"))
        pad = max(2, int((y1 - y0) * 0.15))
        crop = im.crop((max(0, x0 - pad), max(0, y0 - pad), min(im.width, x1 + pad), min(im.height, y1 + pad)))
        if crop.height < 10 or crop.width < 20:
            continue
        name = f"{out}/us_{os.path.basename(d)}_{os.path.basename(jpg)[:-4]}_{total:05d}"
        crop.save(name + ".png")
        open(name + ".gt.txt", "w", encoding="utf-8").write(gt + "\n")
        w, h = crop.size
        open(name + ".box", "w", encoding="utf-8").write(f"WordStr 0 0 {w} {h} 0 #{gt}\n\t 0 0 {w} {h} 0\n")
        kept += 1
print(f"{kept} training lines from {total} lines read")
