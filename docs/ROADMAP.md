# Griffon Roadmap

This document sets the direction of the project and the order of the work.
Detailed tasks live in GitHub issues; each item below should link to its issue
or PR once one exists.

Last reviewed: 2026-10-15

| Phase | Theme | Status |
| --- | --- | --- |
| 1 | Plugin system, testing policy, build | **In progress** |
| 2 | Daemon scheduler, sandboxing | Planned |
| 3 | More plugins, GUI v2, native GUI | Exploratory |

**Phase 1 goal:** someone can write a plugin **outside this repository**, depend
only on a published SDK crate, and it loads, runs and renders correctly. CI
catches it when that stops being true.

---

## Phase 1: Foundations (priority)

### 1.1 Plugin interface cleanup

Today a plugin is only a plugin in name: the contract between the plugin, the
daemon and the GUI is implicit, and first-party plugins use shortcuts that a
third-party plugin cannot.

#### Problems in the current system

- **The GUI hardcodes plugin-specific components.**
  `gui/src/renderer/GriffonElementRenderer.tsx` has cases such as
  `scanner_table`, `cleaner_candidate_list` and `nmap_scan_form`, backed by
  `Scanner*`, `Cleaner*` and `Nmap*` components in `gui/src/components/griffon/`.
  A third-party plugin can never get the same UI, and changing a plugin means
  changing the GUI.
- **The metadata from `init()` is magic strings.** `init()` returns key/value pairs, and
  the runner looks up `"name"`, `"UUID"` and `"function"`, splitting functions on
  `/` or `,`. A typo gives a plugin with no ID or no functions, with no error.
- **`init()` failures are swallowed.** The runner logs and continues with an
  empty name and a nil UUID, so the daemon is never told the plugin is broken.
- **Every plugin parses calls its own way.** The runner sends
  `"fn:<name> <args joined by spaces>"`. Nmap, scanner and docker_helper split it
  and read JSON. The cleaner accepts both `fn:run` and `run`. The template matches
  exact strings.
- **Errors are not errors.** The runner answers `ok: true` for anything a plugin
  returns. Failures come back as `{"ok":false,...}` (nmap), `"ERR cleaner: ..."`
  (cleaner) or `"ACK ..."` (template).
- **Metadata is written twice and has drifted.**
  - Name, UUID and functions exist both in `init()` and in the TOML manifest.
    The names already disagree: `GriffonScan` vs `Griffon Scanner`, and
    `Template Plugin` vs `Plugin Template`.
  - Every manifest repeats the UUID as both `uuid` and `id`.
  - Manifest versions (`1.0.0`) are unrelated to crate versions (`0.3.0`).
- **The manifest has no schema.** `[plugin]`, `[store]`, `[ui]` and
  `[[interactions]]` are defined only by what the GUI happens to parse.
  `plugins/griffon_scanner/griffon_scanner_test.toml` uses UI types the renderer
  does not support, and calls 5 functions the scanner does not declare.
- **Plugin logs are lost.** Plugins log through the `log` crate, but nothing
  installs a logger. Each `.so` also has its own copy of `log`'s global state, so
  even a logger installed by the runner would not see plugin logs.
- **Plugins depend on internal crates.** `griffon_scanner` and `griffon_cleaner`
  list the internal `logger` crate (unused).
- **Two plugin installers.** There are two installers, `plugin-installer/src/main.rs`
  and `gui/src-tauri/src/plugin_installer.rs`. There are also two different
  `plugin_history.rs` files, in `daemon/plugin_manager` and `gui/src-tauri`.
- **The plugin guide is out of date.** `plugins/plugin-guide.md` is written twice
  in the same file and refers to an `interface` crate that is now `plugin_interface`.

#### Target

- **`plugin_interface` becomes the plugin SDK**, the only dependency a plugin needs:
  - a typed `PluginInfo` (name, UUID, functions, API version) instead of
    key/value strings;
  - typed request and response structs, with a standard error shape
    (`{ ok, data | error }`) and the JSON (de)serialization hidden behind a helper or
    macro, so plugins no longer parse `fn:` strings;
  - a `Manifest` type (serde) with `validate()`, and a JSON Schema generated from
    it for editors and the GUI;
  - a logging bridge: the host passes a log callback at `init()`, and the SDK
    installs it as the plugin's `log` backend;
  - an explicit API version, checked by the daemon at load time. abi_stable already
    rejects incompatible layouts through `plugin_interface`'s crate version, so
    bump that version on every interface change.
- **A single source of truth for metadata.** Decide which one (manifest or
  `init()`), and have the contract tests check the other against it. Keep only
  one of `id` / `uuid`.
- **The runner fails loudly.** A plugin whose `init()` fails is reported to the
  daemon as failed, instead of starting with a nil UUID.
