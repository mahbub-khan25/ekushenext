#!/usr/bin/env python3
"""End-to-end check of the Fcitx 5 addon: talks to a running fcitx5 the way an application does
(an input context over D-Bus), types Roman letters and checks what Ekushe writes back.

Run inside a D-Bus session with fcitx5 started (see platform/linux/fcitx5/run-test.sh)."""
import time

import dbus
import dbus.mainloop.glib
from gi.repository import GLib

dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus()

deadline = time.time() + 20
# Wait for the fcitx5 that run-test.sh started (asking for the name earlier would make D-Bus start another)
while not bus.name_has_owner("org.fcitx.Fcitx5"):
    assert time.time() < deadline, "fcitx5 did not start"
    time.sleep(0.2)
while True:
    try:
        im = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/org/freedesktop/portal/inputmethod"), "org.fcitx.Fcitx.InputMethod1")
        controller = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/controller"), "org.fcitx.Fcitx.Controller1")
        path, _uuid = im.CreateInputContext([("program", "ekushe-test"), ("display", "x11:")])
        break
    except dbus.DBusException as e:
        assert time.time() < deadline, f"fcitx5 is not reachable: {e}"
        time.sleep(0.3)

ic = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", path), "org.fcitx.Fcitx.InputContext1")
commits, preedits, deleted = [], [], []
bus.add_signal_receiver(lambda s: commits.append(str(s)), "CommitString", "org.fcitx.Fcitx.InputContext1", path=path)
bus.add_signal_receiver(lambda parts, cursor: preedits.append("".join(str(p[0]) for p in parts)),
                        "UpdateFormattedPreedit", "org.fcitx.Fcitx.InputContext1", path=path)
bus.add_signal_receiver(lambda off, n: deleted.append((int(off), int(n))), "DeleteSurroundingText", "org.fcitx.Fcitx.InputContext1", path=path)

PREEDIT, FORMATTED, SURROUNDING = 1 << 1, 1 << 4, 1 << 6
ic.SetCapability(dbus.UInt64(PREEDIT | FORMATTED | SURROUNDING))
ic.FocusIn()


def pump(seconds=0.25):
    end = time.time() + seconds
    ctx = GLib.MainContext.default()
    while time.time() < end:
        while ctx.pending():
            ctx.iteration(False)
        time.sleep(0.01)


SHIFT, CTRL = 1 << 0, 1 << 2
KEY = {"space": 0x20, "Return": 0xff0d, "BackSpace": 0xff08, "Escape": 0xff1b, "Tab": 0xff09, "Shift_L": 0xffe1, "Shift_R": 0xffe2}


def key(sym, mods=0):
    handled = bool(ic.ProcessKeyEvent(dbus.UInt32(sym), dbus.UInt32(0), dbus.UInt32(mods), False, dbus.UInt32(0)))
    ic.ProcessKeyEvent(dbus.UInt32(sym), dbus.UInt32(0), dbus.UInt32(mods), True, dbus.UInt32(0))
    pump(0.12)
    return handled


def tap_shift(name):
    code = 50 if name == "Shift_L" else 62
    ic.ProcessKeyEvent(dbus.UInt32(KEY[name]), dbus.UInt32(code), dbus.UInt32(0), False, dbus.UInt32(0))
    ic.ProcessKeyEvent(dbus.UInt32(KEY[name]), dbus.UInt32(code), dbus.UInt32(SHIFT), True, dbus.UInt32(0))
    pump(0.05)


def double_shift():
    # Fcitx keeps Left Shift for its own switch, so Ekushe's double-tap is on Right Shift
    tap_shift("Shift_R")
    tap_shift("Shift_R")


controller.SetCurrentIM("ekushe")
pump(0.5)
current = str(controller.CurrentInputMethod())
print("current input method:", current)
assert current == "ekushe", f"Ekushe is not the current input method ({current})"

# Choosing Ekushe in Fcitx means Bangla straight away
assert key(ord("x")), "the Ekushe addon does not take keys"
key(KEY["Escape"])
preedits.clear()
commits.clear()
print("addon loaded and typing")

for ch in "amar":
    assert key(ord(ch)), f"key {ch!r} was not handled"
assert preedits and preedits[-1] == "আমার", f"pre-edit: {preedits[-3:]}"
print("pre-edit:", preedits[-1])
assert key(KEY["space"]), "space was not handled"
assert commits and commits[-1] == "আমার ", f"commits: {commits}"
print("committed:", repr(commits[-1]))

for ch in "sonar":
    key(ord(ch))
assert not key(ord("c"), CTRL), "Ctrl+C must go to the application"
assert commits[-1].startswith("সোনা"), f"word not finished before the shortcut: {commits}"
print("shortcut passed through; word finished as", repr(commits[-1]))

for ch in "bangla":
    key(ord(ch))
assert not key(KEY["Return"]), "Enter must reach the application"
assert commits[-1] == "বাংলা", f"commits: {commits}"
print("Enter committed", repr(commits[-1]))

# Tab picks the next suggestion
for ch in "ki":
    key(ord(ch))
first = preedits[-1]
assert key(KEY["Tab"]), "Tab was not handled"
assert preedits[-1] != first, f"Tab did not change the word: {preedits[-3:]}"
print("Tab:", first, "→", preedits[-1])
key(KEY["space"])
print("committed:", repr(commits[-1]))

# Backspace right after a word reopens it (the application reports the text before the cursor)
for ch in "kemon":
    key(ord(ch))
assert key(KEY["space"])
assert commits[-1] == "কেমন ", f"commits: {commits}"
ic.SetSurroundingText("কেমন ", dbus.UInt32(5), dbus.UInt32(5))
pump(0.2)
assert key(KEY["BackSpace"]), "Backspace after a word was not handled"
assert deleted and deleted[-1] == (-5, 5), f"delete-surrounding: {deleted}"
assert preedits[-1] == "কেমন", f"reopened word: {preedits[-3:]}"
print("Backspace reopened", repr(preedits[-1]))
key(KEY["Escape"])

# Double-tap Shift: English letters go straight to the application
double_shift()
assert not key(ord("a")), "in English, letters must go to the application"
double_shift()
assert key(ord("a")), "back in Bangla, letters are Ekushe's again"
key(KEY["Escape"])
print("double-tap Right Shift switches English ⇄ Bangla")

# Fcitx's own Left Shift: to the English keyboard and back to Ekushe
tap_shift("Shift_L")
pump(0.3)
other = str(controller.CurrentInputMethod())
tap_shift("Shift_L")
pump(0.3)
back = str(controller.CurrentInputMethod())
print(f"Left Shift (Fcitx): ekushe → {other} → {back}")
assert other != "ekushe" and back == "ekushe", "Fcitx's Left Shift switch does not reach English and back"
assert key(ord("a")), "Ekushe types again after coming back"
key(KEY["Escape"])

print("all fcitx5 checks passed")
