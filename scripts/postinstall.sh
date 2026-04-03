#!/bin/sh
set -e

mkdir -p /usr/lib/griffonav/plugins

systemctl daemon-reload
systemctl enable griffonav-daemon
systemctl start griffonav-daemon || true

echo "GriffonAV installed successfully."