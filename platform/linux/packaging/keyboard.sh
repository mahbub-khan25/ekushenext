#!/bin/sh
# Install or remove the Ekushe Bangla keyboard from the AppImage (system-wide, under /usr/local,
# with the IBus registration file in /usr/share/ibus/component where IBus looks for it). When Fcitx 5
# is installed, its addon goes into Fcitx's own folders too (Fcitx only looks there).
set -e
ACTION="$1"; HERE="$2"
PREFIX=/usr/local
FCITX_LIB=""
for d in /usr/lib/x86_64-linux-gnu/fcitx5 /usr/lib64/fcitx5 /usr/lib/fcitx5; do
  if [ -e "$d/libdbus.so" ] || [ -e "$d/libclassicui.so" ]; then FCITX_LIB="$d"; break; fi
done
if [ "$ACTION" = install ]; then
  # The AppImage's files are not readable by root through FUSE: stage them first
  STAGE="$(mktemp -d)"
  cp "$HERE/usr/libexec/ekushe/ibus-engine-ekushe" "$HERE/usr/libexec/ekushe/ekushe-studio-window" "$HERE/usr/bin/ekushe-studio" "$HERE/usr/share/ekushe/ekushe.png" "$STAGE/"
  FCITX_CMD=true
  if [ -n "$FCITX_LIB" ]; then
    cp "$HERE/usr/lib/fcitx5/libekushe.so" "$HERE/usr/share/fcitx5/addon/ekushe.conf" "$STAGE/"
    cp "$HERE/usr/share/fcitx5/inputmethod/ekushe.conf" "$STAGE/ekushe-im.conf"
    FCITX_CMD="install -Dm755 '$STAGE/libekushe.so' '$FCITX_LIB/libekushe.so' &&
               install -Dm644 '$STAGE/ekushe.conf' /usr/share/fcitx5/addon/ekushe.conf &&
               install -Dm644 '$STAGE/ekushe-im.conf' /usr/share/fcitx5/inputmethod/ekushe.conf"
  fi
  sed -e "s|/usr/libexec/ekushe|$PREFIX/libexec/ekushe|" -e "s|/usr/share/ekushe|$PREFIX/share/ekushe|" -e "s|/usr/bin/ekushe-studio|$PREFIX/bin/ekushe-studio|" \
      "$HERE/usr/share/ibus/component/ekushe.xml" > "$STAGE/ekushe.xml"
  pkexec sh -c "install -Dm755 '$STAGE/ibus-engine-ekushe' $PREFIX/libexec/ekushe/ibus-engine-ekushe &&
                install -Dm755 '$STAGE/ekushe-studio-window' $PREFIX/libexec/ekushe/ekushe-studio-window &&
                install -Dm755 '$STAGE/ekushe-studio' $PREFIX/bin/ekushe-studio &&
                install -Dm644 '$STAGE/ekushe.png' $PREFIX/share/ekushe/ekushe.png &&
                install -Dm644 '$STAGE/ekushe.xml' /usr/share/ibus/component/ekushe.xml &&
                $FCITX_CMD &&
                (ibus write-cache --system >/dev/null 2>&1 || true)"
  rm -rf "$STAGE"
  ibus restart >/dev/null 2>&1 || true
  if [ -n "$FCITX_LIB" ] && pgrep -x fcitx5 >/dev/null 2>&1; then fcitx5 -r -d >/dev/null 2>&1 || true; fi
  [ -n "$FCITX_LIB" ] && echo "Fcitx 5: open Fcitx Configuration → Input Method and add \"Ekushe Bangla\"."
  echo "Installed. Open Ekushe Studio and click কিবোর্ড যোগ করুন (GNOME), or add it in your keyboard settings: Input Sources → + → Bangla (Bangladesh) → Ekushe Bangla."
else
  FCITX_FILES=""
  [ -n "$FCITX_LIB" ] && FCITX_FILES="$FCITX_LIB/libekushe.so /usr/share/fcitx5/addon/ekushe.conf /usr/share/fcitx5/inputmethod/ekushe.conf"
  pkexec sh -c "rm -f $FCITX_FILES /usr/share/ibus/component/ekushe.xml $PREFIX/libexec/ekushe/ibus-engine-ekushe $PREFIX/libexec/ekushe/ekushe-studio-window $PREFIX/bin/ekushe-studio $PREFIX/share/ekushe/ekushe.png &&
                rmdir $PREFIX/libexec/ekushe $PREFIX/share/ekushe 2>/dev/null; (ibus write-cache --system >/dev/null 2>&1 || true)"
  ibus restart >/dev/null 2>&1 || true
  echo "Removed the Ekushe keyboard."
fi
