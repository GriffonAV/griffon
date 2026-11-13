# Everything is compiled on Ubuntu 22.04 (glibc 2.35): a binary only runs on
# systems with a glibc at least as new as the one it was built against, so
# building on the oldest supported distro makes the packages work on
# Ubuntu 22.04+, Debian 12+ and Fedora 36+.
# Rust comes from rustup, so the OS age doesn't limit the Rust version.

# --- STAGE 0: base (Ubuntu 22.04 + Rust) ---
FROM ubuntu:22.04 AS rust-base
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Rust version comes from rust-toolchain.toml (same as local dev and CI)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain none
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app
COPY rust-toolchain.toml .
RUN rustup toolchain install


# --- STAGE 1: Build du GUI (Vite + Tauri Binary) ---
FROM rust-base AS gui-builder
# dep for tauri
RUN apt-get update && apt-get install -y --no-install-recommends \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Node.js (same major as the CI frontend job)
ARG NODE_MAJOR=24
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_MAJOR}.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

COPY . .
WORKDIR /app/gui

# Cache NPM dependencies
RUN --mount=type=cache,target=/root/.npm \
    npm ci

# Cache Cargo registry, git, and the target folder (isolated with id=gui-target).
# --no-bundle: only the binary is needed, nfpm does the packaging.
# We use 'find' to only copy the compiled binaries, ignoring the messy deps/ folder.
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,id=gui-target-jammy,target=/app/target \
    npx tauri build --no-bundle && \
    mkdir -p /app/gui-out && \
    find /app/target/release -maxdepth 1 -type f -exec cp {} /app/gui-out/ \;


# --- STAGE 2: Build du workspace Rust (daemon + cli) ---
FROM rust-base AS workspace-builder

COPY . .

# Cache Cargo (isolated with id=ws-target).
# We use 'find' to extract only the daemon binary and plugin .so files.
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,id=ws-target-jammy,target=/app/target \
    cargo build --release --workspace --exclude griffon-gui --locked && \
    mkdir -p /app/ws-out && \
    find /app/target/release -maxdepth 1 -type f -exec cp {} /app/ws-out/ \;


# --- STAGE 3: packaging avec nfpm ---
FROM goreleaser/nfpm:latest AS packager
WORKDIR /app

# Copie des binaires compilés depuis les dossiers NON-CACHÉS
COPY --from=gui-builder /app/gui-out/ /app/target/release/
COPY --from=workspace-builder /app/ws-out/ /app/target/release/

# nfpm.yaml, install scripts, .desktop and systemd unit
COPY packaging/ ./packaging/
COPY assets/logo.png ./assets/logo.png

# copy plugin config files
COPY ./plugins/griffon_cleaner/libgriffon_cleaner.toml ./plugins/griffon_cleaner/libgriffon_cleaner.toml
COPY ./plugins/griffon_scanner/libgriffon_scanner.toml ./plugins/griffon_scanner/libgriffon_scanner.toml

COPY ./daemon/config_griffon_daemon.default.json ./daemon/config_griffon_daemon.default.json
COPY ./plugins/griffon_cleaner/profiles/light.json ./plugins/griffon_cleaner/profiles/light.json
COPY ./plugins/griffon_cleaner/profiles/medium.json ./plugins/griffon_cleaner/profiles/medium.json
COPY ./plugins/griffon_cleaner/profiles/stress.json ./plugins/griffon_cleaner/profiles/stress.json

ENTRYPOINT ["sh", "packaging/build-packages.sh"]