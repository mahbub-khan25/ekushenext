#!/usr/bin/env python3
"""Phase 4c: find English loanwords in the Bangla lexicon (computer → কম্পিউটার, school → স্কুল).

Usage: match_english.py ROMAN_DIR OUT.tsv [lexicon.tsv]
English word list: wordfreq (data CC BY-SA 4.0), top 60,000 English words.
Each English word is respelled the way Bangla speakers pronounce it, reduced to the same skeleton
key as Bangla romanizations, and matched to the closest Bangla word in that bucket.
"""
import sys, os, re
sys.path.insert(0, os.path.dirname(__file__))
from romanize import skeleton
from rank import wdist
from wordfreq import top_n_list, zipf_frequency

RULES = [
    (r"tion$", "shon"), (r"sion$", "shon"), (r"tion", "shon"), (r"ture", "char"), (r"sch", "sk"),
    (r"ph", "f"), (r"ck", "k"), (r"qu", "kw"), (r"x", "ks"), (r"igh", "ai"), (r"gh", ""),
    (r"c(?=[eiy])", "s"), (r"c", "k"), (r"g(?=e$)", "j"), (r"ge(?=[^aeiou])", "je"), (r"dge", "j"),
    (r"ay", "e"), (r"ai", "e"),
    (r"a(?=[^aeiou]e$)", "e"), (r"i(?=[^aeioucsv]e$)", "ai"), (r"u(?=[^aeiou]e$)", "iu"),   # make, time, tube
    (r"oo", "u"), (r"ee", "i"), (r"ea", "i"), (r"^u(?=[^aeiou])", "iu"), (r"(?<=[^aeiou])e$", ""),
    (r"y$", "i"), (r"(er|or|ar|ure)$", "ar"), (r"w", "o"), (r"(.)\1", r"\1"),
]

# letters and signs that English loanwords practically never use in Bangla spelling
NATIVE_MARK = re.compile("[\u0981\u0983\u09A3\u09B7\u098B\u09C3\u09CE\u099E\u09A2\u0988\u098A\u09CC\u0990]|ক্ষ|জ্ঞ")

def bangla_respell(en):
    s = en.lower()
    for a, b in RULES:
        s = re.sub(a, b, s)
    return s

def main():
    roman_dir, out_path = sys.argv[1], sys.argv[2]
    roms, meta = {}, {}
    with open(os.path.join(roman_dir, "romanizations.tsv"), encoding="utf-8") as f:
        f.readline()
        for line in f:
            w, ppm, cls, pref, rs = line.rstrip("\n").split("\t")
            meta[w] = (float(ppm), cls)
            roms[w] = [r.split(":", 1)[1].lower() for r in rs.split("|") if float(r.split(":", 1)[0]) <= 0.3]
    inflected = set()
    lex_path = sys.argv[3] if len(sys.argv) > 3 else None
    if lex_path:
        with open(lex_path, encoding="utf-8") as f:
            f.readline()
            for line in f:
                p = line.rstrip("\n").split("\t")
                if p[6] in ("ের", "র", "কে", "তে", "য়", "ে", "েরা", "দের", "রা"):
                    inflected.add(p[0])                  # লসের, কূপের: inflected native forms
    index = {}
    with open(os.path.join(roman_dir, "roman-index.tsv"), encoding="utf-8") as f:
        f.readline()
        for line in f:
            k, ws = line.rstrip("\n").split("\t")
            index[k] = ws.split(" ")
    out = []
    for en in top_n_list("en", 60000):
        if not en.isalpha() or not en.isascii() or len(en) < 4 or zipf_frequency(en, "en") < 3.5:
            continue
        sp = bangla_respell(en)
        sk = skeleton(sp)
        if len(sk.lstrip("@")) < 2:
            continue
        cands = []
        for w in index.get(sk, [])[:300]:
            ppm, cls = meta[w]
            if ppm < 0.2 or not roms[w]:
                continue
            d = min(wdist(sp, r) for r in roms[w])
            limit = 0.35 if len(en) >= 6 else 0.0
            if d <= limit and cls in ("standard", "variant") and not NATIVE_MARK.search(w) and w not in inflected:
                cands.append((d - 0.35 * __import__("math").log10(ppm), d, w))
        if not cands:
            continue
        cands.sort()
        best, best_ppm = cands[0][0], meta[cands[0][2]][0]
        for score, d, w in cands[:2]:
            if score == best or (score <= best + 0.3 and meta[w][0] >= 0.3 * best_ppm):
                native = en in roms[w]               # English spelling already reaches the word
                out.append((w, en, round(d, 2), sp, native))
    with open(out_path, "w", encoding="utf-8") as f:
        f.write("# bangla\tenglish\tdistance\trespelled\talready_reachable\n")
        for w, en, d, sp, native in out:
            f.write(f"{w}\t{en}\t{d}\t{sp}\t{int(native)}\n")
    print(f"matched {len(out)} English words; {sum(1 for x in out if not x[4])} add a new route", file=sys.stderr)

if __name__ == "__main__":
    main()
