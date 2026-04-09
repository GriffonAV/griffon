# Griffon Cleaner Benchmark Environment

This directory provides a **reproducible benchmarking environment** for the griffon_cleaner module.

It allows you to:

- run the cleaner inside a clean Ubuntu VM
- generate deterministic datasets
- execute benchmarks with multiple configurations
- compare results between runs
- evaluate performance, safety behaviour, and disk cleanup efficiency

This environment is designed to support development and validation of Griffon Cleaner.
***

## Overview

The benchmark environment includes:
```
bench/
├── Vagrantfile
├── configs/
│   ├── light.json
│   ├── medium.json
│   └── stress.json
├── scripts/
│   ├── populate_light.sh
│   ├── populate_medium.sh
│   ├── populate_stress.sh
│   ├── cleanup_dataset.sh
│   └── run_benchmark.sh
├── results/
└── README.md
```
It provides:

- a reproducible Ubuntu VM
- configurable cleaner profiles
- dataset generators
- automated benchmark scripts
- structured result outputs

***

## Requirements

Install on your host machine:

- VirtualBox
- Vagrant
- Git

Optional:

- RustRover (recommended)
- Rust toolchain (if running outside VM)

## Start the Virtual Machine

From inside the bench/ directory:

```
vagrant up
```

Then connect:

```
vagrant ssh
```

To reset the environment:
```
vagrant destroy -f
vagrant up
```

## Repository Layout Inside the VM

Inside the VM the project is mounted at:

```
/home/vagrant/GriffonAV
```

Example:

```
/home/vagrant/GriffonAV/plugins/griffon_cleaner
```


## Dataset Generation

Datasets simulate realistic cache files and temporary artifacts.

Available generators:

```
populate_light.sh
populate_medium.sh
populate_stress.sh
```

Example:

```
cd ~/GriffonAV
./bench/scripts/populate_medium.sh
```

Datasets are typically generated inside:

```
/tmp/
~/.cache/
```

## Cleaning Dataset Between Runs

Always reset datasets before benchmarking:

```
./bench/scripts/cleanup_dataset.sh
```

Then regenerate:

```
./bench/scripts/populate_medium.sh
```
This guarantees reproducible results.


## Running Griffon Cleaner Manually

From inside the VM:

```
cd ~/GriffonAV
cargo run --release --package griffon_cleaner --bin griffon_cleaner
```

Cleaner outputs:
```
griffon_cleaner_report.json
```

## Running Benchmarks Automatically

Use the benchmark script:

```
./bench/scripts/run_benchmark.sh medium
```

Available profiles:

```
light
medium
stress
```
The script automatically:

1. prepares dataset
2. runs cleaner
3. collects results
4. stores logs

## Benchmark Configurations

Configs are located in:

```
bench/configs/
```

Example profiles:

```
profile	purpose
light	quick validation run
medium	realistic standard scenario
stress	heavy cleanup benchmark
```

Each configuration controls:

```
profile mode
dry_run
enabled cache categories
log retention policies
big file thresholds
scan roots
```

## Running Cleaner With Root Privileges

Some system cache locations require root:

Examples:

```
/var/cache
/var/lib/apt/lists
/var/cache/apt/archives
/var/log/journal
```

Run cleaner with:

```
sudo ./target/release/griffon_cleaner
```

Or:

```
sudo cargo run --release --package griffon_cleaner --bin griffon_cleaner
```

Without root:

permission errors are expected and reported.

## Benchmark Output

Results are stored inside:

```
bench/results/
```

Typical structure:

```
bench/results/medium_timestamp/
├── report.json
├── stdout.log
├── stderr.log
├── before.txt
└── after.txt
```

Generated files include:

```
file	purpose
report.json	cleaner execution report
stdout.log	console output
stderr.log	error logs
before.txt	disk usage before run
after.txt	disk usage after run
```

## Metrics Collected

Cleaner reports:

```
total_files_touched
total_bytes_freed
duration_ms
warnings
permission_denied
```

These allow performance comparison between runs.

## Example Workflow

Recommended workflow:

```
vagrant up
vagrant ssh
cd ~/GriffonAV

./bench/scripts/cleanup_dataset.sh
./bench/scripts/populate_medium.sh

cargo build --release --package griffon_cleaner

sudo ./target/release/griffon_cleaner
```

Archive result:

```
bench/results/manual_medium/
```

## Recommended Benchmark Strategy

Use:

```
light → quick validation
medium → standard evaluation
stress → heavy workload scenario
```
Compare:
```
freed space
execution time
warnings
permission_denied
```
Across runs.

## Minimal workflow:

```
vagrant up
vagrant ssh
cd ~/GriffonAV
./bench/scripts/populate_medium.sh
sudo ./target/release/griffon_cleaner
```
Output:
```
griffon_cleaner_report.json
```
stored for analysis inside:
```
bench/results/
```