# justfile

setup-dev-env:
    git config core.hooksPath .githooks

# gui

setup-gui:
    cd gui && npm install


run-gui:
    cd gui && npx tauri dev

build-gui:
    cd gui && npx tauri build --no-bundle

# cli

run-daemon:
    sudo target/debug/daemon_core

run-cli:
    sudo target/debug/cli

#old

lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings
