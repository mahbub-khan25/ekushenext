#!/usr/bin/env python3
"""Ekushe Next Bangla → Roman generator and the forgiving lookup key ("skeleton").

Clean-room design: every Bangla word gets several weighted romanizations covering
(a) precise phonetic typing (case-sensitive: T ট, D ড, N ণ, Sh ষ, R ড়, O ও, I ঈ, U ঊ) and
(b) casual Banglish (amar, valobashi, ase, kmn), each with a small cost.
`skeleton()` maps any Roman string (user input or generated) to a vowel-free, aspiration-free,
case-free key so very different spellings of the same word meet in one bucket.
"""
import heapq, os, re
FINAL_O = float(os.environ.get("EKUSHE_FINAL_O", "0.3"))   # cost of pronouncing a final inherent vowel

HC, NUKTA, ZWJ = "্", "়", "‍"
RRA, RHA, YYA = "\u09DC", "\u09DD", "\u09DF"   # precomposed ড় ঢ় য় (internal only)

CONS = {
    "ক": [("k", 0)], "খ": [("kh", 0), ("k", .6)], "গ": [("g", 0)], "ঘ": [("gh", 0), ("g", .6)],
    "ঙ": [("ng", 0), ("Ng", .2)], "চ": [("c", 0), ("ch", 0)], "ছ": [("ch", 0), ("chh", .2), ("s", .8)],
    "জ": [("j", 0), ("z", .5)], "ঝ": [("jh", 0), ("j", .6)], "ঞ": [("NG", 0), ("n", .3)],
    "ট": [("T", 0), ("t", .1)], "ঠ": [("Th", 0), ("th", .1), ("t", .6)], "ড": [("D", 0), ("d", .1)],
    "ঢ": [("Dh", 0), ("dh", .1), ("d", .6)], "ণ": [("N", 0), ("n", .1)], "ত": [("t", 0)],
    "থ": [("th", 0), ("t", .6)], "দ": [("d", 0)], "ধ": [("dh", 0), ("d", .6)], "ন": [("n", 0)],
    "প": [("p", 0)], "ফ": [("f", 0), ("ph", .2)], "ব": [("b", 0), ("v", .7)], "ভ": [("bh", 0), ("v", .1)],
    "ম": [("m", 0)], "য": [("z", 0), ("j", .1), ("y", .6)], "র": [("r", 0)], "ল": [("l", 0)],
    "শ": [("sh", 0), ("s", .2)], "ষ": [("Sh", 0), ("sh", .1), ("s", .3)], "স": [("s", 0), ("sh", .3)],
    "হ": [("h", 0)], RRA: [("R", 0), ("r", .1), ("rh", .5)], RHA: [("Rh", 0), ("rh", .1), ("r", .4)],
    YYA: [("y", 0), ("", .4)], "ৎ": [("t", 0), ("t``", .3)],
}
SIGNS = {
    "া": [("a", 0), ("aa", .6)], "ি": [("i", 0), ("e", .8)], "ী": [("I", 0), ("i", .1), ("ee", .3)],
    "ু": [("u", 0), ("o", .8)], "ূ": [("U", 0), ("u", .1), ("oo", .4)], "ৃ": [("rri", 0), ("ri", .2)],
    "ে": [("e", 0), ("a", .7), ("ae", .9)], "ৈ": [("OI", 0), ("oi", .1)], "ো": [("O", 0), ("o", .1), ("oo", .7)],
    "ৌ": [("OU", 0), ("ou", .1), ("ow", .5)],
}
INDEP = {
    "অ": [("o", 0), ("a", .5)], "আ": [("a", 0), ("aa", .6)], "ই": [("i", 0)], "ঈ": [("I", 0), ("i", .1), ("ee", .3)],
    "উ": [("u", 0)], "ঊ": [("U", 0), ("u", .1)], "ঋ": [("rri", 0), ("ri", .2)], "এ": [("e", 0), ("a", .7)],
    "ঐ": [("OI", 0), ("oi", .1)], "ও": [("O", 0), ("o", .1)], "ঔ": [("OU", 0), ("ou", .1)],
}
MODS = {"ং": [("ng", 0), ("n", .4)], "ঃ": [(":", 0), ("", .2)], "ঁ": [("^", 0), ("", .1)]}
SPECIAL_CLUSTERS = {
    "ক্ষ": [("kkh", 0), ("kh", .2), ("kSh", .1), ("x", .6)], "জ্ঞ": [("gg", 0), ("gy", .3), ("g", .4), ("jNG", .3)],
    "ঞ্চ": [("nc", 0), ("nch", .1)], "ঞ্জ": [("nj", 0)], "ঙ্ক": [("nk", 0), ("ngk", .1)],
    "ঙ্গ": [("ngg", 0), ("ng", .1)], "হ্ম": [("hm", 0), ("mm", .5)],
    "চ্ছ": [("cch", 0), ("chh", .1), ("ch", .2), ("ss", .6)], "চ্চ": [("cc", 0), ("cch", .1), ("ch", .2)],
}
DOUBLE = object()

