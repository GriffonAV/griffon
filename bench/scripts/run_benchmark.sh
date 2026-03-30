#!/bin/bash

set -e

SCENARIO=$1

if [ -z "$SCENARIO" ]; then
    echo "Usage:"
    echo "./run_benchmark.sh light|medium|stress"
    exit 1
fi

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

RESULT_DIR=~/bench/results/${SCENARIO}_${TIMESTAMP}

mkdir -p "$RESULT_DIR"

echo "Running Griffon Cleaner benchmark..."

cargo build --release

time target/release/griffon_cleaner \
    --output "$RESULT_DIR/report.json" \
    > "$RESULT_DIR/stdout.log" \
    2> "$RESULT_DIR/stderr.log"

echo "Benchmark complete."

echo "Results saved in:"
echo "$RESULT_DIR"