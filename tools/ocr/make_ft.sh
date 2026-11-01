#!/bin/bash
# Render fine-tuning pages (text2image, with its own degradation) and make .lstmf files
set -e
cd "${OCRWORK:-$PWD}/ft"
FONTS=("Kalpurush" "SolaimanLipi" "Hind Siliguri" "Noto Sans Bengali" "Noto Serif Bengali" "Anek Bangla" "Atma" "Baloo Da 2")
# training text: wiki paragraphs not used for eval (lines from train-paras, after the first 20000 used by the LM? fine)
python3 -c "
import random
p=open('../train-paras.txt').read().split('\n')[2000:]
random.seed(5); random.shuffle(p)
open('ft-text.txt','w').write('\n'.join(p[:6000])+'\n')
"
split -n l/40 -d ft-text.txt chunk_
n=0
for f in "${FONTS[@]}"; do
  for c in chunk_*; do
    n=$((n+1)); [ $((n % 3)) -eq 0 ] || continue   # a third of the chunks per font, different ones
    b="pg_${f// /}_${c#chunk_}"
    exp=$(( (n % 3) - 1 ))
    text2image --text "$c" --outputbase "$b" --font "$f" --fonts_dir ~/.fonts --ptsize $((10 + n % 5)) \
      --resolution 300 --xsize 2480 --ysize 3508 --margin 150 --leading 16 --exposure $exp \
      --char_spacing 0 --max_pages 1 >/dev/null 2>&1 || continue
    tesseract "$b.tif" "$b" --tessdata-dir ../td -l ben_best --psm 6 lstm.train >/dev/null 2>&1 || true
    rm -f "$b.tif"
  done
done
ls *.lstmf | wc -l
