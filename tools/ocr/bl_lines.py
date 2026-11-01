"""British Library REID2019 (PAGE XML, region-level text) -> line images + .gt.txt.
Each text region is cut into lines by a horizontal ink profile; kept only when the number of
lines found equals the number of text lines in its transcription."""
import glob, os, re, sys, unicodedata
import xml.etree.ElementTree as ET
import numpy as np, cv2
src, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
kept = skipped = 0
for xf in sorted(glob.glob(f"{src}/*.xml")):
    root = ET.parse(xf).getroot()
    ns = root.tag.split('}')[0][1:]
    q = lambda p: p.replace('p:', '{%s}' % ns)
    page = os.path.basename(xf)[:-4]
    img = cv2.imread(xf[:-4] + ".tif", cv2.IMREAD_GRAYSCALE)
    if img is None: continue
    for ri, reg in enumerate(root.iter(q('p:TextRegion'))):
        u = reg.find(q('p:TextEquiv/p:Unicode'))
        if u is None or not (u.text or '').strip(): continue
        texts = [unicodedata.normalize('NFC', t.strip()) for t in u.text.strip().split('\n') if t.strip()]
        pts = np.array([[int(v) for v in p.split(',')] for p in reg.find(q('p:Coords')).get('points').split()])
        x, y, w, h = cv2.boundingRect(pts)
        mask = np.zeros(img.shape, np.uint8); cv2.fillPoly(mask, [pts], 255)
        crop = np.where(mask[y:y+h, x:x+w] > 0, img[y:y+h, x:x+w], 255).astype(np.uint8)
        _, bw = cv2.threshold(crop, 0, 255, cv2.THRESH_BINARY_INV + cv2.THRESH_OTSU)
        prof = bw.sum(1) / 255.0
        thr = max(2, prof.max() * 0.04)
        rows = prof > thr
        bands = []; s = None
        for i, r in enumerate(list(rows) + [False]):
            if r and s is None: s = i
            if not r and s is not None:
                if i - s > 6: bands.append([s, i])
                s = None
        # merge bands closer than a third of the median height (matras, dots, rephs split off)
        if bands:
            med = np.median([b - a for a, b in bands])
            merged = [bands[0]]
            for a, b in bands[1:]:
                if a - merged[-1][1] < med * 0.35 or (b - a) < med * 0.4: merged[-1][1] = b
                else: merged.append([a, b])
            bands = merged
        if len(bands) != len(texts):
            skipped += len(texts); continue
        for li, ((a, b), t) in enumerate(zip(bands, texts)):
            pad = int((b - a) * 0.25)
            line = crop[max(0, a - pad):min(h, b + pad), :]
            cols = np.where((255 - line).sum(0) > 255 * 2)[0]
            if len(cols) == 0: continue
            line = line[:, max(0, cols[0] - 10):cols[-1] + 10]
            base = f"{out}/{page}_r{ri:02d}_l{li:02d}"
            cv2.imwrite(base + ".png", line)
            open(base + ".gt.txt", "w").write(t + "\n")
            kept += 1
print("lines kept", kept, "skipped", skipped)
