#!/bin/sh
set -e
mkdir -p /out
nfpm pkg --config packaging/nfpm.yaml --packager deb --target /out/griffon-${VERSION}.deb
nfpm pkg --config packaging/nfpm.yaml --packager rpm --target /out/griffon-${VERSION}.rpm