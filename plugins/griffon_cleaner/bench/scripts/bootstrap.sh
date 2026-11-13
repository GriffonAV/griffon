#!/bin/bash

set -e

echo "Updating system..."

apt-get update

apt-get install -y \
    build-essential \
    curl \
    git \
    jq \
    htop \
    pkg-config \
    libssl-dev \
    libglib2.0-dev \
    libgtk-3-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    libwebkit2gtk-4.1-dev

echo "Installing Rust for vagrant user..."

sudo -u vagrant env HOME=/home/vagrant bash -c '
curl https://sh.rustup.rs -sSf | sh -s -- -y
'

echo "Loading cargo env..."

sudo -u vagrant env HOME=/home/vagrant bash -c '
source ~/.cargo/env
rustc --version
cargo --version
'

echo "Creating benchmark directories..."

mkdir -p /tmp/griffon_light
mkdir -p /tmp/griffon_medium
mkdir -p /tmp/griffon_stress

chown -R vagrant:vagrant /tmp/griffon_light
chown -R vagrant:vagrant /tmp/griffon_medium
chown -R vagrant:vagrant /tmp/griffon_stress

echo "Bootstrap complete."