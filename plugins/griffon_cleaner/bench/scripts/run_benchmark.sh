#!/bin/bash
set -e

SCENARIO="$1"

if [ -z "$SCENARIO" ]; then
    echo "Usage: ./plugins/griffon_cleaner/bench/scripts/run_benchmark.sh <profile>"
    echo "  shipped profiles: light|medium|stress"
    echo "  bench configs:    stress_full|test_browser_only|test_dev_only|test_invalid|test_multi"
    exit 1
fi

source /home/vagrant/.cargo/env

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
CLEANER_DIR="/home/vagrant/Griffon/plugins/griffon_cleaner"
RESULT_DIR="${CLEANER_DIR}/bench/results/${SCENARIO}_${TIMESTAMP}"

# Shipped profiles live in profiles/, bench-only configs in bench/configs/.
if [ -f "${CLEANER_DIR}/profiles/${SCENARIO}.json" ]; then
    CONFIG_PATH="${CLEANER_DIR}/profiles/${SCENARIO}.json"
elif [ -f "${CLEANER_DIR}/bench/configs/${SCENARIO}.json" ]; then
    CONFIG_PATH="${CLEANER_DIR}/bench/configs/${SCENARIO}.json"
else
    echo "Unknown profile: ${SCENARIO}"
    exit 1
fi

mkdir -p "$RESULT_DIR"

cd /home/vagrant/Griffon

cargo build --release -p griffon_cleaner

sudo ./target/release/griffon_cleaner \
  --config "$CONFIG_PATH" \
  --output "$RESULT_DIR/report.json" \
  > "$RESULT_DIR/stdout.log" \
  2> "$RESULT_DIR/stderr.log"

echo "Benchmark complete: $RESULT_DIR"