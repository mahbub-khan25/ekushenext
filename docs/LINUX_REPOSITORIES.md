# Fedora COPR and Ubuntu PPA

Every release is also built from source on Fedora's **COPR** and Ubuntu's **Launchpad PPA**, so
people install Ekushe with their own package manager and get updates with the rest of their system:

```sh
# Fedora
sudo dnf copr enable <copr-user>/ekushe
sudo dnf install ekushe

# Ubuntu 22.04, 24.04, 26.04, 26.10
sudo add-apt-repository ppa:<launchpad-user>/ekushe
sudo apt install ekushe
```

`.github/workflows/linux-repos.yml` does the uploads. `release.yml` runs it after each release; to
upload a release that already exists, open **Actions → Linux repositories (COPR and PPA) → Run
workflow** and type its tag. Each half is skipped until its secret and variable below are set.

How it works: `platform/linux/dist/make-source.sh` makes `ekushe-<version>.tar.xz`, which has only
what the Linux build needs plus every Rust library (`vendor/`), because COPR and Launchpad build
without internet. `ekushe.spec` (Fedora) and `debian/` (Ubuntu) build it with
`platform/linux/Makefile`. CI (`ci.yml`, *copr-check* and *ppa-check*) builds both from source and
types Bangla into a real IBus on every push, so a change that would break them shows up there first.

## COPR (Fedora)

1. Create a Fedora account at <https://accounts.fedoraproject.org> (free).
2. Sign in once at <https://copr.fedorainfracloud.org> with that account.
3. Open <https://copr.fedorainfracloud.org/api/>. It shows a block of text starting with `[copr-cli]`.
   Copy all of it.
4. In GitHub, open the repository's **Settings → Secrets and variables → Actions**:
   - **Secrets** tab → *New repository secret*: name `COPR_CONFIG`, value: the block from step 3.
   - **Variables** tab → *New repository variable*: name `COPR_PROJECT`, value `<your Fedora username>/ekushe`.

The first run creates the `ekushe` project on COPR, for every Fedora release COPR offers (Intel/AMD
and ARM). **The COPR token expires after 180 days**: when a run fails at "COPR sign-in", repeat
step 3 and update the secret.

## PPA (Ubuntu)

Uploads to a PPA must be signed with an OpenPGP key registered to your Launchpad account. The
release key for Ekushe has fingerprint `C3385F0E39191191C05A7BD83C47858CCD4AA0E6`
("Ekushe Next <khan.ytube24@gmail.com>"). Keep its private part and passphrase safe and private
(password manager plus an offline copy); never commit them.

1. Create a Launchpad account at <https://launchpad.net> (top right, *Log in / Register*) with the
   same email address as the key.
2. Put the public key on Ubuntu's key server: <https://keyserver.ubuntu.com> → *Submit Key* → paste
   `public-key.asc`.
3. On Launchpad, open your profile → **OpenPGP keys** (pencil icon) → paste the fingerprint →
   *Import Key*. Launchpad emails you a message encrypted to that key. Decrypt it (for example with
   GPG Suite on a Mac, or `gpg --decrypt`), open the link inside and confirm.
4. On your profile, **Create a new PPA**: URL `ekushe`, display name `Ekushe Next`.
   (If Launchpad asks you to accept the Ubuntu Code of Conduct first, do that.)
   Optional: in the PPA's *Change details*, also tick **arm64** so ARM computers get it too.
5. In GitHub **Settings → Secrets and variables → Actions**:
   - Secret `PPA_GPG_KEY`: the whole contents of `PPA_GPG_KEY.asc`
     (from `-----BEGIN PGP PRIVATE KEY BLOCK-----` to the `END` line).
   - Secret `PPA_GPG_PASSPHRASE`: the key's passphrase.
   - Variable `PPA`: `<your Launchpad username>/ekushe`.
   - Optional variable `PPA_SERIES`: which Ubuntu releases to build for, default
     `jammy noble resolute stonking` (22.04, 24.04, 26.04, 26.10). Add a new release's code name
     when it comes out.

Launchpad builds each upload in about 15–30 minutes and emails you if a build fails. Each version
can be uploaded to a PPA only once per Ubuntu release; to rebuild the same version, run
`make-deb-source.sh` with `UPLOAD=2`.

## Getting into the official Fedora and Ubuntu repositories later

COPR and the PPA are the usual first step. The official repositories are run by volunteers who
review new packages: for Fedora, become a packager and file a package review request; for Ubuntu,
the package goes into Debian first (through a Debian sponsor), and Ubuntu copies it from there.
Both want each Rust library as its own distribution package rather than bundled; the Linux build
uses only about 35 common libraries (`build/dist/bundled-crates.txt`), most of which they already
have.
