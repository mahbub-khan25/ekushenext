# Fedora package for COPR, built by .github/workflows/release.yml from the tarball made by
# platform/linux/dist/make-source.sh. @VERSION@ is filled in at build time (e.g. 0.1.0~beta.13).
# The Rust libraries are bundled in vendor/ (listed below as bundled(crate(...))), so the build
# needs no network.

%global debug_package %{nil}

Name:           ekushe
Version:        @VERSION@
Release:        1%{?dist}
Summary:        Bangla phonetic keyboard for IBus, with its own dictionary

# Ekushe's code is MPL-2.0; the dictionary is CC-BY-SA-4.0; Studio embeds fonts under OFL-1.0,
# OFL-1.1 and GPL-2.0-or-later WITH Font-exception-2.0. The bundled Rust libraries are permissive.
License:        MPL-2.0 AND CC-BY-SA-4.0 AND OFL-1.0 AND OFL-1.1 AND GPL-2.0-or-later WITH Font-exception-2.0 AND (MIT OR Apache-2.0) AND MIT AND Unicode-3.0 AND Zlib
URL:            https://ekushenext.com
Source0:        https://github.com/mahbub-khan25/ekushenext/releases/download/@TAG@/ekushe-%{version}.tar.xz

BuildRequires:  cargo >= 1.85
BuildRequires:  rust >= 1.85
BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  pkgconfig(ibus-1.0)
BuildRequires:  pkgconfig(gtk+-3.0)
BuildRequires:  pkgconfig(webkit2gtk-4.1)
BuildRequires:  desktop-file-utils

Requires:       ibus
Requires:       xdg-utils
# Studio's own window (GTK + WebKitGTK); without WebKitGTK Studio opens in the browser instead
Recommends:     webkit2gtk4.1
%global __requires_exclude_from ^%{_libexecdir}/ekushe/ekushe-studio-window$
Provides:       ibus-ekushe = %{version}-%{release}
@BUNDLED@

%description
Ekushe Next is a Bangla keyboard. Type Bangla the way it sounds in English letters
(amar sonar bangla → আমার সোনার বাংলা) and Ekushe writes it with Bangla Academy
standard spellings, using its own dictionary. Suggestions, conjunct hints and
Backspace undo are built in. Everything runs on your computer; nothing you type
is sent anywhere.

Also includes Ekushe Studio: a writing pad, conjunct guide, spell checker and a
Bijoy to Unicode converter, in its own window on any desktop (GNOME, KDE Plasma,
COSMIC, Xfce and others).

%prep
%autosetup -n ekushe-%{version}

%build
export RUSTFLAGS="%{build_rustflags}"
%make_build -f platform/linux/Makefile VERSION=%{version} CARGOFLAGS="--offline --frozen" \
  CFLAGS="%{optflags}" LDFLAGS="%{build_ldflags}"

%install
%make_install -f platform/linux/Makefile VERSION=%{version} CARGOFLAGS="--offline --frozen" \
  PREFIX=%{_prefix} LIBEXECDIR=%{_libexecdir} DATADIR=%{_datadir} BINDIR=%{_bindir}

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/ekushe-studio.desktop
%{buildroot}%{_libexecdir}/ekushe/ibus-engine-ekushe --version

%post
ibus write-cache --system >/dev/null 2>&1 || :

%postun
ibus write-cache --system >/dev/null 2>&1 || :

%preun
# Keyboard lists are per user: tell the person how to take Ekushe out of theirs
if [ "$1" = 0 ]; then
  echo "Ekushe Next: if Ekushe is in your keyboards, run 'ekushe-studio --remove-keyboard' now (before the package is gone), or remove it later from your desktop's keyboard settings." >&2 || :
fi

%files
%license LICENSE THIRD_PARTY_NOTICES.md data/dictionary/LICENSE.md data/dictionary/CC-BY-SA-4.0.txt
%doc README.md
%{_libexecdir}/ekushe/
%{_bindir}/ekushe-studio
%{_datadir}/ibus/component/ekushe.xml
%{_datadir}/ekushe/
%{_datadir}/icons/hicolor/256x256/apps/ekushe.png
%{_datadir}/applications/ekushe-studio.desktop

%changelog
* @DATE@ Ekushe Next <khan.ytube24@gmail.com> - @VERSION@-1
- Release @TAG@: https://github.com/mahbub-khan25/ekushenext/releases/tag/@TAG@
