#!/usr/bin/env python3
"""Phase 3: classify every word: standard spelling, variants, typos, colloquial, sadhu, root + ending.

Usage: classify.py words-all.tsv curated/spelling-seed.tsv OUT_DIR

Writes OUT_DIR/lexicon.tsv and OUT_DIR/review-spelling.tsv. Rules are documented in RULES.md.

Classes
  standard    the preferred spelling (suggested first)
  variant     an accepted but non-standard spelling; suggested below `preferred`
  colloquial  everyday/regional form (করসি, কইরা); suggested, labelled কথ্য, below `preferred`
  sadhu       literary sadhu-bhasha form (করিয়াছে, তাহার); suggested, labelled সাধু
  typo        a misspelling; never suggested, autocorrected to `preferred`
"""
import sys, os, re, collections, unicodedata

SHIP_PPM = 0.03      # words below this are not shipped
CORE_PPM = 0.3
HC, ZWJ = "্", "‍"

def nfc(s): return unicodedata.normalize("NFC", s)

# ---------------------------------------------------------------- load
def load_words(path):
    lex = {}
    with open(path, encoding="utf-8") as f:
        hdr = f.readline().rstrip("\n").split("\t")
        for line in f:
            p = line.rstrip("\n").split("\t")
            lex[p[0]] = {"ppm": float(p[1]), "total": int(p[2]), "docs": int(p[3]),
                         "src": tuple(int(x) for x in p[4:7]), "flags": p[7]}
    return lex

