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


def dismiss_anr():
    """A slow emulator's launcher sometimes freezes and shows "isn't responding" over the app: press Wait."""
    try:
        sh('uiautomator dump /sdcard/ui.xml >/dev/null')
        xml = adb('exec-out', 'cat', '/sdcard/ui.xml')
        for n in ET.fromstring(xml[xml.index('<'):]).iter('node'):
            if n.get('text') == 'Wait' and n.get('package') == 'android':
                x1, y1, x2, y2 = map(int, re.findall(r'\d+', n.get('bounds')))
                sh(f'input tap {(x1 + x2) // 2} {(y1 + y2) // 2}')
                print('dismissed a not-responding box')
                time.sleep(1)
    except (ValueError, ET.ParseError):
        pass


def keys(mode='LETTERS'):
    # Wait for the keyboard to finish sliding in, and for its logged position to stop changing
    last, steady = None, 0
    for _ in range(40):
        time.sleep(0.5)
        log = adb('logcat', '-d', '-s', 'EkusheKeys:I')
        found = [json.loads(m) for m in re.findall(r'EkusheKeys: (\{.*\})', log)]
        found = [f for f in found if f['mode'] == mode]
        cur = found[-1] if found else None
        if cur is not None and cur == last:
            steady += 1
            if steady >= 3:
                return {e['k']: (e['x'], e['y']) for e in cur['keys']}, cur['strip']
        else:
            steady = 0
        last = cur
    raise SystemExit(f'no steady key positions for {mode}')


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
    # uiautomator sometimes catches the screen mid-change; look again a few times
    for _ in range(6):
        try:
            sh('uiautomator dump /sdcard/ui.xml >/dev/null')
            xml = adb('exec-out', 'cat', '/sdcard/ui.xml')
            root = ET.fromstring(xml[xml.index('<'):])
            for n in root.iter('node'):
                if n.get('class') == 'android.widget.EditText' and n.get('package') == PKG:
                    return n.get('text') or '', n.get('bounds')
            # A slow emulator can pop an "isn't responding" box over the app: press Wait
            for n in root.iter('node'):
                if n.get('text') == 'Wait' and n.get('package') == 'android':
                    x1, y1, x2, y2 = map(int, re.findall(r'\d+', n.get('bounds')))
                    sh(f'input tap {(x1 + x2) // 2} {(y1 + y2) // 2}')
                    print('dismissed a not-responding box')
        except (ValueError, ET.ParseError):
            pass
        time.sleep(1.5)
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
sh('ime enable com.google.android.inputmethod.latin/com.android.inputmethod.latin.LatinIME')
sh(f'ime set {IME}')
print('default keyboard:', sh('settings get secure default_input_method').strip())
sh('settings put system screen_off_timeout 1800000')
# The test phone's own launcher is slow to start: don't let its "not responding" box cover the app
sh('settings put secure anr_show_background 0')
sh('settings put global hide_error_dialogs 1')
time.sleep(5)
dismiss_anr()
adb('logcat', '-c')
sh(f'am start -W -n {PKG}/.MainActivity')
time.sleep(3)
dismiss_anr()
shot('01-setup')

