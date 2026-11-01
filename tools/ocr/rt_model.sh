#!/bin/bash
# rt_model.sh CHECKPOINT NAME -> td/NAME.traineddata (int, stock word list, no eng sub-language)
set -e; cd "${OCRWORK:-.}"; X=$(mktemp -d)
lstmtraining --stop_training --convert_to_int --continue_from $1 --traineddata td/ben_best.traineddata --model_output td/$2.traineddata >/dev/null 2>&1
combine_tessdata -u td/$2.traineddata $X/rt_$2. >/dev/null
grep -v tessedit_load_sublangs $X/rt_$2.config > $X/rt_$2.config2 && mv $X/rt_$2.config2 $X/rt_$2.config
combine_tessdata -o td/$2.traineddata $X/rt_$2.config >/dev/null
ls -la td/$2.traineddata
