#!/bin/sh
# Turn a release tag into a Linux package version, the same way the downloadable .deb/.rpm
# (nfpm) number it, so COPR/PPA updates always count as newer than a downloaded package:
#   v0.1.0-beta.13 → 0.1.0~beta.13   (the ~ sorts before 0.1.0, so the final 0.1.0 is an upgrade)
#   v0.1.0         → 0.1.0
echo "$1" | sed -e 's/^v//' -e 's/-/~/'
