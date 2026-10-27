#!/bin/sh
if command -v ibus >/dev/null 2>&1; then
  ibus write-cache --system >/dev/null 2>&1 || true
fi
# Keyboard lists are per user and can't be changed from here: if Ekushe is still in your desktop's
# keyboards, remove it from your own account with `ekushe-studio --remove-keyboard` before
# uninstalling (GNOME: Settings → Keyboard → Input Sources also works afterwards)
echo "Ekushe Next removed. If Ekushe was added to your keyboards, remove it with: ekushe-studio --remove-keyboard (run before uninstalling), or from your desktop's keyboard settings." >&2 || true
exit 0
