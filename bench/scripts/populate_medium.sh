#!/bin/bash

echo "Generating MEDIUM dataset..."

mkdir -p ~/.cache/griffon_medium

for i in {1..500}
do
    head -c 10K </dev/urandom > /tmp/griffon_medium/file_$i.tmp
done

echo "Medium dataset ready."