# Ekushe Next (একুশে নেক্সট) — Modern Cross-Platform Bangla Keyboard & Spell Checker

<p align="center">
  <img src="assets/icons/logo_256.png" width="140" alt="Ekushe Next Logo" />
</p>

**একুশে নেক্সট (Ekushe Next)** is a fast, clean-room, open-source Bangla input method and spell checker built in Rust. Designed so a complete beginner can write Bangla within 60 seconds, while giving power users complete customization.

---

## Key Features

- **Clean-Room Avro-Compatible Phonetic Engine (`ekushe-core`)**:
  - Type naturally: `ami banglay gan gai.` → **আমি বাংলায় গান গাই।**
  - **Effortless Signs (কার / চিহ্ন) & Connected Letters (যুক্তবর্ণ)**:
    - Forgiving multi-alias clusters: `sw` / `sb` → **স্ব** (`swadhin` / `sbadhin` → **স্বাধীন**, `swapno` → **স্বপ্ন**), `shw` / `shb` → **শ্ব** (**বিশ্ব**), `dhw` / `dhb` → **ধ্ব** (**ধ্বনি**), `kkh` / `kSh` → **ক্ষ** (**শিক্ষা**), `gg` / `jNG` → **জ্ঞ** (**জ্ঞান**).
    - **Explicit `+` Joiner**: join *any* two consonants with `+` (`s+b` → **স্ব**, `k+t` → **ক্ত**, `h+m` → **হ্ম**).
    - **Explicit `` ` `` Breaker**: prevent joining (`k`h` → **কহ**).
    - **Live Next-Key Hints**: while typing any consonant, the floating candidate bar shows how to attach every কার, ফলা, and যুক্তবর্ণ with its decomposition (`স্ব = স + ্ + ব`).
- **One-Key Instant Switch (`Double-Tap Shift` by default)**:
  - Tap **Shift twice** (or choose `Ctrl+Space`, `F12`, or `Ctrl+Alt+B` in Settings / System Tray) to toggle between **Normal Default Keyboard (`EN`)** and **Ekushe Next Bangla (`বাং`)** with a floating **বাং / EN Badge**.
- **Backspace Undo Conversion**:
  - Pressing Backspace immediately after committing a word reverts it back to the original Roman word.
- **Fixed Layouts with Smart Kar Reordering**:
  - Ships with **প্রভাত (Probhat)**, **জাতীয় (National / Jatiya)**, **বর্ণনা (Bornona)**, and **মুনীর অপটিমা (Munir Optima)**.
  - Pre-base vowel signs (`ি`, `ে`, `ৈ`) can be typed **before or after** the consonant and automatically reorder to valid Unicode; `ে` + `া` automatically joins into `ো`.
- **Bangla Spell Checker (`spell.rs`)**:
  - Understands common Bangla confusable pairs (`ি↔ী`, `ু↔ূ`, `শ↔ষ↔স`, `ন↔ণ`, `জ↔য`, `র↔ড়↔ঢ়`, `ঙ↔ং`, `ত↔ৎ`) and inflectional suffixes (`-টি`, `-গুলো`, `-ের`, `-কে`, `-দের`).
- **Bijoy (ANSI / SutonnyMJ) ⇄ Unicode & Banglish Converter (`convert.rs`)**.
- **System Tray Resident (`ekushe.exe`) + Ekushe Next Studio UI**:
  - Lives in the Windows System Tray with custom `tray_bangla.ico` / `tray_english.ico` indicators, quick context menu, floating candidate popup, and the **Ekushe Next Studio** web interface.

---

## Quick Start (Windows)

```powershell
# Build all workspace crates
cargo build --workspace

# Run the System Tray & Ekushe Next Studio app
cargo run -p ekushe-tray

# Or install to %LOCALAPPDATA%\Programs\Ekushe with Start Menu shortcut & startup registration
.\scripts\install_windows.ps1
```

> **Note on Windows SmartScreen (Unsigned Binary):**
> If downloading a prebuilt binary before code-signing is configured, click **More info → Run anyway** on first launch.

### Windows input method (preview)

