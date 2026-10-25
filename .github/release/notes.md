**Beta.** Ekushe Next is a Bangla phonetic keyboard for **Windows 10/11 (64-bit)**, **macOS 11 or later** (Apple silicon and Intel), **Linux** (64-bit, IBus) and **Android 7 or later**. It comes with its own dictionary, which puts the Bangla Academy standard spelling first.

### What's new in this beta
- **Linux: one click to add the keyboard.** Installing Ekushe makes it available, but GNOME (Ubuntu, Fedora) doesn't add it to your keyboards by itself. Open **Ekushe Studio** from the applications menu and click **কিবোর্ড যোগ করুন**, or run `ekushe-studio --add-keyboard`. To do it by hand: Settings → Keyboard → Input Sources → + → **Bangla (Bangladesh)** → **Ekushe Bangla**.
- **Linux: the top bar shows অ or EN.** Double-tap Shift switches between Bangla and English while Ekushe stays selected, and the top bar now shows which one you're in. You can also switch from the top bar's menu.

Also in this beta (from beta.15):
- **Much better everyday Bangla.** Chat and casual spellings now land on the right word far more often (`kemon ase` → কেমন আছে, `ami valo achi`, `khaiso`, `thik ache`, `sorry`, `plz`). On our test set of 893 everyday words and sentences, Ekushe now gets 98% right, up from 91%.
- **"A word was wrong" (Windows).** Right-click the tray icon → **✏️ ভুল শব্দ ঠিক করুন**. Your last few words are listed; pick or type the right one and Ekushe learns it at once. The list is kept in memory only (nothing from password boxes), and you can copy your corrections into a report to help improve Ekushe for everyone.
- **Smarter learning.** Ekushe now learns when you pick a different word, not every time you press Space, so it keeps choosing by context (the word before) instead of freezing one choice.
- **Date and time shortcuts:** `;aj` → ২৪ অক্টোবর ২০২৬, `;bar` → শনিবার, `;bongabdo` → ৮ কার্তিক ১৪৩৩ বঙ্গাব্দ (Bangladesh's revised calendar), `;somoy` → সন্ধ্যা ৭:৫০. The popup offers other forms too, like ২৪/১০/২০২৬.
- **Backup and restore** in Studio → Settings (Windows and Linux): one file with your settings, snippets and learned words. Restoring on another computer adds to what's there and never deletes anything.
- **Check for updates (Windows):** the tray menu shows your version and a **🔄 হালনাগাদ দেখুন** item. Ekushe still never goes online by itself; a once-a-day check can be turned on in Studio → Settings.

Also in this beta (from beta.14): **Linux from the Ubuntu PPA and Fedora COPR** (`sudo add-apt-repository ppa:cicadama/ekushe` / `sudo dnf copr enable cicadama/ekushe`), so Ekushe updates with the rest of your system; an Android space bar label that fits every phone; and a **১২৩ / ?123** numbers key.

Also in this beta (from beta.13): **safe English typing on Android.** Email and web addresses are typed exactly as you type them (makhan.aumi@gmail.com stays makhan.aumi@gmail.com). Only Space corrects a word, and never an address, a name in the middle of a sentence, or Banglish in English mode (ami, tomake stay as they are); clear typos are still fixed (teh → the, dont → don't, i → I). Also a full-width number pad, the phone's own Bangla font on the keys, the new shortcuts `;innalillah`, `;barik` and `;brb`, and the testers' zip renamed **Ekushe-Developer-Tools-x64.zip**.

Also in this beta (from beta.12): **Ekushe Next for Android**, the same engine and dictionary as a phone keyboard (**Ekushe-Next-Android.apk**, Android 7 or later).
  - Bangla letter hints on every key, press-and-hold for letters like ট, ৎ, ণ and ষ, suggestions and next-word predictions, ⌫ right after a word to get your letters back, and slide on Space to move the cursor.
  - **English on the same keyboard:** tap **বাং/EN** for English with word completion, spelling fixes when you press Space, and basic next-word suggestions. It uses a 50,000-word list and the phone's own spell checker, and learns the words you use. Prefer Gboard for English (glide typing, voice)? Turn on **ইংরেজির জন্য Gboard** in the app.
  - An emoji key next to Space, and themes: একুশে, **একুশে ডিফল্ট** and **একুশে মসৃণ** (familiar keyboard looks, light and dark), কৃষ্ণচূড়া, পদ্মা, শাপলা, কাশফুল, রাত, your wallpaper's colours, or your own photo behind the keys. Key presses can be classic, 3D, glow, ripple or bounce.
  - No internet permission at all: nothing you type can leave the phone. Password, email and web address boxes get plain English, and nothing is learned in them.

And from beta.11: **Report a problem** from the app. Choose **🐞 সমস্যা জানান (Report a problem)** in the tray menu on Windows, **Report a Problem…** in the input menu on a Mac, **Report a problem** in the IBus menu on Linux, Studio's settings, or the Help section of the Android app. No account is needed.

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

On a Mac, the suggestions appear in macOS's own candidate bar, and Control+Delete flips the last word between Bangla and Roman. **Ekushe-macOS.zip** has the same app for manual install: unzip it and move **Ekushe.app** into `~/Library/Input Methods`.

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
3. Switch keyboards with **Super+Space**. **Ekushe Studio** appears in your applications menu.

Run the AppImage on its own (double-click it) to open Studio without installing anything. IBus only loads keyboards installed for the whole computer, so `--install-keyboard` asks for your password once, and `--uninstall-keyboard` removes the keyboard again.

On Linux the suggestions appear in IBus's own candidate window. Backspace-undo and Ctrl+Backspace need the app to share the text around the cursor; GTK and Qt apps do.

### How to use
- Type Bangla in Roman letters, e.g. `amar sonar bangla`. Space picks the highlighted word, Tab moves to the next suggestion, and Esc keeps what you typed.
- Double-tap **Shift** to switch between Bangla and English.
- For an exact spelling, add a mark: `,,` for hasanta, `^` for chandrabindu, `rr` for রেফ and so on. The full list is at the top of Studio's conjunct guide.
- Press Backspace right after a word to get your Roman letters back. Ctrl+Backspace switches the last word between Bangla and English.
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

If you installed a test build of the Android app before, uninstall it first: test builds were signed with a different key, so the phone won't install this one over them. Updates from now on install straight over this version.