- **The renderer only has generic components** (table, selectable table with row
  actions, form, list, badge, ...). Rule: *a first-party plugin can only use what
  a third-party plugin can use.* Scanner, cleaner and nmap move to the generic
  components; their specific components are deleted.
- **One shared crate for installing plugins and tracking their history**, used by the
  CLI installer, the daemon and the GUI.
- **Documentation:** the plugin guide is rewritten for the SDK, and
  `plugin_template` is the reference plugin. The template is also built and tested
  **outside the workspace**, to prove an external plugin works.

#### Steps (in order)

1. [x] Extract the loader into `daemon/plugin_host` and add contract tests for
   every plugin (`daemon/plugin_host/tests/plugin_contract.rs`).
2. [ ] Add the `Manifest` type and `validate()`. The contract tests deserialize
   every manifest. No runtime change.
3. [ ] Decide what to do with `griffon_scanner_test.toml`: either it is the next
   scanner UI, or it is deleted.
4. [ ] Add typed `PluginInfo`, requests, responses and errors, plus the log bridge,
   to the SDK. Migrate every plugin, the runner and the daemon in one go,
   with the contract tests as the safety net.
5. [ ] Replace the plugin-specific GUI components with generic ones. Add a check
   that every UI `type` in every manifest is supported by the renderer.
6. [ ] Merge the two installers and the two `plugin_history.rs` files into one crate.
7. [ ] Move the plugin template to its own repository, or build it in CI outside the
   workspace. It depends only on the SDK.
8. [ ] Rewrite `plugins/plugin-guide.md`.

### 1.2 Testing policy

The CI already runs fmt and clippy, cargo-deny, the build, Rust tests, plugin
contract tests, and the frontend lint and build, and `release.yml` reuses it.
What is still missing is a clear rule for what must be tested, and the areas
with no tests at all.

#### Policy

- **Every plugin** has contract tests (it loads, its metadata is valid, it matches
  its manifest, it answers calls) **and** its own unit tests for its behavior.
- **Tests run offline.** Anything that needs the network is marked
  `#[ignore = "requires network access"]` and runs in a separate nightly workflow.
- **Every change to a protocol** (IPC, plugin ABI, manifest) comes with
  round-trip or contract tests that are written *before* the change.
- **Branch protection** on `develop` and `main` requires: `Formatting & Lints`,
  `Dependency Audit`, `Build`, `Tests`, `Plugin Contract`, `Frontend`.
- **Local hooks match CI.** The pre-commit hook and `just lint` run the same
  commands as CI (`just lint` is missing `--workspace --all-targets` today).

#### Gaps to close

- [ ] `shared/ipc_protocol`: round-trip encode/decode tests, then replace
  `serde_cbor` (unmaintained, RUSTSEC-2021-0127, currently ignored in `deny.toml`).
- [ ] `daemon/plugin_manager` and `daemon/daemon_core`: no tests today (plugin
  discovery, enable/disable, dispatch, request/response matching).
- [ ] `griffon_cleaner`, `cli` and `plugin-installer`: no tests today.
- [ ] Frontend: no tests. Start with the renderer: one manifest goes in, the
  expected components come out.
- [ ] Nightly workflow for the network tests (downloading the YARA rules).
- [ ] Re-enable `@typescript-eslint/no-explicit-any` as an error once the
  manifest and payload types exist (step 4 of 1.1).
- [ ] Run Prettier once in a single coordinated commit, then add `prettier --check`
  to the `Frontend` job.
- [ ] Use `paths` filters so a GUI-only change does not run the whole Rust pipeline, and
  the other way round.

### 1.3 Build, packaging and release

The release pipeline runs CI, packages in Docker and publishes. The packages
themselves have several defects, and nothing checks them before a release.

#### Problems

- **Some plugins are not packaged.** `nfpm.yaml` only installs `griffon_scanner` and
  `griffon_cleaner`. `griffon_nmap` and `docker_helper` are built but not shipped.
  The plugin list is also repeated in `nfpm.yaml`, `Dockerfile.build` and the
  `justfile`.
- **Wrong runtime dependencies.** The `.deb` depends on `libwebkit2gtk-4.1-dev`
  and `libssl-dev` (and the `.rpm` on `-devel` packages). Users should only need the runtime
  libraries (`libwebkit2gtk-4.1-0`, `libssl3`, ...).
- **Toolchains are not pinned the same way.** The GUI stage of `Dockerfile.build`
  uses `node:20` and an unpinned `rustup` install. The workspace stage uses
  `rust:1.94`, and CI uses Node 24 and Rust stable. The same commit can build
  differently in CI and at release.
- **Versions are scattered.** The workspace is `0.3.2`. Plugin crates are `0.3.0` or
  `0.1.0`, and manifests `1.0.0` or `0.1.0`. `gui/src-tauri` declares
  `rust-version = "1.77.2"`, which is impossible with edition 2024 (1.85+).
  Editions are mixed (`griffon_cleaner` and `gui/src-tauri` are on 2021).
