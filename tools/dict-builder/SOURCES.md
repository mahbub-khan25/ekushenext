# Ekushe Next dictionary — text sources

The resulting dictionary data is licensed CC BY-SA 4.0; see `data/dictionary/LICENSE.md`.

The dictionary contains **word frequency counts** derived from the sources below. No source text is
redistributed. Attribution is required for all three; keep this list in the app's About / credits page.

| Source | Portion used | Licence | Downloaded | Tokens counted |
| --- | --- | --- | --- | --- |
| [Bangla Wikipedia](https://huggingface.co/datasets/wikimedia/wikipedia) (snapshot 2023-11-01) | `20231101.bn`, all 143,070 articles | CC BY-SA 3.0 / GFDL | 2026-10-20 | 47,180,754 |
| [FineWeb-2](https://huggingface.co/datasets/HuggingFaceFW/fineweb-2) by Hugging Face | `ben_Beng` train files 000, 001 (2 of 5) | ODC-By 1.0 (subject to Common Crawl terms of use) | 2026-10-20 | see `build-report.md` |
| [Sangraha](https://huggingface.co/datasets/ai4bharat/sangraha) by AI4Bharat | `verified/ben` files 0–9 (10 of 77) | CC BY 4.0 | 2026-10-20 | 564,586,280 |

Phase 4 also uses the English word list of [wordfreq](https://github.com/rspeer/wordfreq) (data CC BY-SA 4.0)
to find English loanwords (computer → কম্পিউটার). Only English spellings matched to Bangla words are kept.

Attribution text suggested for the app:

> Ekushe Next word frequencies are derived from Bangla Wikipedia (CC BY-SA), FineWeb-2 by Hugging Face
> (ODC-By), and Sangraha by AI4Bharat (CC BY 4.0). English loanword matching uses wordfreq (CC BY-SA 4.0).
