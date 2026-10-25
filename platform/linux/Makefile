# Build and install Ekushe for Linux from source: the IBus keyboard engine and Ekushe Studio.
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
CFLAGS     ?= -O2
LDFLAGS    ?=

TARGET  = target/release
ENGINE  = $(TARGET)/ibus-engine-ekushe
WINDOW  = $(TARGET)/ekushe-studio-window

all: $(ENGINE) $(WINDOW)

$(TARGET)/libekushe_ffi.a $(TARGET)/ekushe-studio:
	$(CARGO) build --release $(CARGOFLAGS) -p ekushe-ffi -p ekushe-studio \
	  --features "ekushe-ffi/embedded-dict ekushe-ffi/studio"

$(ENGINE): platform/linux/ibus/ekushe-engine.c $(TARGET)/libekushe_ffi.a
	$(CC) $(CFLAGS) -Wall -DEKUSHE_VERSION="\"$(VERSION)\"" $< \
	  $$(pkg-config --cflags ibus-1.0) $(TARGET)/libekushe_ffi.a $$(pkg-config --libs ibus-1.0) \
	  -lpthread -ldl -lm $(LDFLAGS) -o $@

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

clean:
	$(CARGO) clean
	rm -f $(ENGINE)

.PHONY: all install clean
