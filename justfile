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
    sudo target/debug/griffonav-daemon

run-cli:
    sudo target/debug/griffonav-cli

run-gui-sudo:
    sudo target/debug/app

lint:
    cargo fmt -- --check
    cargo clippy -- -D warnings

# lint fix at path input
lint-fix:
    cargo fmt
    cargo clippy --fix --allow-dirty

build-deb:
    sudo docker build -t griffonav-builder -f Dockerfile.build .
    sudo docker run --rm -v $(pwd)/dist:/out griffonav-builder

update-plugins:
    cp target/debug/libgriffon_cleaner.so .config/griffon/
    cp target/debug/libgriffon_cleaner.d .config/griffon/
    cp target/debug/libgriffon_scanner.so .config/griffon/
    cp target/debug/libgriffon_scanner.d .config/griffon/
    cp plugins/griffon_cleaner/griffon_cleaner.toml .config/griffon/
    cp plugins/griffon_scanner/griffon_scanner.toml .config/griffon/