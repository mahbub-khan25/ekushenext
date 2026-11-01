"""Find typed (not scanned) PDF pages in data/raw and cut them into line images with their text.
Output: data/typed_lines/<item>/<page>_<n>.png + .txt (raw text from the PDF) + fonts.txt
Only pages with real fonts and no full-page picture; up to 6 pages per PDF."""
import glob, os, re, subprocess, sys, html
from PIL import Image
ROOT = os.environ.get("OCRDATA", "data")
OUT = f"{ROOT}/typed_lines"
os.makedirs(OUT, exist_ok=True)
def run(*a, t=60):
    return subprocess.run(a, capture_output=True, text=True, timeout=t, errors="replace").stdout
import time
T0 = time.time(); DEADLINE = float(os.environ.get("SECONDS_LEFT", "1e9"))
SEEN = f"{ROOT}/typed_seen.txt"
done = set(d for d in os.listdir(OUT) if len(os.listdir(f"{OUT}/{d}")) > 1)
done |= set(open(SEEN).read().split()) if os.path.exists(SEEN) else set()
seen_f = open(SEEN, "a")
pdfs = sorted(glob.glob(f"{ROOT}/raw/*/*/*.pdf") + glob.glob(f"{ROOT}/raw/*/*/*.PDF"))
lim = int(sys.argv[1]) if len(sys.argv) > 1 else 10**9
n_items = n_lines = 0
for ii, p in enumerate(pdfs[:lim]):
    if ii % 50 == 0: print("progress", ii, len(pdfs), flush=True)
    item = os.path.basename(os.path.dirname(p))
    if item in done: continue
    if time.time() - T0 > DEADLINE: break
    seen_f.write(item + "\n"); seen_f.flush()
    try:
        info = run("pdfinfo", p); pages = int(re.search(r"Pages:\s+(\d+)", info).group(1))
        fonts = run("pdffonts", "-l", "8", p, t=20)
        if len(fonts.strip().split("\n")) < 3: continue          # no fonts: a scan
        # a few pages spread through the document
        cand = sorted(set([max(1, pages * k // 7) for k in range(1, 7)]))
        got = 0
        for pg in cand:
            imgs = run("pdfimages", "-list", "-f", str(pg), "-l", str(pg), p)
            big = [l for l in imgs.split("\n")[2:] if l.strip() and int(l.split()[3]) * int(l.split()[4]) > 1500 * 1500]
            if big: continue                                        # a scanned page with hidden text
            bb = run("pdftotext", "-bbox-layout", "-f", str(pg), "-l", str(pg), p, "-")
            m = re.search(r'<page width="([\d.]+)" height="([\d.]+)"', bb)
            if not m: continue
            pw, ph = float(m.group(1)), float(m.group(2))
            base = f"/tmp/typed_{os.getpid()}"
            subprocess.run(["pdftoppm", "-r", "200", "-gray", "-f", str(pg), "-l", str(pg), "-png", "-singlefile", p, base], capture_output=True, timeout=120)
            if not os.path.exists(base + ".png"): continue
            im = Image.open(base + ".png"); sx, sy = im.width / pw, im.height / ph
            for li, lm in enumerate(re.finditer(r'<line xMin="([\d.]+)" yMin="([\d.]+)" xMax="([\d.]+)" yMax="([\d.]+)">(.*?)</line>', bb, re.S)):
                x0, y0, x1, y1 = [float(v) for v in lm.groups()[:4]]
                words = re.findall(r'<word[^>]*>(.*?)</word>', lm.group(5), re.S)
                text = html.unescape(" ".join(w.strip() for w in words)).strip()
                if len(text) < 4 or (y1 - y0) * sy < 12: continue
                pad = (y1 - y0) * sy * 0.15
                crop = im.crop((max(0, int(x0 * sx - pad)), max(0, int(y0 * sy - pad)), min(im.width, int(x1 * sx + pad)), min(im.height, int(y1 * sy + pad))))
                if not got:
                    os.makedirs(f"{OUT}/{item}", exist_ok=True)
                    open(f"{OUT}/{item}/fonts.txt", "w").write(fonts)
                name = f"{OUT}/{item}/p{pg:04d}_{li:03d}"
                crop.save(name + ".png"); open(name + ".txt", "w").write(text + "\n")
                got += 1
            os.remove(base + ".png")
        if got: n_items += 1; n_lines += got; print(item, got, flush=True)
    except Exception as e:
        print("skip", item, e, flush=True)
print("items", n_items, "lines", n_lines)
