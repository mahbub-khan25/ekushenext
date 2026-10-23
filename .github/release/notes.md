**Beta.** Ekushe Next is a Bangla phonetic keyboard for **Windows 10/11 (64-bit)**, **macOS 11 or later** (Apple silicon and Intel) and **Linux** (64-bit, IBus). It comes with its own dictionary, which puts the Bangla Academy standard spelling first.

### What's new in this beta
- **Report a problem from the app.** Choose **🐞 সমস্যা জানান (Report a problem)** in the tray menu on Windows, **Report a Problem…** in the input menu on a Mac, **Report a problem** in the IBus menu on Linux, or the button at the bottom of Studio's settings. It opens a short form on ekushenext.com, with the version, computer, layout and typing mode filled in. Write what happened in Bangla or English; no account is needed, and nothing you have typed in other apps is sent.

Also in this beta (from beta.10): select text in any app and press **Ctrl+Alt+C** to turn old Bijoy or Banglish text into Bangla (Windows), and "English in terminals" now works both ways.

### Download
- **Ekushe-Setup.exe** (or **Ekushe-Setup.zip**, if your browser blocks the .exe) installs it for your user account, with no admin rights needed. It starts with Windows, adds a Start Menu shortcut, and can be removed from Settings → Apps.
- **Ekushe-Portable.exe** is the same program, run without installing. It doesn't start with Windows on its own.
- **Ekushe-Extras-x64.zip** is for testers only. It contains the experimental Windows input method (`ekushe_tsf.dll`) and the command-line tool.

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
| Ubuntu, Debian, Mint, Pop!_OS | `ekushe_…_amd64.deb` | `sudo apt install ./ekushe_*_amd64.deb` |
| Fedora, RHEL, openSUSE | `ekushe-….x86_64.rpm` | `sudo dnf install ./ekushe-*.x86_64.rpm` |
| Any distribution, no package | `Ekushe-…-x86_64.AppImage` | `chmod +x Ekushe-*.AppImage`, then run it with `--install-keyboard` |

After installing:
1. Run `ibus restart`, or log out and back in.
2. Go to **Settings → Keyboard → Input Sources → + → Bengali → Ekushe Bangla**.
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
- The Windows input method in the Extras zip is an early preview. Don't run it at the same time as the main app.

If something goes wrong while installing, the steps are recorded in `%TEMP%\ekushe-install.log`. Please paste its last lines into your report at https://ekushenext.com/report/.

The full guide (install, settings, every typing rule) is at https://ekushenext.com/guide/.

Licence: code MPL-2.0, dictionary data CC BY-SA 4.0. See `THIRD_PARTY_NOTICES.md` for the fonts and data sources.
