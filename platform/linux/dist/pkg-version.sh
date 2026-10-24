#!/bin/sh
# Turn a release tag into a Linux package version that sorts correctly:
#   v0.1.0-beta.13 → 0.1.0~beta13   (the ~ sorts before 0.1.0, so the final 0.1.0 is an upgrade)
#   v0.1.0         → 0.1.0
echo "$1" | sed -e 's/^v//' -e 's/-\([a-z]*\)\.\{0,1\}\([0-9]*\)$/~\1\2/'
