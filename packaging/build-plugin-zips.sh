#!/bin/sh
# Builds one installable ZIP per plugin: the shared library and its TOML
# manifest at the archive root, the layout expected by griffon-plugin-installer.
#
# Usage: build-plugin-zips.sh <release_dir> <repo_dir> <out_dir>
#   release_dir: directory containing the compiled lib<plugin>.so files
#   repo_dir:    repository root (plugin manifests are read from plugins/)
#   out_dir:     where lib<plugin>.zip files are written
set -eu

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <release_dir> <repo_dir> <out_dir>" >&2
    exit 2
fi

release_dir=$1
repo_dir=$2
out_dir=$3

PLUGINS="griffon_scanner griffon_cleaner griffon_nmap docker_helper"

mkdir -p "$out_dir"

for plugin in $PLUGINS; do
    so="$release_dir/lib$plugin.so"
    toml="$repo_dir/plugins/$plugin/lib$plugin.toml"
    zip_path="$out_dir/lib$plugin.zip"

    for file in "$so" "$toml"; do
        if [ ! -f "$file" ]; then
            echo "Missing file for plugin $plugin: $file" >&2
            exit 1
        fi
    done

    rm -f "$zip_path"
    # -j: store files at the archive root; -X: no extra file attributes
    zip -j -X -q "$zip_path" "$so" "$toml"
    echo "Built $zip_path"
done
