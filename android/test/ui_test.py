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


def ensure_app():
    """Wake the test phone, unlock it, and bring the app to the front."""
    sh('input keyevent KEYCODE_WAKEUP')
    sh('wm dismiss-keyguard')
    time.sleep(0.8)
    sh(f'am start -W -n {PKG}/.MainActivity')
    time.sleep(1.5)
    dismiss_anr()


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

# English, typed in Ekushe Next: completions, a typo fixed by Space, ⌫ to take the fix back
tap(*km['LANG'], 0.8)
def typed(word, pause=0.15):
    for ch in word:
        tap(*km[ch] if ch != ' ' else km['SPACE'], pause)
    time.sleep(0.6)
typed(' beau')
shot('10-english-completions')
for _ in range(4):
    tap(*km['DELETE'], 0.12)
typed('teh ')
got = field_text()[0] or ''
expect('english typo fixed by Space', got.endswith('the '), True)
shot('10b-english-next-word')
tap(*km['DELETE'], 0.6)
got = field_text()[0] or ''
expect('⌫ puts back what was typed', got.endswith('teh'), True)
typed(' ')
got = field_text()[0] or ''
expect('the word you took back stays', got.endswith('teh '), True)
typed('dont i ')
got = field_text()[0] or ''
expect("dont → don't, i → I", got.endswith("don't I "), True)
typed('thank ')
shot('10c-english-thank')

def sym(ch):
    """Type a mark from the symbols page and come back to the letters."""
    tap(*km['SYMBOLS'], 0.6)
    sk, _ = keys('SYMBOLS')
    tap(*sk[ch], 0.3)
    tap(*sk['LETTERS'], 0.6)

# An email address in a chat box: nothing changes or disappears (makhan.aumi@ used to become makhan.@)
typed('makhan.aumi')
sym('@')
typed('gmail.com ')
got = field_text()[0] or ''
expect('email address typed as it is', got.endswith('makhan.aumi@gmail.com '), True)
shot('10d-english-email')
# Banglish in English mode is left alone
typed('ami tomake bhalobashi ')
got = field_text()[0] or ''
expect('Banglish in English mode is not corrected', got.endswith('ami tomake bhalobashi '), True)
# a name in the middle of a sentence keeps its spelling
typed('hello ')
tap(*km['SHIFT'], 0.2)
typed('rahim ')
got = field_text()[0] or ''
expect('a name keeps its spelling', got.endswith('hello Rahim '), True)
# a web address
typed('www.ekushenext.com ')
got = field_text()[0] or ''
expect('web address typed as it is', got.endswith('www.ekushenext.com '), True)
tap(*km['LANG'], 0.8)

# A new Bangla shortcut
tap(*km['SPACE'], 0.3)
sym(';')
typed('brb ')
got = field_text()[0] or ''
expect(';brb → এখুনি আসছি', 'এখুনি আসছি' in got, True)

# The emoji key next to Space
tap(*km['EMOJI'], 1)
shot('11-emoji-key')
tap(int(int(re.search(r'(\d+)x', sh('wm size')).group(1)) / 9 / 2), int(strip + 4 * 54 * (int(re.findall(r'(\d+)', sh('wm density'))[-1]) / 160)), 0.8)  # ABC

# The number pad, in a number box
def number_box():
    for _ in range(4):
        sh('uiautomator dump /sdcard/ui.xml >/dev/null')
        xml = adb('exec-out', 'cat', '/sdcard/ui.xml')
        try:
            root = ET.fromstring(xml[xml.index('<'):])
        except (ValueError, ET.ParseError):
            time.sleep(1); continue
        for n in root.iter('node'):
            if n.get('class') == 'android.widget.EditText' and n.get('package') == PKG and ('123' in (n.get('hint') or '') or n.get('text') == '123'):
                return n.get('bounds')
        time.sleep(1)
    return None
