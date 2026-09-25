# Contributing to Griffon

Thanks for helping improve Griffon. Contributions to code, documentation, testing, and bug reports are welcome. Griffon is an experimental, student-built security platform for Linux; please read [SECURITY.md](SECURITY.md) before using it in a sensitive environment.

## Before you start

- Check existing issues before reporting a problem or proposing a change. See [Issues](#issues) for what to include.
- For a feature or substantial architecture change, open an issue first to discuss its scope. Small fixes and documentation improvements can go straight to a pull request.
- **Do not post vulnerabilities in public issues or pull requests.** Follow [SECURITY.md](SECURITY.md) and contact `contact.griffon@proton.me` privately.
- Only test scans and cleanup on files and systems you own or are authorized to use. Use disposable test data for destructive operations. Never commit malware samples, credentials, private data, or generated build artifacts.

## Issues

Search [open and closed issues](https://github.com/GriffonAV/griffon/issues?q=is%3Aissue) before opening a new one. If the same problem has already been reported, add any new reproduction details to that discussion. Keep each issue focused on one problem or proposal and use a descriptive title.

Use the appropriate template from the [new issue page](https://github.com/GriffonAV/griffon/issues/new/choose): **Bug report**, **Feature request**, or **Task**.

### Report a bug

Use **Bug report** and include:

- The affected component: daemon, CLI, GUI, plugin, or installer.
- Your Linux distribution and version, and Griffon version or commit.
- Steps to reproduce the problem, including the exact command or UI action and a minimal, harmless example when possible.
- What you expected to happen and what actually happened.
- Relevant error messages or logs, and screenshots when they help explain a UI problem.

For build problems, also include the output of `rustc --version` and `cargo --version`. Include `node --version` and `npm --version` if the GUI build is affected. Paste commands and logs in fenced code blocks so others can copy and search them. The bug template contains generic device fields; provide the Linux environment details above and mark unrelated fields as not applicable.

Remove tokens, credentials, personal file contents, and other sensitive data from logs and screenshots. Never attach malware samples or private files. Report suspected vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

### Suggest a feature

Use **Feature request**. Describe the problem you want to solve, who would benefit, and the behavior you propose. Mention any alternatives or workarounds you have considered. For substantial changes, discuss the scope with maintainers before starting implementation.

### Describe a task

Use **Task** for a concrete piece of project work, such as a documentation update or an agreed implementation step. Explain what needs to change, the affected area, and how to tell when the task is complete. Link any related bug report or feature discussion instead of repeating it.

### Work on an existing issue

Read the discussion and check for linked pull requests before starting. Leave a short comment explaining your intended approach to help avoid duplicated work. Link the issue in your pull request; use `Fixes #123` when the pull request fully resolves it, or `Related to #123` when it only addresses part of the work.

## Get the project running

You need a Linux development machine, the latest stable Rust toolchain with `rustfmt` and `clippy`, Node.js and npm for the desktop UI, and [`just`](https://github.com/casey/just). Tauri also needs Linux system libraries; consult the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your distribution. On Fedora, the [root README](README.md#running-from-source-development) lists the packages used by this project. Docker is only needed for the packaged build.

### Rust version

For local development, use the **latest stable Rust**, matching the [Rust CI workflow](.github/workflows/ci.yml). CI follows the `stable` channel rather than a fixed version, and the repository does not currently contain a `rust-toolchain.toml` or `rust-toolchain` file.

The packaged build uses **Rust 1.94** (`rust:1.94-slim`) for its non-GUI workspace stage in [Dockerfile.build](Dockerfile.build). The GUI stage installs Rust through rustup without pinning a version. Rust 1.94 is therefore a packaging configuration, not a declared minimum version for the whole project.

The workspace and most crates use **Rust edition 2024**, which requires [Rust 1.85.0 or newer](https://doc.rust-lang.org/edition-guide/rust-2024/index.html). Dependencies may require a newer compiler, and Griffon does not currently declare a workspace-wide minimum supported Rust version (MSRV). The `rust-version = "1.77.2"` entry in [the GUI manifest](gui/src-tauri/Cargo.toml) applies to that package's declaration; it does not establish compatibility for the full workspace.

### Setup

1. Fork the repository, clone your fork, and work on a branch created from `main`:

   ```bash
   git clone https://github.com/YOUR_USERNAME/griffon.git
   cd griffon
   git switch -c fix/short-description
   ```

2. With [rustup](https://rustup.rs/) installed, configure the stable toolchain for this checkout:

   ```bash
   rustup update stable
   rustup override set stable
   rustup component add rustfmt clippy --toolchain stable
   rustc --version
   cargo --version
   ```

   The directory override makes plain `cargo` commands and the Rust commands launched by `just` use stable in this checkout. When reporting a build problem, include the version output above.

3. Install dependencies and build from the repository root:

   ```bash
   just setup-gui
   cargo build
   mkdir -p .config/griffon
   just update-plugins
   ```

   `just update-plugins` copies the scanner and cleaner libraries and manifests into `.config/griffon/`. Run this command again after rebuilding either plugin.

4. In separate terminals, run the components you need:

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
