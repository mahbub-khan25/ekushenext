"""Training/eval lines from real data:
- Mozhi-Bengali (IIIT Hyderabad, CC BY 4.0): word images from scanned books, joined into lines of
  4-8 words with natural gaps (a word image keeps its own height and ink)
- British Library REID2019 (public domain): line images cut from 1713-1914 pages (bl_lines.py)
make_real.py OUT  -> OUT/{train,eval_mozhi,eval_bl}/*.png + .gt.txt + .box"""
import io, os, sys, glob, random, re, unicodedata, shutil
import numpy as np, cv2, pyarrow.parquet as pq
OUT = sys.argv[1]; R = os.environ.get("REALDATA", "realdata")
random.seed(7)
def norm(t):
    t = unicodedata.normalize("NFC", t).replace("‌", "").replace("‍", "").replace("ৰ", "র")
    return re.sub(r"\s+", " ", t).strip()
def write(d, name, img, text):
    os.makedirs(d, exist_ok=True)
    b = f"{d}/{name}"
    cv2.imwrite(b + ".png", img)
    open(b + ".gt.txt", "w").write(text + "\n")
    h, w = img.shape
    open(b + ".box", "w").write(f"WordStr 0 0 {w} {h} 0 #{text}\n\t 0 0 {w} {h} 0\n")
def mozhi(split):
    t = pq.read_table(f"{R}/mozhi_{split}.parquet").to_pylist()
    words = []
    for x in t:
        im = cv2.imdecode(np.frombuffer(x["image"]["bytes"], np.uint8), cv2.IMREAD_GRAYSCALE)
        tx = norm(x["text"])
        if im is None or not tx or " " in tx: continue
        words.append((im, tx))
    random.shuffle(words)
    return words
def join(words):
    h = 48; parts = []; texts = []
    for im, tx in words:
        if im.shape[0] != h: im = cv2.resize(im, (max(1, int(im.shape[1] * h / im.shape[0])), h))
        parts.append(im); texts.append(tx)
        parts.append(np.full((h, random.randint(12, 26)), 255, np.uint8))
    img = np.hstack([np.full((h, 8), 255, np.uint8)] + parts[:-1] + [np.full((h, 8), 255, np.uint8)])
    img = cv2.copyMakeBorder(img, 6, 6, 0, 0, cv2.BORDER_CONSTANT, value=255)
    return img, " ".join(texts)
def lines_from(words, d, prefix, limit):
    i = n = 0
    while i < len(words) and n < limit:
        k = random.randint(4, 8); img, tx = join(words[i:i + k]); i += k
        write(d, f"{prefix}_{n:05d}", img, tx); n += 1
    return n
print("mozhi train lines", lines_from(mozhi("train"), f"{OUT}/train", "mz", 10000))
print("mozhi eval lines", lines_from(mozhi("test"), f"{OUT}/eval_mozhi", "mzt", 400))
# British Library: hold out whole books (shelfmark before the last _) for testing
bl = sorted(glob.glob(f"{R}/bl_lines/*.png"))
books = sorted(set(re.sub(r"_\d+_r\d+_l\d+\.png$", "", os.path.basename(f)) for f in bl))
random.shuffle(books); test_books = set(books[: max(3, len(books) // 6)])
nt = ne = 0
for f in bl:
    book = re.sub(r"_\d+_r\d+_l\d+\.png$", "", os.path.basename(f))
    img = cv2.imread(f, cv2.IMREAD_GRAYSCALE); tx = norm(open(f[:-4] + ".gt.txt").read())
    if not tx: continue
    name = os.path.basename(f)[:-4]
    if book in test_books: write(f"{OUT}/eval_bl", name, img, tx); ne += 1
    else:
        for rep in range(3):  # small set: show it three times
            write(f"{OUT}/train", f"bl{rep}_{name}", img, tx); nt += 1
print("bl train", nt, "bl eval", ne, "test books", sorted(test_books))
