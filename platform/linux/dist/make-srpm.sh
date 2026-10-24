#!/bin/bash
# Make the Fedora source package (.src.rpm) for COPR from the source tarball:
#   platform/linux/dist/make-srpm.sh v0.1.0-beta.13   → build/dist/ekushe-0.1.0~beta.13-1.src.rpm
# Run make-source.sh first. Needs rpmbuild (dnf install rpm-build).
set -euo pipefail
cd "$(dirname "$0")/../../.."
TAG="${1:?usage: make-srpm.sh <tag>}"
VERSION="$(platform/linux/dist/pkg-version.sh "$TAG")"
OUT="$PWD/build/dist"
test -f "$OUT/ekushe-$VERSION.tar.xz" || { echo "Run make-source.sh $TAG first" >&2; exit 1; }
BUNDLED="$(awk '{print "Provides:       bundled(crate(" $1 ")) = " $2}' "$OUT/bundled-crates.txt")"
DATE="$(LC_ALL=C date -u -d "@${SOURCE_DATE_EPOCH:-$(date +%s)}" '+%a %b %d %Y')"
python3 - platform/linux/dist/ekushe.spec "$OUT/ekushe.spec" "$VERSION" "$TAG" "$DATE" "$BUNDLED" <<'PY'
import sys
src, dst, version, tag, date, bundled = sys.argv[1:]
s = open(src).read()
for k, v in (('@VERSION@', version), ('@TAG@', tag), ('@DATE@', date), ('@BUNDLED@', bundled)):
    s = s.replace(k, v)
open(dst, 'w').write(s)
PY
rpmbuild -bs "$OUT/ekushe.spec" --define "_sourcedir $OUT" --define "_srcrpmdir $OUT" --define "dist %{nil}" | tail -1
