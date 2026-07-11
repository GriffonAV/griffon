# justfile

setup-gui:
    cd gui && npm install

run-gui:
    cd gui && npx tauri dev

run-daemon:
    target/debug/griffon-daemon

run-cli:
    target/debug/griffon-cli

## build

build-gui:
    cd gui && npx tauri build --no-bundle

build-workspace:
    cargo build --release --workspace --exclude griffon-gui

## linting and formatting

lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

lint-fix:
    cargo fmt
    cargo clippy --fix --allow-dirty

## packaging

build-deb:
    docker build -t griffon-builder -f Dockerfile.build .
    docker run --rm -v $(pwd)/dist:/out griffon-builder

## dev utils

update-plugins:
    cp target/debug/libgriffon_cleaner.so .config/griffon/
    cp target/debug/libgriffon_scanner.so .config/griffon/
    cp plugins/griffon_cleaner/libgriffon_cleaner.toml .config/griffon/
    cp plugins/griffon_scanner/libgriffon_scanner.toml .config/griffon/

setup-dev-env:
    git config core.hooksPath .githooks

## testing

vagrant-setup:
    vagrant up
    vagrant ssh