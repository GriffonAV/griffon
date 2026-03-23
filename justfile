# justfile

setup-dev-env:
    git config core.hooksPath .githooks

# gui

setup-gui:
    cd gui && npm install


run-gui:
    cd gui && npx tauri dev

#old

build-core:
    cargo build -p griffon_core

build-cli:
    cargo build -p cli

build-daemon:
    cargo build -p daemon


lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

test:
    cargo test --all

clean:
    cargo clean
    cd gui && npm run clean
