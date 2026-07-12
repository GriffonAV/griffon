# Griffon — Modular Security Platform for Linux (Rust)
> Griffon is a modular, Rust-based security and toolbox platform for Linux users! Just write your security tool in a Rust plugin, define a TOML config, and we automatically integrate and generate the UI into our application.

---> [Installation and documentation](https://griffon-av.vercel.app/) <---


<p align="center">
  <img width="125" height="125" src="https://griffon-av.vercel.app/img/logo.png" alt='Griffon logo'>
</p>

![Open Source Love](https://img.shields.io/badge/Open%20Source-%E2%9D%A4-red?style=flat-square)
![GitHub Stars](https://img.shields.io/github/stars/GriffonAV/GriffonAV?style=flat-square)
![GitHub Downloads](https://img.shields.io/github/downloads/GriffonAV/GriffonAV/total?style=flat-square)
![GitHub Release](https://img.shields.io/github/v/release/GriffonAV/GriffonAV?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust)
![Tauri](https://img.shields.io/badge/tauri-v2-blue?style=flat-square&logo=tauri)

## Project purpose

Griffon is a modular antivirus project for Linux.

The goal of the project is to provide a fast, secure, and easy-to-use antivirus solution for Linux users.

Griffon provides:

- a desktop application,
- a background daemon,
- a command-line interface,
- a plugin-based antivirus engine.

This README is intended for basic users who only want to install and use Griffon, as well as contributors who want to build the project and understand its layout.

Advanced users and developers can find the full technical documentation here:

[Developer documentation](https://griffon-av.vercel.app/docs/introduction)

## Key features

- **Modular Architecture** - plug-and-play analysis modules and highly customizable engine.
- **Rust Performance** - memory-safe, fast, concurrent scanning.
- **YARA Integration** - pattern-based detection support.
- **Modern Application** - desktop application powered by [Tauri](https://v2.tauri.app/fr/).
- **CLI Support** - terminal usage through `griffon-cli`.
- **Automatic startup** - Griffon starts automatically after installation.

## Installation

Basic users do not need Rust, Cargo, Node.js, npm, or any development environment.

They only need to install the package matching their Linux distribution.

### Debian / Ubuntu

Install Griffon using the `.deb` package:

```bash
sudo apt install ./griffon.deb
```

### Fedora

Install Griffon using the `.rpm` package:

```bash
sudo dnf install ./griffon.rpm
```

Both packages run `scripts/postinstall.sh` on install (registers the systemd service and, on removal, `scripts/preremove.sh` cleans it up). Once installed, Griffon starts automatically and you can launch the desktop application from your system application menu, or use `griffon-cli` from a terminal.

**Note:** installed (packaged) Griffon and Griffon run from source use different config/plugin locations — don't mix the two. If you're just installing the app, the section above is all you need. The rest of this README (from "Running from source" onward) is for contributors building the project locally.

---

## Running from source (development)

You only need **Node.js**, **Rust/Cargo**, and [`just`](https://github.com/casey/just) installed. On Fedora, Tauri also needs a couple of system packages:

```bash
sudo dnf install @development-tools pkgconf-pkg-config
sudo dnf install pkgconf-pkg-config javascriptcoregtk4.1-devel webkit2gtk4.1-devel
```

Then, from the repo root:

```bash
# 1. Install the GUI's JS dependencies
cd gui && npm i && cd ..

# 2. Build the Rust workspace (daemon, cli, shared crates, plugins)
cargo build

# 3. Move each plugin's .toml + .so into the local config folder the daemon reads from
just update-plugins
```

You can now run each component in its own terminal:

```bash
just run-daemon   # starts the daemon
just run-gui      # starts the desktop app
just run-cli      # starts the CLI, connects to the running daemon
```

### CLI example

```
$ just run-cli
target/debug/griffon-cli
[CLI-NETWORK](DEBUG) Client try connected
[CLI-NETWORK](INFO) Client connected
[CLI-NETWORK](DEBUG) Reader thread started
help
Griffon CLI
Usage:
  griffon-cli
Commands:
  help
      Show this help message
  refresh
      Refresh and display the plugin list from the daemon
  switch_status <plugin_uuid>
      Enable or disable a plugin depending on its current status
  call <plugin_uuid> <fn_name> <arg1|arg2|...>
      Call a plugin function with optional arguments
      Example:
        call 550e8400-e29b-41d4-a716-446655440000 scan /tmp
        call 550e8400-e29b-41d4-a716-446655440000 clean cache|true
  switch_notification <plugin_uuid>
      Enable or disable notifications for a plugin depending on its current notification status
  exit | quit
      Exit the CLI
```

## Folder / module structure

```
griffon/
├── cli/          # griffon-cli: command-line interface (Rust)
├── daemon/       # Background daemon
│   ├── daemon_core/     # Core daemon logic
│   ├── daemon_runner/   # Daemon process entry point / runner
│   ├── plugin_manager/  # Loads, enables/disables, and talks to plugins
│   └── griffon-daemon.service   # systemd unit file (used by the packaged install)
├── gui/          # Desktop application (Tauri v2 + Vite frontend)
├── plugins/      # Built-in extensions
│   ├── griffon_scanner/   # Scanner extension (YARA-based)
│   ├── griffon_cleaner/   # Cleaner extension
│   ├── docker_helper/     # Docker-related helper extension
│   ├── plugin_template/   # Starter template for new extensions
│   └── plugin-guide.md    # Extension development guide
├── shared/       # Crates shared between the daemon and plugins
│   ├── ipc_protocol/     # IPC message types between daemon, GUI, and CLI
│   ├── logger/           # Shared logging utilities
│   └── plugin_interface/ # Stable ABI contract used by all extensions
├── docs/         # Manifest/TOML developer documentation
├── scripts/      # Packaging scripts (used to build the .deb / .rpm)
├── justfile      # Dev command shortcuts (run-daemon, run-gui, run-cli, update-plugins, ...)
└── Cargo.toml    # Workspace root
```

## Key commands

```bash
just update-plugins   # copy plugin .toml + .so files into the dev config folder
just run-daemon       # run the daemon
just run-gui          # run the desktop app
just run-cli          # run the CLI

cargo build            # build the workspace (debug)
cargo build --release  # build the workspace (release)
cargo test              # run tests
```

Run `just --list` to see all available shortcuts.

## Environment variables

Griffon doesn't currently require any environment variables to run. Configuration is handled through config files instead, and those files live in different locations depending on how you're running Griffon:

- **From source:** written to the local dev config folder by `just update-plugins`.
- **Installed via `.deb`/`.rpm`:** managed by the packaged daemon via `daemon/config_griffon_daemon.json` and the systemd service.

## Technical prerequisites

### For basic usage (installed package)

- A supported Linux distribution,
- Administrator privileges,
- The correct package format for your system (`.deb` for Debian/Ubuntu, `.rpm` for Fedora).

No development tools are required for basic usage.

### For running from source

- **Rust** and **Cargo**
- **Node.js** and **npm** (for the `gui/` frontend)
- [`just`](https://github.com/casey/just)
- On Fedora, Tauri's system dependencies:
    
    ```bash
    sudo dnf install @development-tools pkgconf-pkg-config javascriptcoregtk4.1-devel webkit2gtk4.1-devel
    ```
    
    (see the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/) for other distributions)

## Advanced usage and development

This README covers the essentials for installing and running Griffon from source. For plugin/extension development, internal architecture, and IPC protocol details, see:

- [Developer documentation](https://griffon-av.vercel.app/docs/introduction)
- CONTRIBUTING.md
- 
## Documentation

- **User docs:** [griffon-av.vercel.app](https://griffon-av.vercel.app/)
- **Developer docs:** [griffon-av.vercel.app/docs/intro](https://griffon-av.vercel.app/docs/introduction)
- **Internal wiki:** docs/


## Authors

<table>
    <tbody>
        <tr>
            <td align="center">
                <a href="https://github.com/Raphael-Mabille">
                    <img src="https://avatars.githubusercontent.com/u/114607576?s=96&v=4" width="100px;" alt="Sebabacou"/><br />
                    <sub><b>Sebabacou</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/orgs/GriffonAV/people/Sebabacou">
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

You may freely use, modify, and distribute this project under the terms of this license.

## Epitech Project

This project is a final-year study project at Epitech and is not commercial in any way. Here is the link to the [repository being evaluated](https://github.com/GriffonAV/GriffonAV).
