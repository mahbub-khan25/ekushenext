# ekushe-dict-builder

Builds Ekushe Next's own Bangla dictionary from open-licensed text (see `SOURCES.md`).
Nothing here is taken from Avro, Lekho or OpenBangla.

## Pipeline

1. **Extract** — `scripts/extract_text.py` streams document text out of the parquet corpora.
2. **Count** — `ekushe-dict count` (Rust) tokenises Bangla words, normalises them and counts
   occurrences and document frequency. Normalisation (`src/norm.rs`):
   - Unicode NFC (Ekushe's canonical form; য়/ড়/ঢ় stay as letter + nukta, matching `ekushe-core`)
   - অ + া → আ, legacy ত্‍ → ৎ, ঁ moved after the vowel sign, doubled marks removed
   - ZWJ/ZWNJ removed except in র‍্য (র + ZWJ + ্ + য)
   - well-formedness state machine: rejects stray vowel signs, trailing hasanta, bad nukta, etc.
   - words glued to Latin letters/digits are skipped (e.g. the "এ" in "Facebookএ")
3. **Merge** — `scripts/merge_counts.py` keeps words found in ≥2 sources (or very frequent in one),
   scores each by mean frequency-per-million across sources, and flags confusable-spelling
   variants (`variant_of=`) when a neighbour is ≥20× more frequent.
4. **Tiers** — `scripts/split_tiers.py` writes `data/dictionary/words-core.tsv` and `words-extended.tsv`.
5. **Classify** — `scripts/classify.py` decides the standard spelling and tags variants, typos,
   colloquial and sadhu forms, roots and endings → `data/dictionary/lexicon.tsv` and
   `review-spelling.tsv`. Rules: `RULES.md`. Hand decisions: `curated/spelling-seed.tsv`.

6. **Roman index** — `scripts/romanize.py`, `build_index.py`, `match_english.py`, `rank.py` build
   Ekushe's own Roman → Bangla lookup → `data/dictionary/roman/`. Design: `ROMAN.md`.
   Accuracy check: `scripts/eval_pilot.py data/dictionary/roman eval/pilot.tsv --fails`.

7. **Pack** — `ekushe-dict pairs` counts adjacent word pairs (context), and `ekushe-dict pack` writes the
   engine's binary dictionary `data/dictionary/ekushe.dict` plus a compressed `ekushe.dict.z`, which the
   Windows tray app embeds (feature `embedded-dict` of `ekushe-core`). The romanizer and ranker the engine
   uses are in `crates/ekushe-core/src/roman.rs` and `dict.rs`. The Python scripts are the prototype; the Rust
   romanizer matches them exactly.

Run everything: `./build.sh` (≈1 hour on 2 cores, ~25 GB temporary disk).

## Status

Phases 1–5 done (gather, clean, classify, Roman index, binary dictionary + engine integration).
Next: Phase 6 (native-reviewed test set and a scorecard against other keyboards).
