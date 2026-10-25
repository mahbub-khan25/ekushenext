#!/usr/bin/env python3
"""Ekushe on Linux: check what's installed and running, and whether double-tap Shift works.

Run it as yourself (not with sudo) while logged in to your desktop:
    curl -sO https://raw.githubusercontent.com/mahbub-khan25/ekushenext/main/platform/linux/ibus/diagnose.py
    python3 diagnose.py
Paste the output into your report. It prints no text you have typed."""
import os
import subprocess
import time


def run(*cmd):
    try:
        return subprocess.run(cmd, capture_output=True, text=True, timeout=10).stdout.strip()
    except Exception:  # noqa: BLE001
        return ""


print("== Ekushe diagnose ==")
print("package:", run("rpm", "-q", "ekushe") or run("dpkg-query", "-W", "ekushe") or "(not found)")
print("engine file:", run("/usr/libexec/ekushe/ibus-engine-ekushe", "--version") or "(not found)")
print("ibus:", run("ibus", "version"))
print("session:", os.environ.get("XDG_SESSION_TYPE"), os.environ.get("XDG_CURRENT_DESKTOP"))
print("keyboards:", run("gsettings", "get", "org.gnome.desktop.input-sources", "sources"))
print("active engine:", run("ibus", "engine"))
for pid in run("pgrep", "-f", "ibus-engine-ekushe").split():
    try:
        exe = os.readlink(f"/proc/{pid}/exe")
    except OSError:
        exe = "?"
    note = "  <- OLD version still running: log out and back in" if "(deleted)" in exe else ""
    print(f"running engine: pid {pid} {exe}{note}")
cfg = os.path.expanduser("~/.config/Ekushe/config.toml")
try:
    hot = [l.strip() for l in open(cfg, encoding="utf-8") if l.startswith("switch_hotkey")]
    print("switch key setting:", hot[0] if hot else "(default: double-tap Shift)")
except OSError:
    print("switch key setting: (no settings file: double-tap Shift)")

print("\n== Typing test (a private test box; your apps are not touched) ==")
try:
    import gi

    gi.require_version("IBus", "1.0")
    from gi.repository import GLib, IBus

    IBus.init()
    bus = IBus.Bus()
    if not bus.is_connected():
        raise SystemExit("ibus-daemon is not running")
    before = bus.get_global_engine()
    ic = bus.create_input_context("ekushe-diagnose")
    ic.set_capabilities(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS)
    ic.focus_in()
    bus.set_global_engine("ekushe")
    ctx = GLib.MainContext.default()

    def pump(s=0.2):
        end = time.time() + s
        while time.time() < end:
            while ctx.pending():
                ctx.iteration(False)
            time.sleep(0.01)

    def key(kv, mods=0):
        h = ic.process_key_event(kv, 0, mods)
        ic.process_key_event(kv, 0, mods | IBus.ModifierType.RELEASE_MASK)
        pump(0.1)
        return h

    def shift():
        ic.process_key_event(IBus.KEY_Shift_L, 42, 0)
        ic.process_key_event(IBus.KEY_Shift_L, 42, IBus.ModifierType.SHIFT_MASK | IBus.ModifierType.RELEASE_MASK)
        pump(0.05)

    for _ in range(40):
        pump(0.25)
        if key(ord("x")):
            key(IBus.KEY_Escape)
            break
    bn1 = key(ord("a")); key(IBus.KEY_Escape)
    shift(); shift(); pump(0.3)
    en = not key(ord("a")); key(IBus.KEY_Escape)
    shift(); shift(); pump(0.3)
    bn2 = key(ord("a")); key(IBus.KEY_Escape)
    print("Bangla at start:", bn1, "| English after double Shift:", en, "| Bangla again:", bn2)
    print("RESULT:", "the engine switches fine; the desktop isn't passing Shift to it" if (bn1 and en and bn2)
          else "the engine itself did not switch")
    if before is not None and before.get_name() != "ekushe":
        bus.set_global_engine(before.get_name())
except SystemExit as e:
    print("could not run:", e)
except Exception as e:  # noqa: BLE001
    print("could not run:", e.__class__.__name__, e)
