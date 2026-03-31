#!/bin/bash

echo "Cleaning datasets..."

rm -rf /tmp/griffon_light
rm -rf /tmp/griffon_medium
rm -rf /tmp/griffon_stress

rm -rf ~/.cache/griffon_light
rm -rf ~/.cache/griffon_medium
rm -rf ~/.cache/griffon_stress

echo "Cleanup complete."