def load_seed(path):
    prefer, distinct = {}, set()
    for line in open(path, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        pref, variants, kind, source, note = line.rstrip("\n").split("\t")
        pref = nfc(pref); vs = [nfc(v) for v in variants.split(",") if v]
        if kind == "prefer":
            for v in vs:
                prefer[v] = (pref, source, note)
            prefer.setdefault(pref, (pref, source, note))
        else:
            distinct.update([pref, *vs])
    return prefer, distinct

# ---------------------------------------------------------------- deterministic Academy rules
CONS = set(chr(c) for c in range(0x0995, 0x09BA)) | {"ড়", "ঢ়", "য়"}

def rule_reph_doubling(w):
    """র্ + C + ্ + C  →  র্ + C   (কর্ম্ম → কর্ম, সূর্য্য → সূর্য)"""
    out, i, changed = [], 0, False
    while i < len(w):
        if w.startswith("র" + HC, i) and i + 4 <= len(w) and w[i+2] in CONS and w[i+3] == HC and i + 4 < len(w) and w[i+4] == w[i+2]:
            out.append(w[i:i+3]); i += 5; changed = True; continue
        out.append(w[i]); i += 1
    return "".join(out) if changed else None

VISARGA_KEEP = {"পুনঃ", "অন্তঃ", "দুঃ", "নিঃ", "ছিঃ", "উঃ", "আঃ", "বাঃ", "মনঃ", "শিরঃ", "তেজঃ", "বয়ঃ"}

def rule_final_visarga(w):
    """মূলতঃ → মূলত  (no visarga at word end, except interjections like উঃ, আঃ)"""
    if w.endswith("ঃ") and len(w) > 3 and w not in VISARGA_KEEP:
        return w[:-1]
    return None

def rule_dur_prefix(w):
    """দূর্ + consonant → দুর্ (দূর্নীতি → দুর্নীতি). দূর (far) has no hasanta, so it is untouched."""
    if w.startswith("দূর" + HC):
        return "দুর" + HC + w[4:]
    return None

DETERMINISTIC = [("academy:reph-no-doubling", rule_reph_doubling),
                 ("academy:no-final-visarga", rule_final_visarga),
                 ("academy:dur-prefix-short-u", rule_dur_prefix)]

# ---------------------------------------------------------------- colloquial and sadhu
# (regex, replacement, how to find the verb root in the *standard* form for verification)
COLLOQ_SUBS = [
    (r"^(.{2,})স(ি|ে|ো|িলাম|িল|িলো|িলে|িলেন|েন|িস)$", r"\1ছ\2", r"^(.+)ছ"),   # করসি→করছি, কিনসে→কিনছে
    (r"^(.{2,})তাস(ি|ে|ো)$", r"\1ছ\2", r"^(.+)ছ"),                          # করতাসি→করছি
    (r"^(.+)ই(র|ল|খ|ক|ব)া$", r"\1\2ে", r"^(.+)ে$"),                         # কইরা→করে, বইলা→বলে, দেইখা→দেখে
    (r"^(.+)ই(র|ল|খ|ক)া(ছি|ছে|ছিল|ছিলাম)$", r"\1\2ে\3", r"^(.+)ে(?:ছি|ছে|ছিল|ছিলাম)$"),
]
SADHU_SUBS = [
    (r"িয়াছিলেন$", "েছিলেন"), (r"িয়াছিলাম$", "েছিলাম"), (r"িয়াছিল$", "েছিল"),
    (r"িয়াছেন$", "েছেন"), (r"িয়াছি$", "েছি"), (r"িয়াছে$", "েছে"),
    (r"িতেছিলেন$", "ছিলেন"), (r"িতেছিল$", "ছিল"), (r"িতেছেন$", "ছেন"), (r"িতেছি$", "ছি"), (r"িতেছে$", "ছে"),
    (r"িবেন$", "বেন"), (r"িবে$", "বে"), (r"িব$", "ব"), (r"িলেন$", "লেন"), (r"িলাম$", "লাম"), (r"িল$", "ল"),
    (r"িয়া$", "ে"), (r"িতে$", "তে"), (r"িবার$", "বার"),
    (r"^হইয়াছে$", "হয়েছে"), (r"^হইয়া$", "হয়ে"), (r"^হইবে$", "হবে"), (r"^হইল$", "হল"), (r"^হইতে$", "হতে"),
    (r"^যাইয়া$", "গিয়ে"), (r"^যাইবে$", "যাবে"), (r"^দিয়াছে$", "দিয়েছে"), (r"^লইয়া$", "নিয়ে"),
]
SADHU_PRONOUNS = {
    "তাহা": "তা", "তাহার": "তার", "তাহাকে": "তাকে", "তাহারা": "তারা", "তাহাদের": "তাদের", "তাহাদিগকে": "তাদের",
    "যাহা": "যা", "যাহার": "যার", "যাহাকে": "যাকে", "যাহারা": "যারা", "যাহাদের": "যাদের",
    "ইহা": "এটা", "ইহার": "এর", "ইহাকে": "একে", "ইহারা": "এরা", "ইহাদের": "এদের",
    "উহা": "ওটা", "উহার": "ওর", "কাহার": "কার", "কাহাকে": "কাকে", "কেহ": "কেউ", "কিছুই": None,
}

# ---------------------------------------------------------------- ি/ী, ু/ূ, ন/ণ, শ/ষ/স groups
SKEL = str.maketrans({"ী": "ি", "ূ": "ু", "ঈ": "ই", "ঊ": "উ", "ণ": "ন", "ষ": "স", "শ": "স"})
KEEP_II_SUFFIXES = ["ীয়", "ীকরণ", "ীভূত", "কারী", "চারী", "বাদী", "ধারী", "জীবী", "দর্শী", "ভাষী", "বাসী",
                    "গামী", "যোগী", "ভোগী", "প্রার্থী", "দায়ী", "স্থায়ী", "শালী", "প্রেমী", "কামী", "ত্যাগী",
                    "মন্ত্রী", "কর্মী", "যাত্রী", "নেত্রী", "পত্নী", "শিল্পী", "জ্ঞানী", "অনুরাগী", "সঙ্গী", "স্বামী", "প্রাণী",
                    "রোগী", "লোভী", "জয়ী", "বিজয়ী", "দুঃখী", "সুখী", "ধনী", "গুণী", "তরুণী", "জননী"]
FOREIGN_KARI = ("সরকার", "দরকার", "তরকার")
TAIL_RE = r"(র|তে|কে|য়|টা|টি|গুলো|গুলি|দের|রা|ও|ই|য়ে|তেও|কেও|রও|দেরকে|রাই|দেরও)"
SADHU_EXCEPTIONS = {"উড়িয়া", "ওড়িয়া"}
NOUN_SUFFIX_RE = None  # built later

def harmonize(stem):
    """Chalit perfective vowel harmony: রাখ→রেখ (রাখিয়া→রেখে), আস→এস (আসিয়া→এসে)."""
    if stem.startswith("আ"):
        return "এ" + stem[1:]
    idx = stem.rfind("া")
    if idx != -1 and idx == len(stem) - 2:
        return stem[:idx] + "ে" + stem[idx+1:]
    return stem

def ii_position_rule(a, b, ppm):
    """a and b differ only by ি↔ী / ু↔ূ at positions. Return (preferred, rule, conf) or None."""
    diff = [i for i, (x, y) in enumerate(zip(a, b)) if x != y]
    if len(a) != len(b) or len(diff) != 1:
        return None
    i = diff[0]
    lo, hi = (a, b) if a[i] in "িুইউ" else (b, a)   # lo = short form, hi = long form
    if hi[i] in "ীঈ":
        # tatsama suffix contexts keep ী
        for suf in KEEP_II_SUFFIXES:
            pos = hi.rfind(suf)
            if pos <= 0 or not (pos <= i < pos + len(suf)):
                continue
            tail = hi[pos + len(suf):]
            if tail and not re.fullmatch(TAIL_RE, tail):
                continue                      # suffix must end the word (plus a case ending)
            if suf == "কারী" and any(r in hi for r in FOREIGN_KARI):
                return lo, "academy:foreign-root-short-i", "high"
            stem = hi[:pos]
            # a real tatsama suffix attaches to a real word (উপকার+ী, এলাকা+বাসী, জাতি→জাত+ীয়);
            # শিকারি, বেকারি, পায়চারি only look like it
            letters = sum(1 for ch in stem if ch in CONS or "\u0985" <= ch <= "\u0994")
            need = 5.0 if suf in ("কারী", "চারী") else 1.0
            if letters < 2 or max(ppm(stem), ppm(stem + "ি")) < need or stem.endswith("য়"):
                continue
            if ppm(hi) >= 0.25 * ppm(lo):
                return hi, "academy:tatsama-suffix-keeps-ii", "high"
            return lo, "data:short-i-dominant", "low"
        # word-final (or before a case/classifier ending) ী in a doubly-valid/non-tatsama word → ি
        stem_end = len(hi) - 1
        tail = hi[i+1:]
        if i == stem_end or re.fullmatch(TAIL_RE, tail or "-"):
            # Academy: non-tatsama (and doubly-valid tatsama) words take ি. Tatsama-only ী words
            # (প্রাণী, তরুণী) show up as an overwhelming ী majority, so the rule yields to data there.
            if ppm(lo) >= 0.25 * ppm(hi):
                return lo, "academy:non-tatsama-final-short-i", "medium"
            return hi, "data:likely-tatsama-keeps-ii", "low"
    return None

def main():
    words_path, seed_path, out_dir = sys.argv[1:4]
    lex = load_words(words_path)
    seed_prefer, seed_distinct = load_seed(seed_path)
    ship = {w for w, e in lex.items() if e["ppm"] >= SHIP_PPM}
    ppm = lambda w: lex[w]["ppm"] if w in lex else 0.0
    def is_verb_root(x):
        """x is a verb root if its chalit infinitive (x+তে) and another finite form are common."""
        return len(x) >= 1 and ppm(x + "তে") >= 1.0 and max(ppm(x + "বে"), ppm(x + "ছে"), ppm(x + "লে"), ppm(x + "ল")) >= 0.3
    R = {w: {"cls": "standard", "pref": "", "rule": "", "conf": "", "root": "", "suffix": "", "tags": set()} for w in ship}
    review = []   # (priority ppm, word, suggested, alternative, rule, reason)

    def mark(w, cls, pref, rule, conf):
        r = R[w]
        if r["rule"].startswith("curated"):
            return
        r.update(cls=cls, pref=pref, rule=rule, conf=conf)

    for w, e in lex.items():
        if w in R and "single_source" in e["flags"]:
            R[w]["tags"].add("single_source")

    # 1. curated seed (highest authority)
    for w in ship:
        if w in seed_distinct:
            R[w].update(rule="curated:distinct", conf="high"); R[w]["tags"].add("distinct_word")
        if w in seed_prefer:
            pref, source, note = seed_prefer[w]
            if pref != w:
                R[w].update(cls="variant", pref=pref, rule=f"curated:{source}", conf="high")
            else:
                R[w].update(rule=f"curated:{source}", conf="high")

    # 2. deterministic Academy rules
    for w in ship:
        for name, fn in DETERMINISTIC:
            t = fn(w)
            if t and t != w:
                cls = "variant" if ppm(w) >= 0.1 * ppm(t) or ppm(t) == 0 else "typo"
                mark(w, cls, t, name, "high")
                break

    # 3. colloquial and sadhu forms
    for w in ship:
        if R[w]["cls"] != "standard" or R[w]["rule"].startswith("curated"):
            continue
        if w in SADHU_PRONOUNS:
            mark(w, "sadhu", SADHU_PRONOUNS[w] or "", "register:sadhu-pronoun", "high"); continue
        done = False
        for pat, rep in SADHU_SUBS:
            if re.search(pat, w):
                t = re.sub(pat, rep, w, count=1)
                stem = w[:re.search(pat, w).start()]
                if "িয়া" in pat and not pat.startswith("^"):
                    t = harmonize(stem) + rep
                verb_ok = pat.startswith("^") or (is_verb_root(stem) and len(stem) >= 2 and w not in SADHU_EXCEPTIONS)
                if t != w and verb_ok and ppm(t) >= max(1.0, 2 * ppm(w)) and len(t) >= 2:
                    mark(w, "sadhu", t, "register:sadhu-verb", "medium"); done = True; break
        if done:
            continue
        for pat, rep, root_pat in COLLOQ_SUBS:
            if re.search(pat, w):
                t = re.sub(pat, rep, w, count=1)
                if "ই(র|ল|খ|ক" in pat:
                    g = re.match(pat, w)
                    t = harmonize(g.group(1) + g.group(2)) + rep.split("\\2", 1)[1].replace("\\3", g.group(3) if g.lastindex >= 3 else "")
                    root = g.group(1) + g.group(2)
                    m = re.match(r"(.+)", root)
                else:
                    m = re.search(root_pat, t)
                if t != w and m and ppm(t) >= max(1.0, 5 * ppm(w)) and (t.startswith("গে") or is_verb_root(m.group(1))):
                    mark(w, "colloquial", t, "register:colloquial", "medium"); break

    # 4. confusable spelling groups (ি/ী ু/ূ ন/ণ শ/ষ/স)
    groups = collections.defaultdict(list)
    for w in ship:
        groups[w.translate(SKEL)].append(w)
    for skel, ws in groups.items():
        if len(ws) < 2:
            continue
        ws.sort(key=lambda x: (-ppm(x), x))   # ties by word: same output every run
        top = ws[0]
        for w in ws[1:]:
            if R[w]["cls"] != "standard" or R[w]["rule"].startswith("curated") or R[top]["rule"] == "curated:distinct":
                continue
            rule = ii_position_rule(top, w, ppm)
            if rule:
                pref, rname, conf = rule
                other = w if pref == top else top
                if R[other]["rule"].startswith("curated") or R[other]["cls"] != "standard":
                    continue
                cls = "variant" if ppm(other) >= ppm(pref) / 20 else "typo"
                mark(other, cls, pref, rname, conf)
                if ppm(other) > ppm(pref):
                    review.append((ppm(other), other, pref, other, rname, "rule prefers the less common spelling"))
                elif conf == "low" and ppm(pref) >= 1:
                    review.append((ppm(other), other, pref, other, rname, "kept ী because usage is overwhelming; Academy may prefer ি"))
                continue
            ratio = ppm(top) / max(ppm(w), 1e-9)
            if ppm(w) >= 1.0:
                # common enough that it may be a separate word (দিন/দীন, সব/শব): never call it a typo
                if ratio >= 10 and len(w) >= 4:
                    mark(w, "variant", top, "data:confusable-common-misspelling", "low")
                    review.append((ppm(w), w, top, w, "data:confusable-common-misspelling", "common misspelling, or a separate word?"))
                elif ratio < 50:
                    review.append((ppm(w), w, top, w, "data:confusable-pair", "both common: variant or separate word?"))
                continue
            if ratio >= 20 and len(w) >= 4:
                mark(w, "typo", top, "data:confusable-rare", "medium")
            elif ratio >= 20:
                R[w]["tags"].add(f"suspect_typo_of={top}")
            elif ratio >= 3:
                mark(w, "variant", top, "data:confusable-less-common", "low")
                review.append((ppm(w), w, top, w, "data:confusable-less-common", "less common spelling of the same word?"))

    # 5. typo filter: edit distance 1 (non-confusable edits) against much more frequent core words
    core = [w for w in ship if ppm(w) >= CORE_PPM and R[w]["cls"] == "standard"]
    dindex = collections.defaultdict(list)
    for w in core:
        for i in range(len(w)):
            dindex[w[:i] + w[i+1:]].append(w)
        dindex[w].append(w)
    def ed1_targets(w):
        keys = {w[:i] + w[i+1:] for i in range(len(w))} | {w}
        cands = set()
        for k in keys:
            cands.update(dindex.get(k, ()))
        cands.discard(w)
        return cands
    def edit_ok(w, t):
        """Reject edits that usually separate real words: last-letter edits, ঁ, and confusables (handled above)."""
        if abs(len(w) - len(t)) > 1:
            return False
        if "ঁ" in (set(w) ^ set(t)):
            return False
        n = min(len(w), len(t))
        first = next((i for i in range(n) if w[i] != t[i]), n)
        if first >= max(len(w), len(t)) - 1 and HC not in (w[-1:], t[-1:]):
            return False
        if w.translate(SKEL) == t.translate(SKEL):
            return False
        return dl1(w, t)
    for w in ship:
        r = R[w]
        if r["cls"] != "standard" or r["rule"] or ppm(w) >= 1.0 or len(w) < 4:
            continue
        best, best_kind = None, None
        for t in ed1_targets(w):
            if ppm(t) >= 30 * ppm(w) and edit_ok(w, t) and (best is None or ppm(t) > ppm(best)):
                best, best_kind = t, typo_shape(w, t)
        if not best:
            continue
        if best_kind and ppm(best) >= 50 * ppm(w) and ppm(w) < 0.5:
            mark(w, "typo", best, f"data:typo-{best_kind}", "medium")
        else:
            r["tags"].add(f"suspect_typo_of={best}")

    # 6. roots and endings
    for w in ship:
        root, suf = analyse(w, lex)
        if root:
            R[w]["root"], R[w]["suffix"] = root, suf

    # resolve chains (a → b → c) so every pointer lands on a standard word
    for w, r in R.items():
        seen, p = {w}, r["pref"]
        while p and p in R and R[p]["pref"] and p not in seen:
            seen.add(p); p = R[p]["pref"]
        r["pref"] = p

    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "lexicon.tsv"), "w", encoding="utf-8") as f:
        f.write("word\tppm\ttier\tclass\tpreferred\troot\tsuffix\trule\tconfidence\ttags\n")
        for w in sorted(ship, key=lambda x: (-ppm(x), x)):
            r = R[w]
            tier = "core" if ppm(w) >= CORE_PPM else "extended"
            f.write(f"{w}\t{ppm(w):.4f}\t{tier}\t{r['cls']}\t{r['pref']}\t{r['root']}\t{r['suffix']}\t{r['rule']}\t{r['conf']}\t{','.join(sorted(r['tags']))}\n")
    review.sort(key=lambda x: (-x[0], x[1], x[2]))
    with open(os.path.join(out_dir, "review-spelling.tsv"), "w", encoding="utf-8") as f:
        f.write("word\tsuggested_standard\tppm_word\tppm_suggested\trule\tquestion\tdecision\n")
        seen = set()
        for _, w, pref, alt, rule, why in review:
            key = tuple(sorted((w, pref)))
            if key in seen:
                continue
            seen.add(key)
            f.write(f"{w}\t{pref}\t{ppm(w):.3f}\t{ppm(pref):.3f}\t{rule}\t{why}\t\n")
    c = collections.Counter(r["cls"] for r in R.values())
    print("classes:", dict(c), "review:", len(seen), file=sys.stderr)

