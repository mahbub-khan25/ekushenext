#!/usr/bin/env bash
# Build the engine for Android phones and copy it, with the dictionary, into the app.
#   ANDROID_NDK_HOME must point at an Android NDK (r26 or later); needs `cargo install cargo-ndk`.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$HERE/.."
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android >/dev/null
(cd "$ROOT" && cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -P 24 \
  -o "$HERE/app/src/main/jniLibs" build --release -p ekushe-android)
mkdir -p "$HERE/app/src/main/assets"
cp "$ROOT/data/dictionary/ekushe.dict.z" "$HERE/app/src/main/assets/ekushe.dict.z"
mkdir -p "$HERE/app/src/main/assets/en"
cp "$ROOT/data/english/words.txt" "$ROOT/data/english/next.tsv" "$HERE/app/src/main/assets/en/"
mkdir -p "$HERE/app/src/main/assets/fonts"
cp "$ROOT/assets/fonts/HindSiliguri-Regular.ttf" "$ROOT/assets/fonts/HindSiliguri-SemiBold.ttf" "$HERE/app/src/main/assets/fonts/"
ls -la "$HERE/app/src/main/jniLibs"/*/ "$HERE/app/src/main/assets/"
