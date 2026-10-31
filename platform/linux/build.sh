#!/bin/bash
# Build Ekushe for Linux (x86_64): the IBus keyboard engine, the Fcitx 5 addon and Ekushe Studio,
# packaged as .deb, .rpm and .AppImage. Needs Rust, gcc, g++, pkg-config, libibus-1.0-dev, libfcitx5core-dev,
# libwebkit2gtk-4.1-dev, plus nfpm and appimagetool (downloaded if missing).
#   platform/linux/build.sh [version]          build packages into build/linux
#   platform/linux/build.sh [version] --test   also type into a real ibus-daemon and fcitx5 to check both
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
echo "› Fcitx 5 addon"
g++ -std=c++20 -O2 -Wall -Wextra -fPIC -shared platform/linux/fcitx5/ekushe-fcitx5.cpp \
  $(pkg-config --cflags Fcitx5Core) target/release/libekushe_ffi.a $(pkg-config --libs Fcitx5Core) \
  -lpthread -ldl -lm -Wl,--no-undefined -Wl,--exclude-libs,ALL -o "$OUT/libekushe.so"
echo "› Studio window (GTK + WebKitGTK)"
gcc -O2 -Wall -Wextra -DEKUSHE_VERSION="\"$VERSION\"" platform/linux/studio/ekushe-studio-window.c \
  $(pkg-config --cflags --libs webkit2gtk-4.1 gtk+-3.0) -o "$OUT/ekushe-studio-window"
strip "$OUT/ibus-engine-ekushe" target/release/ekushe-studio -o /dev/null 2>/dev/null || true

echo "› Files"
install -Dm755 "$OUT/ibus-engine-ekushe" "$STAGE/usr/libexec/ekushe/ibus-engine-ekushe"
install -Dm755 target/release/ekushe-studio "$STAGE/usr/bin/ekushe-studio"
install -Dm755 "$OUT/ekushe-studio-window" "$STAGE/usr/libexec/ekushe/ekushe-studio-window"
strip "$STAGE/usr/libexec/ekushe/ibus-engine-ekushe" "$STAGE/usr/libexec/ekushe/ekushe-studio-window" "$STAGE/usr/bin/ekushe-studio"
mkdir -p "$STAGE/usr/share/ibus/component"
sed -e "s|@LIBEXECDIR@|/usr/libexec/ekushe|" -e "s|@DATADIR@|/usr/share|" -e "s|@BINDIR@|/usr/bin|" -e "s|@VERSION@|$VERSION|" \
  platform/linux/ibus/ekushe.xml.in > "$STAGE/usr/share/ibus/component/ekushe.xml"
install -Dm755 "$OUT/libekushe.so" "$STAGE/usr/lib/fcitx5/libekushe.so"
strip --strip-unneeded "$STAGE/usr/lib/fcitx5/libekushe.so"
mkdir -p "$STAGE/usr/share/fcitx5/addon"
sed -e "s|@VERSION@|$VERSION|" platform/linux/fcitx5/ekushe-addon.conf.in > "$STAGE/usr/share/fcitx5/addon/ekushe.conf"
install -Dm644 platform/linux/fcitx5/ekushe-im.conf "$STAGE/usr/share/fcitx5/inputmethod/ekushe.conf"
install -Dm644 assets/icons/logo_256.png "$STAGE/usr/share/ekushe/ekushe.png"
install -Dm644 assets/icons/logo_256.png "$STAGE/usr/share/icons/hicolor/256x256/apps/ekushe.png"
install -Dm644 platform/linux/packaging/ekushe-studio.desktop "$STAGE/usr/share/applications/ekushe-studio.desktop"
install -Dm644 LICENSE "$STAGE/usr/share/doc/ekushe/LICENSE"
install -Dm644 THIRD_PARTY_NOTICES.md "$STAGE/usr/share/doc/ekushe/THIRD_PARTY_NOTICES.md"

# Packaging tools are fixed versions, checked against these SHA-256 sums before they run
fetch_verified() { # url file sha256
  curl -fsSL -o "$2" "$1"
  if ! echo "$3  $2" | sha256sum -c --status -; then
    echo "Checksum mismatch for $1" >&2
    rm -f "$2"
    exit 1
  fi
}

echo "› .deb and .rpm"
NFPM=$(command -v nfpm || true)
if [ -z "$NFPM" ]; then
  fetch_verified https://github.com/goreleaser/nfpm/releases/download/v2.41.1/nfpm_2.41.1_Linux_x86_64.tar.gz \
    "$TOOLS/nfpm.tar.gz" b3cf95aa6dabed836d09ad7f0c190a13c74c5b1304db60846f0f702ee407f430
  tar xzf "$TOOLS/nfpm.tar.gz" -C "$TOOLS" nfpm
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
  fetch_verified https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage \
    "$TOOLS/appimagetool" ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0
  chmod +x "$TOOLS/appimagetool"; TOOL="$TOOLS/appimagetool"
fi
# The AppImage runtime (the small program at the front of the AppImage), also a fixed version
fetch_verified https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64 \
  "$TOOLS/runtime-x86_64" 2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 VERSION="$VERSION" "$TOOL" --no-appstream --runtime-file "$TOOLS/runtime-x86_64" "$APPDIR" "$OUT/Ekushe-${VERSION}-x86_64.AppImage"

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
  echo "› Fixed layout (UniJoy) in a real ibus-daemon"
  LAYOUT_HOME="$(mktemp -d)"
  mkdir -p "$LAYOUT_HOME/Ekushe"
  printf 'active_layout = "UniJoy"\n' > "$LAYOUT_HOME/Ekushe/config.toml"
  XDG_CONFIG_HOME="$LAYOUT_HOME" dbus-run-session -- bash -c '
    ibus-daemon --xim=false --panel=disable --emoji-extension=disable --cache=none --config=disable &
    for i in $(seq 1 50); do F=$(ls "$XDG_CONFIG_HOME"/ibus/bus/* 2>/dev/null | head -1); [ -n "$F" ] && break; sleep 0.2; done
    export $(grep "^IBUS_ADDRESS=" "$F")
    ${PYTHON:-python3} platform/linux/ibus/test_fixed_layout.py
  '
  echo "› Typing test against a real fcitx5"
  platform/linux/fcitx5/run-test.sh "$OUT"
fi
