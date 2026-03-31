#!/bin/bash

set -e

echo "Generating MEDIUM dataset..."

mkdir -p /tmp/griffon_medium
mkdir -p ~/.cache/griffon_medium

for i in {1..500}
do
    head -c 10K </dev/urandom > /tmp/griffon_medium/file_$i.tmp
done

echo "Medium dataset ready."