nb = number_box()
if nb:
    nx1, ny1, nx2, ny2 = map(int, re.findall(r'\d+', nb))
    tap((nx1 + nx2) // 2, (ny1 + ny2) // 2, 2)
    nk, _ = keys('NUMBERS')
    for ch in '2026':
        tap(*nk[ch], 0.2)
    time.sleep(0.5)
    shot('12b-number-pad')
    widths = [nk[c][0] for c in '123']
    expect('number pad spans the keyboard', nk['1'][0] < 250 and nk['3'][0] > 600, True)
    tap(x2 - 40, y2 - 25, 1.5)
else:
    failures.append('number box')

# Dark theme
sh('input keyevent KEYCODE_BACK')
time.sleep(0.8)
sh('cmd uimode night yes')
time.sleep(2)
ensure_app()
tap((x1 + x2) // 2, (y1 + y2) // 2, 1.5)
shot('12-dark')
sh('cmd uimode night no')
time.sleep(1)

# Settings page
sh('input keyevent KEYCODE_BACK')
time.sleep(1)
ensure_app()
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


ensure_app()
THEMES = ['Ekushe', 'Ekushe Default', 'Ekushe Smooth', 'Krishnachura', 'Padma', 'Shapla', 'Kashful', 'Night', 'Follow system', 'Match my phone', 'Classic light', 'Classic dark']
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

# The two look-alike themes follow the phone into dark mode
sh('cmd uimode night yes'); time.sleep(2); ensure_app()
for t in ['Ekushe Default', 'Ekushe Smooth']:
    to_top()
    at = find(lambda n, t=t: n.get('content-desc') == f'{t} theme')
    if at:
        tap(at[0], at[1], 0.8)
        keyboard_shot('29d-dark-' + t.lower().replace(' ', '-'))
sh('cmd uimode night no'); time.sleep(2); ensure_app()

# Flat keys (borders off), in the signature theme
to_top()
at = find(lambda n: n.get('content-desc') == 'Ekushe theme')
if at: tap(at[0], at[1], 0.6)
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

# Gboard for English, when it's turned on: EN hands over, and Gboard's globe key comes back in Bangla
to_top()
at = find(lambda n: (n.get('text') or '').startswith('ইংরেজির জন্য Gboard'))
if at:
    tap(at[0], at[1], 0.8)
    to_top()
    text, b = field_text()
    bx1, by1, bx2, by2 = map(int, re.findall(r'\d+', b))
    tap(bx2 - 40, by2 - 25, 1.5)
    km3, _ = keys()
    tap(*km3['LANG'], 2.5)
    now_ime = sh('settings get secure default_input_method').strip()
    expect('EN hands over to Gboard (when on)', now_ime.startswith('com.google.android.inputmethod.latin'), True)
    time.sleep(2)
    tap(bx2 - 40, by2 - 25, 3)
    shot('34-gboard-english')
    sh(f'ime set {IME}')
    time.sleep(2)
    dismiss_anr()
    tap(bx2 - 40, by2 - 25, 2)
    km3, _ = keys()
    tap(*km3['SPACE'], 0.3)
    for ch in 'ami ':
        tap(*km3[ch] if ch != ' ' else km3['SPACE'], 0.15)
    time.sleep(0.5)
    got = field_text()[0] or ''
    expect('back in Bangla after Gboard', 'আমি ' in got, True)
    sh('input keyevent KEYCODE_BACK'); time.sleep(0.8)
    # and off again
    to_top()
    at = find(lambda n: (n.get('text') or '').startswith('ইংরেজির জন্য Gboard'))
    if at: tap(at[0], at[1], 0.8)
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

# A physical keyboard (a tablet's keyboard cover, Bluetooth): keys typed on it go through Ekushe,
# only the suggestion strip is shown, double-tap Shift switches language, ⌫ reopens, Tab picks.
# `input keyboard …` marks the keys as a keyboard's, as a real one does (plain `input keyevent` doesn't)
to_top()
text, b = field_text()
bx1, by1, bx2, by2 = map(int, re.findall(r'\d+', b))
tap(bx2 - 40, by2 - 25, 1.5)
keys()
start = field_text()[0] or ''
sh('input keyboard text ami%sbhalo%sachi%s'); time.sleep(1)
got = field_text()[0] or ''
expect('physical keyboard types Bangla', got[len(start):], 'আমি ভালো আছি ')
try:
    keys('STRIP_ONLY')
    expect('physical keyboard: only the strip is shown', True, True)
except SystemExit:
    expect('physical keyboard: only the strip is shown', False, True)
shot('50-physical-keyboard-strip')
sh('input keyboard keyevent KEYCODE_SHIFT_LEFT KEYCODE_SHIFT_LEFT'); time.sleep(0.6)
start = field_text()[0] or ''
sh('input keyboard text hello%s'); time.sleep(0.8)
got = field_text()[0] or ''
expect('double-tap Shift: English', got[len(start):], 'hello ')
sh('input keyboard keyevent KEYCODE_SHIFT_LEFT KEYCODE_SHIFT_LEFT'); time.sleep(0.6)
start = field_text()[0] or ''
sh('input keyboard text office%s'); time.sleep(0.8)
sh('input keyboard keyevent KEYCODE_DEL'); time.sleep(0.8)
got = field_text()[0] or ''
expect('⌫ reopens the word', got[len(start):], 'অফিস')
shot('51-physical-keyboard-reopened')
sh('input keyboard keyevent KEYCODE_TAB'); time.sleep(0.4)
sh('input keyboard keyevent KEYCODE_SPACE'); time.sleep(0.8)
got = field_text()[0] or ''
expect('Tab picks the next suggestion', got[len(start):], 'অফিসে ')
print('\n'.join(re.findall(r'EkusheHard: .*', adb('logcat', '-d', '-s', 'EkusheHard:I'))[-40:]))
sh('input keyevent KEYCODE_BACK'); time.sleep(0.8)

# A keyboard crash is invisible on screen (Android restarts the keyboard, and hide_error_dialogs is
# on above): fail if the keyboard process crashed at any point during the test
crashes = adb('logcat', '-d', '-b', 'crash', check=False)
if PKG in crashes or 'ekushe' in crashes.lower():
    print(crashes[-6000:])
    failures.append('keyboard crashed (see logcat crash buffer above)')
print('FAILURES:', failures)
sys.exit(1 if failures else 0)
