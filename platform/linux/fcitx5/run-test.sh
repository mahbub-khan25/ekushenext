#!/bin/bash
# Type into a real fcitx5 (no display: only its D-Bus front end) with Ekushe's addon.
#   platform/linux/fcitx5/run-test.sh <dir with libekushe.so>   the freshly built addon
#   platform/linux/fcitx5/run-test.sh --installed               the addon the package installed
# Needs fcitx5 (with its dbus and dbusfrontend modules), dbus-run-session and python3-dbus + python3-gi.
set -euo pipefail
cd "$(dirname "$0")/../../.."
T="$(mktemp -d)"
mkdir -p "$T/data/fcitx5/addon" "$T/data/fcitx5/inputmethod" "$T/config/fcitx5"
if [ "$1" != --installed ]; then
  LIBDIR="$(realpath "$1")"
  SYS=""
  for d in "$(pkg-config --variable=libdir Fcitx5Core 2>/dev/null)/fcitx5" /usr/lib/x86_64-linux-gnu/fcitx5 /usr/lib64/fcitx5 /usr/lib/fcitx5; do
    if [ -e "$d/libdbus.so" ] || [ -e "$d/libclassicui.so" ]; then SYS="$d"; break; fi
  done
  export FCITX_ADDON_DIRS="$LIBDIR:$SYS"
  sed -e "s|@VERSION@|test|" platform/linux/fcitx5/ekushe-addon.conf.in > "$T/data/fcitx5/addon/ekushe.conf"
  cp platform/linux/fcitx5/ekushe-im.conf "$T/data/fcitx5/inputmethod/ekushe.conf"
fi
cat > "$T/config/fcitx5/profile" <<'EOF'
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=ekushe

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=ekushe
Layout=

[GroupOrder]
0=Default
EOF
export HOME="$T" XDG_CONFIG_HOME="$T/config" XDG_DATA_HOME="$T/data"
PY="${PYTHON:-python3}"
dbus-run-session -- bash -c "
  fcitx5 --disable=all --enable=dbus,dbusfrontend,ekushe,keyboard --verbose=default=3 > '$T/fcitx5.log' 2>&1 &
  FCITX=\$!
  set +e
  $PY platform/linux/fcitx5/test_fcitx5.py; R=\$?
  kill \$FCITX 2>/dev/null
  [ \$R = 0 ] || { echo '--- fcitx5 log ---'; tail -40 '$T/fcitx5.log'; }
  exit \$R
"
