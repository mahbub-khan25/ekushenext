**Ekushe Next is here — first public beta.** A free, open-source Bangla keyboard for **Windows 10/11 (64-bit)**, **macOS 11 or later** (Apple silicon and Intel), **Linux** (64-bit, IBus) and **Android 7 or later**.

Type Bangla the way it sounds, in English letters (`amar sonar bangla` → আমার সোনার বাংলা), and Ekushe writes it with the spelling you meant.

**New in beta.2–beta.6:** a larger, better-checked dictionary. `byabostha` and `byakti` now give ব্যবস্থা and ব্যক্তি, chat verbs like `khaiso` and `korsi` find the standard word with the spoken form next to it, common names (`rahim` → রহিম, `mohammad` → মোহাম্মদ) and chat words (`ok` → ওকে, `koi` → কই) come out right, and 483 spellings were confirmed by native-speaker reviewers.

On Android tablets, a **physical keyboard** (a keyboard cover or a Bluetooth keyboard) now types Bangla through Ekushe: only the suggestion strip shows, double-tap Shift switches Bangla ⇄ English, Tab moves through the suggestions and Backspace right after a word reopens it. New installs start with the **Ekushe Default** theme. The suggestion strip now shows whenever a physical keyboard is attached (also above the taskbar in a tablet's desktop mode), and unplugging it brings the on-screen keys back.

**A new suggestion bar on Windows and Mac**, in the look of the phone keyboards: light frosted glass, rounded word chips, the chosen word in blue and a small EN on English words. On Windows, Bangla letters in the bar are now smooth instead of jagged. Fast typing on Windows is fixed: in beta.5 a key could reach Word or the browser unconverted while the bar redrew, leaving a Latin letter or a doubled letter inside a word.

**Help for the marks, one click away.** `+` makes a conjunct (`as+te` → আস্তে), `,,` shows a hasanta (`as,,te` → আস্‌তে) and `` ` `` keeps letters apart (``as`te`` → আসতে). The full list opens from the **?** button on the Android keyboard, **Marks & Conjuncts Guide** in the Mac and Linux menus, and the popup's সহায়িকা or the tray menu on Windows.

### What makes it different
- **Its own dictionary, built for Ekushe.** Not copied from any other keyboard. The **Bangla Academy standard spelling (প্রমিত বানান) comes first**, even when a non-standard spelling is more common online.
- **Understands how people really type.** Chat spellings like `kemon aso`, `valo achi`, `thik ache` land on the right word, and colloquial forms (করসি, গেসি) are offered as suggestions. On our test set of 1,034 typed words and sentences, 99.1% come out right first time.
- **Fix a wrong word without retyping.** Press Backspace right after a word and it opens again with its suggestions.
- **One keyboard on every system**, with the same engine and dictionary on Windows, Mac, Linux and Android.
- **Ekushe Studio**: a writing pad, a conjunct (যুক্তবর্ণ) guide, a spell checker, a Bijoy ⇄ Unicode converter, snippets, and backup and restore of your settings and learned words.
- **Shortcuts for dates and phrases**: `;aj` → today's date, `;bongabdo` → the Bangla calendar date, `;somoy` → the time.
- **Private by design.** Ekushe works offline, nothing you type ever leaves your computer or phone, and password boxes get plain English. The Android app has no internet permission at all.

This is a beta: please tell us what goes wrong, or what word Ekushe got wrong, with **Report a problem** in the app or at https://ekushenext.com/report/ (no account needed).

### Download
- **Ekushe-Setup.exe** (or **Ekushe-Setup.zip**, if your browser blocks the .exe) installs it for your user account, with no admin rights needed. It starts with Windows, adds a Start Menu shortcut, and can be removed from Settings → Apps.
- **Ekushe-Portable.exe** is the same program, run without installing. It doesn't start with Windows on its own.
- **Ekushe-Developer-Tools-x64.zip** is for testers and developers only. It contains the experimental Windows input method (`ekushe_tsf.dll`) and the command-line tool (`ekushe-cli.exe`). Not needed for everyday typing.

Windows builds are code-signed by **A S M Mahbub Afzal Khan**. The certificate is new and still building reputation, so SmartScreen may show a warning. If it does, click **More info → Run anyway**; the publisher shown should be A S M Mahbub Afzal Khan.

### Mac
1. Download **Ekushe-macOS.pkg** and open it. Because the installer isn't notarized by Apple yet, macOS will say it can't verify it. Go to **System Settings → Privacy & Security**, scroll down and click **Open Anyway**, then enter your password.
2. The installer puts Ekushe in `/Library/Input Methods` and adds it to your input sources. If it doesn't show up, log out and back in, then go to **System Settings → Keyboard → Text Input → Edit… → +**, choose **Bengali → Ekushe Bangla** and click Add.
3. Pick **Ekushe Bangla** from the input menu in the menu bar (or press Control+Space / the 🌐 key to switch).
4. The Ekushe item in that menu opens **Studio** and **Settings** in your browser.

