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

7. **Pack** — `ekushe-dict pairs` counts adjacent word pairs (context; counted with the words in step 2,
   one corpus file at a time, then summed by `ekushe-dict merge-pairs`), and `ekushe-dict pack` writes the
   engine's binary dictionary `data/dictionary/ekushe.dict` plus a compressed `ekushe.dict.z`, which the
   Windows tray app embeds (feature `embedded-dict` of `ekushe-core`). The romanizer and ranker the engine
   uses are in `crates/ekushe-core/src/roman.rs` and `dict.rs`. The Python scripts are the prototype (the
   Rust romanizer has since gained a few spellings, such as s for ছ, that the prototype lacks).

   Curated inputs to `pack` (all in `curated/`, all hand-editable):
   - `chat-phrases.txt` — everyday chat phrases; their word pairs get a strong count (context).
   - `spoken-verbs.tsv` (`--spoken`, written by `scripts/spoken_forms.py`) — chat verb forms (খাইছো,
     করতেছি) with the spellings people type and their standard form: the standard word is offered first,
     the spoken one right after it. Only forms that occur in the corpus are listed.
   - `names.tsv`, `chat-words.tsv` (`--keys`) — how people type common names and chat words; the spellings
     go to whichever candidate spelling the corpus uses most (rahim → রহিম).
   - `vulgar-prefixes.tsv` — words offered only when typed exactly.

Run everything: `./build.sh` (5-6 hours on 2 cores; about 8 GB of disk, since each corpus file is
downloaded, counted and deleted in turn; an interrupted build resumes where it stopped). `COUNT_ONLY=1`
stops after counting. To re-pack without the corpus (a ranking or curated-list change), take the pairs from
the current dictionary: `cargo run --release -p ekushe-core --example dump_pairs -- data/dictionary/ekushe.dict.z pairs.tsv`.

## Review and test data from people

- **Spelling review:** `scripts/review_sheet.py data/dictionary/review-spelling.tsv review.xlsx` makes a
  workbook of the open spelling questions, most common first, with a dropdown per row;
  `scripts/import_review.py review.xlsx curated/spelling-seed.tsv NAME` brings the answers in as reviewed
  decisions.
- **Typing kit:** `scripts/typing_kit.py kit.xlsx` makes 100 everyday sentences for someone to type the way
  they normally do; `scripts/import_typing.py eval/community.tsv kit1.xlsx kit2.xlsx …` turns the kits into
  the held-out test set `eval/community.tsv`.
- **Why does a word rank where it does?** `cargo run --release -p ekushe-core --example why_rank --
  data/dictionary/ekushe.dict.z byabostha` prints the scores, frequencies and spellings behind it.

## Status

Phases 1–5 done (gather, clean, classify, Roman index, binary dictionary + engine integration).
Next: Phase 6 (native-reviewed test set and a scorecard against other keyboards).
