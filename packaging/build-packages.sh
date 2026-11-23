#!/bin/sh
set -e
mkdir -p /out
nfpm pkg --config packaging/nfpm.yaml --packager deb --target /out/griffon-${VERSION}.deb
nfpm pkg --config packaging/nfpm.yaml --packager rpm --target /out/griffon-${VERSION}.rpm

# Plugin ZIPs built in the workspace stage, renamed with the release version
for zip in /app/plugin-zips/*.zip; do
    name=$(basename "$zip" .zip)
    cp "$zip" "/out/${name}-${VERSION}.zip"
done
