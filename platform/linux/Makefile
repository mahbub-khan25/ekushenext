# Build and install Ekushe for Linux from source: the IBus keyboard engine, the Fcitx 5 addon (when
# Fcitx 5's development files are installed) and Ekushe Studio.
# Used by the Fedora (COPR) and Ubuntu (PPA) packages, which build from the source tarball made by
# platform/linux/dist/make-source.sh. Run from the top of the source tree:
#   make -f platform/linux/Makefile VERSION=0.1.0
#   make -f platform/linux/Makefile install DESTDIR=/tmp/stage
# CARGOFLAGS="--offline --frozen" builds from the bundled vendor/ folder without a network.

VERSION    ?= 0.1.0
PREFIX     ?= /usr
LIBEXECDIR ?= $(PREFIX)/libexec
DATADIR    ?= $(PREFIX)/share
BINDIR     ?= $(PREFIX)/bin
CARGO      ?= cargo
CARGOFLAGS ?=
CC         ?= cc
CXX        ?= c++
CFLAGS     ?= -O2
CXXFLAGS   ?= $(CFLAGS)
LDFLAGS    ?=
# Fcitx 5 addon: built when Fcitx5Core is found (FCITX5=no skips it, FCITX5=yes insists on it)
FCITX5     ?= auto

TARGET  = target/release
ENGINE  = $(TARGET)/ibus-engine-ekushe
WINDOW  = $(TARGET)/ekushe-studio-window
FCITX   = $(TARGET)/fcitx5/libekushe.so

ifeq ($(FCITX5),auto)
FCITX5 := $(shell [ -f platform/linux/fcitx5/ekushe-fcitx5.cpp ] && pkg-config --exists Fcitx5Core && echo yes)
endif
ifeq ($(FCITX5),yes)
FCITX5_ADDONDIR ?= $(shell pkg-config --variable=libdir Fcitx5Core)/fcitx5
FCITX5_DATADIR  ?= $(DATADIR)/fcitx5
all: $(FCITX)
endif

all: $(ENGINE) $(WINDOW)

$(TARGET)/libekushe_ffi.a $(TARGET)/ekushe-studio:
	$(CARGO) build --release $(CARGOFLAGS) -p ekushe-ffi -p ekushe-studio \
	  --features "ekushe-ffi/embedded-dict ekushe-ffi/studio"

$(ENGINE): platform/linux/ibus/ekushe-engine.c $(TARGET)/libekushe_ffi.a
	$(CC) $(CFLAGS) -Wall -DEKUSHE_VERSION="\"$(VERSION)\"" $< \
	  $$(pkg-config --cflags ibus-1.0) $(TARGET)/libekushe_ffi.a $$(pkg-config --libs ibus-1.0) \
	  -lpthread -ldl -lm $(LDFLAGS) -o $@

# Fcitx 5 addon: a shared library Fcitx loads; the Rust engine is linked in and kept private
$(FCITX): platform/linux/fcitx5/ekushe-fcitx5.cpp $(TARGET)/libekushe_ffi.a
	mkdir -p $(TARGET)/fcitx5
	$(CXX) -std=c++20 $(CXXFLAGS) -Wall -fPIC -shared $< \
	  $$(pkg-config --cflags Fcitx5Core) $(TARGET)/libekushe_ffi.a $$(pkg-config --libs Fcitx5Core) \
	  -lpthread -ldl -lm $(LDFLAGS) -Wl,--no-undefined -Wl,--exclude-libs,ALL -o $@

# Studio's own window (GTK 3 + WebKitGTK 4.1): any desktop, not only GNOME
$(WINDOW): platform/linux/studio/ekushe-studio-window.c
	$(CC) $(CFLAGS) -Wall -DEKUSHE_VERSION="\"$(VERSION)\"" $< \
	  $$(pkg-config --cflags --libs webkit2gtk-4.1 gtk+-3.0) $(LDFLAGS) -o $@

install: all
	install -Dm755 $(ENGINE) $(DESTDIR)$(LIBEXECDIR)/ekushe/ibus-engine-ekushe
	install -Dm755 $(WINDOW) $(DESTDIR)$(LIBEXECDIR)/ekushe/ekushe-studio-window
	install -Dm755 $(TARGET)/ekushe-studio $(DESTDIR)$(BINDIR)/ekushe-studio
	install -d $(DESTDIR)$(DATADIR)/ibus/component
	sed -e "s|@LIBEXECDIR@|$(LIBEXECDIR)/ekushe|" -e "s|@DATADIR@|$(DATADIR)|" -e "s|@BINDIR@|$(BINDIR)|" \
	  -e "s|@VERSION@|$(VERSION)|" platform/linux/ibus/ekushe.xml.in > $(DESTDIR)$(DATADIR)/ibus/component/ekushe.xml
	chmod 644 $(DESTDIR)$(DATADIR)/ibus/component/ekushe.xml
	install -Dm644 assets/icons/logo_256.png $(DESTDIR)$(DATADIR)/ekushe/ekushe.png
	install -Dm644 assets/icons/logo_256.png $(DESTDIR)$(DATADIR)/icons/hicolor/256x256/apps/ekushe.png
	install -Dm644 platform/linux/packaging/ekushe-studio.desktop $(DESTDIR)$(DATADIR)/applications/ekushe-studio.desktop
ifeq ($(FCITX5),yes)
	install -Dm755 $(FCITX) $(DESTDIR)$(FCITX5_ADDONDIR)/libekushe.so
	install -d $(DESTDIR)$(FCITX5_DATADIR)/addon
	sed -e "s|@VERSION@|$(VERSION)|" platform/linux/fcitx5/ekushe-addon.conf.in > $(DESTDIR)$(FCITX5_DATADIR)/addon/ekushe.conf
	chmod 644 $(DESTDIR)$(FCITX5_DATADIR)/addon/ekushe.conf
	install -Dm644 platform/linux/fcitx5/ekushe-im.conf $(DESTDIR)$(FCITX5_DATADIR)/inputmethod/ekushe.conf
endif

clean:
	$(CARGO) clean
	rm -f $(ENGINE) $(FCITX)

.PHONY: all install clean
