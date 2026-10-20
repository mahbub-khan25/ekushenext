# Ekushe Next Roman → Bangla lookup (Phase 4)

Clean-room design. It is not Avro's regex-dictionary search, and nothing is copied from Avro, Lekho or OpenBangla.

## 1. Every word gets its own romanizations — `scripts/romanize.py`
Each Bangla word is split into syllable units (consonant cluster + vowel sign + marks), and each unit
has weighted Roman options. A beam search keeps the cheapest 8 (core words) or 5 (extended).

- **Precise** (cost 0): case-sensitive phonetic letters: `T` ট, `D` ড, `N` ণ, `Sh` ষ, `R` ড়, `O` ও/ো, `I` ঈ/ী, `U` ঊ/ূ.
- **Casual** (small cost): `v` for ভ, `s` for শ/ষ, `ch` for ছ and also `s` (ase = আছে), `aa` for া, dropped
  inherent vowels (kmn), doubled consonants for য-ফলা (sotti, bakko), `w`/`b`/nothing for ব-ফলা
  (swadhin, sbadhin, shadhin), `dukkho` for দুঃখ, `bank` for ব্যাংক.
- **Inherent vowel**: kept inside words (kemon), optional at the end (bol / bolo, cost 0.3), pronounced after a
  final cluster (kormo, rokto) and before ং (ebong).

Example: ভালোবাসি → bhalObasi, bhalobasi, valObasi, valobasi, bhalObashi, valobashi …

## 2. Forgiving key — `skeleton()`
User input and every generated romanization reduce to the same key: lower-case, aspiration `h` dropped
after consonants, `sh`→`s`, `ch`→`c`, `v`→`b`, `z`→`j`, `w`→`b`, `ng`→`n`, vowels removed (a leading vowel
leaves `@`), doubled letters collapsed. `valobashi`, `bhalObasi` and `vlbs` all become `blbs`.
`roman-index.tsv` maps each key to its words, most frequent first (114,507 keys).

## 3. Ranking — `scripts/rank.py` (reference; Phase 5 ports it to Rust)
```
score = cheapest( romanization cost + weighted edit distance(typed, romanization) − exact-match bonus )
        − 0.55 × log10(frequency per million)
        + class penalty (variant 0.9, colloquial 0.7, sadhu 1.2)
```
- **Edit distance:** vowel swaps, `h`, `y`/`w` and doubled letters are cheap. The final vowel costs more,
  because it carries meaning (করো/করে).
- **Typo tolerance:** if the key's bucket is small, buckets one consonant away are searched too.
- **Standard form always offered:** a colloquial, sadhu or variant match also brings in its standard form,
  so `korsi` offers করছি and `sorkari` offers সরকারি.
- **Vulgar words** (`curated/vulgar-prefixes.tsv`) only appear when typed exactly.

## 4. English loanwords — `scripts/match_english.py`
The top 60,000 English words (wordfreq, CC BY-SA 4.0) are respelled the way Bangla speakers say them:
school→skul, mobile→mobail, -tion→shon. Each respelling is matched to a Bangla word with the same key and
a near-identical romanization. Filters drop inflected forms and words with native-only letters (ণ ষ ঁ ঃ ঋ …).
The 6,185 matches become extra keys at a higher cost, so native spellings still win.
About 85% of a 100-pair sample were right. The wrong ones sit behind native matches and rarely surface.

## Weights
The weights were tuned on two-thirds of `eval/pilot.tsv`. The remaining third was held out
(see `data/dictionary/build-report.md`).
