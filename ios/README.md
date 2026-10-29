# Ekushe Next for iPhone and iPad

The iOS keyboard: the same engine, dictionary and typing rules as every other version, as a
keyboard you turn on in Settings. It lives here:

| Folder | What it is |
|---|---|
| `App/` | The Ekushe Next app: getting started, the **writing pad** (`PadView.swift`) and the marks |
| `Keyboard/` | The keyboard extension: `KeyboardView` (drawing, touch, VoiceOver), `Typist` (the typing logic), `KeyboardViewController` (iOS), the in-keyboard settings and marks |
| `Shared/` | The engine bridge, layouts (iPhone, iPad, iPad split), themes, settings, emoji, English help, the marks list and the pad's key router |
| `Tests/` | Typing, layout and VoiceOver tests on a pretend text box, and a picture of every theme and layout |
| `UITests/` | The app on a simulator: every screen pictured, and the pad typing Bangla from the simulator's keyboard |
| `../crates/ekushe-mobile` | The C bridge to the Rust engine |

The keyboard doesn't ask for **Full Access**, so iOS itself keeps it off the internet. The dictionary
is mapped straight from the app rather than unpacked into memory, because iOS stops keyboards that
use too much memory. (Without Full Access the keyboard and the app can't share a container either,
so the keyboard's settings live in the keyboard, under ⚙︎.)

### iPhone and iPad

- **iPhone**, upright and on its side (the keyboard is lower in landscape and follows a rotation).
- **iPad**: its own layout in Apple's proportions (Delete top right, a wide Return on the indented
  second row, Shift on both sides with `,` and `.`, a key to hide the keyboard), bigger keys and
  letters, no key balloons. In landscape it can be **split** for thumbs (⚙︎ → ভাঙা কিবোর্ড).
- **VoiceOver** reads and presses every key and suggestion.
- **? in the strip** shows the marks (`+` joins, `,,` shows a hasanta, `` ` `` keeps apart …).

### Physical keyboards and the writing pad

iPadOS doesn't let any third-party keyboard read a physical keyboard: its keys go straight to the
app. So the Ekushe app has a **writing pad**, where any keyboard, an iPad's keyboard cover or a
Bluetooth one included (or Apple's own on-screen keyboard), types phonetic Bangla through the same
engine and suggestion strip: Tab moves through the suggestions, Space places one, Backspace right
after a word reopens it, Esc keeps what's shown, and Shift tapped twice switches Bangla ⇄ English.
The text stays between visits; copy or share it when done. The pad uses the keyboard's own
dictionary, from inside the keyboard, so the app carries it once.

The version comes from `../.github/release/VERSION` (`build-rust.sh` writes `Version.xcconfig`), and
both the app and the keyboard carry Apple's privacy manifest (`PrivacyInfo.xcprivacy`).

## Run it on your own iPhone or iPad (free, with Xcode)

You need a Mac with **Xcode** (from the App Store), and an Apple ID.

1. Install the tools, once:
   ```sh
   curl https://sh.rustup.rs -sSf | sh        # Rust
   brew install xcodegen                       # makes the Xcode project (https://brew.sh for brew)
   ```
2. Build the engine and make the project:
   ```sh
   git clone https://github.com/mahbub-khan25/ekushenext.git
   cd ekushenext/ios
   ./build-rust.sh
   xcodegen
   open EkusheNext.xcodeproj
   ```
3. In Xcode, click the project in the left column, then for **both** targets **EkusheNext** and
   **EkusheKeyboard** open **Signing & Capabilities**, tick **Automatically manage signing** and
   choose your **Team** (your Apple ID; add it under Xcode → Settings → Accounts if it isn't listed).
   If Xcode says the bundle identifier is taken, change `com.ekushenext.app` to something of your own
   (for example `com.yourname.ekushe`) in both targets, keeping `.keyboard` at the end of the keyboard's.
4. Connect the iPad or iPhone with a cable (or pick it in the device list if it's on the same Wi-Fi),
   choose it at the top of the window, and press **Run** (▶).
5. The first time, the device asks you to trust the developer: **Settings → General → VPN & Device
   Management → your Apple ID → Trust**. On iOS 16 and later also turn on **Settings → Privacy &
   Security → Developer Mode** when asked.
6. Turn the keyboard on: **Settings → Ekushe Next → Keyboards → Ekushe Next** (or Settings → General →
   Keyboard → Keyboards → Add New Keyboard → Ekushe Next). It doesn't need "Allow Full Access".
7. In any app, touch and hold 🌐 and choose **Ekushe Next**.

With a free Apple ID the app works for 7 days, then press Run again. With the Apple Developer
Program (99 US$ a year) it lasts a year and can go to TestFlight and the App Store.

## On a Mac without the device: the tests

```sh
xcodebuild test -project EkusheNext.xcodeproj -scheme EkusheNext -destination 'platform=iOS Simulator,name=iPhone 16'
```

The GitHub workflow `.github/workflows/ios.yml` runs the tests on an iPhone and an iPad simulator on
every change, and keeps the pictures of every theme, layout and app screen (`ios-keyboard-pictures`).
