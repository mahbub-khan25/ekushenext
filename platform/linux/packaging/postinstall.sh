#!/bin/sh
# Refresh IBus's list of input methods; the user then adds "Ekushe Bangla" in their settings.
if command -v ibus >/dev/null 2>&1; then
  ibus write-cache --system >/dev/null 2>&1 || true
fi
cat <<'MSG'

Ekushe is installed. To start typing Bangla:
  IBus (GNOME and most desktops):
    1. Restart IBus (or log out and back in):   ibus restart
    2. Settings → Keyboard → Input Sources → + → Bengali → "Ekushe Bangla"
    3. Switch with Super+Space.
  Fcitx 5 (often on KDE Plasma):
    1. Restart Fcitx (or log out and back in):  fcitx5 -r -d
    2. Fcitx Configuration → Input Method → add "Ekushe Bangla"
    3. Switch with Ctrl+Space (or Left Shift). Inside Ekushe, double-tap Right Shift for English.
  "Ekushe Studio" is in your applications menu.

MSG
exit 0