def parse(word):
    """Split a Bangla word into units: ("C", [consonants], vowel_sign, mods) or ("V", vowel, mods)."""
    w = word.replace(ZWJ, "").replace("\u09A1" + NUKTA, RRA).replace("\u09A2" + NUKTA, RHA).replace("\u09AF" + NUKTA, YYA)
    units, i, n = [], 0, len(w)
    while i < n:
        c = w[i]
        if c in INDEP:
            u = ["V", c, []]; i += 1
            while i < n and w[i] in MODS: u[2].append(w[i]); i += 1
            units.append(u); continue
        if c in CONS:
            cl = [c]; i += 1
            while i + 1 < n and w[i] == HC and w[i+1] in CONS:
                cl.append(w[i+1]); i += 2
            sign = None
            if i < n and w[i] in SIGNS: sign = w[i]; i += 1
            if i < n and w[i] == HC: i += 1          # stray final hasanta
            mods = []
            while i < n and w[i] in MODS: mods.append(w[i]); i += 1
            units.append(["C", cl, sign, mods]); continue
        if c in MODS:
            units.append(["V", "", [c]]); i += 1; continue
        i += 1
    return units

def _beam(option_lists, k):
    beams = [(0.0, "")]
    for opts in option_lists:
        nb = []
        for cost, s in beams:
            for r, c in opts:
                if r is DOUBLE:
                    r = _last_consonant(s)
                nb.append((cost + c, s + r))
        beams = heapq.nsmallest(k, set(nb))
    return beams

def _last_consonant(s):
    m = re.search(r"([bcdfghjklmnpqrstvwxyzBCDFGHJKLMNPQRSTVWXYZ]+)$", s)
    return m.group(1)[-1].lower() if m else ""

def _cluster_options(cl, sign):
    """Option lists for a consonant cluster (+ its vowel sign)."""
    key = HC.join(cl)
    if key in SPECIAL_CLUSTERS:
        return [SPECIAL_CLUSTERS[key]], sign
    lists = []
    for j, c in enumerate(cl):
        if j == 0:
            if c == "র" and len(cl) > 1:
                lists.append([("r", 0), ("rr", .3)]); continue      # reph
            lists.append(CONS[c]); continue
        prev = cl[j-1]
        if j == 1 and cl[0] == "র":                                 # after reph: plain consonant
            lists.append(CONS[c]); continue
        two = prev + HC + c
        if two in SPECIAL_CLUSTERS and j == 1 and len(cl) == 2:
            pass
        if c == "য":                                                # য-ফলা
            if sign == "া" and j == len(cl) - 1:
                lists.append([("ya", 0), ("a", .3), ("e", .5), ("ae", .6)]); sign = "_consumed"
            else:
                lists.append([("y", 0), (DOUBLE, .1), ("", .3)])
        elif c == "ব" and prev != "ম":                              # ব-ফলা
            lists.append([("w", 0), ("b", .3), ("", .4), (DOUBLE, .4)])
        elif c == "ম" and j == len(cl) - 1 and prev not in "ন":     # ম-ফলা
            lists.append([("m", 0), ("", .4), (DOUBLE, .5)])
        elif c == "র":
            lists.append([("r", 0)])
        else:
            lists.append(CONS[c])
    return lists, sign

# spoken verb endings that start with a consonant: the stem vowel before them is not said (matches roman.rs)
VERB_ENDINGS = {"লাম", "লে", "লেন", "লো", "লি", "লিস", "তে", "তাম", "তেন", "তো", "তিস", "ছি", "ছে", "ছেন",
                "ছো", "ছিস", "ছিল", "ছিলাম", "ছিলে", "ছিলেন", "ছিলো", "বো", "বে", "বেন", "বি", "বিস"}
VERB_STEM_O = 0.1

def _units_text(units):
    out = []
    for u in units:
        if u[0] == "V":
            out.append(u[1] or ""); out.extend(u[2])
        else:
            out.append("\u09cd".join(u[1])); out.append(u[2] or ""); out.extend(u[3])
    return "".join(out)

