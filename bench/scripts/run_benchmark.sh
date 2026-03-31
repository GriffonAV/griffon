#!/bin/bash

set -e

SCENARIO="$1"

if [ -z "$SCENARIO" ]; then
    echo "Usage: ./scripts/run_benchmark.sh light|medium|stress"
    exit 1
fi

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULT_DIR="/home/vagrant/GriffonAV/bench/results/${SCENARIO}_${TIMESTAMP}"

mkdir -p "$RESULT_DIR"

echo "Loading Rust environment..."
source /home/vagrant/.cargo/env

echo "Building griffon_cleaner..."
cd /home/vagrant/GriffonAV

cargo build --release

echo "Running Griffon Cleaner benchmark..."

sudo time ./target/release/griffon_cleaner \
    --scenario "$SCENARIO" \
    --output "$RESULT_DIR/report.json" \
    > "$RESULT_DIR/stdout.log" \
    2> "$RESULT_DIR/stderr.log"

echo "Benchmark complete."
echo "Results stored in: $RESULT_DIR"