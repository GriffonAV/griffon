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
    cargo build --release --workspace --exclude griffon-gui

run-daemon:
    sudo target/debug/griffon-daemon

run-cli:
    sudo target/debug/griffon-cli

run-gui-sudo:
    sudo target/debug/griffon-gui

lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

# lint fix at path input
lint-fix:
    cargo fmt
    cargo clippy --fix --allow-dirty

build-deb:
    sudo docker build -t griffon-builder -f Dockerfile.build .
    sudo docker run --rm -v $(pwd)/dist:/out griffon-builder

update-plugins:
    cp target/debug/libgriffon_cleaner.so .config/griffon/
    cp target/debug/libgriffon_cleaner.d .config/griffon/
    cp target/debug/libgriffon_scanner.so .config/griffon/
    cp target/debug/libgriffon_scanner.d .config/griffon/
    cp plugins/griffon_cleaner/libgriffon_cleaner.toml .config/griffon/
    cp plugins/griffon_scanner/griffon_scanner.toml .config/griffon/
