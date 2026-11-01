# Bangla OCR: measuring and improving

Ekushe reads printed Bangla from pictures with Tesseract (`assets/ocr/`, run in Studio as
WebAssembly) and then cleans the text with its dictionary (`crates/ekushe-core/src/ocr.rs`). These
scripts measure accuracy and produce the data the clean-up uses.

Work in a scratch folder (`OCRWORK`), with `tessdata/` models in `td/` and fonts installed for
`text2image`. Needs `tesseract`, `text2image`, `lstmtraining`, Python 3 with `opencv-python`,
`rapidfuzz`, `pyarrow`, `Pillow`.

1. `prepare_text.py wiki-bn.parquet eval-paras.txt train-paras.txt`: clean Bangla paragraphs;
   the first 400 are kept for evaluation only.
2. `render_eval.py N`: renders pages in each font (`FONTS=` to choose, `OUT=`, `SRC=`, `START=`)
   and makes three copies: `clean` (300 dpi), `scan` (200 dpi, blur, skew, noise, threshold) and
   `photo` (perspective, uneven light, blur, JPEG, low resolution).
3. `prep.py in out`: the picture clean-up (lighting, letter size, straightening). Studio does the
   same in JavaScript (`ocrPrepare` in `studio_ui.rs`).
4. `eval_ocr.py NAME 'command with {img} {out}'`: runs an OCR setup on every page and prints the
   character and word error rates per condition (`FONTS=1` per font, `DIR=` for another set).
5. `correct.py learn DIR RUN rules.tsv`: learns which misreads OCR makes (intended → read, with
   probabilities) by aligning OCR output with the truth. The result is `data/ocr/rules.tsv`.
6. `correct2.py`, `tune.py`: the Python prototype of the clean-up and its parameter search; the
   engine's `ocr.rs` is the real implementation (`examples/ocr_correct.rs` runs it on a folder).
7. `make_ft.sh` then `lstmtraining`: fine-tunes Tesseract's Bangla model on rendered lines.

8. `gen_lines.py OUT N SEED`: renders training lines in every font with print, scan and photo
   damage (tried for a second fine-tune; it hurt real pages, see the handbook).
9. `score_real.py OUTDIR`: scores OCR output against hand-typed crops of real pages (`gt/*.png` +
   `gt/*.gt.txt`), the test that matters most.
10. `cargo run -p ekushe-core --features embedded-dict --example text_layer FILE`: what Ekushe would
    use from a PDF page's text (`pdftotext -f N -l N file.pdf FILE`).

11. Real data (2026-11-01): `make_real.py OUT` builds training and test lines from
    **Mozhi-Bengali** (IIIT Hyderabad, CC BY 4.0, word images from scanned books, joined into lines)
    and the **British Library REID2019** pages (public domain, 1713-1914; cut into lines by
    `bl_lines.py`); `lstmtraining --continue_from` Tesseract's `ben` model on them plus the rendered
    pages of step 7; `rt_model.sh CHECKPOINT NAME` makes the int model (`benreal`);
    `eval_lines.py DIR MODEL [--correct]` scores line images.

12. `mine_typed_pdfs.py`: finds typed (not scanned) PDF pages in a collection and cuts them into
    line images with the PDF's own text (Bijoy text converted with `ekushe-cli bijoy2uni`; lines kept
    only when the text is real Bangla and a plain OCR pass roughly agrees). Used as a private test set.
13. `data/ocr/rules_real.tsv`: misreads learned from held-out real scanned lines (Mozhi validation
    split) with `correct.py learn`, Bangla letters only.

Results so far are in the project handbook (log and dictionary pages).

## Pages people send from Studio

Studio's "একুশেকে পাঠান" button (after the person agrees) posts the page pictures, the lines as read
and the corrected text to `ekushenext.com/api/ocr-sample` (website repo), which keeps them in R2.

- `fetch_samples.py OUT_DIR`: downloads everything new (`EKUSHE_OCR_KEY` = the `OCR_ADMIN_KEY`
  secret).
- `user_lines.py SAMPLES_DIR OUT_DIR`: cuts each line out of the page and labels it with the
  person's correction (matched word by word, kept only when at most 40% of the letters changed).
