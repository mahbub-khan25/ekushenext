"""Download the pages people sent from Studio ("একুশেকে পাঠান") to a folder; files already there
are skipped, so run it any time to get the new ones.

    EKUSHE_OCR_KEY=<the OCR_ADMIN_KEY secret> python3 fetch_samples.py OUT_DIR [BASE_URL]

OUT_DIR/<date>/<id>/ gets page-0001.jpg, page-0001.json (lines as read, with boxes), corrected.txt,
original.txt, meta.json. Then make training lines with user_lines.py."""
import json, os, sys, urllib.parse, urllib.request

out = sys.argv[1]
base = (sys.argv[2] if len(sys.argv) > 2 else "https://ekushenext.com").rstrip("/") + "/api/ocr-sample"
key = os.environ.get("EKUSHE_OCR_KEY") or sys.exit("set EKUSHE_OCR_KEY")


def get(q):
    req = urllib.request.Request(f"{base}?{urllib.parse.urlencode(q)}", headers={"Authorization": f"Bearer {key}", "User-Agent": "ekushe-fetch"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read()


cursor, new, seen = None, 0, 0
while True:
    q = {"list": "1"}
    if cursor:
        q["cursor"] = cursor
    page = json.loads(get(q))
    for o in page["objects"]:
        seen += 1
        dest = os.path.join(out, *o["key"].split("/")[1:])
        if os.path.exists(dest) and os.path.getsize(dest) == o["size"]:
            continue
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        data = get({"get": o["key"]})
        with open(dest + ".part", "wb") as f:
            f.write(data)
        os.replace(dest + ".part", dest)
        new += 1
    cursor = page.get("cursor")
    if not cursor:
        break
print(f"{seen} files on the server, {new} new downloaded to {out}")