`ekushe_tsf.dll` is Ekushe as a real Windows input method (Text Services Framework). It writes the
word you are typing as a TSF composition instead of simulating key presses, so it works in Store apps
and elevated windows and never flickers. It is a **preview**: there is no candidate window yet (the top
word shows inline, Tab cycles), and only 64-bit applications are covered. To try it, quit the tray app
first (running both converts every key twice), then from an administrator prompt:

```powershell
mkdir "C:\Program Files\Ekushe Next" -Force
copy target\release\ekushe_tsf.dll "C:\Program Files\Ekushe Next\"
regsvr32 "C:\Program Files\Ekushe Next\ekushe_tsf.dll"      # adds "Ekushe Next Bangla" under Bangla (Bangladesh)
regsvr32 /u "C:\Program Files\Ekushe Next\ekushe_tsf.dll"   # removes it
```

Every program loads a registered input method, so it only registers from a folder that only
administrators can change (Program Files or Windows\System32); from anywhere else `regsvr32` fails with
`0x80070005`.

Then pick it with Win+Space. The installer copies the DLL but does not register it.

---

## Ekushe's Own Dictionary

Ekushe Next ships its own clean-room Bangla dictionary (418,743 words), built from open-licensed text
(Bangla Wikipedia, FineWeb-2, AI4Bharat Sangraha; see `tools/dict-builder/SOURCES.md`). Nothing is taken
from Avro, Lekho or OpenBangla.

- **Standard spelling first** (Bangla Academy প্রমিত বানান), with colloquial (কথ্য) and sadhu (সাধু) forms as labelled suggestions
- **Casual Banglish and precise phonetic typing** both work (`valobashi`, `bhalObasi`, `kmn`, `sotti`)
- **Smart mode is the default**: about 97% of pilot words commit correctly on Space; ~0.7 ms per keystroke

| Path | What it is |
| --- | --- |
| `data/dictionary/ekushe.dict.z` | Compressed engine dictionary, embedded in the tray app (`embedded-dict` feature) |
| `data/dictionary/lexicon.tsv` | Every word with its frequency, class, standard form, root and ending |
| `data/dictionary/review-spelling.tsv` | Spelling questions for native-speaker review |
| `data/dictionary/build-report.md` | How the dictionary was built and how well it performs |
| `tools/dict-builder/` | The builder (`build.sh`), rules (`RULES.md`), lookup design (`ROMAN.md`), pilot tests |

```powershell
cargo test -p ekushe-core     # includes engine tests against the real dictionary
cargo run --release -p ekushe-core --example engine_eval -- data/dictionary/ekushe.dict.z tools/dict-builder/eval/pilot.tsv
```

---

## Privacy

Ekushe Next does not send anything over the network. Typing, learned words and settings stay on your
computer (`%APPDATA%\Ekushe` on Windows, `~/Library/Application Support/Ekushe` on macOS, `~/.config/Ekushe`
on Linux). Ekushe Studio runs on `127.0.0.1` only and refuses requests from other websites. There is no
telemetry, no update check and no account.

## What the installer changes

The Windows installer, for your user account only (no administrator rights):
- copies `ekushe.exe` to `%LOCALAPPDATA%\Programs\Ekushe`
- installs the bundled Bangla fonts for your user
- adds a Start Menu shortcut, and a desktop shortcut and "start with Windows" if you leave those ticked
- registers Ekushe in **Settings → Apps**, where **Uninstall** removes all of the above (your settings
  and learned words are kept)

The macOS `.pkg` puts Ekushe in `/Library/Input Methods`; the Linux packages install the IBus engine,
Ekushe Studio and an applications-menu entry. Removing the package removes them.

## License

Ekushe Next is open source.

| Part | Licence |
| --- | --- |
| Code (all crates, `tools/`, scripts) | [Mozilla Public License 2.0](LICENSE) |
| Dictionary data (`data/dictionary/`) | [CC BY-SA 4.0](data/dictionary/LICENSE.md) |
| Bundled fonts (`assets/fonts/`) | Each font's own licence (SIL OFL, GPL with font exception); see [`assets/fonts/README.md`](assets/fonts/README.md) |

Credits for third-party material are in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). Contributions are
welcome; see [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md).
