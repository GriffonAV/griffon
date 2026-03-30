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
    libssl-dev

echo "Installing Rust..."

curl https://sh.rustup.rs -sSf | sh -s -- -y

source /home/vagrant/.cargo/env

echo "Creating benchmark directories..."

mkdir -p /tmp/griffon_light
mkdir -p /tmp/griffon_medium
mkdir -p /tmp/griffon_stress

echo "Bootstrap complete."