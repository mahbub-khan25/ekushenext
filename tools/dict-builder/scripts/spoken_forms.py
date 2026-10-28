#!/usr/bin/env python3
"""Spoken (Dhaka chat) verb forms → curated/spoken-verbs.tsv for `ekushe-dict pack --spoken`.

The source texts are written Bangla, so chat forms like খাইছো, খাইতেছি or করতেছ are rare there
(often under one per million words) and lose to unrelated common words. For the verbs below this
writes each spoken form that occurs in the lexicon, the spellings people type for it (khaiso,
kortesi), and the standard form it stands for (খেয়েছো, করছি), so the standard spelling is offered
first and the spoken one right after it. Forms that never occur in the corpus are left out: nothing
here invents a word.

Usage: spoken_forms.py data/dictionary/lexicon.tsv > curated/spoken-verbs.tsv
"""
import sys

# Spoken endings per person: (ছ-spelling, স-spelling, standard ending, typed endings)
PERSONS = [
    ("ি", "ি", "ি", ["i"]),          # আমি: খাইছি / খাইসি → খেয়েছি
    ("ো", "ো", "ো", ["o"]),          # তুমি: খাইছো / খাইসো → খেয়েছো
    ("", None, "", ["o", ""]),      # তুমি: খাইছ → খেয়েছ (a bare খাইস is not written)
    ("ে", "ে", "ে", ["e"]),          # সে: খাইছে / খাইসে → খেয়েছে
    ("েন", "েন", "েন", ["en"]),      # আপনি/তিনি: খাইছেন → খেয়েছেন
    ("স", "স", "িস", ["os", "is"]),  # তুই: খাইছস / খাইসস → খেয়েছিস
]

# Vowel-stem verbs: perfect (আমি খাইছি = খেয়েছি) and continuous (খাইতেছি = খাচ্ছি).
# (spoken stem, typed stem, standard perfect stem, standard continuous stem)
VOWEL_VERBS = [
    ("খাই", "khai", "খেয়েছ", "খাচ্ছ"),
    ("পাই", "pai", "পেয়েছ", "পাচ্ছ"),
    ("চাই", "chai", "চেয়েছ", "চাচ্ছ"),
    ("দি", "di", "দিয়েছ", "দিচ্ছ"),
    ("নি", "ni", "নিয়েছ", "নিচ্ছ"),
    ("হই", "hoi", "হয়েছ", "হচ্ছ"),
    ("ঘুমাই", "ghumai", "ঘুমিয়েছ", "ঘুমাচ্ছ"),
    ("বানাই", "banai", "বানিয়েছ", "বানাচ্ছ"),
    ("পাঠাই", "pathai", "পাঠিয়েছ", "পাঠাচ্ছ"),
    ("দেখাই", "dekhai", "দেখিয়েছ", "দেখাচ্ছ"),
    ("যাই", "jai", None, "যাচ্ছ"),      # perfect গেছি is standard already
]
# Consonant-stem verbs: continuous (করতেছি = করছি) and the তুই perfect (করছস = করেছিস).
# (stem, typed stem(s), standard perfect stem)
CONSONANT_VERBS = [
    ("কর", ["kor"], "করেছ"), ("বল", ["bol"], "বলেছ"), ("দেখ", ["dekh"], "দেখেছ"),
    ("শুন", ["shun", "sun"], "শুনেছ"), ("লিখ", ["likh", "lekh"], "লিখেছ"), ("পড়", ["por", "por"], "পড়েছ"),
    ("ধর", ["dhor"], "ধরেছ"), ("রাখ", ["rakh"], "রেখেছ"), ("বস", ["bos", "bosh"], "বসেছ"),
    ("আস", ["ash", "as"], "এসেছ"), ("থাক", ["thak"], "থেকেছ"), ("বুঝ", ["bujh", "buj"], "বুঝেছ"),
    ("জান", ["jan"], "জেনেছ"), ("পার", ["par"], "পেরেছ"), ("উঠ", ["uth"], "উঠেছ"),
    ("কিন", ["kin"], "কিনেছ"), ("ভাব", ["bhab", "vab"], "ভেবেছ"), ("শিখ", ["shikh", "sikh"], "শিখেছ"),
    ("হাস", ["has", "hash"], "হেসেছ"), ("চল", ["chol"], "চলেছ"), ("খেল", ["khel"], "খেলেছ"),
    ("ঘুর", ["ghur"], "ঘুরেছ"), ("ফির", ["fir", "phir"], "ফিরেছ"), ("লাগ", ["lag"], "লেগেছ"),
    ("খুঁজ", ["khuj", "khuj"], "খুঁজেছ"), ("মার", ["mar"], "মেরেছ"), ("ছাড়", ["char", "sar"], "ছেড়েছ"),
]


