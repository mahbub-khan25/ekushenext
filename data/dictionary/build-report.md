# Dictionary build report — Phases 1–2

Built 2026-10-20 by `tools/dict-builder` (see its README).

## Corpus

| Source | Documents | Words counted |
| --- | --- | --- |
| Bangla Wikipedia (2023-11-01) | 143,070 | 47,180,754 |
| FineWeb-2 `ben_Beng` (files 000–001) | 5,982,002 | 2,132,078,875 |
| Sangraha verified Bengali (files 0–9) | 1,492,633 | 564,586,280 |
| **Total** | **7,617,705** | **2,743,845,909** |

## Output

| File | Words | Rule |
| --- | --- | --- |
| `words-core.tsv` | 102,747 | ≥ 0.3 per million words |
| `words-extended.tsv` | 315,996 | 0.03–0.3 per million |

Columns: `word, ppm (mean per-million across sources), total, docs, wiki, web, sangraha, flags`.
Flags: `single_source` (frequent in only one source, mostly names), `variant_of=X` (a confusable
spelling X is ≥20× more common).

## Spot check (random 60 words from the lower half of each tier)

- **Core:** about 5 of 60 problematic: misspellings (ব্যাবসায়, নিদের্শনা, পূণ্যের), one flagged
  (পুর্বের), one fragment (অঁ). The rest are real words, names and inflected forms.
- **Extended:** about 8 of 60 problematic (প্রকৃয়ার, দূর্নিতীর, কদাচিং …). Use it for recognition and
  spell-check acceptance only after Phase 3 cleaning, not for top suggestions.

## Findings that shape Phase 3

- Popular ≠ standard: শ্রেণী (42.7/M) is more common than the Academy form শ্রেণি (34.9/M);
  সরকারী, তৈরী, বাড়ী are also common. Phase 3 must put the standard spelling first regardless.
- Context words: কোন (1246/M) and কোনো (1074/M) are both very common; they are different words,
  so ranking needs next-word context, not a spelling rule.
- Colloquial forms are present and countable: কইরা 6.7/M, করসি 0.17/M, গেসি 0.10/M.
- Typo filtering needs more than confusable pairs: hasanta insertions (অপরাধ্মূলক) and
  letter swaps (নিদের্শনা) slip through. Phase 3 adds an edit-distance check against frequent words.

---

# Phase 3 — classification

Output: `lexicon.tsv` (418,743 words) and `review-spelling.tsv` (10,558 questions). Rules: `tools/dict-builder/RULES.md`.

| Class | Core | Extended | Total |
| --- | --- | --- | --- |
| standard | 97,940 | 292,407 | 390,347 |
| variant (suggested below the standard form) | 3,086 | 11,959 | 15,045 |
| typo (never suggested, autocorrected) | 1,245 | 11,292 | 12,537 |
| sadhu (labelled সাধু) | 460 | 293 | 753 |
| colloquial (labelled কথ্য) | 16 | 45 | 61 |

Also: 116,862 words annotated with root + ending; 61,883 tagged `suspect_typo_of` (kept, rank lower).

## How decisions were made

| Basis | Words decided |
| --- | --- |
| Curated list (draft, needs review) | 130 |
| Bangla Academy rules | 11,097 |
| Register rules (sadhu/colloquial) | 814 |
| Frequency data | 16,426 |

## Spot checks (random samples of 30–60 per class, judged by Claude)

| Class / rule | Looks right |
| --- | --- |
| sadhu verbs (করিয়াছে → করেছে, রাখিয়া → রেখে) | ~95% |
| colloquial (কইরা → করে, গেসি → গেছি) | ~95% |
| typo: hasanta / swap / letter confusion | ~85–90% (misses: names, e.g. হেমলেট "Hamlet") |
| typo: rare ি/ী ন/ণ শ/ষ/স spelling | ~85% (misses: rare real words, e.g. মণের, সঙ্কর) |
| common misspellings (কারনে → কারণে, পুরষ্কার → পুরস্কার) | ~95% |
| final ি rule (সরকারী → সরকারি, জানুয়ারী → জানুয়ারি) | right by Academy style; applies to names too (মোদী → মোদি) |

## Known limits

- **Colloquial coverage is small (61 words).** Only verb patterns are detected. More everyday forms will
  come from the Phase 6 test set and from user learning.
- **Proper names follow the same ি/ী rule** as common words. They stay suggestable, but someone should
  review how names are handled.
- **English loanword tagging moved to Phase 4**, because it needs the Roman → Bangla index.
- **review-spelling.tsv needs a native speaker.** It is sorted by frequency: the first ~500 rows cover most
  of what people type. Fill the `decision` column (keep / use suggested / separate words).

---

# Phase 4 — Roman → Bangla index

Output in `data/dictionary/roman/`:

| File | Contents |
| --- | --- |
| `romanizations.tsv` | 406,202 suggestable words × up to 8 weighted romanizations |
| `roman-index.tsv` | 114,507 forgiving keys → words, most frequent first |
| `english-matches.tsv` | 6,185 English loanword spellings (school → স্কুল, computer → কম্পিউটার) |

## Pilot accuracy (`tools/dict-builder/eval/pilot.tsv`, 271 typed words)

