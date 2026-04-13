#!/bin/sh
set -e
mkdir -p /out
nfpm pkg --packager deb --target /out/griffonav.deb
nfpm pkg --packager rpm --target /out/griffonav.rpm