# ---------------------------------------------------------------- helpers
TYPO_SUBS = [set("চছ"), set("তৎ"), set("জয"), {"ড়", "র"}, set("ঙং"), set("ঁ")]
def typo_shape(w, t):
    """Name the edit if it is a typical Bangla typing mistake, else None."""
    lw, lt = len(w), len(t)
    if lw != lt:
        a, b = (w, t) if lw > lt else (t, w)      # a is longer
        i = next((k for k in range(len(b)) if a[k] != b[k]), len(b))
        extra = a[i]
        if extra == HC:
            return "hasanta"
        if ((i > 0 and a[i-1] == extra) or (i + 1 < len(a) and a[i+1] == extra)) and i < len(a) - 4:
            return "doubled-letter"
        if extra == "া" and a[max(0, i-2):i] == HC + "য":
            return "ya-phala-aa"
        return None
    diff = [k for k in range(lw) if w[k] != t[k]]
    if len(diff) == 2 and diff[1] == diff[0] + 1:
        return "swap"
    if len(diff) == 1:
        pair = {w[diff[0]], t[diff[0]]}
        if any(pair <= g for g in TYPO_SUBS if len(g) > 1):
            return "letter-confusion"
    return None

def dl1(a, b):
    """True if Damerau-Levenshtein distance(a, b) == 1."""
    if a == b: return False
    la, lb = len(a), len(b)
    if abs(la - lb) > 1: return False
    i = 0
    while i < min(la, lb) and a[i] == b[i]: i += 1
    if la == lb:
        if a[i+1:] == b[i+1:]: return True
        return i + 1 < la and a[i] == b[i+1] and a[i+1] == b[i] and a[i+2:] == b[i+2:]
    if la > lb: return a[i+1:] == b[i:]
    return a[i:] == b[i+1:]

