# Ekushe Next for iPhone and iPad

The iOS keyboard: the same engine, dictionary and typing rules as every other version, as a
keyboard you turn on in Settings. It lives here:

| Folder | What it is |
|---|---|
| `App/` | The Ekushe Next app: how to turn the keyboard on, a box to try it, help |
| `Keyboard/` | The keyboard extension: `KeyboardView` (drawing and touch), `Typist` (the typing logic), `KeyboardViewController` (iOS) and the in-keyboard settings |
| `Shared/` | The engine bridge, layouts, themes, settings, emoji and English help |
| `Tests/` | Typing tests on a pretend text box, and a picture of every theme |
| `../crates/ekushe-mobile` | The C bridge to the Rust engine |

The keyboard doesn't ask for **Full Access**, so iOS itself keeps it off the internet. The dictionary
is mapped straight from the app rather than unpacked into memory, because iOS stops keyboards that
use too much memory.

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

The GitHub workflow `.github/workflows/ios.yml` does all of this on every change and keeps the
pictures of every theme.