# Focus the try-it box: the keyboard opens
text, bounds = field_text()
x1, y1, x2, y2 = map(int, re.findall(r'\d+', bounds))
tap((x1 + x2) // 2, (y1 + y2) // 2, 2.5)
km, strip = keys()
print('keys:', {k: km[k] for k in ['q', 'a', 'z', 'm', 'SPACE', 'DELETE']}, 'strip', strip)
print(sh('dumpsys window | grep -E "InputMethod|mFrame" | head -20'))
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

# English: the EN key hands over to Gboard
tap(*km['LANG'], 2.5)
now_ime = sh('settings get secure default_input_method').strip()
expect('EN hands over to Gboard', now_ime.startswith('com.google.android.inputmethod.latin'), True)
shot('10-gboard-english')
# Gboard's globe key comes back here (the test does it directly), and in Bangla
sh(f'ime set {IME}')
time.sleep(2)
dismiss_anr()
before = field_text()[0] or ''
km, strip = keys()
tap(*km['SPACE'], 0.3)
type_word('ami ', km)
time.sleep(0.5)
got = field_text()[0] or ''
expect('back in Bangla after Gboard', got.endswith('আমি '), True)

# The emoji key next to Space
tap(*km['EMOJI'], 1)
shot('11-emoji-key')
tap(int(int(re.search(r'(\d+)x', sh('wm size')).group(1)) / 9 / 2), int(strip + 4 * 54 * (int(re.findall(r'(\d+)', sh('wm density'))[-1]) / 160)), 0.8)  # ABC

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

# ── Themes: tap each one in the gallery, then open the keyboard and photograph it
def dump_nodes():
    for _ in range(4):
        try:
            sh('uiautomator dump /sdcard/ui.xml >/dev/null')
            xml = adb('exec-out', 'cat', '/sdcard/ui.xml')
            return list(ET.fromstring(xml[xml.index('<'):]).iter('node'))
        except (ValueError, ET.ParseError):
            time.sleep(1)
    return []


def center(n):
    x1, y1, x2, y2 = map(int, re.findall(r'\d+', n.get('bounds')))
    return (x1 + x2) // 2, (y1 + y2) // 2, y1, y2


def find(match, tries=14):
    """Scroll down the settings page until a view matching `match` is fully on screen."""
    for _ in range(tries):
        for n in dump_nodes():
            if match(n):
                x, y, y1, y2 = center(n)
                if y1 > 150 and y2 < 2050:
                    return x, y
        sh('input swipe 540 1700 540 1000 500')
        time.sleep(0.8)
    return None


def to_top():
    for _ in range(6):
        sh('input swipe 540 500 540 2100 150')
    time.sleep(0.8)


def keyboard_shot(name, word='bangla'):
    dismiss_anr()
    to_top()
    text, b = field_text()
    if b is None:
        print('no text box for', name); failures.append(name); return
    bx1, by1, bx2, by2 = map(int, re.findall(r'\d+', b))
    tap((bx1 + bx2) // 2, (by1 + by2) // 2, 1.5)
    km2, _ = keys()
    if word:
        tap(*km2['SPACE'], 0.3)
        type_word(word, km2)
        time.sleep(0.8)
    shot(name)
    sh('input keyevent KEYCODE_BACK')
    time.sleep(0.8)


THEMES = ['Ekushe', 'Krishnachura', 'Padma', 'Shapla', 'Kashful', 'Night', 'Follow system', 'Match my phone', 'Classic light', 'Classic dark']
for i, t in enumerate(THEMES):
    to_top()
    at = find(lambda n, t=t: n.get('content-desc') == f'{t} theme')
    if at is None:
        print('no theme tile', t)
        if t != 'Match my phone':
            failures.append('theme ' + t)
        continue
    if i == 0:
        sh('input swipe 540 1500 540 1100 400')  # show the gallery's first rows well
        time.sleep(0.8)
        shot('20-theme-gallery')
        at = find(lambda n, t=t: n.get('content-desc') == f'{t} theme')
    tap(at[0], at[1], 0.8)
    keyboard_shot(f'2{i + 1}-theme-' + t.lower().replace(' ', '-'))

# Flat keys (borders off), in the signature theme
to_top()
at = find(lambda n: n.get('content-desc') == 'Ekushe theme'); tap(at[0], at[1], 0.6)
at = find(lambda n: (n.get('text') or '').startswith('কীর আলাদা ঘর'))
if at: tap(at[0], at[1], 0.8); keyboard_shot('31-flat-keys')
else: failures.append('borders switch')
to_top()
at = find(lambda n: (n.get('text') or '').startswith('কীর আলাদা ঘর'))
if at: tap(at[0], at[1], 0.6)

# A photo behind the keys (a test picture put where the app keeps its own copy)
subprocess.run(['adb', 'push', str(Path(__file__).with_name('photo.png')), '/data/local/tmp/photo.png'], check=True, capture_output=True)
sh(f'run-as {PKG} cp /data/local/tmp/photo.png files/keyboard_photo.jpg')
sh('input keyevent KEYCODE_HOME'); time.sleep(1)
sh(f'am start -W -n {PKG}/.MainActivity'); time.sleep(2)
at = find(lambda n: (n.get('text') or '').startswith('ছবির ব্যাকগ্রাউন্ড'))
if at:
    tap(at[0], at[1], 1.2)
    shot('32-photo-settings')
    keyboard_shot('33-photo-keyboard')
else:
    failures.append('photo switch')

# Our own English (Gboard setting off), then each key-press effect while a key is held
to_top()
at = find(lambda n: (n.get('text') or '').startswith('ইংরেজির জন্য Gboard'))
if at:
    tap(at[0], at[1], 0.8)
    to_top()
    text, b = field_text()
    bx1, by1, bx2, by2 = map(int, re.findall(r'\d+', b))
    tap((bx1 + bx2) // 2, (by1 + by2) // 2, 1.5)
    km3, _ = keys()
    tap(*km3['LANG'], 0.8)
    for ch in ' hello ':
        tap(*km3[ch] if ch != ' ' else km3['SPACE'], 0.15)
    time.sleep(0.4)
    got = field_text()[0] or ''
    expect('english typing in Ekushe Next', got.endswith('hello ') or got.endswith('Hello '), True)
    shot('34-own-english')
    tap(*km3['LANG'], 0.6)
    sh('input keyevent KEYCODE_BACK'); time.sleep(0.8)
else:
    failures.append('gboard switch')

# Photo off, so the effects are easy to see
to_top()
at = find(lambda n: (n.get('text') or '').startswith('ছবির ব্যাকগ্রাউন্ড'))
if at: tap(at[0], at[1], 0.8)

for label, name in [('থ্রিডি', '3d'), ('আলো', 'glow'), ('ঢেউ', 'ripple'), ('লাফ', 'bounce')]:
    to_top()
    at = find(lambda n, l=label: (n.get('text') or '').lstrip('○● ').startswith(l))
    if not at:
        failures.append('effect ' + name); continue
    tap(at[0], at[1], 0.8)
    to_top()
    text, b = field_text()
    bx1, by1, bx2, by2 = map(int, re.findall(r'\d+', b))
    tap((bx1 + bx2) // 2, (by1 + by2) // 2, 1.5)
    km4, _ = keys()
    # hold the f key (it has no long-press choices) down while the screen is captured
    gx, gy = km4['f']
    hold = subprocess.Popen(['adb', 'shell', f'input swipe {gx} {gy} {gx} {gy} 2500'])
    time.sleep(1.0 if name != 'ripple' else 0.12)
    shot(f'4{["3d", "glow", "ripple", "bounce"].index(name)}-effect-{name}')
    hold.wait()
    sh('input keyevent KEYCODE_BACK'); time.sleep(0.8)

print('FAILURES:', failures)
sys.exit(1 if failures else 0)
