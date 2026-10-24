#!/usr/bin/env bash
# Rebuild the Ekushe Next word-frequency lists from scratch.
# Needs: Rust (cargo), Python 3 + pyarrow, curl, ~25 GB free disk, ~1 hour on 2 cores.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
WORK="${WORK:-$HERE/work}"
OUT="${OUT:-$HERE/../../data/dictionary}"
mkdir -p "$WORK/raw" "$WORK/counts" "$OUT"
cargo build --release --manifest-path "$HERE/Cargo.toml"
BIN="$HERE/target/release/ekushe-dict"
HF=https://huggingface.co/datasets

fetch() { [ -s "$2" ] || curl -sSL -o "$2" "$1"; }
for i in 0 1; do fetch "$HF/wikimedia/wikipedia/resolve/main/20231101.bn/train-0000$i-of-00002.parquet" "$WORK/raw/wiki-$i.parquet"; done
for i in 000 001; do fetch "$HF/HuggingFaceFW/fineweb-2/resolve/main/data/ben_Beng/train/${i}_00000.parquet" "$WORK/raw/fw-$i.parquet"; done
for i in $(seq 0 9); do fetch "$HF/ai4bharat/sangraha/resolve/main/verified/ben/data-$i.parquet" "$WORK/raw/sg-$i.parquet"; done

count() { python3 "$HERE/scripts/extract_text.py" "${@:2}" | "$BIN" count --out "$WORK/counts/$1.tsv"
          awk -F'\t' 'NR==1 || $2>=2' "$WORK/counts/$1.tsv" > "$WORK/counts/$1.min2.tsv"; }
count wiki "$WORK"/raw/wiki-*.parquet
count fw-000 "$WORK/raw/fw-000.parquet"
count fw-001 "$WORK/raw/fw-001.parquet"
count sangraha "$WORK"/raw/sg-*.parquet

python3 "$HERE/scripts/merge_counts.py" --out "$WORK/words-all.tsv" \
  wiki="$WORK/counts/wiki.min2.tsv" \
  web="$WORK/counts/fw-000.min2.tsv,$WORK/counts/fw-001.min2.tsv" \
  sangraha="$WORK/counts/sangraha.min2.tsv"
python3 "$HERE/scripts/split_tiers.py" "$WORK/words-all.tsv" "$OUT"

# Phase 3: classification (standard / variant / colloquial / sadhu / typo, roots and endings)
python3 "$HERE/scripts/classify.py" "$WORK/words-all.tsv" "$HERE/curated/spelling-seed.tsv" "$OUT"

# Phase 4: Roman → Bangla index (+ English loanword keys)
pip install --quiet wordfreq >/dev/null 2>&1 || true
python3 "$HERE/scripts/build_index.py" "$OUT/lexicon.tsv" "$OUT/roman"
python3 "$HERE/scripts/match_english.py" "$OUT/roman" "$OUT/roman/english-matches.tsv" "$OUT/lexicon.tsv"
python3 "$HERE/scripts/build_index.py" "$OUT/lexicon.tsv" "$OUT/roman" "$OUT/roman/english-matches.tsv"
python3 "$HERE/scripts/eval_pilot.py" "$OUT/roman" "$HERE/eval/pilot.tsv"

# Phase 5: word pairs (context) + binary dictionary for the engine
awk -F'\t' 'NR>1 && $3=="core" && $4!="typo"{print $1}' "$OUT/lexicon.tsv" > "$WORK/core-vocab.txt"
python3 "$HERE/scripts/extract_text.py" "$WORK"/raw/wiki-*.parquet "$WORK"/raw/sg-*.parquet "$WORK/raw/fw-000.parquet" \
  | "$BIN" pairs --vocab "$WORK/core-vocab.txt" --out "$WORK/pairs.tsv"
"$BIN" pack --lexicon "$OUT/lexicon.tsv" --english "$OUT/roman/english-matches.tsv" --pairs "$WORK/pairs.tsv" \
  --phrases "$HERE/curated/chat-phrases.txt" \
  --vulgar "$HERE/curated/vulgar-prefixes.tsv" --out "$OUT/ekushe.dict"
# engine checks (run from the repository root)
( cd "$HERE/../.." && cargo test -p ekushe-core && \
  cargo run --release -q -p ekushe-core --example engine_eval -- data/dictionary/ekushe.dict tools/dict-builder/eval/pilot.tsv )
