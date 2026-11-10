#!/usr/bin/env bash
# Run from any directory. All process/network effects in unit tests are injected.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p target/plugin-coverage
cargo llvm-cov --locked -p griffon_cleaner -p griffon_nmap --all-targets \
  --ignore-filename-regex '(^|/)([^/]*tests)\.rs$' \
  --json --output-path target/plugin-coverage/coverage.json
python3 scripts/check-plugin-coverage.py target/plugin-coverage/coverage.json
cargo llvm-cov report --html \
  --ignore-filename-regex '(^|/)([^/]*tests)\.rs$' \
  --output-dir target/plugin-coverage
