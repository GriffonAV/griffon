#!/bin/sh
set -e

systemctl stop griffonav-daemon || true
systemctl disable griffonav-daemon || true