CLASSIFIERS = ["", "টা", "টি", "টো", "খানা", "খানি", "গুলো", "গুলি", "গুলা", "রা", "েরা", "দের", "দেরকে", "দিগের"]
CASES = ["", "কে", "র", "ের", "তে", "য়", "ে", "য়ে", "েতে", "রে"]
EMPH = ["", "ই", "ও"]
SUFFIXES = sorted({c + k + e for c in CLASSIFIERS for k in CASES for e in EMPH} - {""}, key=len, reverse=True)

def analyse(w, lex):
    """Longest known ending whose remaining stem is a frequent word. Annotation only."""
    for suf in SUFFIXES:
        if len(suf) >= len(w) - 1 or not w.endswith(suf):
            continue
        stem = w[:-len(suf)]
        e = lex.get(stem)
        letters = sum(1 for ch in stem if ch in CONS or "\u0985" <= ch <= "\u0994")
        if not e or e["ppm"] < 0.3 or letters < 2:
            continue
        if suf in ("ে", "ের", "েতে", "েরা") and (stem[-1] not in CONS or e["ppm"] < 1.0):
            continue
        if suf in ("র", "য়", "য়ে", "রা", "রে") and stem[-1] in CONS:
            continue
        return stem, suf
    return "", ""

if __name__ == "__main__":
    main()
