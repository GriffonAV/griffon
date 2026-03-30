#!/bin/bash

echo "Generating LIGHT dataset..."

mkdir -p ~/.cache/griffon_light

for i in {1..100}
do
    head -c 1K </dev/urandom > /tmp/griffon_light/file_$i.tmp
done

echo "Light dataset ready."