#!/bin/sh
# Install or remove the Ekushe Bangla keyboard from the AppImage (system-wide, under /usr/local,
# with the IBus registration file in /usr/share/ibus/component where IBus looks for it).
set -e
ACTION="$1"; HERE="$2"
PREFIX=/usr/local
if [ "$ACTION" = install ]; then
  # The AppImage's files are not readable by root through FUSE: stage them first
  STAGE="$(mktemp -d)"
  cp "$HERE/usr/libexec/ekushe/ibus-engine-ekushe" "$HERE/usr/bin/ekushe-studio" "$HERE/usr/share/ekushe/ekushe.png" "$STAGE/"
  sed -e "s|/usr/libexec/ekushe|$PREFIX/libexec/ekushe|" -e "s|/usr/share/ekushe|$PREFIX/share/ekushe|" -e "s|/usr/bin/ekushe-studio|$PREFIX/bin/ekushe-studio|" \
      "$HERE/usr/share/ibus/component/ekushe.xml" > "$STAGE/ekushe.xml"
  pkexec sh -c "install -Dm755 '$STAGE/ibus-engine-ekushe' $PREFIX/libexec/ekushe/ibus-engine-ekushe &&
                install -Dm755 '$STAGE/ekushe-studio' $PREFIX/bin/ekushe-studio &&
                install -Dm644 '$STAGE/ekushe.png' $PREFIX/share/ekushe/ekushe.png &&
                install -Dm644 '$STAGE/ekushe.xml' /usr/share/ibus/component/ekushe.xml &&
                (ibus write-cache --system >/dev/null 2>&1 || true)"
  rm -rf "$STAGE"
  ibus restart >/dev/null 2>&1 || true
  echo "Installed. Add it in Settings → Keyboard → Input Sources → + → Bengali → Ekushe Bangla."
else
  pkexec sh -c "rm -f /usr/share/ibus/component/ekushe.xml $PREFIX/libexec/ekushe/ibus-engine-ekushe $PREFIX/bin/ekushe-studio $PREFIX/share/ekushe/ekushe.png &&
                rmdir $PREFIX/libexec/ekushe $PREFIX/share/ekushe 2>/dev/null; (ibus write-cache --system >/dev/null 2>&1 || true)"
  ibus restart >/dev/null 2>&1 || true
  echo "Removed the Ekushe keyboard."
fi
