#!/usr/bin/env python3
"""Type on the real keyboard in an emulator and check what reaches the app; save screenshots.

    python3 android/test/ui_test.py app-debug.apk out-dir

The debug keyboard logs where each key is ("EkusheKeys"); taps go to those points with adb.
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
    r = subprocess.run(['adb', *a], capture_output=True, text=True, timeout=120)
    if check and r.returncode:
        print('adb', a, r.stderr)
    return r.stdout


def sh(cmd):
    return adb('shell', cmd)


def shot(name):
    with open(OUT / f'{name}.png', 'wb') as f:
        f.write(subprocess.run(['adb', 'exec-out', 'screencap', '-p'], capture_output=True, timeout=60).stdout)
    print('screenshot', name)


def keys(mode='LETTERS'):
    time.sleep(1.0)
    for _ in range(20):
        log = adb('logcat', '-d', '-s', 'EkusheKeys:I')
        found = [json.loads(m) for m in re.findall(r'EkusheKeys: (\{.*\})', log)]
        found = [f for f in found if f['mode'] == mode]
        if found:
            k = found[-1]
            return {e['k']: (e['x'], e['y']) for e in k['keys']}, k['strip']
        time.sleep(0.5)
    raise SystemExit(f'no key positions for {mode}')


def tap(x, y, pause=0.12):
    sh(f'input tap {x} {y}')
    time.sleep(pause)


def type_word(word, km):
    for ch in word:
        if ch == ' ':
            tap(*km['SPACE'], 0.35)
        else:
            tap(*km[ch])


def field_text():
    sh('uiautomator dump /sdcard/ui.xml >/dev/null')
    xml = adb('exec-out', 'cat', '/sdcard/ui.xml')
    root = ET.fromstring(xml[xml.index('<'):])
    for n in root.iter('node'):
        if n.get('class') == 'android.widget.EditText' and n.get('package') == PKG:
            return n.get('text') or '', n.get('bounds')
    return None, None


def expect(label, got, want):
    ok = got == want
    print(('PASS' if ok else 'FAIL'), label, repr(got), '' if ok else f'(want {want!r})')
    if not ok:
        failures.append(label)


print(adb('install', '-r', '-g', APK))
# The system takes a moment to notice a newly installed keyboard
for _ in range(40):
    if IME in sh('ime list -a -s'):
        break
    time.sleep(0.5)
print('keyboards:', sh('ime list -a -s').split())
sh(f'ime enable {IME}')
sh(f'ime set {IME}')
print('default keyboard:', sh('settings get secure default_input_method').strip())
sh('settings put system screen_off_timeout 1800000')
adb('logcat', '-c')
sh(f'am start -W -n {PKG}/.MainActivity')
time.sleep(3)
shot('01-setup')

# Focus the try-it box: the keyboard opens
text, bounds = field_text()
x1, y1, x2, y2 = map(int, re.findall(r'\d+', bounds))
tap((x1 + x2) // 2, (y1 + y2) // 2, 2.5)
km, strip = keys()
shot('02-keyboard-bangla')

type_word('amar', km)
time.sleep(0.6)
shot('03-suggestions-amar')
type_word(' sonar bangla ', km)
time.sleep(0.6)
expect('phonetic words', field_text()[0], 'আমার সোনার বাংলা ')
shot('04-amar-sonar-bangla')

# Backspace right after a word reopens its letters
type_word('office', km)
time.sleep(0.5)
shot('05-office')
tap(*km['SPACE'], 0.5)
expect('dictionary word', field_text()[0], 'আমার সোনার বাংলা অফিস ')
tap(*km['DELETE'], 0.6)
shot('06-backspace-reopens')
expect('backspace reopens the letters', field_text()[0], 'আমার সোনার বাংলা অফিস')
tap(*km['SPACE'], 0.5)
expect('word placed again', field_text()[0], 'আমার সোনার বাংলা অফিস ')

# Long press: the capital letters and marks
x, y = km['t']
sh(f'input motionevent DOWN {x} {y}')
time.sleep(0.9)
shot('07-longpress-t')
sh(f'input motionevent UP {x} {y}')
time.sleep(0.4)

# Next-word suggestions after শুভ
for ch in 'shuvo ':
    tap(*km[ch] if ch != ' ' else km['SPACE'], 0.15)
time.sleep(0.6)
shot('08-next-word')

# Numbers and symbols
tap(*km['SYMBOLS'], 0.8)
shot('09-symbols')
sk, _ = keys('SYMBOLS')
tap(*sk['LETTERS'], 0.8)

# English
tap(*km['LANG'], 0.6)
shot('10-english')
for ch in 'hello ':
    tap(*km[ch] if ch != ' ' else km['SPACE'], 0.15)
time.sleep(0.4)
got = field_text()[0]
expect('english typing', got.endswith('Hello ') or got.endswith('hello '), True)
tap(*km['LANG'], 0.6)

# Emoji panel from the toolbar (shown while nothing is being typed)
density = int(re.findall(r'(\d+)', sh('wm density'))[-1]) / 160
width = int(re.search(r'(\d+)x', sh('wm size')).group(1))
tap(int(width - 2.5 * 52 * density - 8 * density), strip, 1)
shot('11-emoji')
tap(int(width / 9 / 2), int(strip + 4 * 54 * density), 0.8)  # ABC

# Dark theme
sh('cmd uimode night yes')
time.sleep(2)
tap((x1 + x2) // 2, (y1 + y2) // 2, 1.5)
shot('12-dark')
sh('cmd uimode night no')
time.sleep(1)

# Settings page
sh('input keyevent KEYCODE_BACK')
time.sleep(1)
sh('input swipe 500 1800 500 400 400')
time.sleep(1)
shot('13-settings')
sh('input swipe 500 1800 500 300 400')
time.sleep(1)
shot('14-settings-more')

print('FAILURES:', failures)
sys.exit(1 if failures else 0)
