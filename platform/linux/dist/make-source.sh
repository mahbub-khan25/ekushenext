#!/bin/bash
# Make the Linux source tarball that the Fedora (COPR) and Ubuntu (PPA) packages build from:
#   platform/linux/dist/make-source.sh v0.1.0-beta.13   → build/dist/ekushe-0.1.0~beta.13.tar.xz
# It holds only what the Linux build needs (engine, Studio, dictionary, icons, fonts), plus every
# Rust library in vendor/, because Launchpad and COPR build without internet. Libraries that only
# Windows, macOS, phones or tests use keep their Cargo.toml but no code, so the tarball stays small.
set -euo pipefail
cd "$(dirname "$0")/../../.."
TAG="${1:?usage: make-source.sh <tag, e.g. v0.1.0-beta.13>}"
VERSION="$(platform/linux/dist/pkg-version.sh "$TAG")"
NAME="ekushe-$VERSION"
OUT=build/dist; WORK="$OUT/$NAME"
rm -rf "$WORK"; mkdir -p "$WORK"

echo "› Files"
copy() { for f in "$@"; do mkdir -p "$WORK/$(dirname "$f")"; cp -a "$f" "$WORK/$f"; done; }
copy Cargo.toml Cargo.lock LICENSE README.md THIRD_PARTY_NOTICES.md .github/release/VERSION \
     crates/ekushe-core crates/ekushe-ffi crates/ekushe-studio \
     crates/ekushe-tray/src/studio_http.rs crates/ekushe-tray/src/studio_ui.rs \
     assets/icons/logo_256.png assets/fonts \
     data/dictionary/ekushe.dict.z data/dictionary/LICENSE.md data/dictionary/CC-BY-SA-4.0.txt \
     platform/linux/Makefile platform/linux/ekushe_ffi.h platform/linux/ibus platform/linux/studio platform/linux/packaging/ekushe-studio.desktop
# Newer Studio pieces (older release tags, rebuilt with this script, don't have them)
for f in crates/ekushe-tray/src/studio_layouts.rs; do if [ -f "$f" ]; then copy "$f"; fi; done
rm -rf "$WORK"/crates/*/target

# A workspace of just the three Linux crates. Cargo drops the other members' libraries from
# Cargo.lock and keeps every remaining version exactly as locked.
python3 - "$WORK/Cargo.toml" <<'PY'
import re, sys
p = sys.argv[1]; s = open(p).read()
s, n = re.subn(r'members = \[.*?\]',
    'members = [\n    "crates/ekushe-core",\n    "crates/ekushe-ffi",\n    "crates/ekushe-studio",\n]', s, count=1, flags=re.S)
assert n == 1, "workspace members not found"
open(p, 'w').write(s)
PY
(cd "$WORK" && cargo metadata --format-version 1 -q >/dev/null)

echo "› Rust libraries (vendor/)"
(cd "$WORK" && cargo vendor --locked -q vendor > /dev/null)
# Which libraries a Linux build actually compiles (normal and build dependencies, x86_64 and arm64)
NEEDED="$(cd "$WORK" && for t in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
  cargo tree --locked --offline -p ekushe-ffi -p ekushe-studio --features "ekushe-ffi/embedded-dict ekushe-ffi/studio" \
    --target "$t" -e normal,build --prefix none --format '{p}' ; done | awk '{print $1" "$2}' | sort -u)"
python3 - "$WORK/vendor" "$NEEDED" <<'PY'
# Empty out the libraries the Linux build never compiles: keep Cargo.toml (Cargo still reads it to
# resolve Cargo.lock), drop the code, and record no file checksums so Cargo accepts the stub.
import json, os, shutil, sys, tomllib
vendor, needed = sys.argv[1], set()
for line in filter(str.strip, sys.argv[2].splitlines()):
    name, ver = line.split()
    needed.add((name, ver.lstrip('v')))
kept = emptied = 0
bundled = []
for d in sorted(os.listdir(vendor)):
    path = os.path.join(vendor, d)
    meta = tomllib.load(open(os.path.join(path, 'Cargo.toml'), 'rb'))['package']
    if (meta['name'], meta['version']) in needed:
        kept += 1
        bundled.append(f"{meta['name']} {meta['version']}")
        # Debian's clean step deletes *.orig and *.rej files, which would break Cargo's checksums:
        # drop them now (Cargo.toml.orig is only a copy of the author's manifest)
        sums_path = os.path.join(path, '.cargo-checksum.json')
        sums = json.load(open(sums_path))
        for f in [f for f in sums['files'] if f.endswith(('.orig', '.rej'))]:
            os.remove(os.path.join(path, f))
            del sums['files'][f]
        json.dump(sums, open(sums_path, 'w'))
        continue
    checksum = json.load(open(os.path.join(path, '.cargo-checksum.json')))
    for entry in os.listdir(path):
        if entry not in ('Cargo.toml', '.cargo-checksum.json'):
            full = os.path.join(path, entry)
            shutil.rmtree(full) if os.path.isdir(full) and not os.path.islink(full) else os.remove(full)
    os.makedirs(os.path.join(path, 'src'))
    open(os.path.join(path, 'src', 'lib.rs'), 'w').close()
    json.dump({'files': {}, 'package': checksum.get('package')}, open(os.path.join(path, '.cargo-checksum.json'), 'w'))
    emptied += 1
print(f"  {kept} libraries bundled, {emptied} not needed on Linux (emptied)")
open(os.path.join(vendor, '..', '..', 'bundled-crates.txt'), 'w').write('\n'.join(sorted(bundled)) + '\n')
PY
mkdir -p "$WORK/.cargo"
cat > "$WORK/.cargo/config.toml" <<'TOML'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
TOML

echo "› Check: it builds without internet"
if [ "${SKIP_CHECK:-}" != 1 ]; then
  (cd "$WORK" && CARGO_TARGET_DIR="$PWD/../check-target" cargo check --offline --frozen -q -p ekushe-ffi -p ekushe-studio \
     --features "ekushe-ffi/embedded-dict ekushe-ffi/studio")
  rm -rf "$OUT/check-target"
fi

echo "› Tarball"
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}" \
    -C "$OUT" -cJf "$OUT/$NAME.tar.xz" "$NAME"
rm -rf "$WORK"
ls -la "$OUT/$NAME.tar.xz"
