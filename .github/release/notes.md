**Beta.** Ekushe Next is a Bangla phonetic keyboard for **Windows 10/11 (64-bit)** and, new in this beta, **macOS 11 or later** (Apple silicon and Intel). It comes with its own dictionary, which puts the Bangla Academy standard spelling first.

### Download
- **Ekushe-Setup.exe** (or **Ekushe-Setup.zip**, if your browser blocks the .exe) installs it for your user account, with no admin rights needed. It starts with Windows, adds a Start Menu shortcut, and can be removed from Settings → Apps.
- **Ekushe-Portable.exe** is the same program, run without installing. It doesn't start with Windows on its own.
- **Ekushe-Extras-x64.zip** is for testers only. It contains the experimental Windows input method (`ekushe_tsf.dll`) and the command-line tool.

Windows SmartScreen may warn you because the app isn't code-signed yet. Click **More info → Run anyway**.

### Mac (first Mac beta)
1. Download **Ekushe-macOS.pkg** and open it. Because the installer isn't notarized by Apple yet, macOS will say it can't verify it. Go to **System Settings → Privacy & Security**, scroll down and click **Open Anyway**, then enter your password.
2. The installer puts Ekushe in `/Library/Input Methods` and adds it to your input sources. If it doesn't show up, log out and back in, then go to **System Settings → Keyboard → Text Input → Edit… → +**, choose **Bengali → Ekushe Bangla** and click Add.
3. Pick **Ekushe Bangla** from the input menu in the menu bar (or press Control+Space / the 🌐 key to switch).
4. The Ekushe item in that menu opens **Studio** and **Settings** in your browser.

On a Mac, the suggestions appear in macOS's own candidate bar, and Control+Delete flips the last word between Bangla and Roman. **Ekushe-macOS.zip** has the same app for manual install: unzip it and move **Ekushe.app** into `~/Library/Input Methods`.

### How to use
- Type Bangla in Roman letters, e.g. `amar sonar bangla`. Space picks the highlighted word, Tab moves to the next suggestion, and Esc keeps what you typed.
- Double-tap **Shift** to switch between Bangla and English.
- Press Backspace right after a word to get your Roman letters back. Ctrl+Backspace switches the last word between Bangla and English.
- Right-click the tray icon to open **Studio**. It has a writing pad, a conjunct guide, a spell checker, a Bijoy ⇄ Unicode converter and settings.

### Known limits in this beta
- The frosted-glass blur behind the suggestion popup needs Windows "Transparency effects" turned on. At 100% opacity in Studio, the popup is solid.
- Some apps, such as Chrome and Word, don't report where the text cursor is. In those apps the popup appears near the mouse instead.
- Password boxes in browsers and Store apps aren't detected yet. Switch to English (double-tap Shift) before typing a password there.
- The Windows input method in the Extras zip is an early preview. Don't run it at the same time as the main app.

If something goes wrong while installing, the steps are recorded in `%TEMP%\ekushe-install.log`. Please attach that file when you report the problem.

Licence: code MPL-2.0, dictionary data CC BY-SA 4.0. See `THIRD_PARTY_NOTICES.md` for the fonts and data sources.
