#!/bin/sh
set -e
mkdir -p /out
nfpm pkg --packager deb --target /out/griffon-${VERSION}.deb
nfpm pkg --packager rpm --target /out/griffon-${VERSION}.rpm