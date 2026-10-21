# SignPath Foundation application (free code signing for open source)

Apply at <https://signpath.org/>. Answers to their questions, prepared from the project as it is.

## Before you apply (you)

- [ ] Turn on **two-factor authentication** on your GitHub account (Settings → Password and
      authentication). SignPath requires MFA for everyone with access to the repository and to SignPath.
- [ ] Check the repository is public and the latest release builds from GitHub Actions (it does).

## Project details

| Question | Answer |
|---|---|
| Project name | Ekushe Next |
| Repository | https://github.com/mahbub-khan25/ekushenext |
| Homepage / download page | https://github.com/mahbub-khan25/ekushenext/releases |
| Licence | MPL-2.0 (code). Dictionary data CC BY-SA 4.0. Bundled fonts under SIL OFL 1.0/1.1 and GPL-2.0+ with font exception (see `assets/fonts/README.md`) |
| What it does | A Bangla (Bengali) phonetic keyboard for Windows, macOS and Linux with its own dictionary, a spell checker and a Bijoy ⇄ Unicode converter |
| Files to sign | `ekushe.exe` (Windows app and installer), `ekushe_tsf.dll` (Windows input method preview), `ekushe-cli.exe` |
| Build system | GitHub Actions, `.github/workflows/release.yml`, from tagged versions in `.github/release/VERSION` |
| Team | Khan (mahbub-khan25): author, reviewer and approver |
| Network / privacy | No network access at all; see the README "Privacy" section |
| System changes | Listed in the README "What the installer changes"; uninstall from Settings → Apps |
| Maintained | Active; regular beta releases |

## Things they may ask about

- **Keyboard hook.** The Windows app reads keystrokes with a low-level keyboard hook (`WH_KEYBOARD_LL`)
  because that is how a typing app converts Roman letters to Bangla in every program. It never stores
  or sends what is typed; password fields are skipped. The TSF input method (`ekushe_tsf.dll`) is the
  long-term replacement that uses the official Windows text input API.
- **OFL 1.0 fonts.** Kalpurush and SolaimanLipi are SIL Open Font License 1.0 (fonts, not code).
  If SignPath insists on OSI-approved licences for every bundled file, the fallback is to bundle only
  Hind Siliguri and Noto Sans Bengali (OFL 1.1, OSI-approved) and let users install the others.

## After approval

1. Add the "Code signing policy" section SignPath asks for to the README, for example:

   > Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by
   > [SignPath Foundation](https://signpath.org). Team: Khan (author, reviewer, approver).
   > Privacy: this program does not transfer any information to other networked systems.

2. Give the SignPath project, signing policy and artifact configuration slugs to whoever wires it into
   `release.yml` (the SignPath GitHub action replaces the Azure step).
