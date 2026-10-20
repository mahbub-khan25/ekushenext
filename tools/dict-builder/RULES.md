# Ekushe Next spelling and classification rules (Phase 3)

`scripts/classify.py` assigns every shipped word one class. Order of authority:
**curated list → deterministic Academy rules → register (sadhu/colloquial) → spelling groups → typo filter.**
A word set by an earlier step is never overridden by a later one.

| Class | Suggested? | Meaning |
| --- | --- | --- |
| `standard` | yes, first | the preferred spelling |
| `variant` | yes, below `preferred` | accepted but non-standard spelling |
| `colloquial` | yes, below `preferred`, labelled কথ্য | everyday/regional form |
| `sadhu` | yes, below `preferred`, labelled সাধু | literary sadhu-bhasha form |
| `typo` | no | misspelling; autocorrect target is `preferred` |

Tag `suspect_typo_of=X`: one typing-edit away from a much more frequent word, but the edit is also how
real words differ, so the word is kept. The engine can rank it lower. Tag `distinct_word`: the
curated list says it is a separate word from its look-alike (কোন/কোনো, কি/কী).

## 1. Curated list — `curated/spelling-seed.tsv`
Hand-written decisions; highest authority. `prefer` rows set the standard form; `distinct` rows protect
look-alike words from being merged. Rows marked `draft` were written by Claude from the Bangla Academy
rules and need a native speaker's confirmation; change `source` to `reviewed` when confirmed.

## 2. Deterministic Academy rules (confidence: high)
- **No consonant doubling after reph**: কার্য্য → কার্য, কর্ম্ম → কর্ম, সূর্য্য → সূর্য.
- **No visarga at word end**: মূলতঃ → মূলত, ক্রমশঃ → ক্রমশ (kept: পুনঃ, অন্তঃ, দুঃ, নিঃ and interjections).
  Also catches the colon misuse "বলেছেনঃ".
- **দুর্ prefix has short u**: দূর্নীতি → দুর্নীতি, দূর্ঘটনা → দুর্ঘটনা (দূর "far" is unaffected).

## 3. Register
- **Sadhu verbs**: করিয়াছে → করেছে, বলিতেছে → বলছে, করিবে → করবে, with chalit vowel harmony
  (রাখিয়া → রেখে, আসিয়া → এসে). Only applied when the stem is a real verb root (its চলিত
  infinitive such as রাখতে is common), so nouns like কাজিয়া or ভারতিয়া are left alone.
- **Sadhu pronouns**: তাহার → তার, যাহা → যা, ইহা → এটা …
- **Colloquial verbs**: করসি → করছি, গেসে → গেছে, কইরা → করে, রাইখা → রেখে, দেইখা → দেখে.
  Same verb-root check.

## 4. Spelling groups (ি/ী, ু/ূ, ন/ণ, শ/ষ/স)
Words that differ only in these letters form a group.
- **Final ী vs ি** (also before a case ending such as -র, -তে, -কে): the Academy rule for non-tatsama
  and doubly-valid tatsama words gives **ি** (সরকারি, শ্রেণি, জানুয়ারি, কোম্পানি). Tatsama-only ী words
  (প্রাণী, তরুণী) show up as an overwhelming ী majority (ি-form < 25% of the ী-form), so there the rule
  yields to the data and the pair goes to review.
- **Tatsama suffixes keep ী** when they end the word and attach to a real word: -ীয় (জাতীয়, কেন্দ্রীয়),
  -কারী (উপকারী, সহকারী), -বাসী, -জীবী, -বাদী, -যোগী, -মন্ত্রী … Foreign roots (সরকার, দরকার, তরকার)
  take ি.
- **Other positions** (no rule applies): the more frequent form leads.
  - rarer form < 1 per million and ≥20× rarer, 4+ letters → `typo`
  - 3–20× rarer → `variant` (low confidence) + review
  - common (≥1 per million) and ≥10× rarer → `variant` "common misspelling" (কারনে → কারণে) + review
  - common and close in frequency → kept as `standard` + review (homophones: আসা/আশা, বিষ/বিশ).

## 5. Typo filter (edit distance 1)
A rare word (< 0.5 per million, 4+ letters) that is one edit from a ≥50× more frequent word becomes a
`typo` only if the edit is a typical typing mistake:
hasanta added/removed (কিনতু → কিন্তু), adjacent swap (নাগিরক → নাগরিক), doubled letter outside the
ending (হযয়েছে → হয়েছে), extra া after য-ফলা (ব্যাবসা), or a letter confusion চ/ছ, ত/ৎ, জ/য, ড়/র, ঙ/ং.
Edits at the last letter, chandrabindu edits and other substitutions only add `suspect_typo_of`.

## 6. Roots and endings (annotation only)
Longest known ending (classifier টা/টি/গুলো…, plural রা/দের, case কে/র/ের/তে/য়/ে, emphatic ই/ও)
whose remaining stem is a frequent word with 2+ letters: বিকল্পগুলিতে = বিকল্প + গুলিতে.
