<p align="center">
  <img width="125" height="125" src="logo.png" alt="Griffon logo">
</p>

<h1 align="center">Griffon</h1>

<p align="center">
  A modular, Rust-based security toolbox and antivirus for Linux.
</p>

<p align="center">
  <a href="https://griffon-av.vercel.app/">Website &amp; docs</a> ·
  <a href="https://github.com/GriffonAV/griffon/releases/latest">Download</a> ·
  <a href="https://discord.gg/2mP5bBC7HZ">Discord</a>
</p>

<p align="center">
  <img src="https://img.shields.io/github/v/release/GriffonAV/griffon?style=flat-square" alt="GitHub Release">
  <img src="https://img.shields.io/github/downloads/GriffonAV/griffon/total?style=flat-square" alt="GitHub Downloads">
  <img src="https://img.shields.io/github/stars/GriffonAV/griffon?style=flat-square" alt="GitHub Stars">
  <a href="https://discord.gg/2mP5bBC7HZ"><img src="https://img.shields.io/badge/discord-join-5865F2?style=flat-square&logo=discord&logoColor=white" alt="Discord"></a>
  <img src="https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/tauri-v2-blue?style=flat-square&logo=tauri" alt="Tauri v2">
</p>

---

## What is Griffon?

Griffon is a fast, secure, and easy-to-use antivirus and security toolbox for Linux. It ships as:

