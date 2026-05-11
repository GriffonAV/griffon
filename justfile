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