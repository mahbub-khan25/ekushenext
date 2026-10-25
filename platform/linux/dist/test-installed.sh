#!/bin/bash
# After installing the ekushe package (Fedora or Ubuntu): check the engine starts and type Bangla
# into a real IBus. Needs python3-gi (python3-gobject), dbus. Run from the source tree.
set -euo pipefail
cd "$(dirname "$0")/../../.."
if ldd /usr/libexec/ekushe/ibus-engine-ekushe | grep -i "not found"; then exit 1; fi
/usr/libexec/ekushe/ibus-engine-ekushe --version
# Studio's own window: installed, and it runs when WebKitGTK is there (it is only recommended)
test -x /usr/libexec/ekushe/ekushe-studio-window
if ! ldd /usr/libexec/ekushe/ekushe-studio-window | grep -q "not found"; then
  /usr/libexec/ekushe/ekushe-studio-window --version
else
  echo "WebKitGTK not installed: Studio will open in the browser"
fi
grep -q "<exec>/usr/libexec/ekushe/ibus-engine-ekushe " /usr/share/ibus/component/ekushe.xml
rm -rf ~/.config/ibus/bus
dbus-run-session -- bash -c 'ibus-daemon --xim=false --panel=disable --emoji-extension=disable --cache=none --config=disable &
  for i in $(seq 1 50); do F=$(ls ~/.config/ibus/bus/* 2>/dev/null | head -1); [ -n "$F" ] && break; sleep 0.2; done
  export $(grep "^IBUS_ADDRESS=" "$F"); timeout 120 python3 platform/linux/ibus/test_engine.py'