YA_PHALA_A = 0.2  # word-initial য-ফলা typed with an a (matches roman.rs)

def romanize(word, k=8):
    """Up to k (cost, roman) pairs, cheapest first."""
    units = parse(word)
    lists = []
    for idx, u in enumerate(units):
        final = idx == len(units) - 1
        if u[0] == "V":
            if u[1]:
                lists.append(INDEP[u[1]])
        else:
            cl, sign, mods = u[1], u[2], u[3]
            cl_lists, sign = _cluster_options(cl, sign)
            lists.extend(cl_lists)
            before = len(lists)
            if sign == "_consumed":
                pass
            elif sign:
                lists.append(SIGNS[sign])
            elif cl == ["ৎ"]:
                pass
            elif cl == [YYA] and final:
                if units[idx - 1][0] == "C" and units[idx - 1][2] in ("ী", "ি"):
                    lists.append([("o", 0), ("", .2)])                # দ্বিতীয় ditiyo
                # হয় hoy, যায় jay: nothing
            elif final and len(cl) > 1:
                lists.append([("o", 0), ("", .3)])                    # কর্ম kormo, রক্ত rokto
            elif mods:
                lists.append([("o", 0), ("", .4)])                    # এবং ebong, দুঃখ
            elif final:
                lists.append([("", 0), ("o", FINAL_O)])
            else:
                nxt = units[idx + 1]
                # inherent vowel before য় is usually pronounced (হয় hoy); before a vowel unit it is dropped
                if nxt[0] == "C" and idx > 0 and _units_text(units[idx + 1:]) in VERB_ENDINGS:
                    lists.append([("", 0), ("o", VERB_STEM_O)])          # করতে korte, আসলাম ashlam
                else:
                    lists.append([("o", 0), ("", .3)] if nxt[0] == "C" else [("", 0), ("o", .3)])
            # word-initial consonant + য-ফলা, no vowel sign: "æ", typed a or o (ব্যস্ত byasto/byosto)
            if (idx == 0 and not sign and len(cl) == 2 and cl[1] == "য" and len(lists) > before
                    and any(o[0] == "o" for o in lists[-1])):
                lists[-1] = lists[-1] + [("a", YA_PHALA_A)]
        mods = u[2] if u[0] == "V" else u[3]
        for m in mods:
            lists.append(MODS[m])
    out = _beam(lists, k * 3)
    # দুঃখ is typed "dukkho": visarga before a consonant doubles it
    extra = []
    for cost, r in out:
        m = re.search(r":([bcdfghjklmnpqrstvwxz])", r)
        if m:
            extra.append((cost + .1, r[:m.start()] + m.group(1) + r[m.start() + 1:]))
    out = out + extra
    seen, res = set(), []
    for cost, r in sorted(out):
        if r and r not in seen:
            seen.add(r); res.append((round(cost, 2), r))
        if len(res) >= k:
            break
    return res

_SK = [("chh", "c"), ("ch", "c"), ("sh", "s"), ("ph", "f"), ("kkh", "k"), ("``", ""), ("ng", "n")]
def skeleton(roman):
    """Forgiving key: case-free, aspiration-free, vowel-free (except a leading-vowel marker), no doubles."""
    s = roman.lower()
    for a, b in _SK:
        s = s.replace(a, b)
    s = re.sub(r"(?<=[bcdfgjklmnpqrstvwxz])h", "", s)   # drop aspiration h
    s = s.replace("v", "b").replace("z", "j").replace("q", "k").replace("x", "ks").replace("w", "b")
    lead = "@" if s[:1] in "aeiouy" else ""
    s = re.sub(r"[aeiouy^:]", "", s)
    s = re.sub(r"(.)\1+", r"\1", s)
    return lead + s

if __name__ == "__main__":
    import sys
    for w in sys.argv[1:] or ["আমি", "বাংলা", "স্বাধীন", "কেমন", "ভালোবাসি", "আছে", "বিশ্ববিদ্যালয়", "শিক্ষা", "জ্ঞান",
                              "ব্যাংক", "হয়েছে", "সত্যি", "কম্পিউটার", "চাঁদ", "দুঃখ", "হঠাৎ", "প্রথম", "কর্ম", "পড়াশোনা", "মেয়ে"]:
        rs = romanize(w)
        print(w, "|", skeleton(rs[0][1]), "|", "  ".join(f"{r}({c})" for c, r in rs))