On a Mac, the suggestions appear in Ekushe's frosted bar under the word you're typing (click a word to place it), and Control+Delete flips the last word between Bangla and Roman. **Ekushe-macOS.zip** has the same app for manual install: unzip it and move **Ekushe.app** into `~/Library/Input Methods`.

### Linux
Ekushe is an **IBus** keyboard. IBus is the standard on Ubuntu, Fedora, Debian and GNOME, and also works on KDE.

| You use | Download | Install |
|---|---|---|
| Ubuntu 22.04–26.10 (also Mint, Pop!_OS) | nothing, from the PPA | `sudo add-apt-repository ppa:cicadama/ekushe` then `sudo apt install ekushe` |
| Fedora 43–45 | nothing, from COPR | `sudo dnf copr enable cicadama/ekushe` then `sudo dnf install ekushe` |
| Ubuntu, Debian, Mint, Pop!_OS | `ekushe_…_amd64.deb` | `sudo apt install ./ekushe_*_amd64.deb` |
| Fedora, RHEL, openSUSE | `ekushe-….x86_64.rpm` | `sudo dnf install ./ekushe-*.x86_64.rpm` |
| Any distribution, no package | `Ekushe-…-x86_64.AppImage` | `chmod +x Ekushe-*.AppImage`, then run it with `--install-keyboard` |

After installing:
1. Run `ibus restart`, or log out and back in.
2. Open **Ekushe Studio** from your applications menu and click **কিবোর্ড যোগ করুন** (or run `ekushe-studio --add-keyboard`). By hand: **Settings → Keyboard → Input Sources → + → Bangla (Bangladesh) → Ekushe Bangla**.
3. Switch keyboards with **Super+Space**. **Ekushe Studio** appears in your applications menu and opens in its own window.

To take Ekushe out of your keyboards again, click **কিবোর্ড বাদ দিন** in Studio (or run `ekushe-studio --remove-keyboard`) before removing the package.

Run the AppImage on its own (double-click it) to open Studio without installing anything. IBus only loads keyboards installed for the whole computer, so `--install-keyboard` asks for your password once, and `--uninstall-keyboard` removes the keyboard again.

On Linux the suggestions appear in IBus's own candidate window. Backspace-undo and Ctrl+Backspace need the app to share the text around the cursor; GTK and Qt apps do.

### How to use
- Type Bangla in Roman letters, e.g. `amar sonar bangla`. Space picks the highlighted word, Tab moves to the next suggestion, and Esc keeps what you typed.
- Double-tap **Shift** to switch between Bangla and English.
- For an exact spelling, add a mark: `,,` for hasanta, `^` for chandrabindu, `rr` for রেফ and so on. The full list is at the top of Studio's conjunct guide.
- Picked the wrong word? Press Backspace right after it: the word opens again with its suggestions (the English spelling is one of them). Ctrl+Backspace switches the last word between Bangla and English.
- Select text in any app and press **Ctrl+Alt+C** to turn Bijoy or Banglish into Bangla (Windows).
- Right-click the tray icon to open **Studio**. It has a writing pad, a conjunct guide, a spell checker, a Bijoy ⇄ Unicode converter and settings.

### Known limits in this beta
- The frosted-glass blur behind the suggestion popup needs Windows "Transparency effects" turned on. At 100% opacity in Studio, the popup is solid.
- Some apps, such as Chrome and Word, don't report where the text cursor is. In those apps the popup appears near the mouse instead.
- On Windows, a password box is recognised a moment after you click into it. Typing straight away usually works; if you see Bangla in a password box, switch to English (double-tap Shift) and type it again.
- The Windows input method in the Developer Tools zip is an early preview. Don't run it at the same time as the main app.

If something goes wrong while installing, the steps are recorded in `%TEMP%\ekushe-install.log`. Please paste its last lines into your report at https://ekushenext.com/report/.

The full guide (install, settings, every typing rule) is at https://ekushenext.com/guide/.

Licence: code MPL-2.0, dictionary data CC BY-SA 4.0. See `THIRD_PARTY_NOTICES.md` for the fonts and data sources.

### Android
1. On your phone, download **Ekushe-Next-Android.apk** and open it. If the phone asks, allow installing apps from this source. If Google Play Protect says it doesn't recognise the app, tap **More details → Install anyway**.
2. Open the **Ekushe Next** app and finish its two steps: turn the keyboard on, then choose it.
3. Type in any app. Tap **বাং/EN** to switch between Bangla and English.

Already using an earlier beta? This version installs straight over it on every system; your settings, snippets and learned words stay.
