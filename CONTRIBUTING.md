# Contributing to Griffon

Thanks for helping improve Griffon. Contributions to code, documentation, testing, and bug reports are welcome. Griffon is an experimental, student-built security platform for Linux; please read [SECURITY.md](SECURITY.md) before using it in a sensitive environment.

## Before you start

- For a bug, check existing issues and open a new one with your Linux distribution and version, Griffon version or commit, expected and actual behavior, reproduction steps, and relevant logs. Remove tokens, personal files, and other sensitive data from logs.
- For a feature or substantial architecture change, open an issue first to discuss its scope. Small fixes and documentation improvements can go straight to a pull request.
- **Do not post vulnerabilities in public issues or pull requests.** Follow [SECURITY.md](SECURITY.md) and contact `contact.griffon@proton.me` privately.
- Only test scans and cleanup on files and systems you own or are authorized to use. Use disposable test data for destructive operations. Never commit malware samples, credentials, private data, or generated build artifacts.

## Get the project running

You need a Linux development machine, a recent stable Rust toolchain with `rustfmt` and `clippy`, Node.js and npm for the desktop UI, and [`just`](https://github.com/casey/just). Tauri also needs Linux system libraries; consult the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your distribution. On Fedora, the [root README](README.md#running-from-source-development) lists the packages used by this project. Docker is only needed for the packaged build.

1. Fork the repository, clone your fork, and work on a branch created from `main`:

   ```bash
   git clone https://github.com/YOUR_USERNAME/griffon.git
   cd griffon
   git switch -c fix/short-description
   ```

2. Install dependencies and build from the repository root:

   ```bash
   just setup-gui
   cargo build
   just update-plugins
   ```

   `just update-plugins` copies the scanner and cleaner libraries and manifests into `.config/griffon/`. If that directory does not exist, create it first with `mkdir -p .config/griffon`. Run this command again after rebuilding either plugin.

3. In separate terminals, run the components you need:

   ```bash
   just run-daemon
   just run-gui
   just run-cli
   ```

   Build before using `just run-daemon` or `just run-cli`: these recipes launch binaries from `target/debug/`. The CLI and GUI expect the daemon when exercising connected features. The [README](README.md) has an example CLI session.

   Optional: `just setup-dev-env` enables the repository's local pre-commit hook. The CI checks remain the authoritative gate for pull requests.

## Find the right place to change

| Area | Directory | What belongs there |
| --- | --- | --- |
| Background service | `daemon/` | Daemon entry point, core behavior, plugin loading |
| Desktop app | `gui/` | React/Vite frontend and Tauri integration |
| CLI | `cli/` | Terminal commands and daemon interactions |
| Shared contracts | `shared/` | IPC messages, plugin ABI, logging |
| Built-in plugins | `plugins/` | Scanner, cleaner, Docker helper, plugin template |
| Installer and packaging | `plugin-installer/`, `scripts/`, `nfpm.yaml` | Plugin installation and distribution packaging |
| Docs | `docs/`, `README.md`, `plugins/plugin-guide.md` | User and developer guidance |

For a new plugin, start with `plugins/plugin_template/` and the [plugin guide](plugins/plugin-guide.md). Plugins are Rust `cdylib` libraries using the shared `plugin_interface` and `abi_stable` types at the ABI boundary. Add a manifest, document each exposed command, and verify that the daemon can load and call the plugin. Changes to `shared/plugin_interface` or `shared/ipc_protocol` can affect plugins, daemon, CLI, and GUI; explain compatibility implications in the pull request.

## Make and verify a change

- Keep each pull request focused. Explain behavior changes in the relevant user or developer documentation.
- Add or update tests for changed behavior, especially IPC, plugin loading, detection, quarantine, and cleanup. For cleanup or quarantine, test that unselected files remain intact. Use harmless fixtures rather than live malware.
- Handle errors explicitly. A plugin runs inside the daemon's process, so review paths, permissions, untrusted input, and potentially destructive actions carefully. Avoid panics and unintended file deletion at plugin boundaries.
- Follow the existing style in nearby files and [GUI conventions](gui/STANDARDS.md) when editing the frontend.

Run the checks relevant to your changes before opening a pull request:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The first two commands match the [Rust CI workflow](.github/workflows/ci.yml). For GUI changes, also run:

```bash
cd gui
npm run build
npm run lint
```

Exercise the affected feature manually when automation cannot cover it, and include the steps and results in your pull request. If a check cannot run because of your environment or an existing issue, say so explicitly.

## Open a pull request

1. Push your branch and open a pull request against `main` in the repository you intend to contribute to. If you are contributing upstream from a fork, target `GriffonAV/griffon`.
2. Describe the problem, your solution, related issue, and any user-visible or compatibility changes. Add screenshots for UI changes.
3. List the commands you ran and any manual verification. Highlight security implications, file operations, migrations, or changes to the plugin ABI or IPC format.
4. Respond to review comments and keep the branch current when needed. CI runs Rust formatting and Clippy on pull requests to `main` and `develop`; contributors should still run the other relevant checks locally.

All contributions are provided under the project's [Apache 2.0 license](LICENSE). Thank you for contributing.
