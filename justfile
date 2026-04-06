# justfile

setup-dev-env:
    git config core.hooksPath .githooks

setup-gui:
    cd gui && npm install


run-gui:
    cd gui && npx tauri dev

build-gui:
    cd gui && npx tauri build --no-bundle

build-workspace:
    cargo build --release --workspace --exclude griffonav-gui

run-daemon:
    sudo target/debug/daemon_core

run-cli:
    sudo target/debug/cli

run-gui-sudo:
    sudo target/debug/app

lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

build-deb:
    sudo docker build -t griffonav-builder -f Dockerfile.build .
    sudo docker run --rm -v $(pwd)/dist:/out griffonav-builder