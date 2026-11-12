# Griffon Roadmap

This document gives the direction of the project and the order of the work.
It does not list every task. Tasks live in [GitHub issues](https://github.com/GriffonAV/griffon/issues),
and each item below links to its issue once one exists.

Technical debt is tracked as issues with the
[`tech-debt`](https://github.com/GriffonAV/griffon/issues?q=is%3Aissue+is%3Aopen+label%3Atech-debt)
label, not in this file.

Last reviewed: 2026-11-12

## Current focus

| Track | Goal | Depends on | Status |
| --- | --- | --- | --- |
| [ABI v2](#1-abi-v2) | A typed, versioned contract between the daemon and plugins | Plugin SDK (done) | Next |
| [UI v2](#2-ui-v2) | A GUI driven only by manifests, using generic components | ABI v2 (manifest schema) | Next |
| [Sandboxing](#3-sandboxing) | A plugin only gets the rights it declares | ABI v2 (permissions in the manifest) | Planned |
| [Plugin scheduling](#4-plugin-scheduling) | The daemon runs plugin functions on a schedule | ABI v2 (schedules in the manifest) | Planned |

**ABI v2 comes first:** the three other tracks need a typed manifest to declare
their components, permissions and schedules.

**End goal:** someone can write a plugin **outside this repository**, depend
only on the published SDK, declare what it needs, and it loads, runs, renders
and stays confined. CI catches it when that stops being true.

---

## 1. ABI v2

The [plugin SDK](shared/plugin_sdk) already hides the current ABI: plugins
register typed functions, and errors have one shape. Underneath, the
contract is still strings: `init()` returns key/value pairs, calls are
`"fn:<name> <args>"`, and errors are JSON inside a successful reply.

- **Typed contract:** `PluginInfo` (name, UUID, functions, API version) instead
  of key/value strings, and typed requests and responses, with errors returned as
  real errors instead of `ok: true` plus a JSON error body.
- **Versioning:** the daemon checks an explicit API version at load time and
  rejects incompatible plugins with a clear message.
- **The runner fails loudly:** a plugin whose `init()` fails is reported to the
  daemon as failed, instead of starting with an empty name and a nil UUID.
- **Manifest type:** a `Manifest` type (serde) with `validate()`, and a JSON
  Schema generated from it for editors and the GUI. Manifest validation becomes
  part of the contract tests.
- **One source of truth for metadata:** name, UUID and functions are declared once
  (manifest or code), and the contract tests check the other against it. Keep
  only one of `id` / `uuid`.
- **Plugin logs:** the host passes a log callback at `init()`, and the SDK
  installs it as the plugin's `log` backend.
- **Migration:** the change stays inside the SDK, so plugins keep their code and
  are rebuilt against the new SDK version.
- **Proof:** `plugin_template` is built and tested **outside the workspace**,
  depending only on the SDK.

## 2. UI v2

Today the renderer has cases specific to some plugins (`scanner_*`, `cleaner_*`,
`nmap_*`), so a third-party plugin can never get the same UI.

- **Rule:** *a first-party plugin can only use what a third-party plugin can use.*
- **Generic components only:** table, selectable table with row actions,
  form, list, badge, ... Scanner, cleaner and nmap move to them, and their
  specific components are deleted.
- **Typed manifests:** TypeScript types generated from the ABI v2 JSON Schema,
  and a CI check that every UI `type` used in a manifest is supported by the
  renderer.
- **Frontend tests**, starting with the renderer: one manifest goes in, the
  expected components come out.
- **Product:** a dashboard (daemon status, scheduled jobs, recent results) and
  plugin management (install, permissions, enable or disable, updates).
- Related: [#239](https://github.com/GriffonAV/griffon/issues/239) (web app SDK for plugin front-ends).

## 3. Sandboxing

Each plugin already runs in its own `griffon-daemon-runner` process, so a crash
doesn't take down the daemon. But the daemon runs as `root`
(`daemon/griffon-daemon.service`), and the runners inherit all of its rights.
A buggy or malicious plugin has full control of the machine.

- **Permissions in the manifest:** each plugin declares what it needs (paths it
  reads and writes, network access, external programs such as `nmap` or
  `docker`, specific capabilities). Installation shows them to the user.
- **Least privilege per runner:**
  - drop to an unprivileged user unless the plugin declares that it needs root, and
    keep only the capabilities it declared;
  - restrict the filesystem with Landlock, based on the declared paths;
  - restrict syscalls with a seccomp filter;
  - set `no_new_privs`, and resource limits (memory, CPU, open files).
- **Harden the daemon itself** with systemd options: `ProtectSystem`,
  `ProtectHome`, `PrivateTmp`, `NoNewPrivileges`, `CapabilityBoundingSet`, ...
- **Plugin integrity:** checksums, then signatures, checked at install and at load.
- **Tests:** a test plugin tries forbidden actions, and the runner must block
  every one of them.

## 4. Plugin scheduling

Every action is triggered by the GUI or the CLI today. There is no scheduling
in the daemon.

- **Jobs:** a plugin function plus its arguments, run on an interval, managed inside the daemon.
  expression (for example a nightly scan, or weekly YARA rule updates).
- **Persistence:** jobs are stored in the daemon config and survive restarts. They
  catch up on missed runs according to a per-job policy.
- **Limits:** at most one run of a given job at a time, a global concurrency limit,
  and a timeout per call.
- **Results:** the run history is stored and visible from the CLI and the GUI.
  A failure sends a notification.
- **Defaults:** plugins can declare default schedules in their manifest.

---

## Later (exploratory)

Each item below needs a short design note before any work starts.

- **More plugins, built with the public SDK only:** integrity monitoring (pairs with
  scheduling), system hardening audit, package vulnerability check, firewall
  overview.
- **Native GTK4 / libadwaita GUI:** it integrates natively with GNOME and has no
  webview, but it would be a second UI to maintain. Decide after UI v2, once the
  UI is pure data.

## Done

- Plugin loader extracted into `daemon/plugin_host`, with contract tests for every plugin.
- Plugin SDK (`shared/plugin_sdk`): typed functions, one error shape, panic
  safety, and the function list generated automatically. Every first-party plugin uses it.
- CI: formatting and lints, dependency audit, build, tests, plugin contract tests,
  frontend, plugin coverage (70% for Cleaner and Nmap), and package build and
  install tests.
- Packaging: correct runtime dependencies, a build on Ubuntu 22.04 for glibc
  compatibility, and the Rust toolchain pinned in `rust-toolchain.toml`.
