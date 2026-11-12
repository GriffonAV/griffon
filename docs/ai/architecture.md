# Architecture map for AI-assisted work

Use this map to locate code, not as a guarantee of current behavior. Read the
implementation and tests before answering or reporting a bug.

## Request path

```text
React UI → Tauri commands ─┐
                          ├→ Unix socket → daemon dispatcher → plugin manager
CLI ──────────────────────┘                                      ↓
                                                       runner process
                                                              ↓
                                                   plugin host → plugin
```

Responses return through the IPC layers; the GUI also receives Tauri events.
Trace request identifiers, error propagation and notifications in both directions.

| Responsibility | Entry points |
| --- | --- |
| React bindings and pending requests | [PluginContext](../../gui/src/bindings/PluginContext.tsx), [request manager](../../gui/src/services/requestManager.ts) |
| Tauri commands and connection | [main](../../gui/src-tauri/src/main.rs), [daemon module](../../gui/src-tauri/src/daemon/mod.rs) |
| CLI commands | [CLI main](../../cli/src/main.rs) |
| Client socket and dispatch | [network](../../daemon/daemon_core/src/network.rs), [dispatcher](../../daemon/daemon_core/src/dispatcher.rs) |
| Runner lifecycle and plugin calls | [plugin manager](../../daemon/plugin_manager/src/plugin_manager.rs), [runner](../../daemon/daemon_runner/src/main.rs) |
| Dynamic library loading | [plugin host](../../daemon/plugin_host/src/lib.rs) |
| Wire formats | [IPC source](../../shared/ipc_protocol/src/lib.rs), [protocol guide](../../shared/ipc_protocol/ipc-protocol.md) |
| ABI and handler routing | [plugin interface](../../shared/plugin_interface/src/lib.rs), [SDK router](../../shared/plugin_sdk/src/router.rs) |
| Manifest-driven UI | [manifest parser](../../gui/src-tauri/src/manifests/mod.rs), [renderer](../../gui/src/renderer/GriffonElementRenderer.tsx), [TOML guide](../guiTomlHowTo.md) |
| Privileged plugin installation | [installer](../../plugin-installer/src/main.rs), [Tauri integration](../../gui/src-tauri/src/plugin_installer.rs) |

Plugins are loaded into runner processes. Process separation alone does not prove
restricted permissions, filesystem isolation or protection from malicious plugins.

## Plugin-specific review

- [Scanner](../../plugins/griffon_scanner/src/lib.rs): inspect engine, archive
  extraction, rule updates and quarantine operations. Check path handling,
  resource bounds, failure recovery and restoration collisions.
- [Cleaner](../../plugins/griffon_cleaner/src/lib.rs): inspect selection, dry-run,
  path validation and Docker operations. `ExecutionContext.root_paths` alone does
  not isolate real cleanup of known caches; see [integration tests](../plugin-integration-tests.md).
- [Nmap](../../plugins/griffon_nmap/src/lib.rs): inspect argument construction,
  targets, subprocess failures, result parsing and history. Use mocked executables
  or explicitly authorized local test targets.
- [Plugin guide](../../plugins/plugin-guide.md): consult alongside the actual SDK,
  manifest and exported function definitions when changing a plugin contract.

## Environment and existing evidence

Debug and installed builds use different paths. The daemon socket is selected in
[network.rs](../../daemon/daemon_core/src/network.rs): `./griffon.sock` in debug
and `/run/griffon/griffon.sock` in release. Check each component's path selection
and working directory; do not assume changing one root redirects all filesystem use.

Use [README](../../README.md), [justfile](../../justfile),
[rust-toolchain.toml](../../rust-toolchain.toml), [CI](../../.github/workflows/ci.yml)
and [packaging tests](../packaging-testing.md) for environment setup.
[Resilience notes](../resilience.md) contain historical evidence and known gaps;
revalidate their findings against the current revision instead of reporting them
as newly discovered or still present without checking.
