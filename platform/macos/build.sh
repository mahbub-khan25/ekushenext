#!/bin/bash
# Build Ekushe.app (universal: Apple silicon + Intel), Ekushe-macOS.pkg and Ekushe-macOS.zip.
# Runs on macOS with Xcode command-line tools and Rust. Usage: platform/macos/build.sh [version]
set -euo pipefail
cd "$(dirname "$0")/../.."
VERSION="${1:-0.1.0}"
BUILD="${GITHUB_RUN_NUMBER:-1}"
OUT=build/macos
APP="$OUT/Ekushe.app"
rm -rf "$OUT"; mkdir -p "$OUT"

echo "› Rust engine (arm64 + x86_64)"
rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
for t in aarch64-apple-darwin x86_64-apple-darwin; do
  MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release -p ekushe-ffi --features "embedded-dict studio" --target "$t"
done

echo "› Swift input method"
SRC=platform/macos/Ekushe
for arch in arm64 x86_64; do
  t=$([ "$arch" = arm64 ] && echo aarch64-apple-darwin || echo x86_64-apple-darwin)
  swiftc -O -whole-module-optimization -module-name Ekushe \
    -target "$arch-apple-macos11.0" \
    -import-objc-header "$SRC/ekushe_ffi.h" \
    "$SRC"/*.swift \
    "target/$t/release/libekushe_ffi.a" \
    -framework Cocoa -framework InputMethodKit -framework Carbon \
    -o "$OUT/Ekushe-$arch"
done

echo "› App bundle"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create "$OUT/Ekushe-arm64" "$OUT/Ekushe-x86_64" -output "$APP/Contents/MacOS/Ekushe"
sed -e "s/__VERSION__/${VERSION#v}/" -e "s/__BUILD__/$BUILD/" platform/macos/Info.plist > "$APP/Contents/Info.plist"
cp -R platform/macos/Resources/en.lproj "$APP/Contents/Resources/"
# Menu-bar icon (16 pt, with a @2x version) and app icon
sips -s format tiff assets/icons/logo_16.png --out "$OUT/m16.tiff" >/dev/null
sips -s format tiff assets/icons/logo_32.png --out "$OUT/m32.tiff" >/dev/null
tiffutil -cathidpicheck "$OUT/m16.tiff" "$OUT/m32.tiff" -out "$APP/Contents/Resources/MenuIcon.tiff" >/dev/null
ICONSET="$OUT/AppIcon.iconset"; mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z $s $s assets/icons/logo_512.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  d=$((s * 2)); [ $d -le 1024 ] && sips -z $d $d assets/icons/logo_512.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
plutil -lint "$APP/Contents/Info.plist"

echo "› Sign (ad-hoc until a Developer ID is set up)"
if [ -n "${MACOS_SIGN_IDENTITY:-}" ]; then
  codesign --force --options runtime --timestamp --sign "$MACOS_SIGN_IDENTITY" "$APP"
else
  codesign --force --sign - "$APP"
fi
codesign --verify --verbose "$APP"

echo "› Package"
ditto -c -k --keepParent "$APP" "$OUT/Ekushe-macOS.zip"
STAGE="$OUT/pkgroot"; mkdir -p "$STAGE"; cp -R "$APP" "$STAGE/"
pkgbuild --root "$STAGE" --install-location "/Library/Input Methods" \
  --identifier org.ekushe.inputmethod.pkg --version "${VERSION#v}" \
  --scripts platform/macos/pkg-scripts "$OUT/Ekushe-macOS.pkg"
lipo -info "$APP/Contents/MacOS/Ekushe"
ls -la "$OUT"
