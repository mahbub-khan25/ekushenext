"""Render the held-out paragraphs as pages in each font, then make degraded copies.
Writes eval/<font>_<page>_<cond>.png and the truth eval/<font>_<page>.gt.txt"""
import os, subprocess, sys, random, io
import numpy as np, cv2
from PIL import Image

FONTS_DEFAULT = ["Kalpurush", "SolaimanLipi", "Siyam Rupali", "Hind Siliguri", "Noto Sans Bengali",
         "Noto Serif Bengali", "Tiro Bangla", "Anek Bangla", "Mina", "Atma", "Baloo Da 2"]
FONTS = os.environ["FONTS"].split(",") if os.environ.get("FONTS") else FONTS_DEFAULT
SRC = os.environ.get("SRC", "eval-paras.txt"); paras = open(SRC).read().split("\n")
out = os.environ.get("OUT", "eval"); os.makedirs(out, exist_ok=True)
rng = random.Random(int(os.environ.get("SEED", "7")))

def degrade(img, cond, r):
    a = np.array(img.convert("L"))
    if cond == "clean":
        return a
    if cond == "scan":  # 200 dpi office scanner: slight blur, skew, noise, threshold
        a = cv2.resize(a, None, fx=2/3, fy=2/3, interpolation=cv2.INTER_AREA)
        a = cv2.GaussianBlur(a, (3, 3), 0.8)
        h, w = a.shape
        m = cv2.getRotationMatrix2D((w/2, h/2), r.uniform(-1.5, 1.5), 1)
        a = cv2.warpAffine(a, m, (w, h), borderValue=255)
        a = np.clip(a + np.random.default_rng(r.randint(0, 1 << 30)).normal(0, 12, a.shape), 0, 255).astype(np.uint8)
        _, a = cv2.threshold(a, 170, 255, cv2.THRESH_BINARY)
        return a
    if cond == "photo":  # phone photo: perspective, uneven light, blur, JPEG, low-ish res
        h, w = a.shape
        d = 0.04
        src = np.float32([[0, 0], [w, 0], [w, h], [0, h]])
        dst = np.float32([[w*r.uniform(0, d), h*r.uniform(0, d)], [w*(1-r.uniform(0, d)), h*r.uniform(0, d)],
                          [w*(1-r.uniform(0, d)), h*(1-r.uniform(0, d))], [w*r.uniform(0, d), h*(1-r.uniform(0, d))]])
        a = cv2.warpPerspective(a, cv2.getPerspectiveTransform(src, dst), (w, h), borderValue=255)
        yy, xx = np.mgrid[0:h, 0:w]
        light = 0.75 + 0.25 * np.cos((xx / w - r.uniform(0, 1)) * 3) * np.cos((yy / h - r.uniform(0, 1)) * 2)
        a = (a.astype(np.float32) * light + 30 * (1 - light)).clip(0, 255).astype(np.uint8)
        a = cv2.GaussianBlur(a, (5, 5), 1.2)
        a = cv2.resize(a, None, fx=0.5, fy=0.5, interpolation=cv2.INTER_AREA)
        ok, buf = cv2.imencode(".jpg", a, [cv2.IMWRITE_JPEG_QUALITY, 55])
        return cv2.imdecode(buf, cv2.IMREAD_GRAYSCALE)
    raise ValueError(cond)

pages_per_font = int(sys.argv[1]) if len(sys.argv) > 1 else 2
pi = int(os.environ.get("START", "0"))
for font in FONTS:
    for page in range(pages_per_font):
        text = []
        while sum(len(t) for t in text) < 1200:
            text.append(paras[pi % len(paras)]); pi += 1
        base = f"{out}/{font.replace(' ', '')}_{page}"
        open(base + ".in.txt", "w").write("\n".join(text) + "\n")
        subprocess.run(["text2image", "--text", base + ".in.txt", "--outputbase", base + ".raw", "--font", font,
                        "--fonts_dir", os.path.expanduser("~/.fonts"), "--ptsize", "12", "--resolution", "300",
                        "--xsize", "2480", "--ysize", "3508", "--margin", "150", "--leading", "18"],
                       check=True, capture_output=True)
        # The truth is what was rendered (from the box file: the text that fit on the page)
        rendered = []
        for line in open(base + ".raw.box", encoding="utf-8"):
            ch = line.rstrip("\n").rsplit(" ", 5)[0]
            rendered.append("\n" if ch == "\t" else ch)
        gt = "".join(rendered)
        # box files hold grapheme clusters, tabs at line ends; map back to words with spaces
        open(base + ".gt.txt", "w").write(gt)
        img = Image.open(base + ".raw.tif")
        for cond in ["clean", "scan", "photo"]:
            cv2.imwrite(f"{base}_{cond}.png", degrade(img, cond, rng))
        os.remove(base + ".raw.tif")
    print(font, "done", flush=True)
