# Third-party notices

Ekushe Next's own code is licensed under the [Mozilla Public License 2.0](LICENSE). The project also
includes or uses the following third-party material, each under its own licence.

## Fonts (`assets/fonts/`, embedded in the Windows tray app)

Kalpurush and SolaimanLipi (SIL OFL 1.0), Hind Siliguri and Noto Sans Bengali (SIL OFL 1.1), and Siyam Rupali
(GNU GPL v2+ with font exception; Latin glyphs from Droid Sans, Apache-2.0). Authors, versions and full
licence texts are in [`assets/fonts/README.md`](assets/fonts/README.md).

## Dictionary data (`data/dictionary/`, embedded in the Windows tray app)

Ekushe Next dictionary, CC BY-SA 4.0. Derived from Bangla Wikipedia (CC BY-SA 3.0), FineWeb-2 by Hugging Face
(ODC-By 1.0), Sangraha by AI4Bharat (CC BY 4.0) and wordfreq (CC BY-SA 4.0). See
[`data/dictionary/LICENSE.md`](data/dictionary/LICENSE.md).

## English words for the phone keyboards (`data/english/`)

`words.txt` comes from wordfreq by Robyn Speer (CC BY-SA 4.0). The counted part of `next.tsv` (likely
next words) is made from the English sentences of Tatoeba (https://tatoeba.org), CC BY 2.0 FR. See
[`data/english/README.md`](data/english/README.md).

## Bijoy (SutonnyMJ) glyph tables (`crates/ekushe-core/src/bijoy.rs`)

The glyph-to-letter tables of the Bijoy ⇄ Unicode converter come from bijoy_helper
(https://github.com/fahidsarker/bijoy_helper), with a few corrections; the reordering code is
Ekushe's own.

```
MIT License

Copyright (c) 2022 Muhammad Fahid Sarker

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Rust libraries

All Rust dependencies use permissive licences: MIT, Apache-2.0, BSD-0, Zlib, Unlicense, Unicode-3.0, or
MPL-2.0 (`option-ext`). To list them with versions:

```
cargo metadata --format-version 1 | jq -r '.packages[] | "\(.name) \(.version) \(.license)"'
```

Key ones: `windows-sys` (MIT/Apache-2.0), `serde`/`serde_json` (MIT/Apache-2.0), `miniz_oxide`
(MIT/Zlib/Apache-2.0), `parking_lot`, `once_cell`, `dirs` (MIT/Apache-2.0).

## Picture to text (`assets/ocr/`)

Studio reads pictures of printed Bangla with these, under the Apache License 2.0 unless noted (full texts
next to the files):

- **Tesseract OCR** compiled to WebAssembly, from [tesseract.js-core](https://github.com/naptha/tesseract.js-core)
  7.0.0 (`tesseract-core-*.wasm.js`), and **tesseract.js** 7.0.0 (`tesseract.min.js`, `worker.min.js`;
  bundles regenerator-runtime (MIT), the browser `buffer` module (MIT), ieee754 (BSD-3-Clause) and
  zlib.js (MIT)). `LICENSE-tesseract.js.txt`.
- **The Bangla model** `ben.traineddata` starts from Tesseract's `tessdata_best` Bengali model
  (Apache-2.0, `LICENSE-tessdata.txt`); Ekushe fine-tuned it on rendered Bangla text and gave it
  Ekushe's own word list. `benold.traineddata` is the `tessdata_best` Bengali model itself, converted
  to integer weights (used alongside, for old print).
  `benreal.traineddata` is that model fine-tuned by Ekushe on real scans: the **Mozhi-Bengali**
  dataset by CVIT, IIIT Hyderabad (word images from scanned books, CC BY 4.0; M. Mathew, A. Mondal and
  C. V. Jawahar, "Towards Deployable OCR Models for Indic Languages", ICPR 2022,
  https://ilocr.iiit.ac.in/dataset/4/) and the British Library's ground truth for historical Bengali
  printed texts (REID2019, public domain, DOI 10.23636/1168), with rendered Bangla text.
- **pdf.js** 5.7.284 by Mozilla (`pdf.min.mjs`, `pdf.worker.min.mjs`), for scanned PDFs. `LICENSE-pdf.js.txt`.
  Its image decoders in `wasm/`: JBIG2 (from PDFium, BSD-3-Clause), OpenJPEG (BSD-2-Clause) and qcms
  (MIT), with Mozilla's wrappers (Apache-2.0 / BSD-2-Clause); all texts in `wasm/LICENSES.txt`.

The misread tables `data/ocr/rules.tsv` (learned from Ekushe's own test pages) and
`data/ocr/rules_extra.tsv` are Ekushe's own data.

## Voice typing (Windows, experimental; `crates/ekushe-voice`)

The voice helpers (`ekushe-voice*.exe`, downloaded with a speech model when the person asks) use:

- **sherpa-onnx** by the k2-fsa project (Apache-2.0) with **ONNX Runtime** by Microsoft (MIT), for the
  live model; **whisper.cpp** / ggml by Georgi Gerganov and contributors (MIT) through **whisper-rs**
  (Unlicense), for the Whisper models; **cpal** (Apache-2.0) for the microphone.
- **Silero VAD** v5.1.2 (MIT, Copyright (c) 2020-present Silero Team), to find where speech starts and
  stops: `assets/ggml-silero-v5.1.2.bin` (converted by the whisper.cpp project, MIT) and
  `assets/silero_vad.onnx` (as published by sherpa-onnx). Details in `crates/ekushe-voice/assets/NOTICE.txt`.

Speech models (separate downloads from Ekushe's GitHub releases, each with its own notice):

- **Live model** `bn-live-1.ekm`: the Bangla Zipformer transducer (int8 ONNX) by Sayed Shaun
  (https://huggingface.co/SayedShaun/bangla-zipformer-onnx, CC BY-SA 4.0), repackaged unchanged into
  one file by Ekushe (the pack includes `CREDITS.txt`) and shared under the same licence.
- **Accurate model** `bn-medium-q5_0.bin`: Bengali.AI's Whisper-medium Bangla model, converted to the
  whisper.cpp format by Ekushe's `voice-model.yml` workflow (release `voice-model-1`, with NOTICE.txt).

## Promo video project (`ekushe-promo/`)

Built with [Remotion](https://www.remotion.dev/), installed from npm (not bundled here). Remotion has its own
licence: free for individuals and small teams, and companies above its size limit need a company licence.
Check [remotion.dev/license](https://www.remotion.dev/license) before commercial use. The promo background
music is generated by `ekushe-promo/generate-music.js`. The images in `ekushe-promo/public/assets/media_*.png`
were AI-generated for this project, and the maintainer holds the rights to publish them.
