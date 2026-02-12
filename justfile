# justfile

build-core:
    cargo build -p griffon_core

build-cli:
    cargo build -p cli

build-daemon:
    cargo build -p daemon

build-runner:
    cargo build -p runner

setup-gui:
    cd gui && npm install

build-gui:
    cd gui && npm run build

dev-gui:
    cd gui && npm run dev

tauri-dev: build-runner
    cd gui && npx tauri dev

lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

test:
    cargo test --all

clean:
    cargo clean
    cd gui && npm run clean
