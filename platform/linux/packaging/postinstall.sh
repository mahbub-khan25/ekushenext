#!/bin/sh
# Refresh IBus's list of input methods; the user then adds "Ekushe Bangla" in their settings.
if command -v ibus >/dev/null 2>&1; then
  ibus write-cache --system >/dev/null 2>&1 || true
fi
cat <<'MSG'

Ekushe is installed. To start typing Bangla:
  1. Restart IBus (or log out and back in):   ibus restart
  2. Settings → Keyboard → Input Sources → + → Bengali → "Ekushe Bangla"
  3. Switch with Super+Space. "Ekushe Studio" is in your applications menu.

MSG
exit 0
