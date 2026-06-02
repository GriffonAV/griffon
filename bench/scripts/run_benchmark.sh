#!/bin/bash
set -e

SCENARIO="$1"

if [ -z "$SCENARIO" ]; then
    echo "Usage: ./bench/scripts/run_benchmark.sh light|medium|stress"
    exit 1
fi

source /home/vagrant/.cargo/env

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULT_DIR="/home/vagrant/Griffon/bench/results/${SCENARIO}_${TIMESTAMP}"
CONFIG_PATH="/home/vagrant/Griffon/bench/configs/${SCENARIO}.json"

mkdir -p "$RESULT_DIR"

cd /home/vagrant/Griffon

cargo build --release -p griffon_cleaner

sudo ./target/release/griffon_cleaner \
  --config "$CONFIG_PATH" \
  --output "$RESULT_DIR/report.json" \
  > "$RESULT_DIR/stdout.log" \
  2> "$RESULT_DIR/stderr.log"

echo "Benchmark complete: $RESULT_DIR"