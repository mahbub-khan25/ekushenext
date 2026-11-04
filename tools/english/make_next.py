#!/usr/bin/env python3
"""Make data/english/next.tsv: likely next words, from Tatoeba's English sentences.

  curl -O https://downloads.tatoeba.org/exports/per_language/eng/eng_sentences.tsv.bz2 && bunzip2 eng_sentences.tsv.bz2
  python3 tools/english/make_next.py eng_sentences.tsv

The hand-written lines at the top of next.tsv (up to the "# generated" line) are kept first, since
they cover everyday messages; the counted pairs and triples follow. Only words in words.txt are
used, and never names (Tom and Mary are in half of Tatoeba's sentences) or rude words.
Tatoeba (https://tatoeba.org) is licensed under CC BY 2.0 FR.
"""
import collections, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "..", "..", "data", "english")
words = [w.strip() for w in open(os.path.join(DATA, "words.txt"), encoding="utf-8") if w.strip()]
rank = {}
for i, w in enumerate(words):
    rank.setdefault(w.lower(), i)
# names and places (capitalised in the list) are not suggested
names = {w.lower() for w in words if w[0].isupper() and w.lower() not in {x for x in words if x[0].islower()}}
quiet = set("fuck fucking fucked fucker shit shitty bitch bitches cunt dick dicks pussy cock cocks asshole bastard slut whore fag faggot nigger nigga retard retarded porn sex sexy nude nudes boobs tits penis vagina rape motherfucker damn crap piss".split())
ok = lambda w: w in rank and w not in names and w not in quiet and rank[w] < 30000

# Tatoeba's own names (Tom, Mary, Ken …): words written with a capital letter in the middle of a
# sentence most of the time
caps = collections.Counter(); low = collections.Counter()
for line in open(sys.argv[1], encoding="utf-8"):
    parts = line.rstrip("\n").split("\t")
    if len(parts) < 3:
        continue
    for w in re.findall(r"(?<=[a-z,] )[A-Za-z]+", parts[2]):
        (caps if w[0].isupper() else low)[w.lower()] += 1
names |= {w for w, n in caps.items() if n >= 20 and n > 2 * low[w] and w != "i"}

big = collections.defaultdict(collections.Counter)
tri = collections.defaultdict(collections.Counter)
tok = re.compile(r"[a-z]+(?:'[a-z]+)?")
for line in open(sys.argv[1], encoding="utf-8"):
    parts = line.rstrip("\n").split("\t")
    if len(parts) < 3:
        continue
    # a sentence end stops the chain (what follows "." isn't a next word)
    for clause in re.split(r"[.!?;:\"()]", parts[2].replace("’", "'").lower()):
        ws = tok.findall(clause)
        for i in range(len(ws) - 1):
            a, b = ws[i], ws[i + 1]
            if ok(a) and ok(b):
                big[a][b] += 1
                if i + 2 < len(ws) and ok(ws[i + 2]):
                    tri[(a, b)][ws[i + 2]] += 1

out = []
for a, c in sorted(big.items(), key=lambda kv: rank[kv[0]]):
    total = sum(c.values())
    if total < 20:
        continue
    nxt = [b for b, n in c.most_common(8) if n >= 3 and n / total >= 0.02][:6]
    if nxt:
        out.append(a + "\t" + "\t".join(nxt))
pairs = sorted(tri.items(), key=lambda kv: -sum(kv[1].values()))
kept = 0
for (a, b), c in pairs:
    total = sum(c.values())
    if total < 40 or kept >= 15000:
        break
    nxt = [w for w, n in c.most_common(6) if n >= 4 and n / total >= 0.03][:4]
    if nxt:
        out.append(f"{a} {b}\t" + "\t".join(nxt))
        kept += 1

path = os.path.join(DATA, "next.tsv")
hand = []
for line in open(path, encoding="utf-8"):
    if line.startswith("# generated"):
        break
    hand.append(line.rstrip("\n"))
with open(path, "w", encoding="utf-8") as f:
    f.write("\n".join(hand) + "\n")
    f.write("# generated from Tatoeba (https://tatoeba.org, CC BY 2.0 FR) by tools/english/make_next.py\n")
    f.write("\n".join(out) + "\n")
print(len(out), "lines;", kept, "word pairs")