- **Packaging is only tested at release time.** A packaging bug shows up on the day
  the tag is pushed.
- **The GUI is never fully built in CI.** The `Build` job excludes `griffon-gui`.
  Only clippy and the frontend job cover it.
- **Development friction.** `vite` has no `strictPort`. If port 5173 is taken, the
  dev GUI silently loads another app.

#### Steps

- [ ] Generate the packaged plugin list from one place, and ship every
  first-party plugin (or document why one is excluded).
- [ ] Fix the `.deb` and `.rpm` runtime dependencies. Test-install the package in a
  clean container (`apt install ./griffon.deb`, the daemon starts, the CLI answers).
- [ ] Pin the same Rust and Node versions in CI and in `Dockerfile.build`
  (`rust-toolchain.toml`, `.nvmrc`).
- [ ] One version policy: `version.workspace = true` everywhere, with manifest
  versions either derived from the crate version or explicitly independent. Align
  the editions on 2024 and fix `rust-version`.
- [ ] Add a `package` CI job that runs `Dockerfile.build` on PRs to `main`.
- [ ] Use a dedicated dev port with `strictPort: true` (`gui/vite.config.ts` and
  `devUrl` in `tauri.conf.json`).

---

## Phase 2: Daemon (medium priority)

Starts once phase 1 is stable. Both items depend on the typed plugin
contract from 1.1.

### 2.1 Daemon scheduler

There is no scheduling in the daemon today. Every action is triggered by
the GUI or the CLI. The only periodic work is a hand-written loop in
`plugin_template`.

- Scheduled jobs: a plugin function plus its arguments, run on an interval or cron
  expression (for example a nightly scan, or weekly YARA rule updates).
- Jobs are persisted in the daemon config, survive restarts, and catch up on
  missed runs according to a per-job policy.
- Limits: at most one run of a given job at a time, a global concurrency limit,
  and a timeout per call.
- Results and history are stored and visible from the CLI and the GUI. A failure
  sends a notification (`daemon_core/src/notification.rs`).
- Plugins can declare default schedules in their manifest. This needs the
  manifest schema from 1.1.

### 2.2 Sandboxing

Each plugin already runs in its own `griffon-daemon-runner` process, so a crash
does not take down the daemon. But the daemon runs as `root`
(`daemon/griffon-daemon.service`), and the runners inherit full root rights.
A buggy or malicious plugin has full control of the machine.

- **Permissions in the manifest.** Each plugin declares what it needs (paths it
  reads and writes, network access, running external programs such as `nmap` or
  `docker`, specific capabilities). Installation shows them to the user.
- **Least privilege per runner:**
  - drop to an unprivileged user unless the plugin declares it needs root, and
    keep only the capabilities it declared;
  - restrict the filesystem with Landlock, using the declared paths;
  - restrict syscalls with a seccomp filter;
  - set `no_new_privs`, and resource limits (memory, CPU, open files) with
    cgroups or rlimits.
- **Harden the daemon itself** with systemd options: `ProtectSystem`,
  `ProtectHome`, `PrivateTmp`, `NoNewPrivileges`, `CapabilityBoundingSet`, and so on.
- **Plugin integrity:** checksums, and later signatures, verified at install and at load.
- Tests: a test plugin that tries forbidden actions, and the runner must
  block every one of them.

---

## Phase 3: Product (low priority, exploratory)

Each item below needs a short design note before any work starts.

### 3.1 More plugins

Built with the public SDK only, to prove the phase 1 goal. Candidates:

- **Integrity monitoring:** file hashes and change detection, which pairs
  with the scheduler.
- **System hardening audit:** SSH config, open ports, sudoers, SUID binaries.
- **Package vulnerability check:** installed packages checked against an
  advisory database.
- **Firewall overview** (nftables / firewalld).

Each new plugin goes through the same contract tests and packaging as the
first-party ones.

### 3.2 GUI v2

- Only generic components (see 1.1), driven entirely by manifests.
- Typed manifest and payload types shared with Rust (generated from the JSON Schema).
- Dashboard: daemon status, scheduled jobs, recent results.
- Plugin management: install, permissions, enable or disable, updates.

### 3.3 Native GTK / libadwaita GUI (question)

The question is whether to replace or complement the Tauri GUI with a native GTK4 +
libadwaita app (for example with `relm4` or `gtk4-rs`).

- **For:** it integrates natively with GNOME, there is no webview,
  and the runtime is lighter.
- **Against:**
  - it is a second UI to maintain;
  - the manifest-driven renderer would need a native version;
  - it looks foreign outside GNOME.
- **Prerequisite:** GUI v2's generic components and typed manifest. Once the UI is
  pure data, a second renderer becomes a realistic project.
- **Decision to make:** replace Tauri, or add a native frontend alongside it,
  or drop the idea. Decide after GUI v2.