- a **desktop application** (built with [Tauri](https://v2.tauri.app/)),
- a **background daemon** that starts automatically,
- a **command-line interface** (`griffon-cli`),
- a **plugin-based engine**: write your tool as a Rust plugin, describe its UI in a TOML file, and Griffon generates the interface for you.

Built-in plugins include a **YARA-based scanner** (with quarantine and rule updates) and a **system cleaner** (caches, logs, packages, big files, Docker).

> [!WARNING]
> Griffon is an academic project in pre-release. It has not been professionally audited — see [SECURITY.md](SECURITY.md).

## Installation

Download the latest `.deb` or `.rpm` from the [releases page](https://github.com/GriffonAV/griffon/releases/latest), then install it (replace `<version>` with the version you downloaded):

```bash
# Debian / Ubuntu
sudo apt install ./griffon-<version>.deb

# Fedora
sudo dnf install ./griffon-<version>.rpm
```

The install enables and starts the `griffon-daemon` systemd service, and adds your user to the `griffon` group so the GUI and CLI can talk to the daemon without `sudo`.

**Log out and back in** (or run `newgrp griffon`) for the group change to take effect. Then launch **Griffon** from your application menu, or run `griffon-cli` in a terminal.

To uninstall: `sudo apt remove griffon` or `sudo dnf remove griffon`.

## Development

> Installed Griffon and Griffon run from source use **different** config, plugin, and socket locations (`/usr/lib/griffon`, `/etc/griffon`, `/run/griffon` vs. the repository folder). You can have both, but the dev CLI/GUI only talk to the dev daemon.

### Prerequisites

- [Rust](https://rustup.rs/) (stable) and Cargo
- [Node.js](https://nodejs.org/) and npm
- [`just`](https://github.com/casey/just)
- Tauri's system dependencies:

  ```bash
  # Fedora
  sudo dnf install @development-tools pkgconf-pkg-config openssl-devel webkit2gtk4.1-devel javascriptcoregtk4.1-devel

  # Debian / Ubuntu
  sudo apt install build-essential pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
  ```

  For other distributions, see the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/).

- Docker (only for building the `.deb` / `.rpm` packages)

### Build and run from source

From the repository root:

```bash
just setup-gui        # install the GUI's npm dependencies
cargo build           # build the whole workspace (daemon, CLI, GUI, plugins)
just update-plugins   # copy the scanner and cleaner plugins into .config/griffon/
```

Then run each component in its own terminal, **starting with the daemon**:

```bash
just run-daemon   # start the daemon (creates ./griffon.sock)
just run-gui      # start the desktop app in dev mode
just run-cli      # start the CLI
```

Re-run `cargo build && just update-plugins` whenever you change a plugin.

### Common commands

| Command | Description |
| --- | --- |
| `just --list` | List all available shortcuts |
| `just setup-dev-env` | Enable the git pre-commit hook (fmt + clippy) |
| `just lint` / `just lint-fix` | Check / fix formatting and Clippy lints |
| `cargo test` | Run the tests |
| `cargo build --release` | Build the workspace in release mode |
| `just build-gui` | Build the GUI binary (release, no installer bundle) |
| `just build-deb` | Build the `.deb` **and** `.rpm` packages in Docker, output to `dist/` |

CI runs `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`; run them before opening a pull request.

### Project layout

```
griffon/
├── cli/                 # griffon-cli: interactive command-line client
├── daemon/
│   ├── daemon_core/     # griffon-daemon: socket server, dispatcher, notifications
│   ├── daemon_runner/   # griffon-daemon-runner: isolated process that hosts a plugin
│   ├── plugin_manager/  # loads, enables/disables and talks to plugins
│   └── griffon-daemon.service   # systemd unit (packaged install)
├── gui/                 # desktop app (React + Vite frontend, Tauri v2 in src-tauri/)
├── plugin-installer/    # privileged helper used by the GUI to install plugins
├── plugins/
│   ├── griffon_scanner/ # YARA + hash based scanner, quarantine, rule updater
│   ├── griffon_cleaner/ # system cleaner
│   ├── docker_helper/   # Docker helper plugin
│   ├── plugin_template/ # starter template for new plugins
│   └── plugin-guide.md  # plugin development guide
├── shared/
│   ├── ipc_protocol/    # IPC messages between daemon, runner, GUI and CLI
│   ├── logger/          # shared logging
│   └── plugin_interface/ # stable ABI contract implemented by every plugin
├── docs/                # TOML manifest / GUI generation docs
├── bench/               # benchmark VM (Vagrant) and datasets
├── scripts/             # packaging scripts (post-install, pre-remove)
├── nfpm.yaml            # .deb / .rpm package definition
└── justfile             # dev shortcuts
```

### Writing a plugin

Start from [`plugins/plugin_template`](plugins/plugin_template), then read the [plugin guide](plugins/plugin-guide.md) and the [TOML / GUI guide](docs/guiTomlHowTo.md). The full developer documentation is at [griffon-av.vercel.app/docs](https://griffon-av.vercel.app/docs/introduction).

## Community & contributing

- 💬 Join us on [Discord](https://discord.gg/2mP5bBC7HZ) for questions, ideas and help.
- 🐛 Report bugs and request features via [GitHub issues](https://github.com/GriffonAV/griffon/issues).
- 🔒 Report security vulnerabilities privately — see [SECURITY.md](SECURITY.md).
- 🤝 Read [CONTRIBUTING.md](CONTRIBUTING.md) and our [Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

## Authors

<table>
    <tbody>
        <tr>
            <td align="center">
                <a href="https://github.com/Sebabacou">
                    <img src="https://avatars.githubusercontent.com/u/114607576?s=96&v=4" width="100px;" alt="Sebabacou"/><br />
                    <sub><b>Sebabacou</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/Raphael-Mabille">
                    <img src="https://avatars.githubusercontent.com/u/114739950?s=96&v=4" width="100px;" alt="Raphael_m"/><br />
                    <sub><b>Raphael_m</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/ewen1507">
                    <img src="https://avatars.githubusercontent.com/u/114604459?s=96&v=4" width="100px;" alt="ewen1507"/><br />
                    <sub><b>ewen1507</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/DiaboloAB">
                    <img src="https://avatars.githubusercontent.com/u/109909203?s=96&v=4" width="100px;" alt="Alexis Boitel"/><br />
                    <sub><b>Alexis Boitel</b></sub>
                </a>
            </td>
        </tr>
    </tbody>
</table>

## License

Licensed under the [Apache License 2.0](LICENSE).
