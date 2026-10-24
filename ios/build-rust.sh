#!/usr/bin/env bash
# Build the engine for iPhone and iPad (and the simulator), and put the dictionary and the English
# word list where the keyboard bundles them. Needs a Mac with Xcode and Rust.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$HERE/.."
export IPHONEOS_DEPLOYMENT_TARGET=16.0
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios >/dev/null
for t in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios; do
  (cd "$ROOT" && cargo build --release -p ekushe-mobile --target "$t")
done
mkdir -p "$HERE/rust/iphoneos" "$HERE/rust/iphonesimulator" "$HERE/Data"
cp "$ROOT/target/aarch64-apple-ios/release/libekushe_mobile.a" "$HERE/rust/iphoneos/"
lipo -create "$ROOT/target/aarch64-apple-ios-sim/release/libekushe_mobile.a" "$ROOT/target/x86_64-apple-ios/release/libekushe_mobile.a" \
  -output "$HERE/rust/iphonesimulator/libekushe_mobile.a"
# The keyboard maps the dictionary straight from the app instead of unpacking it into memory
# (iOS stops keyboards that use too much), so it ships uncompressed; the app download is still compressed.
python3 -c "import zlib,sys; open(sys.argv[2],'wb').write(zlib.decompress(open(sys.argv[1],'rb').read()))" \
  "$ROOT/data/dictionary/ekushe.dict.z" "$HERE/Data/ekushe-dictionary.bin"
cp "$ROOT/data/english/words.txt" "$ROOT/data/english/next.tsv" "$HERE/Data/"
ls -la "$HERE/rust"/*/ "$HERE/Data/"
