#!/usr/bin/env python3
"""End-to-end check of the IBus engine: talks to a running ibus-daemon the way an application
does (an input context), types Roman letters and checks what Ekushe writes back.

Run inside a D-Bus session with ibus-daemon started (see platform/linux/build.sh --test)."""
import sys
import time

import gi

gi.require_version("IBus", "1.0")
from gi.repository import GLib, IBus  # noqa: E402

IBus.init()
bus = IBus.Bus()
deadline = time.time() + 10
while not bus.is_connected() and time.time() < deadline:
    time.sleep(0.2)
    bus = IBus.Bus()
assert bus.is_connected(), "ibus-daemon is not reachable"

names = [e.get_name() for e in bus.list_engines()]
assert "ekushe" in names, f"engine not registered; engines: {names}"
print("engine registered")

ic = bus.create_input_context("ekushe-test")
assert ic is not None, "could not create an input context"
commits, preedits = [], []
ic.connect("commit-text", lambda _ic, t: commits.append(t.get_text()))
ic.connect("update-preedit-text", lambda _ic, t, _c, _v: preedits.append(t.get_text()))
caps = IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS | IBus.Capabilite.LOOKUP_TABLE
ic.set_capabilities(caps)
ic.focus_in()
# Newer IBus (1.5.30+) ignores SetEngine from ordinary clients; the global engine (what
# `ibus engine ekushe` sets) applies to the focused input context everywhere
bus.set_global_engine("ekushe")
ic.set_engine("ekushe")
for _ in range(40):
    desc = ic.get_engine()
    if desc is not None and desc.get_name() == "ekushe":
        break
    time.sleep(0.25)
    ic.focus_in()
    bus.set_global_engine("ekushe")
desc = ic.get_engine()
print("input context engine:", desc.get_name() if desc else None, "| IBus", IBus.MAJOR_VERSION, IBus.MINOR_VERSION, IBus.MICRO_VERSION)


def pump(seconds=0.3):
    end = time.time() + seconds
    ctx = GLib.MainContext.default()
    while time.time() < end:
        while ctx.pending():
            ctx.iteration(False)
        time.sleep(0.01)


def key(keyval, mods=0):
    handled = ic.process_key_event(keyval, 0, mods)
    ic.process_key_event(keyval, 0, mods | IBus.ModifierType.RELEASE_MASK)
    pump(0.15)
    return handled


# ibus-daemon starts the engine process on demand: wait until it answers (a probe key, then Escape)
deadline = time.time() + 20
while True:
    pump(0.5)
    if key(ord("x")):
        key(IBus.KEY_Escape)
        break
    assert time.time() < deadline, "the Ekushe engine did not start (see ibus-daemon output above)"
preedits.clear()
commits.clear()
print("engine started")
for ch in "amar":
    assert key(ord(ch)), f"key {ch!r} was not handled"
assert preedits and preedits[-1] == "আমার", f"pre-edit: {preedits[-3:]}"
print("pre-edit:", preedits[-1])
assert key(IBus.KEY_space), "space was not handled"
assert commits and commits[-1] == "আমার ", f"commits: {commits}"
print("committed:", repr(commits[-1]))

for ch in "sonar":
    key(ord(ch))
handled = key(ord("c"), IBus.ModifierType.CONTROL_MASK)
assert not handled, "Ctrl+C must go to the application"
assert commits[-1].startswith("সোনা"), f"word not finished before the shortcut: {commits}"
print("shortcut passed through; word finished as", repr(commits[-1]))

for ch in "bangla":
    key(ord(ch))
assert not key(IBus.KEY_Return), "Enter must reach the application"
assert commits[-1] == "বাংলা", f"commits: {commits}"
print("Enter committed", repr(commits[-1]))

# A password field: keys go straight to the application, nothing is pre-edited or learned
ic.set_content_type(IBus.InputPurpose.PASSWORD, 0)
pump(0.3)
before = len(preedits), len(commits)
for ch in "amar":
    assert not key(ord(ch)), f"key {ch!r} was taken in a password field"
assert (len(preedits), len(commits)) == before or not any(preedits[before[0]:]), f"pre-edit in a password field: {preedits[before[0]:]}"
print("password field: keys passed through")
# Back to an ordinary field: Bangla again
ic.set_content_type(IBus.InputPurpose.FREE_FORM, 0)
pump(0.3)
for ch in "amar":
    assert key(ord(ch)), f"key {ch!r} was not handled after leaving the password field"
assert preedits[-1] == "আমার", f"pre-edit: {preedits[-3:]}"
key(IBus.KEY_Escape)
print("ordinary field again: Bangla")

# Double-tap Shift switches to English and back, and the top bar (the "InputMode" symbol) follows
# Property updates go to the panel (GNOME Shell on a desktop): stand in for it
symbols = []
panel = IBus.PanelService.new(bus.get_connection())
bus.request_name("org.freedesktop.IBus.Panel", 0)
labels = []
panel.connect("update-property", lambda _p, p: p.get_key() == "InputMode"
              and (symbols.append(p.get_symbol().get_text()), labels.append(p.get_label().get_text())))
pump(0.5)
ic.focus_out(); ic.focus_in(); pump(0.3)


def shift_tap():
    ic.process_key_event(IBus.KEY_Shift_L, 50, 0)
    ic.process_key_event(IBus.KEY_Shift_L, 50, IBus.ModifierType.SHIFT_MASK | IBus.ModifierType.RELEASE_MASK)
    pump(0.05)


shift_tap(); shift_tap(); pump(0.3)
assert not key(ord("a")), "after double-tap Shift, letters must go through as English"
assert symbols and symbols[-1] == "EN", f"top bar symbol after switching to English: {symbols}"
assert labels[-1] == "English", f"the label must name the language in use: {labels}"
shift_tap(); shift_tap(); pump(0.3)
assert key(ord("a")), "a second double-tap Shift must bring Bangla back"
key(IBus.KEY_Escape)
assert symbols[-1] == "এ", f"top bar symbol after switching back: {symbols}"
assert labels[-1] == "বাংলা", f"the label must name the language in use: {labels}"
print("double-tap Shift: English and back; top bar shows", symbols[-2:])
print("OK")
sys.exit(0)
