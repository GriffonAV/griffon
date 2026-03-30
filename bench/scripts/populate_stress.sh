#!/bin/bash

echo "Generating STRESS dataset..."

mkdir -p ~/.cache/griffon_stress

for i in {1..2000}
do
    head -c 100K </dev/urandom > /tmp/griffon_stress/file_$i.tmp
done

echo "Stress dataset ready."