"""End-to-end check of a fixed layout in the IBus engine: UniJoy, its link key, a pre-base kar typed
first, AltGr (Mod5) for the third layer, and the standard-spelling hint.

Run like test_engine.py, with Ekushe's settings choosing UniJoy and Bangla at the start
(platform/linux/build.sh --test does this in its own settings folder)."""
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

ic = bus.create_input_context("ekushe-layout-test")
commits, preedits, tables = [], [], []
ic.connect("commit-text", lambda _ic, t: commits.append(t.get_text()))
ic.connect("update-preedit-text", lambda _ic, t, _c, _v: preedits.append(t.get_text()))
ic.connect("update-lookup-table", lambda _ic, t, _v: tables.append(
    [t.get_candidate(i).get_text() for i in range(t.get_number_of_candidates())]))
ic.set_capabilities(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS | IBus.Capabilite.LOOKUP_TABLE)
ic.focus_in()
bus.set_global_engine("ekushe")
ic.set_engine("ekushe")
for _ in range(40):
    desc = ic.get_engine()
    if desc is not None and desc.get_name() == "ekushe":
        break
    time.sleep(0.25)
    ic.focus_in()
    bus.set_global_engine("ekushe")


def pump(seconds=0.15):
    end = time.time() + seconds
    ctx = GLib.MainContext.default()
    while time.time() < end:
        while ctx.pending():
            ctx.iteration(False)
        time.sleep(0.01)


def key(keyval, mods=0):
    handled = ic.process_key_event(keyval, 0, mods)
    ic.process_key_event(keyval, 0, mods | IBus.ModifierType.RELEASE_MASK)
    pump()
    return handled


def typ(keys):
    for k in keys:
        assert key(ord(k)), f"key {k!r} was not handled"


def double_shift():
    for _ in range(2):
        ic.process_key_event(IBus.KEY_Shift_L, 50, 0)
        ic.process_key_event(IBus.KEY_Shift_L, 50, IBus.ModifierType.SHIFT_MASK | IBus.ModifierType.RELEASE_MASK)
        pump(0.05)


# ibus-daemon starts the engine on demand: wait until it answers (a probe key, then Escape)
deadline = time.time() + 20
while True:
    pump(0.5)
    if key(ord("x")):
        key(IBus.KEY_Escape)
        break
    double_shift()
    pump(0.3)
    if key(ord("x")):
        key(IBus.KEY_Escape)
        break
    assert time.time() < deadline, "the Ekushe engine did not start"
preedits.clear()
commits.clear()

typ("jgk")
assert preedits[-1] == "ক্ত", f"link key: {preedits[-3:]}"
key(IBus.KEY_space)
assert commits[-1] == "ক্ত ", commits
print("link key: j g k ->", repr(commits[-1]))

typ("cjf")  # ে typed first, then ক and া
key(IBus.KEY_space)
assert commits[-1] == "কো ", commits
print("pre-base kar: c j f ->", repr(commits[-1]))

assert key(ord("d"), IBus.ModifierType.MOD5_MASK), "AltGr+d was not handled"
key(IBus.KEY_space)
assert commits[-1] == "ই ", commits
print("AltGr+d ->", repr(commits[-1]))

typ("nvjfvD")  # সরকারী: the standard সরকারি is offered next to it
assert preedits[-1] == "সরকারী", preedits[-3:]
assert any("সরকারি" in t for t in tables[-2:]), f"no standard-spelling hint: {tables[-2:]}"
key(IBus.KEY_space)
assert commits[-1] == "সরকারী ", commits
print("spelling hint offered: সরকারী -> সরকারি (Space kept the word as typed)")
print("all fixed-layout IBus checks passed")
