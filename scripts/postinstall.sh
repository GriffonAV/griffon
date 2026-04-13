#!/bin/sh
set -e

mkdir -p /usr/lib/griffonav/plugins

systemctl daemon-reload
systemctl enable griffonav-daemon
systemctl start griffonav-daemon || true

echo "GriffonAV installed successfully."

# Refresh icon cache so the icon shows immediately
gtk-update-icon-cache /usr/share/icons/hicolor/ -f -t || true

# Refresh app listing
update-desktop-database /usr/share/applications || true