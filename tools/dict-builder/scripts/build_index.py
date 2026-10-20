#!/usr/bin/env python3
"""Phase 4: build Ekushe's Roman → Bangla index from lexicon.tsv.

Usage: build_index.py lexicon.tsv OUT_DIR [english-matches.tsv]
Writes OUT_DIR/romanizations.tsv  word, ppm, class, preferred, romanizations (cost:roman, cheapest first)
       OUT_DIR/roman-index.tsv    skeleton key → words (most frequent first)
"""
import sys, os, collections
sys.path.insert(0, os.path.dirname(__file__))
from romanize import romanize, skeleton

def load_lexicon(path):
    rows = []
    with open(path, encoding="utf-8") as f:
        f.readline()
        for line in f:
            p = line.rstrip("\n").split("\t")
            rows.append({"word": p[0], "ppm": float(p[1]), "tier": p[2], "cls": p[3], "pref": p[4], "tags": p[9]})
    return rows

def main():
    lex_path, out_dir = sys.argv[1], sys.argv[2]
    english = {}
    if len(sys.argv) > 3 and os.path.exists(sys.argv[3]):
        for line in open(sys.argv[3], encoding="utf-8"):
            if line.startswith("#"):
                continue
            w, en, dist = line.rstrip("\n").split("\t")[:3]
            english.setdefault(w, []).append(en)
    rows = load_lexicon(lex_path)
    vulgar = []
    vpath = os.path.join(os.path.dirname(__file__), "..", "curated", "vulgar-prefixes.tsv")
    if os.path.exists(vpath):
        vulgar = [l.split("\t")[0].strip() for l in open(vpath, encoding="utf-8") if l.strip() and not l.startswith("#")]
    index = collections.defaultdict(list)
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "romanizations.tsv"), "w", encoding="utf-8") as out:
        out.write("word\tppm\tclass\tpreferred\tromanizations\n")
        for r in rows:
            if r["cls"] == "typo":
                continue
            k = 8 if r["tier"] == "core" else 5
            roms = romanize(r["word"], k)
            is_vulgar = any(r["word"].startswith(v) for v in vulgar)
            if is_vulgar:
                r["cls"] = "vulgar"
            for en in ([] if is_vulgar else english.get(r["word"], [])):
                roms.append((0.5, en))                         # English spelling of a loanword (extra cost)
            if not roms:
                continue
            out.write(f"{r['word']}\t{r['ppm']:.4f}\t{r['cls']}\t{r['pref']}\t" + "|".join(f"{c}:{x}" for c, x in roms) + "\n")
            for key in {skeleton(x) for _, x in roms}:
                index[key].append((r["ppm"], r["word"]))
    with open(os.path.join(out_dir, "roman-index.tsv"), "w", encoding="utf-8") as out:
        out.write("key\twords\n")
        for key in sorted(index):
            ws = [w for _, w in sorted(index[key], reverse=True)]
            out.write(f"{key}\t{' '.join(ws)}\n")
    sizes = sorted(len(v) for v in index.values())
    print(f"words={len(rows)} keys={len(index)} entries={sum(sizes)} max_bucket={sizes[-1]} median={sizes[len(sizes)//2]}", file=sys.stderr)

if __name__ == "__main__":
    main()
