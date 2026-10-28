#!/usr/bin/env bash
# Rebuild the Ekushe Next word-frequency lists from scratch.
# Needs: Rust (cargo), Python 3 + pyarrow, curl, about 8 GB free disk and 5-6 hours on 2 cores.
# Corpus files are processed one at a time (download, count words and word pairs, delete), so the
# raw text never has to fit on disk at once; KEEP_RAW=1 keeps the downloads. An interrupted build
# resumes where it stopped (each file's counts are kept in $WORK/counts).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
WORK="${WORK:-$HERE/work}"
OUT="${OUT:-$HERE/../../data/dictionary}"
mkdir -p "$WORK/raw" "$WORK/counts" "$OUT"
cargo build --release --manifest-path "$HERE/Cargo.toml"
BIN="$HERE/target/release/ekushe-dict"
HF=https://huggingface.co/datasets

# The corpus (see SOURCES.md): all of Bangla Wikipedia, FineWeb-2 and Sangraha's verified Bengali
WIKI_FILES="${WIKI_FILES:-0 1}"
FW_FILES="${FW_FILES:-000 001 002 003 004}"
SG_FILES="${SG_FILES:-$(seq -s ' ' 0 76)}"

# Word pairs (context) are counted in the same pass, for the core words of the current lexicon.
# On a first build without data/dictionary/lexicon.tsv, run the script twice: the second run
# counts pairs for the lexicon the first one made.
VOCAB="$WORK/core-vocab.txt"
if [ -s "$OUT/lexicon.tsv" ]; then
  awk -F'\t' 'NR>1 && $3=="core" && $4!="typo"{print $1}' "$OUT/lexicon.tsv" > "$VOCAB"
else
  : > "$VOCAB"
fi

fetch() { [ -s "$2" ] || { curl -sSL --retry 5 --retry-delay 10 --retry-all-errors -o "$2.part" "$1" && mv "$2.part" "$2"; }; }

# one NAME URL: count the words and word pairs of one corpus file
one() {
  local name="$1" url="$2" raw="$WORK/raw/$1.parquet" c="$WORK/counts/$1"
  if [ -s "$c.min2.tsv" ] && [ -s "$c.pairs.tsv" ]; then return; fi
  echo "› $name" >&2
  fetch "$url" "$raw"
  rm -f "$c.fifo"; mkfifo "$c.fifo"
  "$BIN" pairs --vocab "$VOCAB" --min 2 --out "$c.pairs.part" < "$c.fifo" & local pairs_pid=$!
  python3 "$HERE/scripts/extract_text.py" "$raw" | tee "$c.fifo" | "$BIN" count --out "$c.tsv"
  wait "$pairs_pid"
  rm -f "$c.fifo"
  mv "$c.pairs.part" "$c.pairs.tsv"
  awk -F'\t' 'NR==1 || $2>=2' "$c.tsv" > "$c.min2.part" && mv "$c.min2.part" "$c.min2.tsv" && rm -f "$c.tsv"
  [ "${KEEP_RAW:-}" = 1 ] || rm -f "$raw"
}
for i in $WIKI_FILES; do one "wiki-$i" "$HF/wikimedia/wikipedia/resolve/main/20231101.bn/train-0000$i-of-00002.parquet"; done
for i in $FW_FILES; do one "fw-$i" "$HF/HuggingFaceFW/fineweb-2/resolve/main/data/ben_Beng/train/${i}_00000.parquet"; done
for i in $SG_FILES; do one "sg-$i" "$HF/ai4bharat/sangraha/resolve/main/verified/ben/data-$i.parquet"; done
[ "${COUNT_ONLY:-}" = 1 ] && { echo "counted; run again without COUNT_ONLY=1 to build" >&2; exit 0; }

list() { local f="" n; for n in "$@"; do f="$f${f:+,}$WORK/counts/$n.min2.tsv"; done; echo "$f"; }
python3 "$HERE/scripts/merge_counts.py" --out "$WORK/words-all.tsv" \
  wiki="$(list $(for i in $WIKI_FILES; do echo "wiki-$i"; done))" \
  web="$(list $(for i in $FW_FILES; do echo "fw-$i"; done))" \
  sangraha="$(list $(for i in $SG_FILES; do echo "sg-$i"; done))"
python3 "$HERE/scripts/split_tiers.py" "$WORK/words-all.tsv" "$OUT"

# Phase 3: classification (standard / variant / colloquial / sadhu / typo, roots and endings)
python3 "$HERE/scripts/classify.py" "$WORK/words-all.tsv" "$HERE/curated/spelling-seed.tsv" "$OUT"

# Phase 4: Roman → Bangla index (+ English loanword keys)
pip install --quiet wordfreq >/dev/null 2>&1 || true
python3 "$HERE/scripts/build_index.py" "$OUT/lexicon.tsv" "$OUT/roman"
python3 "$HERE/scripts/match_english.py" "$OUT/roman" "$OUT/roman/english-matches.tsv" "$OUT/lexicon.tsv"
python3 "$HERE/scripts/build_index.py" "$OUT/lexicon.tsv" "$OUT/roman" "$OUT/roman/english-matches.tsv"
python3 "$HERE/scripts/eval_pilot.py" "$OUT/roman" "$HERE/eval/pilot.tsv"

# Phase 5: word pairs (context, counted with the words above) + binary dictionary for the engine
"$BIN" merge-pairs --min 3 --out "$WORK/pairs.tsv" "$WORK"/counts/*.pairs.tsv
"$BIN" pack --lexicon "$OUT/lexicon.tsv" --english "$OUT/roman/english-matches.tsv" --pairs "$WORK/pairs.tsv" \
  --phrases "$HERE/curated/chat-phrases.txt" --spoken "$HERE/curated/spoken-verbs.tsv" \
  --keys "$HERE/curated/names.tsv,$HERE/curated/chat-words.tsv" \
  --vulgar "$HERE/curated/vulgar-prefixes.tsv" --out "$OUT/ekushe.dict"
# engine checks (run from the repository root)
( cd "$HERE/../.." && cargo test -p ekushe-core && \
  cargo run --release -q -p ekushe-core --example engine_eval -- data/dictionary/ekushe.dict tools/dict-builder/eval/pilot.tsv )
