#!/bin/bash
# Build Ekushe for Linux (x86_64): the IBus keyboard engine and Ekushe Studio, packaged as .deb,
# .rpm and .AppImage. Needs Rust, gcc, pkg-config, libibus-1.0-dev, plus nfpm and appimagetool
# (downloaded if missing).
#   platform/linux/build.sh [version]          build packages into build/linux
#   platform/linux/build.sh [version] --test   also type into a real ibus-daemon to check the engine
set -euo pipefail
cd "$(dirname "$0")/../.."
VERSION="${1:-0.1.0}"; VERSION="${VERSION#v}"
OUT=build/linux; STAGE="$OUT/stage"; TOOLS="$OUT/tools"
rm -rf "$OUT"; mkdir -p "$STAGE" "$TOOLS"

echo "› Rust engine and Studio"
cargo build --release -p ekushe-ffi --features "embedded-dict studio"
cargo build --release -p ekushe-studio

echo "› IBus engine"
gcc -O2 -Wall -Wextra -DEKUSHE_VERSION="\"$VERSION\"" platform/linux/ibus/ekushe-engine.c \
  $(pkg-config --cflags ibus-1.0) target/release/libekushe_ffi.a $(pkg-config --libs ibus-1.0) \
  -lpthread -ldl -lm -o "$OUT/ibus-engine-ekushe"
strip "$OUT/ibus-engine-ekushe" target/release/ekushe-studio -o /dev/null 2>/dev/null || true

echo "› Files"
install -Dm755 "$OUT/ibus-engine-ekushe" "$STAGE/usr/libexec/ekushe/ibus-engine-ekushe"
install -Dm755 target/release/ekushe-studio "$STAGE/usr/bin/ekushe-studio"
strip "$STAGE/usr/libexec/ekushe/ibus-engine-ekushe" "$STAGE/usr/bin/ekushe-studio"
mkdir -p "$STAGE/usr/share/ibus/component"
sed -e "s|@LIBEXECDIR@|/usr/libexec/ekushe|" -e "s|@DATADIR@|/usr/share|" -e "s|@BINDIR@|/usr/bin|" -e "s|@VERSION@|$VERSION|" \
  platform/linux/ibus/ekushe.xml.in > "$STAGE/usr/share/ibus/component/ekushe.xml"
install -Dm644 assets/icons/logo_256.png "$STAGE/usr/share/ekushe/ekushe.png"
install -Dm644 assets/icons/logo_256.png "$STAGE/usr/share/icons/hicolor/256x256/apps/ekushe.png"
install -Dm644 platform/linux/packaging/ekushe-studio.desktop "$STAGE/usr/share/applications/ekushe-studio.desktop"
install -Dm644 LICENSE "$STAGE/usr/share/doc/ekushe/LICENSE"
install -Dm644 THIRD_PARTY_NOTICES.md "$STAGE/usr/share/doc/ekushe/THIRD_PARTY_NOTICES.md"

echo "› .deb and .rpm"
NFPM=$(command -v nfpm || true)
if [ -z "$NFPM" ]; then
  curl -sSL https://github.com/goreleaser/nfpm/releases/download/v2.41.1/nfpm_2.41.1_Linux_x86_64.tar.gz | tar xz -C "$TOOLS" nfpm
  NFPM="$TOOLS/nfpm"
fi
export EKUSHE_VERSION="$VERSION"
"$NFPM" package --config platform/linux/packaging/nfpm.yaml --packager deb --target "$OUT/ekushe_${VERSION}_amd64.deb"
"$NFPM" package --config platform/linux/packaging/nfpm.yaml --packager rpm --target "$OUT/ekushe-${VERSION}.x86_64.rpm"

echo "› AppImage"
APPDIR="$OUT/Ekushe.AppDir"
cp -a "$STAGE" "$APPDIR"
rm -rf "$APPDIR/usr/share/doc"
install -Dm755 platform/linux/packaging/AppRun "$APPDIR/AppRun"
install -Dm755 platform/linux/packaging/keyboard.sh "$APPDIR/usr/share/ekushe/keyboard.sh"
cp platform/linux/packaging/ekushe-studio.desktop "$APPDIR/ekushe-studio.desktop"
cp assets/icons/logo_256.png "$APPDIR/ekushe.png"
TOOL=$(command -v appimagetool || true)
if [ -z "$TOOL" ]; then
  curl -sSL -o "$TOOLS/appimagetool" https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
  chmod +x "$TOOLS/appimagetool"; TOOL="$TOOLS/appimagetool"
fi
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 VERSION="$VERSION" "$TOOL" --no-appstream "$APPDIR" "$OUT/Ekushe-${VERSION}-x86_64.AppImage"

ls -la "$OUT"/*.deb "$OUT"/*.rpm "$OUT"/*.AppImage

if [ "${2:-}" = "--test" ]; then
  echo "› Typing test against a real ibus-daemon"
  export IBUS_COMPONENT_PATH="$(pwd)/$OUT/test-component"
  mkdir -p "$IBUS_COMPONENT_PATH"
  sed -e "s|@LIBEXECDIR@|$(pwd)/$OUT|" -e "s|@DATADIR@|/usr/share|" -e "s|@BINDIR@|/usr/bin|" -e "s|@VERSION@|$VERSION|" \
    platform/linux/ibus/ekushe.xml.in > "$IBUS_COMPONENT_PATH/ekushe.xml"
  rm -rf "$HOME/.config/ibus/bus"
  dbus-run-session -- bash -c '
    ibus-daemon --xim=false --panel=disable --emoji-extension=disable --cache=none --config=disable &
    for i in $(seq 1 50); do F=$(ls "$HOME"/.config/ibus/bus/* 2>/dev/null | head -1); [ -n "$F" ] && break; sleep 0.2; done
    export $(grep "^IBUS_ADDRESS=" "$F")
    ${PYTHON:-python3} platform/linux/ibus/test_engine.py
  '
fi
