#!/usr/bin/env python3
"""A tablet with a physical keyboard, in desktop mode: the suggestion strip must stay above the taskbar.

    python3 android/test/desktop_test.py app-debug.apk out-dir

With a physical keyboard the taskbar stays on screen when the keyboard opens, but Android tells the
keyboard it's only the slim bar it shrinks to, so a short strip can end up behind it. This opens the
app in a desktop window, types on a "physical" keyboard (`input keyboard`), and compares where the
strip is with where the taskbar is, with gesture navigation and with the three buttons.
"""
import json
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
from pathlib import Path

APK, OUT = sys.argv[1], Path(sys.argv[2])
OUT.mkdir(parents=True, exist_ok=True)
PKG = 'com.ekushenext.keyboard'
IME = f'{PKG}/.EkusheIME'
failures = []


def adb(*a, check=True):
    r = subprocess.run(['adb', *a], capture_output=True, text=True, timeout=180)
    if check and r.returncode != 0:
        print(r.stdout, r.stderr)
    return r.stdout


def sh(cmd):
    return adb('shell', cmd, check=False)


def shot(name):
    with open(OUT / f'{name}.png', 'wb') as f:
        f.write(subprocess.run(['adb', 'exec-out', 'screencap', '-p'], capture_output=True, timeout=60).stdout)


def wait_boot():
    adb('wait-for-device')
    for _ in range(120):
        if sh('getprop sys.boot_completed').strip() == '1':
            break
        time.sleep(2)
    time.sleep(8)


def strip_top_bottom():
    """The strip's top and bottom on screen (px), from the debug keyboard's log, once it holds still."""
    last, steady = None, 0
    for _ in range(40):
        time.sleep(0.5)
        log = adb('logcat', '-d', '-s', 'EkusheKeys:I')
        found = [json.loads(m) for m in re.findall(r'EkusheKeys: (\{.*\})', log)]
        found = [f for f in found if f['mode'] == 'STRIP_ONLY']
        cur = found[-1]['strip'] if found else None
        if cur is not None and cur == last:
            steady += 1
            if steady >= 3:
                return cur
        else:
            steady = 0
        last = cur
    return None


def taskbar_top():
    """The top of the taskbar window (px) and whether it is stashed (shrunk to a slim bar)."""
    out = sh('dumpsys window windows')
    top = None
    for block in re.split(r'\n  Window #', out):
        head = block.split('\n')[0]
        if 'Taskbar' in head and 'mHasSurface=true' in block:
            m = re.search(r'mFrame=\[(\d+),(\d+)\]\[(\d+),(\d+)\]', block) or re.search(r'frame=\[(\d+),(\d+)\]\[(\d+),(\d+)\]', block)
            if m:
                top = int(m.group(2))
                print('taskbar window:', head.strip(), m.group(0))
    stash = sh('dumpsys activity service TouchInteractionService')
    lines = [l.strip() for l in stash.split('\n') if re.search(r'(?i)stash', l)]
    print('\n'.join(lines[:12]))
    stashed = any(re.search(r'isStashed=true|mIsStashed=true', l) for l in lines)
    return top, stashed


def field():
    for _ in range(6):
        sh('uiautomator dump /sdcard/ui.xml >/dev/null')
        xml = adb('exec-out', 'cat', '/sdcard/ui.xml')
        try:
            root = ET.fromstring(xml[xml.index('<'):])
        except (ValueError, ET.ParseError):
            time.sleep(0.6)
            continue
        for n in root.iter('node'):
            if n.get('class') == 'android.widget.EditText' and n.get('package') == PKG:
                x1, y1, x2, y2 = map(int, re.findall(r'\d+', n.get('bounds')))
                return (x1 + x2) // 2, (y1 + y2) // 2
        time.sleep(0.6)
    return None


adb('install', '-r', '-g', APK)
sh('settings put global hide_error_dialogs 1')
# desktop windows: freeform windows and the desktop mode on a tablet-sized screen
adb('root'); time.sleep(3); adb('wait-for-device')
sh('settings put global enable_freeform_support 1')
sh('settings put global force_resizable_activities 1')
sh('setprop persist.wm.debug.desktop_mode_enforce_device_restrictions false')
sh('setprop persist.wm.debug.desktop_mode true')
sh('setprop persist.wm.debug.desktop_mode_2 true')
adb('reboot')
wait_boot()
sh(f'ime enable {IME}')
sh(f'ime set {IME}')
sh('settings put secure show_ime_with_hard_keyboard 0')
print('keyboard', sh('settings get secure default_input_method').strip())
print('screen', sh('wm size').strip(), sh('wm density').strip())
print('configuration', sh('dumpsys activity configuration | grep -m1 -i "keys\\|qwerty"').strip()[:300])

for nav in ['gestural', 'threebutton']:
    sh(f'cmd overlay enable-exclusive --category com.android.internal.systemui.navbar.{nav}')
    time.sleep(4)
    sh('input keyevent KEYCODE_WAKEUP'); sh('wm dismiss-keyguard'); sh('input keyevent KEYCODE_HOME'); time.sleep(2)
    sh(f'am force-stop {PKG}')
    # Android picks its own keyboard again after a restart: choose Ekushe now, and check it held
    for _ in range(5):
        sh(f'ime enable {IME}'); sh(f'ime set {IME}'); time.sleep(2)
        if IME.split('/')[0] in sh('settings get secure default_input_method'):
            break
    print(nav, 'keyboard', sh('settings get secure default_input_method').strip())
    # a desktop window (freeform), as when the tablet is in desktop mode
    sh(f'am start -W --windowingMode 5 -n {PKG}/.MainActivity'); time.sleep(3)
    shot(f'60-desktop-{nav}-app')
    at = field()
    if not at:
        failures.append(f'{nav}: no text box found'); continue
    adb('logcat', '-c')
    sh(f'input tap {at[0]} {at[1]}'); time.sleep(1.5)
    sh('input keyboard text ami'); time.sleep(1.5)
    strip = strip_top_bottom()
    bar, stashed = taskbar_top()
    cover = re.findall(r'EkusheTaskbar: (.*)', adb('logcat', '-d', '-s', 'EkusheTaskbar:I'))
    shot(f'61-desktop-{nav}-typing')
    dens = int(re.findall(r'(\d+)', sh('wm density'))[-1]) / 160
    # the taskbar's top as apps are told it (what its window frame shows is taller than the bar)
    m = [json.loads(c) for c in cover if c.startswith('{')]
    if m:
        bar = m[-1]['taskbarTop']
    print(f'{nav}: strip middle={strip} taskbar top={bar} stashed={stashed} measured={cover[-1:] if cover else None} density={dens}')
    if strip is None:
        failures.append(f'{nav}: the strip did not show')
    elif bar is not None and strip + 23 * dens > bar + 2:
        failures.append(f'{nav}: the strip (middle {strip}, bottom {strip + 23 * dens:.0f}) is behind the taskbar (top {bar})')
    (OUT / f'logcat-{nav}.txt').write_text(adb('logcat', '-d', check=False))
    print('keyboard now', sh('settings get secure default_input_method').strip(), '| ime:', ' '.join(re.findall(r'mInputShown=\S+|mCurMethodId=\S+|mCurClient=\S+', sh('dumpsys input_method'))[:3]))
    sh('input keyevent KEYCODE_ESCAPE'); sh('input keyevent KEYCODE_HOME'); time.sleep(1)

crashes = adb('logcat', '-d', '-b', 'crash', check=False)
if PKG in crashes:
    print(crashes[-4000:]); failures.append('keyboard crashed')
print('FAILURES:', failures)
sys.exit(1 if failures else 0)