| Set | Words | Right word first | Right word in top 3 |
| --- | --- | --- | --- |
| Held-out third (never used for tuning) | 90 | 96.7% | 97.8% |
| Whole pilot | 271 | 95.9% | 98.2% |
| — typed both ways (ami, kemon, prothom) | 147 | 99.3% | 100% |
| — Avro-style precise (bhalObasi, shohId, zuddho) | 37 | 100% | 100% |
| — casual Banglish (valo, kmn, sotti, office) | 87 | 88.5% | 94.3% |

**Caution:** I wrote the pilot myself, and it measures single words without context, so these numbers are
encouraging but not proof. Phase 6 replaces it with a larger, native-reviewed set and a head-to-head
comparison against other keyboards.

## What still goes wrong

- **Words that need context:** `ase` (আছে / আসে), `aso` (আছো / আসো), `bolo` (বলো / বল). Next-word data in
  Phase 5 can settle most of these.
- **Inflected form beats base form:** `phone` → ফোনে before ফোন; `hospital` → হসপিটাল before হাসপাতাল.
- **Loanword keys are about 85% right** (spot check of 100). The wrong ones rank below native matches, but
  `school` still shows সকল first; the English key cost needs a per-word check.
- **Rare casual spellings:** `haan` for হ্যাঁ, `hoyse` for হয়েছে.
- **Prefix completion** (showing বাংলাদেশ while you type `bangl`) isn't built yet. It's part of Phase 5.
- **Speed:** the Python reference takes ~36 ms per word. The Rust port in Phase 5 must stay under 1 ms.

---

# Phase 5 — binary dictionary and engine integration

## What the engine now uses

| Piece | Where | Notes |
| --- | --- | --- |
| Romanizer + lookup key | `crates/ekushe-core/src/roman.rs` | Rust port; identical output to `romanize.py` on 30,000 random words and 286 typed strings |
| Dictionary + ranker | `crates/ekushe-core/src/dict.rs` | Zero-copy binary format, typo-tolerant lookup, word completion, word-pair context |
| `ekushe.dict.z` | `data/dictionary/` | 418,743 words, 114,506 keys, up to 16 followers per common word; 13.7 MB compressed, 39.9 MB in memory |
| Windows tray | `crates/ekushe-tray/Cargo.toml` | Embeds the dictionary (feature `embedded-dict`); unpacks at startup in ~110 ms |
| CLI / FFI | — | Pick up `ekushe.dict(.z)` from next to the executable, `%LOCALAPPDATA%\Ekushe\`, or `data/dictionary/` |

Word pairs were counted over 1.07 billion adjacent-word pairs (Wikipedia, Sangraha, FineWeb-2 file 000),
keeping the 16 most frequent followers of each common word.

## Accuracy (pilot, 271 words, typed through the full engine; the word Space commits)

| Mode | Committed correctly |
| --- | --- |
| Smart (**default** since 2026-10-20) | **97.4%** |
| Phonetic-First | 94.1% |
| Ranker alone, top 1 / top 3 | 96.3% / 98.5% |

Phonetic-First loses mostly where Avro rules make a plain `o` the inherent vowel: `valo` → ভাল, `koro` → কর,
`dekho` → দেখ. Smart mode picks ভালো, করো, দেখো.

## Context (25 phrases, `tools/dict-builder/eval/context.tsv`)

Smart mode gets 18/25 with or without context. Context fixes cases the corpus covers well (`kemon aso` →
আছো) but can push toward news-style pairs (`ki koro` → করে, from the very common কি করে). The corpora are
formal, so conversational pairs (তুমি-forms, imperatives) are thin. Conversational data and user learning
are the way to improve this.

## Speed (2-core cloud machine, release build)

| Measure | Value |
| --- | --- |
| Ranking one typed prefix (1,414 prefixes) | mean 0.65 ms, 95th percentile 1.3 ms |
| Full keystroke through the engine | mean ~0.7 ms, worst ~3–11 ms (spikes while other jobs ran) |
| Startup (unpack embedded dictionary) | ~110 ms, once |

## Engine changes

- **Candidates:** with the dictionary, `SuggestionEngine` uses Ekushe's ranking instead of the 117-word
  list. Snippets, user-learned words, user autocorrect, emoji and the English word are kept. Colloquial,
  sadhu and variant candidates carry the labels কথ্য, সাধু, ভিন্ন বানান.
- **Smart is the default.** The inline text always shows exactly what Space will commit. It's always a
  whole-word match (completions appear only in the bar), and the first keystroke shows the plain letter.
- **Phonetic-First** (optional): the literal phonetic spelling leads when it's a common, standard word in the
  ranking's top 3. Otherwise the dictionary's choice leads. Existing users who saved Phonetic-First keep it.
- **Context:** the last committed word is carried as context across a plain space.
- **Spell checker:** uses the dictionary. Typos get their recorded correction first (কিনতু → কিন্তু).
  Confident non-standard spellings are reported as `non_standard` (সরকারী → সরকারি). Precomposed য়/ড়/ঢ়
  typed with other keyboards is accepted.
- **Fixes:** the য় confusable-pair bug in the spell checker; a single `r` before a consonant is now র
  (korte → করতে, dorkar → দরকার); রেফ via `rr` or the dictionary (kormo → কর্ম).
- **Tests:** 18 pass, including 5 new tests that run the engine with the real dictionary. A Windows
  `cargo check` of the whole workspace passes. The tray app has not been run on Windows yet.
