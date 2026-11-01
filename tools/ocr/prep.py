"""Clean up a page image for OCR: grey, even out lighting, scale text to a good size, straighten."""
import sys, cv2, numpy as np

def text_height(bw):
    # median height of ink components = rough letter size
    n, _, st, _ = cv2.connectedComponentsWithStats(255 - bw, 8)
    hs = st[1:, cv2.CC_STAT_HEIGHT]
    hs = hs[(hs > 4) & (hs < 200)]
    return float(np.median(hs)) if len(hs) else 0

def prep(path, out, mode="full"):
    g = cv2.imread(path, cv2.IMREAD_GRAYSCALE)
    # 1. even out lighting: divide by a heavily blurred background
    bg = cv2.morphologyEx(g, cv2.MORPH_CLOSE, cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (25, 25)))
    flat = cv2.divide(g, bg, scale=255)
    # 2. scale so letters are ~38 px tall (Tesseract likes 30-40 px x-height-ish for Bangla)
    _, bw = cv2.threshold(flat, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    h = text_height(bw)
    if h > 0:
        f = max(0.5, min(4.0, 38.0 / h))
        if abs(f - 1) > 0.15:
            flat = cv2.resize(flat, None, fx=f, fy=f, interpolation=cv2.INTER_CUBIC if f > 1 else cv2.INTER_AREA)
    # 3. straighten (deskew) using the angle of the text lines
    _, bw = cv2.threshold(flat, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    best, ang = -1, 0.0
    small = cv2.resize(255 - bw, None, fx=0.25, fy=0.25, interpolation=cv2.INTER_AREA)
    for a in np.arange(-5, 5.01, 0.25):
        m = cv2.getRotationMatrix2D((small.shape[1] / 2, small.shape[0] / 2), a, 1)
        r = cv2.warpAffine(small, m, (small.shape[1], small.shape[0]))
        s = np.var(r.sum(axis=1))
        if s > best: best, ang = s, a
    if abs(ang) > 0.1:
        m = cv2.getRotationMatrix2D((flat.shape[1] / 2, flat.shape[0] / 2), ang, 1)
        flat = cv2.warpAffine(flat, m, (flat.shape[1], flat.shape[0]), flags=cv2.INTER_CUBIC, borderValue=255)
    if mode == "bin":
        flat = cv2.adaptiveThreshold(flat, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C, cv2.THRESH_BINARY, 41, 15)
    cv2.imwrite(out, flat)

if __name__ == "__main__":
    prep(sys.argv[1], sys.argv[2], sys.argv[3] if len(sys.argv) > 3 else "full")
