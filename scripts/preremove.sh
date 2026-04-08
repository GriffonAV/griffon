#!/bin/sh
set -e

if command -v systemctl >/dev/null 2>&1; then
    systemctl stop griffonav-daemon || true
    systemctl disable griffonav-daemon || true
    systemctl daemon-reload || true
fi