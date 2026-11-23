# FONTLOG for Shishir Bangla

## Basic font information

Shishir Bangla is an open Bengali font family for Shishir (a Rust desktop) and Ekushe Next (a Bangla
keyboard). It takes SolaimanLipi 2.002's letterforms, Noto Sans Bengali's proportions next to Latin, its
paragraph colour and its Bengali digits, and Inter's Latin. It has four styles: Regular, Medium, SemiBold
and Bold. It is built from source in this repository: `python build.py` turns the pinned upstream fonts in
`sources/upstream/` into the family in `fonts/`. No upstream file is edited by hand; every change is code
in `shishir_bangla/`.

## Licence

Shishir Bangla is released under the **SIL Open Font License, Version 1.1** (`OFL.txt`).

All three upstreams are under OFL 1.1: SolaimanLipi 2.002 (its name ID 13: "This Font Software is
licensed under the SIL Open Font License, Version 1.1."), Noto Sans Bengali 3.011 and Inter 4.1. So the
whole family, including Inter's Latin (Task 5) and Noto's digits and conjuncts (Task 3), is one OFL 1.1
Font Software. (Until 22 November 2026 the family was built from SolaimanLipi 1.6.1, which is OFL 1.0;
the OFL FAQ, entry 8.8, does not let a derivative of an OFL 1.0 font move to 1.1. Khan's pick of 2.002
closed that question. The reasoning is in git history, in this file at commit `63d4958`.)

**What the licence covers.** The OFL defines "Font Software" as font files, data files, source code,
build scripts and documentation. In this repository, the build code (`build.py`, `shishir_bangla/`),
the tests (`tests/*.py`) and the documentation are part of the Font Software, under the same OFL 1.1 as
the fonts. There is no separate code licence. The one exception is the dictionary words in
`tests/corpus/` (`words-sample.txt`, `words-baseline.txt`). They are test data taken from Ekushe Next,
licensed separately under CC BY-SA 4.0 (`tests/corpus/NOTICE.md`), and they are not part of any font
release.

### Reserved Font Names

OFL 1.1 defines a Reserved Font Name as "any names specified as such after the copyright statement(s)".
Unlike OFL 1.0, it reserves nothing by default.

- **SolaimanLipi 2.002 declares no Reserved Font Name.** Checked on 2026-11-22 in all three fonts
  (Normal, Bold, Thin): every name record (IDs 0 to 19) was read, and none says "Reserved Font Name".
  The copyright statement (ID 0) is "Copyright © 2022 All Rights Reserved by Ekushey", with no name after
  it; ID 7 (trademark) repeats the same sentence and names no trademark. The zip
  (`https://ekushey.org/static/fonts/SolaimanLipi.zip`) holds only the three fonts, with no licence,
  FONTLOG or readme file, and Ekushey's SolaimanLipi page (`https://ekushey.org/font/solaimanlipi/`)
  states no licence terms. Even so, "SolaimanLipi" is not used in any Shishir Bangla font name, so the
  two are never confused; it appears only as credit in the copyright notice.
- **"All Rights Reserved" is an upstream wording quirk.** SolaimanLipi 2.002's copyright line (name IDs
  0 and 7) says "Copyright © 2022 All Rights Reserved by Ekushey", next to its licence record (ID 13),
  which grants the SIL Open Font License, Version 1.1. **The OFL grant governs**: it is the licence the
  copyright holder attached to the font, and "All Rights Reserved" is the usual copyright formula, not a
  second licence. Nothing in Shishir Bangla's licence changes because of it. The line is kept verbatim in
  `OFL.txt`, as OFL 1.1 condition 2 asks ("each copy contains the above copyright notice and this
  license"). Flagged for Khan, who may want Ekushey to confirm the wording.
- **Noto Sans Bengali and Inter declare no Reserved Font Name.** Checked again on 2026-11-22 for
  Task 5 (the Latin merge), in all four weights of each: no name record says "Reserved", and the
  licence files' copyright lines name none ("Copyright (c) 2016 The Inter Project Authors
  (https://github.com/rsms/inter)" and "Copyright 2022 The Noto Project Authors
  (https://github.com/notofonts/bengali)", with no "with Reserved Font Name" after either). Both names
  are trademarks according to their own name ID 7: Noto's says "Noto is a trademark of Google Inc.",
  and Inter's says "Inter UI and Inter is a trademark of rsms.". So neither name is used in any
  Shishir Bangla font name; they appear only as credit (name ID 0). The tests check, in every style,
  that no name record but ID 0 contains "SolaimanLipi", "Noto", "Inter" or "Ekushey".
- **Licence files in a release.** The release folder carries `OFL.txt` (the family's licence, with
  SolaimanLipi 2.002's notice) and, in `licenses/`, Inter's and Noto Sans Bengali's own licence files
  with their copyright lines, as OFL 1.1 condition 2 asks.

## Upstream credits

| Part | Upstream | Copyright | Licence |
| --- | --- | --- | --- |
| Bengali letterforms (Regular from Normal, Bold from Bold) | SolaimanLipi 2.002, designed by Solaiman Karim, version 2.002 developed by Al Mamun Hossen, distributed by Ekushey (https://ekushey.org) | Copyright © 2022 All Rights Reserved by Ekushey (the fonts' name ID 0) | SIL OFL 1.1 |
| Proportions next to Latin, paragraph colour, Bengali digits, the imported letters, signs and conjuncts listed in the ChangeLog (Tasks 2, 3 and 4) | Noto Sans Bengali 3.011, Joana Ranito (Universal Thirst) and Jelle Bosma (Monotype Design Team) | Copyright 2025 The Noto Project Authors (https://github.com/notofonts/bengali), as in the fonts' name ID 0 (the release's `OFL.txt` says 2022) | SIL OFL 1.1 |
| Latin (Task 5) | Inter 4.1, by Rasmus Andersson | Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter), as in the fonts' name ID 0 | SIL OFL 1.1 |

Where each file came from, its version and its sha256 are in `sources/upstream/SOURCES.md`; the build
checks every hash before it runs.

The corpus words in `tests/corpus/` come from the Ekushe Next dictionary and are CC BY-SA 4.0, not OFL;
see `tests/corpus/NOTICE.md`. They are test data, not part of the fonts.

## ChangeLog

### 22 November 2026 (Mahbub Khan) Shishir Bangla version 0.003, Task 5: Latin, naming, Windows, packaging

- **Inter's Latin** in every weight, from Inter 4.1 of the same weight (`shishir_bangla/latin.py`),
  scaled from 2048 to 1000 units per em (the scale Task 2 measured Inter's x-height and cap height at):
  Basic Latin, Latin-1, Latin Extended-A and -B, Latin Extended Additional, spacing modifiers and
  combining marks, general punctuation, super- and subscripts, currency, letterlike symbols, number
  forms, arrows and mathematical operators: 1191 code points. With them come Inter's `kern`, `mark`
  and `mkmk`, under a new `latn` script and in `DFLT`; `beng` and `bng2` are not touched.
- **Kept from the Bengali side:** the word space and no-break space (spaced for Bengali by Task 4b,
  257 to 264 units), ZWNJ, ZWJ, danda, double danda, om and the dotted circle; and SolaimanLipi's
  degree glyph, which also draws ৽ (U+09FD), so Inter's degree sign is a glyph of its own
  (`degree.latn`).
- **Inter's word space in Latin runs** (ruling R-5c): one space glyph, the Bengali one; in `latn` and
  `DFLT` a single adjustment (first in `kern`) widens or narrows it to Inter's, +24 / +8 / −9 / −27
  units in Regular / Medium / SemiBold / Bold (Inter 281 / 270 / 260 / 237 against 257 / 262 / 269 /
  264), and Inter's kerning pairs with its space (comma + space …) apply on top. Bengali runs keep
  the Bengali word space.
- **Stacked-accent capitals inside the line box** (ruling R-5a). Inter's Ǘ Ǚ Ǜ Ǻ Ḕ Ḗ Ḯ Ṍ Ṏ Ṑ Ṓ Ṧ Ṹ Ṻ
  and the Vietnamese Ấ Ầ Ẩ Ẫ Ắ Ằ Ẳ Ẵ Ế Ề Ể Ễ Ố Ồ Ổ Ỗ (30; their accents reach 1105 to 1108, above
  1085, the line box less its margin) are taken with their accents brought down
  (`latin.lower_accents`): the accent stack, unchanged in shape and keeping Inter's side-by-side
  Vietnamese design, moves down by its overshoot (1 to 27 units, by weight), closing the gap to the
  letter no further than Inter's tightest single-accent capital (24 to 26 units). Where that is not
  enough (Ǻ, whose ring already sits that close), the gap between ring and acute closes to the same
  limit and the rest comes from compressing the ring-and-acute stack vertically from its foot (at
  most 7.5%), in a copy (`ringacute.uc.downN`). So every glyph fits the line box 1090 / 472, which is
  unchanged, and so does every covered character typed decomposed (HarfBuzz composes it; before,
  without the precomposed glyphs, it decomposed them and stacked marks to 1141 to 1157), and every
  letter with one combining mark (test).
- **Typing shortcuts removed** (ruling R-5b, displayed text equals stored text;
  `shaping.drop_shortcuts`). SolaimanLipi's `rlig` showed ঈঃঃ as ৺, সাঃঃ as ﷺ (also in a second
  lookup), হঃঃ as ঽ, ৭ঃঃ as ঀ, and `<<` and `>>` (in every script, English and code too) as two glyphs
  at the unassigned code points U+098D and U+098E; its `nukt` showed ব + ় as র. All are removed: a
  ligature whose glyph has a code point of its own now must be canonically equivalent to what was typed
  (ড + ় → ড় stays). ৺ ﷺ ঽ ঀ র keep their own code points. U+098D and U+098E are unmapped (no
  unassigned Bengali code point is mapped, test), and their glyphs `u098D` and `u098E`, which nothing
  reaches any more, are removed from the font.
- **Repeated points cleaned before hinting** (`package.clean`): where rounding in the path operations
  brought two points together (about 300 glyphs a weight), the outline is redrawn without them (no
  outline changes by more than 0.5% of its area). ttfautohint had left 8 / 12 / 12 / 3 glyphs without
  instructions because of them (ঞ among them). Still unhinted, 1 / 5 / 8 / 1 conjunct glyphs, for a
  reason not found: ttfautohint's debug log shows it finds and fits their edges and then writes no
  bytecode; they have no repeated points and no overlapping contours. They render unhinted (smooth,
  not snapped to the pixel grid). Regular: `uni099509CD09A409CD09F1.akhn` (ক্ত্ব). Medium: the same,
  `uni099909CD0996.half` (ঙ্খ half form), `uni09A409CD09B2.akhn` (ত্ল), `uni09AD09CD09F009C1.blwf`
  (ভ্রু), `uni09B609CD09B2.akhn` (শ্ল). SemiBold: `uni099509CD09A409CD09F1.akhn`,
  `uni099909CD09AE.half` (ঙ্ম half form), `uni099C09CD099D.akhn` (জ্ঝ), `uni099C09CD09F1.blwf`
  (জ্ব below form), `uni09A409CD09B2.akhn`, `uni09AE09CD09AD.akhn` (ম্ভ), `uni09AE09CD09AD09CD09F0.akhn`
  (ম্ভ্র), `uni09B809CD09A5.akhn` (স্থ). Bold: `uni099E09CD099D.akhn` (ঞ্ঝ). `package.UNHINTED`
  holds the list; a test fails if it changes.
- **Bengali unchanged:** every Bengali glyph, advance and code point, and the shaping of every corpus
  word, are identical before and after the Latin step, in all four weights (tests). The corpus test and
  the 200 hard words pass in all four weights with the Latin in.
- **Names and Windows** (`shishir_bangla/naming.py`): family "Shishir Bangla"; Regular and Bold
  style-linked as one legacy family; Medium and SemiBold as "Shishir Bangla Medium" / "SemiBold" with
  style Regular, and the typographic family and style in name IDs 16 and 17; `usWeightClass`
  400 / 500 / 600 / 700; `fsSelection` BOLD for Bold, REGULAR for the others, USE_TYPO_METRICS kept;
  `macStyle` bold for Bold only; OS/2 Unicode-range and code-page bits from the cmap; `fsType` 0;
  vendor `SHSB`; `sxHeight` and `sCapHeight` from Inter's x and H. The copyright notice now credits
  Inter. Version 0.003.
- **Hinting and files** (`shishir_bangla/package.py`): ttfautohint 1.8.4 (bundled in the pinned
  `ttfautohint-py`), Bengali as default and fallback script, Windows compatibility on;
  `fonts/ttf/` and `fonts/woff2/` for all four styles, and the release folder
  `build/release/ShishirBangla-0.003/` with `OFL.txt`, this file, `README.md` and the licence files of
  Inter and Noto Sans Bengali in `licenses/`.

#### Open items for Khan (Task 5)

- **Bengali next to Inter at text sizes, hinted**, on Windows (GDI and DirectWrite) and Linux: a manual
  check. ttfautohint hints the Bengali with its Bengali blue zones (headline and base).
- The lowered Vietnamese and stacked-accent capitals (and the compressed Ǻ stack): a manual check.
- A capital with two combining marks that do not compose into one character (À + ◌̀, Q + ◌̈ + ◌́)
  still stacks above the line box, as it does in Inter itself; no orthography uses them.
- The vendor ID `SHSB` is not registered with Microsoft (version 0.003 and `SHSB` kept, controller's
  ruling).

### 22 November 2026 (Mahbub Khan) Shishir Bangla, Task 4b: one width scale, Noto-like colour

- **One width scale.** In every weight each glyph's advance is Regular's times Noto Sans Bengali's ratio
  of mean consonant advance for that weight (Medium 1.0082, SemiBold 1.0174, Bold 1.0280). Bold uses it
  too, so it is no longer the narrowest weight: the sample string is 20089 → 20257 → 20439 → 20649 units
  (Noto 24113 → 24335 → 24573 → 24865); before, Bold was 19720, narrower than SemiBold's 21414. The
  outlines are not stretched; the extra advance goes into the side bearings.
- **Paragraph colour like Noto.** Every weight's stroke is grown until the ink density of the sample
  string is 0.88 of Noto's in the same weight; Regular grows too. Stems 56 / 68 / 83 / 102 (were
  54 / 62 / 73 / 84; Noto 77 / 94 / 114 / 137), matra 58 / 67 / 76 / 92. Density against Noto's, sample
  string: 0.879 / 0.882 / 0.879 / 0.867 (were 0.850 / 0.799 / 0.758 / 0.753); a paragraph:
  0.889 / 0.891 / 0.890 / 0.875.
- **One scale per weight.** The drawn Bold's compatible Bengali glyphs (ও ং ঃ ো ৌ, hasanta, nukta, some
  half forms) are first scaled to Regular's size at Bold's stroke, so in Medium, SemiBold and Bold they no
  longer shrink beside the growing consonants.
- **Counters** keep at least the share Noto keeps in its own letters over the same step (Regular to
  Medium 0.784, Regular to SemiBold 0.573, SemiBold to Bold 0.675; was a flat 0.25). Where growing
  would take a counter below that, the counter is reopened: the strokes grow less into it only.
- Gaps narrower than 15% of the drawing's stem (the drawn Bold's ink traps and near-joins, 4 to 12 units)
  may close; wider gaps are kept open as before. This is a judgement about SolaimanLipi's drawn Bold, not
  something Noto does (Noto merges strokes in none of its 365 compared glyphs from Regular to Bold):
  Khan's ruling via the controller, 22 Nov 2026, with the clogging exception R-4b-1. A join may close
  only where it joins stroke to stroke; where the white it encloses would fill (a loop or a spiral's
  inside: ল্ল, শু, গু, ণ্ড, জ, ল, ফ …), the join may not close and the glyph grows less until the opening
  stays open. Bold: 34 glyphs join, 45 grow less to keep a loop open; Medium 2, SemiBold 1 (ল্ফ, স্ফ).
- **Never clip, letters and conjuncts with vowel signs below.** Every consonant and every SolaimanLipi
  conjunct shaped with ু ূ ৃ ৄ ৢ ৣ (and with ্র / ্য before them) now ends at or above −467: where a vowel
  sign went deeper (হ্ণূ −513, ত্লৣ −544, স্ক্লৣ −579 in Regular; হ্ণু −485 in Bold …), the anchor it
  sits on is raised by the overshoot (`shaping.fit_below_marks`), in every weight.
- Regular is now also made by the weights step (`build.build` writes the family's Regular).
- The line box is unchanged (1090 / 472) and holds the full lexicon in all four weights.

#### Open items for Khan (Task 4b)

- **Bold is still the drawn Bold's design.** Its letters are more compact than Regular-to-SemiBold, so
  Bold's shapes change from SemiBold (ত্ত, ক্ষ, ঙ্গ). Manual check in `docs/specimen-weights.png` and the
  ladder at the top of `docs/specimen-paragraph.png`.
- Where a counter is reopened, the stroke grows less into it (Medium 154 glyphs, SemiBold 117, Bold 162),
  so those letters are a little lighter, at text sizes too: ব and প in Medium have stem 68 against the
  69.4 target, ব in Bold 102 against 106.
- Some glyphs take a smaller growth so that no counter closes and no loop clogs (Medium 12, SemiBold 5,
  Bold 67, listed in `docs/weights.md`); in a paragraph they look a little lighter. In Bold this
  includes common letters: জ (not grown), খ 2.2, ফ 3.3 and ল 3.3 of 11 units. ক্ম (`uni099509CD09AE.akhn`)
  is not grown at all in Regular and Medium (its large counter has a narrow neck that pinches shut at any
  growth); জ্জ্ব and ন্ত্ব take 1.1 of 11 in Bold.

### 22 November 2026 (Mahbub Khan) Shishir Bangla, Task 4: weights

- **Four weights.** Regular is SolaimanLipi 2.002 Normal and Bold is 2.002's drawn Bold; **Medium and
  SemiBold** are made between them (`shishir_bangla/weights.py`, details in `docs/weights.md`), at the
  positions Noto Sans Bengali's own stems give (Medium 0.2833, SemiBold 0.6167 of the way from Regular to
  Bold, from Noto's vertical stems 77, 94, 114, 137).
- **Interpolated** where Normal and Bold are point-compatible (147 glyphs, mostly Latin and symbols, plus
  ং ঃ ও ো ৌ ্ ৗ ় ৽ ৱ ঀ ঽ), with fontTools varLib through a temporary two-master design space.
- **Offset** everywhere else (278 glyphs, every consonant and nearly every conjunct): the Regular outline
  moved outward with skia-pathops by the matching stem amount (Medium 4.25 units a side, SemiBold 9.25),
  with the matra height and the baseline kept, the advance widened, the overlaps removed. Where two
  strokes would meet, the gap between them is kept open (ী, ৳, ঙ্গ and a few conjuncts); two rare glyphs
  take a smaller growth. Stems: Regular 54, Medium 62, SemiBold 73, Bold 84 (matra 56, 62, 68, 76).
- Medium and SemiBold go through the same shaping step as Regular and Bold, with **Noto Sans Bengali
  Medium and SemiBold**: their digits are Noto's of that weight, and the imported letters and conjuncts
  (listed under Task 3) come from Noto of that weight, thinned to the weight's own stroke.
- The line box is unchanged (1090 / 472, line height 1.562) and holds the whole lexicon in all four
  weights. The corpus test and the 200 hard words pass in all four.
- Outlines made with path operations no longer keep degenerate slivers (contours with no area after
  rounding); this also removes one from Bold's ৣ.

#### Open items for Khan (Task 4)

- **Bold and SemiBold look alike in places.** The drawn Bold is narrower and has more contrast than the
  offset SemiBold, so SemiBold can look as dark as Bold at text sizes, though its stems are thinner
  (73 against 84). Manual check in `docs/specimen-weights.png`.
- Medium and SemiBold are SolaimanLipi Normal's design made heavier, not the drawn Bold's design: the
  shapes change from SemiBold to Bold.
- In স্ফ the strokes already touch in Regular (a gap under 4 units); in Medium and SemiBold they meet.

### 22 November 2026 (Mahbub Khan) Shishir Bangla, Task 3: shaping

- **Reph height kept as SolaimanLipi draws it; line height 1.562 — Khan, 22 Nov 2026.**
- **Modern Indic shaping.** SolaimanLipi 2.002 already has both script tags, `bng2` (the modern Indic
  model that HarfBuzz, DirectWrite and Android use) and `beng` (the old model, kept as the fallback),
  with their own `blwf` and `pstf` lookups for each model. The build checks that both are there with the
  same features, and every lookup it adds is registered in `DFLT`, `beng` and `bng2`
  (`shishir_bangla/shaping.py`). For the old model, `beng`'s own below-base lookup now also forms
  ra-phala across a ZWJ (ট্র‍্যাক).
- **ন্স্ট** now shapes as the half form of ন followed by SolaimanLipi's স্ট ligature (a contextual lookup
  before the conjunct ligatures; no new glyph), instead of the ন্স ligature with a visible hasanta.
- **Bengali digits** ০–৯ are Noto Sans Bengali's, in the matching weight (Noto Regular for Regular, Noto
  Bold for Bold), scaled into the font's proportions (Regular: scale 1, Bold: scale 0.9857) with Noto's
  own stroke (Khan's pick). They are tabular (one advance: 592 in Regular, 600 in Bold), and the `tnum`
  feature is added for software that asks for tabular digits.
- **Glyphs imported from Noto Sans Bengali** (Regular from Noto Regular, Bold from Noto Bold). Each is
  scaled into the font's proportions (Noto's base and matra onto the font's) and thinned to
  SolaimanLipi's stroke weight (the outline is moved inward by half the difference of the mean stem
  widths: 9.5 units in Regular, 21.4 in Bold). Letters and conjuncts take SolaimanLipi's marks, and the
  new marks use SolaimanLipi's anchors, placed as Noto places them.

  | Glyph | Text | Code points | Name | Kind |
  | --- | --- | --- | --- | --- |
  | `u098C` | ঌ | U+098C | BENGALI LETTER VOCALIC L | letter |
  | `u09C4` | ৄ | U+09C4 | BENGALI VOWEL SIGN VOCALIC RR | mark |
  | `u09E0` | ৠ | U+09E0 | BENGALI LETTER VOCALIC RR | letter |
  | `u09E1` | ৡ | U+09E1 | BENGALI LETTER VOCALIC LL | letter |
  | `u09E2` | ৢ | U+09E2 | BENGALI VOWEL SIGN VOCALIC L | mark |
  | `u09E3` | ৣ | U+09E3 | BENGALI VOWEL SIGN VOCALIC LL | mark |
  | `u09F2` | ৲ | U+09F2 | BENGALI RUPEE MARK | sign |
  | `u09F4` | ৴ | U+09F4 | BENGALI CURRENCY NUMERATOR ONE | sign |
  | `u09F5` | ৵ | U+09F5 | BENGALI CURRENCY NUMERATOR TWO | sign |
  | `u09F6` | ৶ | U+09F6 | BENGALI CURRENCY NUMERATOR THREE | sign |
  | `u09F7` | ৷ | U+09F7 | BENGALI CURRENCY NUMERATOR FOUR | sign |
  | `u09F8` | ৸ | U+09F8 | BENGALI CURRENCY NUMERATOR ONE LESS THAN THE DENOMINATOR | sign |
  | `u09F9` | ৹ | U+09F9 | BENGALI CURRENCY DENOMINATOR SIXTEEN | sign |
  | `u09FB` | ৻ | U+09FB | BENGALI GANDA MARK | sign |
  | `u09FC` | ৼ | U+09FC | BENGALI LETTER VEDIC ANUSVARA | letter |
  | `u09FE` | ৾ | U+09FE | BENGALI SANDHI MARK | mark |
  | `uni09A109CD09AE.akhn` | ড্ম | U+09A1 U+09CD U+09AE | conjunct (Noto's two glyphs merged into one) | conjunct |
  | `uni09AC09CD099D.akhn` | ব্ঝ | U+09AC U+09CD U+099D | conjunct (Noto's glyphs merged into one) | conjunct |

  With these, the whole Unicode Bengali block (96 assigned code points) plus danda and double danda is
  covered, and the corpus has no failing word in either weight.
- The copyright notice (name ID 0) now also credits Noto Sans Bengali (The Noto Project Authors).
- **ড্ম never clips.** Noto stacks ম low under ড in ড্ম and hangs ু below it; with SolaimanLipi's longer
  ু that reached −491 (Regular) and −589 (Bold), past the 472 descender. ড্ম now takes Noto's shallow
  side form of ম (the one Noto uses in দ্ম), with the vowel signs placed as Noto places them under দ্ম.
  As a guarantee for every imported letter and conjunct, the build shapes it with ু ূ ৃ ৄ ৢ ৣ, ্র, ্য, a
  reph and ঁ, and raises (or lowers) its anchor by any overshoot past the line box less its 5-unit margin
  (`shaping.fit_line_box`; ড্ম is raised 73 units in Regular and 58 in Bold, ব্ঝ 11 in Bold).

#### Open items for Khan (Task 3)

- **Thinned imports have small notches in Bold.** Thinning moves the outline inward with round joins,
  so inner corners round off: small notches show where the matra meets the stems in ড্ম and ব্ঝ (and,
  less visibly, the other imports). Manual check; a hand fix or a different thinning may follow.
- Noto's digits keep Noto's stroke and look heavier than SolaimanLipi's letters, most of all in Bold.
- **In Bold, ু and ূ under ড্ম touch the ম.** The line-box fit raises the vowel signs under ড্ম so that
  even ৣ and ৄ stay inside the box; in Bold that brings ু and ূ up against the bowl of ম. Visual only:
  the ink stays inside the line box. Manual check.

### 22 November 2026 (Mahbub Khan) Shishir Bangla version 0.002

- The letterforms now come from **SolaimanLipi 2.002** (Normal for Regular, the drawn Bold for Bold),
  Khan's pick on the version sheet. SolaimanLipi 1.6.1 is removed from `sources/upstream/`.
- The family moves to the **SIL Open Font License, Version 1.1** (`OFL.txt`, name IDs 0, 13 and 14).
- Corpus baseline re-run on 2.002 (`docs/baseline.md`), and the point compatibility of 2.002 Normal and
  Bold recorded for Task 4 (`docs/compatibility.md`).
- Proportions next to Latin (`shishir_bangla/proportions.py`, `docs/metrics.md`). The Bengali side of
  each master is transformed on its own: Regular is scaled by 0.9242 and raised by 48 units, and Bold by
  0.8497 and 82. So both stand on the baseline, with the matra at 622, relating to Inter's x-height and
  cap height as Noto Sans Bengali's does. Spacing, mark anchors and kerning are scaled with it.
- Vertical metrics never clip: ascender 1090, descender 472 (line height 1.562), the same in both
  weights, holding every stack of the full lexicon. `USE_TYPO_METRICS` is on. The upstream hints are
  dropped until Task 5 rehints.

### 22 November 2026 (Mahbub Khan) Shishir Bangla version 0.001

- Repository, pinned upstreams, licence and the corpus test.
- `build.py` copies SolaimanLipi 1.6.1 through the naming step: family "Shishir Bangla", style Regular,
  `fonts/ttf/ShishirBangla-Regular.ttf`. The outlines, metrics and shaping are SolaimanLipi's, unchanged.

## Acknowledgements

N: Solaiman Karim
W: https://ekushey.org/designer/solaiman-karim/
D: Designer of SolaimanLipi, the source of the Bengali letterforms

N: Al Mamun Hossen
W: https://ekushey.org
D: Developer of SolaimanLipi version 2.002 (Normal, Bold, Thin)

N: Ekushey
W: https://ekushey.org
D: Distributor of SolaimanLipi

N: The Noto Project Authors
W: https://github.com/notofonts/bengali
D: Noto Sans Bengali, the reference for proportions next to Latin, and the Bengali digits

N: The Inter Project Authors (Rasmus Andersson)
W: https://github.com/rsms/inter
D: Inter, the source of the Latin

N: Mahbub Khan
D: Shishir Bangla
