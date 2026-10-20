# Contributing to Ekushe Next

Thank you for helping build the next step for Bangla writing. Contributions of every size are welcome:
bug reports, spelling reviews, test cases, code and documentation.

## Ways to help

| Area | How |
| --- | --- |
| **Spelling review** (native speakers) | Open `data/dictionary/review-spelling.tsv`, fill the `decision` column (`keep` / `use suggested` / `separate words`) for some rows, and open a pull request. The rows are sorted by frequency, so the top ones matter most. |
| **Curated spellings** | Add or correct rows in `tools/dict-builder/curated/spelling-seed.tsv`. Mark a `draft` row as `reviewed` when you confirm it. |
| **Typing test cases** | Add `typed<TAB>expected` lines to `tools/dict-builder/eval/pilot.tsv` (single words) or `eval/context.tsv` (phrases), in the casual and precise styles people really use. |
| **Bugs** | Open an issue with what you typed, what you expected, what you got, the app, and your Windows version. |
| **Code** | See below. |

## Code

- Rust stable. The core engine is `crates/ekushe-core`; the Windows app is `crates/ekushe-tray`.
- Before a pull request:
  ```
  cargo fmt
  cargo test -p ekushe-core
  cargo run --release -p ekushe-core --example engine_eval -- data/dictionary/ekushe.dict.z tools/dict-builder/eval/pilot.tsv
  ```
  Ranking changes should not lower the pilot score. Mention the before/after numbers in the pull request.
- Dictionary changes go through `tools/dict-builder` (see its `README.md`, `RULES.md` and `ROMAN.md`). Don't
  edit generated files by hand.
- **Clean-room rule:** don't copy code, rules, dictionaries or word lists from Avro, OpenBangla, Lekho or other
  keyboards. Any new data source must be openly licensed; add it to `tools/dict-builder/SOURCES.md`.
- Keep pull requests focused: one change, with a clear description.

## Licensing of contributions

By contributing, you agree that your code is licensed under the project's
[MPL-2.0](LICENSE) licence, and your dictionary or data contributions under
[CC BY-SA 4.0](data/dictionary/LICENSE.md).

Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).