def typed(stem_r, mid_ch, mid_s, ends):
    out = []
    for e in ends:
        out += [stem_r + mid_s + e, stem_r + mid_ch + e]
    return out


def main():
    ppm = {}
    with open(sys.argv[1], encoding="utf-8") as f:
        next(f)
        for line in f:
            p = line.split("\t", 3)
            ppm[p[0]] = float(p[1])

    groups = []  # (spoken forms, typed spellings, standard candidates, note)

    def std_of(cands):
        return next((c for c in cands if c in ppm), "")

    for stem, sr, perf, cont in VOWEL_VERBS:
        for ch_e, s_e, std_e, ends in PERSONS:
            if perf:
                std = std_of([perf + std_e] + ([perf + "ো"] if std_e == "" else []))
                forms = [stem + "ছ" + ch_e] + ([stem + "স" + s_e] if s_e is not None else [])
                groups.append((forms, typed(sr, "ch", "s", ends), std, "perfect"))
            stdc = std_of([cont + std_e] + ([cont + "ো"] if std_e == "" else []))
            forms = [stem + "তে" + "ছ" + ch_e, stem + "তা" + "ছ" + ch_e] + ([stem + "তে" + "স" + s_e, stem + "তা" + "স" + s_e] if s_e is not None else [])
            groups.append((forms, typed(sr, "tech", "tes", ends) + typed(sr, "tach", "tas", ends), stdc, "continuous"))

    for stem, srs, perf in CONSONANT_VERBS:
        cont = stem + "ছ"
        for ch_e, s_e, std_e, ends in PERSONS:
            stdc = std_of([cont + std_e] + ([cont + "ো"] if std_e == "" else []))
            forms = [stem + "তে" + "ছ" + ch_e, stem + "তা" + "ছ" + ch_e] + ([stem + "তে" + "স" + s_e, stem + "তা" + "স" + s_e] if s_e is not None else [])
            keys = []
            for sr in srs:
                keys += typed(sr, "tech", "tes", ends) + typed(sr, "tach", "tas", ends)
            groups.append((forms, keys, stdc, "continuous"))
            if ch_e == "স":  # তুই perfect: করছস / করসস → করেছিস (other persons collide with standard words)
                std = std_of([perf + "িস"])
                keys = []
                for sr in srs:
                    keys += typed(sr, "ch", "s", ends)
                groups.append(([stem + "ছস", stem + "সস"], keys, std, "perfect"))

    print("# Spoken verb forms (generated by scripts/spoken_forms.py from its verb table; do not edit by hand).")
    print("# word\ttyped spellings\tstandard form\t[minimum frequency per million; default 3]")
    seen = set()
    n = 0
    for forms, keys, std, note in groups:
        found = [f for f in dict.fromkeys(forms) if f in ppm and f not in seen and f != std]
        if not found:
            continue
        found.sort(key=lambda f: -ppm[f])
        keys = [k for k in dict.fromkeys(keys) if k]
        for i, f in enumerate(found):
            seen.add(f)
            # the most common spelling gets the typed spellings and an everyday frequency; the others
            # are only marked as spoken forms of the standard word (frequency as counted)
            print(f"{f}\t{','.join(keys)}\t{std}" if i == 0 else f"{f}\t\t{std}\t0")
            n += 1
    print(f"{n} spoken forms", file=sys.stderr)


if __name__ == "__main__":
    main()
