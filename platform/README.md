# Phase 2–5 Platform Targets (Linking `ekushe-ffi` / `ekushe-core`)

- `platform/macos`: Swift `InputMethodKit` (`IMKInputController`) app linking `libekushe_ffi.a` (Universal binary: Apple Silicon `arm64` + `x86_64`).
- `platform/linux`: `IBus` engine and `Fcitx5` addon linking `libekushe_ffi.so`, packaged for `.deb`, `.rpm`, and Arch Linux `PKGBUILD` (AUR).
- `platform/android`: Android `InputMethodService` (Kotlin + Jetpack Compose) calling `ekushe-ffi`.
- `platform/ios`: iOS Keyboard Extension (`UIInputViewController` + SwiftUI) calling `ekushe-ffi` XCFramework.
