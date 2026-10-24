#!/bin/bash
# Make the Ubuntu source package for one release, for the Launchpad PPA:
#   platform/linux/dist/make-deb-source.sh v0.1.0-beta.13 noble
#     → build/dist/deb/ekushe_0.1.0~beta.13-1~noble1_source.changes (+ .dsc, .debian.tar.xz, .orig.tar.xz)
# Run make-source.sh first. Needs dpkg-dev and debhelper. The result is unsigned; sign it with
# debsign before uploading with dput (the release workflow does both).
# UPLOAD=2 makes a second upload of the same version (1~noble2) if Launchpad needs a rebuild.
set -euo pipefail
cd "$(dirname "$0")/../../.."
TAG="${1:?usage: make-deb-source.sh <tag> <ubuntu series>}"
SERIES="${2:?usage: make-deb-source.sh <tag> <ubuntu series>}"
VERSION="$(platform/linux/dist/pkg-version.sh "$TAG")"
DEBVER="$VERSION-1~${SERIES}${UPLOAD:-1}"
DIST="$PWD/build/dist"; OUT="$DIST/deb"
test -f "$DIST/ekushe-$VERSION.tar.xz" || { echo "Run make-source.sh $TAG first" >&2; exit 1; }

# The default Rust is too old before Ubuntu 26.10; those releases have Rust 1.91 as an update
case "$SERIES" in
  jammy|noble|resolute) RUST="cargo-1.91, rustc-1.91" ;;
  *)                    RUST="cargo (>= 1.85), rustc (>= 1.85)" ;;
esac

mkdir -p "$OUT"; rm -rf "$OUT/ekushe-$VERSION"
cp "$DIST/ekushe-$VERSION.tar.xz" "$OUT/ekushe_$VERSION.orig.tar.xz"
tar -C "$OUT" -xJf "$OUT/ekushe_$VERSION.orig.tar.xz"
SRC="$OUT/ekushe-$VERSION"
cp -a platform/linux/dist/debian "$SRC/debian"
sed "s/@RUST@/$RUST/" "$SRC/debian/control.in" > "$SRC/debian/control"; rm "$SRC/debian/control.in"
cat > "$SRC/debian/changelog" <<EOF
ekushe ($DEBVER) $SERIES; urgency=medium

  * Release $TAG: https://github.com/mahbub-khan25/ekushenext/releases/tag/$TAG

 -- Ekushe Next <khan.ytube24@gmail.com>  $(LC_ALL=C date -R -u -d "@${SOURCE_DATE_EPOCH:-$(date +%s)}")
EOF
# -sa: include the .orig tarball (Launchpad needs it for each new version); -d: build
# dependencies are checked by Launchpad, not here
(cd "$SRC" && dpkg-buildpackage -S -sa -d -us -uc -nc >/dev/null)
rm -rf "$SRC"
ls "$OUT"/ekushe_"$DEBVER"*
