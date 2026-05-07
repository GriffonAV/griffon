#!/bin/sh
set -e

mkdir -p /usr/lib/griffonav/plugins

if command -v systemctl >/dev/null 2>&1; then
    systemctl daemon-reload || true
    systemctl enable griffonav-daemon || true
    systemctl start griffonav-daemon || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache /usr/share/icons/hicolor -f -t || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

echo "GriffonAV installed successfully."