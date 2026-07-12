#!/bin/sh
set -e

if command -v systemctl >/dev/null 2>&1; then
    systemctl stop griffon-daemon || true
    systemctl disable griffon-daemon || true
    systemctl daemon-reload || true
fi