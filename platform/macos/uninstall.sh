#!/bin/bash
# Remove Ekushe Next from this Mac:
#   sudo "/Library/Input Methods/Ekushe.app/Contents/Resources/uninstall.sh"
# Takes out the Shishir Bangla files the installer put in /Library/Fonts (only those listed in
# /Library/Application Support/Ekushe/fonts-installed.txt; a font you installed yourself stays),
# then Ekushe.app. Your settings and learned words (~/Library/Application Support/Ekushe) are kept.
set -euo pipefail
if [ "$(id -u)" != 0 ]; then
  echo "Run with sudo: sudo \"$0\"" >&2
  exit 1
fi
RECORD="/Library/Application Support/Ekushe/fonts-installed.txt"
if [ -f "$RECORD" ]; then
  while IFS= read -r name; do
    case "$name" in
      ShishirBangla-*.ttf) ;;      # only Ekushe's own file names, never a path
      *) continue ;;
    esac
    case "$name" in */*) continue ;; esac
    rm -f "/Library/Fonts/$name" && echo "removed /Library/Fonts/$name"
  done < "$RECORD"
  rm -f "$RECORD"
  rmdir "/Library/Application Support/Ekushe" 2>/dev/null || true
fi
/usr/bin/pkill -x Ekushe 2>/dev/null || true
rm -rf "/Library/Input Methods/Ekushe.app"
/usr/sbin/pkgutil --forget org.ekushe.inputmethod.pkg >/dev/null 2>&1 || true
echo "Ekushe Next removed. Remove it from System Settings > Keyboard > Input Sources if it is still listed."
