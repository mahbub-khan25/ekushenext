#!/bin/bash
# Render both versions: 16:9 (index.html) and 9:16 (variants/vertical.html, swapped in for its render;
# its scenes are compositions-v/, copies of compositions/ at 1080×1920 — rebuild them after a change:
#   for f in compositions/*.html; do sed 's/data-width="1920" data-height="1080"/data-width="1080" data-height="1920"/' $f > compositions-v/$(basename $f); done
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p renders
npx hyperframes render -q delivery -o renders/ekushe-next-film-1080.mp4
cp index.html renders/.index16.html
trap 'cp renders/.index16.html index.html' EXIT
cp variants/vertical.html index.html
npx hyperframes render -q delivery -o renders/ekushe-next-film-vertical.mp4
