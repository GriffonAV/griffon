# Griffon: Resilience and Error Handling

> Purpose: show how Griffon behaves when things go wrong: invalid input, components failing, services going down, unexpected load. Also show what the user sees and what gets logged.
>
> Scope: daemon (`daemon/`), IPC protocol (`shared/ipc_protocol`), plugin SDK and plugins (`plugins/`), GUI (`gui/`), CLI (`cli/`), privileged plugin installer (`plugin-installer/`).
>
> Last updated: 2026-11-10, version `0.3.3`, branch `develop`.

---

## 0. Status summary

| Requirement | Status | Where |
|---|---|---|
| Validate every piece of external data and decide what happens when it's invalid | 🟡 Mostly done: strong at the IPC, plugin-SDK, nmap, cleaner and installer boundaries; gaps in quarantine names, GUI manifest names and config files | [§1](#1-validating-external-data) |
| List anticipated failure cases (behaviour / user sees / logs) | ✅ Catalogue below | [§2](#2-anticipated-failure-cases) |
| Decide behaviour when a dependency goes down | 🟡 Decided and implemented for daemon↔GUI/CLI, Docker and nmap; plugin-process death is detected but not recovered | [§3](#3-behaviour-when-a-dependency-goes-down) |
| Run ≥ 2 resilience scenarios (expected vs. actual) | ✅ 4 live scenarios plus 87 automated failure-path tests | [§4](#4-resilience-scenarios) |
| Usable logs (what, when, where, context) | 🟡 What and where (component prefix) plus IDs are present. **No timestamps** in our own format (journald adds them under systemd). Level is fixed at Debug | [§5](#5-logging) |

Prioritized follow-up work is in [§6](#6-known-gaps-and-action-plan).

---

## 1. Validating external data

Griffon receives external data at the following points. Each row covers what is checked, what happens on invalid data, and which test covers it.

### 1.1 IPC socket (GUI/CLI → daemon)

The wire format is a 12-byte header (`magic 0xBEEF | version | type | request_id | length`) followed by a CBOR body.

| Check | Code | On invalid data | Test |
|---|---|---|---|
| Magic, version and message type | `shared/ipc_protocol/src/ipc_header.rs:83-98` | `InvalidData` error. The daemon closes that connection and logs `WARN … bad magic`. Other clients are unaffected | Live scenario S2 |
| Payload ≤ 1 MiB, checked **before** allocating memory | `ipc_header.rs:7, 103-110` | `InvalidData "payload too large"`. Connection closed, `WARN` | Live scenario S2 |
| CBOR body decodes to a known `InterfaceRequest` | `ipc_payload_interface.rs:116-156` | Connection closed, `WARN … unexpected code at offset N` | Live scenario S2 |
| Plugin UUID exists | `plugin_manager.rs:200-205, 302-306` | Client gets `Error{"… plugin not found"}`; the connection stays open | `network.rs` unit tests |
| Plugin enabled | `plugin_manager.rs:308-317` | Client gets `Error{"send_call failed: plugin is disabled"}`; `plugin_call_rejected` is written to the plugin history | none |
| Socket access | `network.rs:44-69` | Socket is `root:griffon 0660`; other users get `EACCES` | Manual (packaging VM) |

### 1.2 Plugin requests (daemon → plugin, `shared/plugin_sdk`)

Every plugin built on the SDK gets JSON-RPC-style validation for free:

| Invalid input | Response | Code |
|---|---|---|
| Unknown method | `{"ok":false,"code":-32601}` | `router.rs:66-76` |
| Malformed JSON / wrong param types | `{"ok":false,"code":-32602,"message":"invalid params: …"}` | `router.rs:23-33, 78-88` |
| Panic inside a handler | `{"ok":false,"code":-32603,"message":"plugin panicked: …"}`; the runner keeps running | `router.rs:90-110` |
| Panic inside `init()` | Caught, returned as an error | `lib.rs:46-60` |

Tests: `test_unknown_method`, `test_invalid_params`, `test_business_error`, `test_panic_safety`, `test_no_params_variants`. Every plugin also passes the contract test in `daemon/plugin_host/tests/plugin_contract.rs`, which calls an unknown function and checks that the plugin doesn't panic or abort.

### 1.3 Plugin-specific external data

| Plugin | Data | Validation | On invalid data |
|---|---|---|---|
| **nmap** | Target | IP or strict RFC hostname (≤253 chars, label rules) (`griffon_nmap/src/lib.rs:286-306`) | Rejected before any process is spawned |
| | Ports | Numeric ports and ranges only (`:326-346`) | Rejected |
| | Profile, timing, scan type, executor | Fixed allow-lists (`:316-384, 458-479`) | Rejected |
| | Command construction | `Command::args` with `--` before the target, never a shell (`:461, 475, 651, 667`), so no command injection | n/a |
| | nmap XML output | Parsed with `roxmltree`; output must be UTF-8 (`:728-733, 877`) | "invalid Nmap XML result" error; nothing written to history |
| | History file | Missing means empty; written atomically via temp file and rename (`:181-204`) | Corrupt file returns an error |
| **cleaner** | Paths to delete | `validate_delete_target` rejects empty paths, nonexistent paths, symlinks, system directories and anything outside the allowed cache roots after canonicalisation (`modules/cache.rs:317-368`) | Path refused and reported in `failed[]` |
| | Request args | `dry_run` type, list items, empty selection (`lib.rs:346-423`) | Error before any file is touched |
| | Config JSON | Must parse; at least one category must be enabled (`config.rs:42-89`) | `Config("Invalid config JSON: …")` |
| | Docker output | Reclaimable-size parsing (`modules/docker.rs:65-117`) | Malformed values become 0 |
| **scanner** | Scanned files | Files > 50 MB skipped; media files skipped by magic bytes; directory walk doesn't follow symlinks; only regular files; system pseudo-filesystems excluded (`scanner_engine/scan_file.rs:12-42`, `scan_dir.rs:8-99`) | Marked `skipped` or `error` in the result |
| | ZIP archives | Nesting depth ≤ 5, entry ≤ 100 MB, total ≤ 500 MB, compression ratio ≤ 100× as a zip-bomb guard (`scan_file.rs:9-10`, `archive/extract.rs:5-57`) | Entry skipped and `WARN` logged |
| | YARA rules | Compiled file by file (`yara_engine/mod.rs:26-63`) | Bad file logged at `ERROR` and skipped; other rules still load |
| | Remote rule manifest | Parse checked, HTTP status checked (`scanner_updater/mod.rs:54-96`) | `"Update failed: …"` |
| **docker_helper** | `docker ps` output | Line-by-line parse (`lib.rs:66-83`) | Bad lines skipped with `WARN`; missing fields become `"Unknown"` |

### 1.4 User input (GUI and CLI)

| Input | Validation | On invalid input |
|---|---|---|
| CLI command / arguments (`cli/src/main.rs`) | Arity check; UUID parsed with `Uuid::parse_str` | Usage message or `Invalid UUID format`; the prompt continues |
| GUI plugin call (`gui/src-tauri/src/main.rs:296-307`) | UUID parsed, function name must not be empty | `Err` returned to the frontend |
| GUI plugin manifests (TOML) | Deserialized into fixed structs (`manifests/mod.rs:81-86`) | Invalid manifest logged and skipped; other plugins still listed |
| Nmap form (target / ports) | None on the client side; validated by the plugin (§1.3) | Error message returned by the plugin |

### 1.5 Plugin packages (privileged installer, runs as root via `pkexec`)

| Check | Code |
|---|---|
| Must run as root; archive path canonicalised; must be a regular file ending in `.zip` | `plugin-installer/src/main.rs:85-116` |
| Zip-slip protection via `enclosed_name()`; only the file name is used for the destination | `:135-173` |
| Only `.toml` and `.so` extracted; exactly one of each is required | `:146-196` |
| 256 MiB per-file cap | `:12, 150-155` |
| Extracted to a temp directory, installed with copy → chmod 0644 → atomic rename | `:241-276` |
| On removal, paths are canonicalised and must stay inside the plugin directory | `:355-360, 438-443` |

What the user sees: the `pkexec` exit code is mapped to a message. 126 → "cancelled by the user", 127 → "authorization denied", otherwise the installer's stderr (`gui/src-tauri/src/plugin_installer.rs:387-410`).

### 1.6 Known validation gaps

These are listed honestly and tracked in §6:

- **Scanner quarantine:** `q_restore` and `q_delete` join a caller-supplied name onto the quarantine directory without checking it (`scanner_quarantine/restore_file.rs:7-10, 76-84`).
- **GUI `get_plugin_manifest`:** builds `{dir}/{name}.toml` without sanitizing `name`. A `sanitize_plugin_name` function exists but is unused (`gui/src-tauri/src/main.rs:158-197`).
- **Daemon config (`/etc/griffon/config_griffon_daemon.json`):** malformed JSON silently falls back to defaults and is then overwritten on the next save. Nothing is logged.
- **Plugin installer:** no signature or checksum check; the size cap trusts the size declared in the archive.
- **Scanner rule updates:** no integrity check; the remote `filename` is not sanitized.
- **Missing YARA rules directory:** the engine reports "ready" with 0 rules instead of failing.
- **CLI `call`:** the JSON payload isn't validated on the client (the plugin SDK rejects it, see §1.2).

---

## 2. Anticipated failure cases

| # | Failure | What the system does | What the user sees | What is logged |
|---|---|---|---|---|
| F1 | Daemon not running / socket missing (CLI) | CLI exits immediately | `Failed to connect to daemon at ./griffon.sock: No such file or directory`, exit code 1 | `[CLI](ERROR) Failed to connect …` |
| F2 | Daemon not running / connection lost (GUI) | Reconnect loop: retries every 10 s, forever | Daemon icon in the title bar turns grey; tooltip "Background Service disconnected" with the error | `[GUI-NETWORK](ERROR) …` + `Waiting 10 seconds before reconnecting...` |
| F3 | Connection lost mid-session (CLI) | Reader thread stops | `[FATAL] Connection to daemon lost: …`, exit code 1 | stderr |
| F4 | Malformed / oversized / non-CBOR frame | That connection is closed; daemon and other clients unaffected | Connection closed (no error frame) | `[DAEMON-INTERFACE-NETWORK](WARN) Client disconnected or invalid request: bad magic \| payload too large \| unexpected code at offset N` |
| F5 | Request for an unknown or disabled plugin | Request rejected; connection kept | `Error{"… plugin not found"}` / `"plugin is disabled"` | `[DISPATCHER](ERROR)` + `plugin_call_rejected` in the plugin history |
| F6 | Request type not implemented (`KillPlugin`) | Rejected | `Error{"Request not implemented yet"}` | `WARN` |
| F7 | Plugin `.so` fails to load (ABI mismatch, missing symbol) | Runner exits 1; handshake fails; plugin marked disabled; other plugins load normally | Plugin listed with status inactive | `[PLUGIN_MANAGER](ERROR) handshake failed` + `plugin_enable_failed` in the plugin history |
| F8 | Plugin panics while handling a call (SDK plugins) | Panic caught; runner survives | `{"ok":false,"code":-32603,"message":"plugin panicked: …"}` | Returned to the caller |
| F9 | Plugin runner process crashes / is killed | Daemon detects EOF; **plugin stays listed Active**; next call fails fast | `Error{"send_call failed: Broken pipe (os error 32)"}` | `[PLUGIN_MANAGER-NETWORK](INFO) … closed / recv error`, `[DISPATCHER](ERROR) Plugin runner <pid> closed: …`, `plugin_closed` history line |
| F10 | Invalid plugin arguments | SDK rejects them before the handler runs | `code -32602 invalid params: …` | Returned to the caller |
| F11 | nmap binary missing / nmap fails / invalid XML | Scan aborted; history not modified | "Could not start Nmap" / "Nmap failed: <stderr>" | Error returned to the caller (the nmap plugin has no logger) |
| F12 | No host up | Success with a diagnostic | Message suggesting `-Pn` | none |
| F13 | Docker daemon down (cleaner) | Docker module adds a warning; cache cleaning results are kept | Report with a Docker warning | `warnings[]` in the report |
| F14 | Docker daemon down (nmap containers / docker_helper) | Request fails | "Docker unavailable…" / `"Docker CLI error: …"` | `[LIBDOCKERH] ERROR` |
| F15 | Cleaner can't delete a file (permission) | Continues with the next file | Counted in `permission_denied` and listed in `warnings` / `failed[]` | In the report |
| F16 | Path outside the allowed scope given to the cleaner | Refused | `failed[{path, error}]` | In the report |
| F17 | Scanner: unreadable file, file too large, media file | Skipped; scan continues | Counted in `total_errors` / `total_skipped` | Too large / media: `skipped` flag; unreadable: not logged (gap) |
| F18 | Scanner: zip bomb / too deep / too large | Entry skipped | Scan completes | `WARN` with the archive path |
| F19 | Scanner: malformed YARA rule file | Skipped; other rules still loaded | Scan works with the remaining rules | `ERROR Failed to compile rule file <path>: <err>` |
| F20 | Scanner: engine not started / still loading | Request rejected | "Engine is not running, call start first" / "still loading" | none |
| F21 | Rule update: network down / HTTP error | Update aborted; existing rules kept | `"Update failed: …"` / "Failed to check for updates: …" | none |
| F22 | Plugin install cancelled / denied in `pkexec` | No change | "Installation … was cancelled by the user" / "authorization was denied" | none |
| F23 | Corrupt ZIP / zip-slip entry / missing `.so` or `.toml` | Installation aborted; temp directory discarded | "Installation failed: Unsafe path found in ZIP archive" etc. | Installer stderr |
| F24 | Runner binary not found at daemon start | Dispatcher thread panics; daemon keeps accepting connections | Every request: `Error{"Failed to queue task: …"}` | Panic on stderr (gap: should exit and let systemd restart) |
| F25 | Socket bind fails (permission) | Daemon exits non-zero; systemd restarts it after 5 s (`Restart=on-failure`) | Clients can't connect (F1/F2) | stderr → journald |

---

## 3. Behaviour when a dependency goes down

| Dependency | Strategy | Detail |
|---|---|---|
| **Daemon** (for the GUI) | **Retry** | Infinite reconnect loop with a fixed 10 s interval; connection status shown in the title bar (`gui/src-tauri/src/main.rs:532-576`). Commands return `"Daemon not connected"` immediately while disconnected |
| **Daemon** (for the CLI) | **Fail fast** | Explicit error, exit code 1 |
| **Daemon process** (under systemd) | **Restart** | `Restart=on-failure`, `RestartSec=5` (`daemon/griffon-daemon.service`) |
| **Plugin runner process** | **Isolate + degrade** | Each plugin runs in its own `griffon-daemon-runner` process over a socketpair. A plugin crash cannot take down the daemon or other plugins. Calls to the dead plugin fail fast with `Broken pipe`. *No automatic restart yet* (see §6) |
| **Docker daemon** (cleaner) | **Degrade gracefully** | Docker cleanup is skipped with a warning; cache cleanup still runs and its results are kept (tested: `docker_daemon_failure_preserves_cache_results`) |
| **Docker daemon** (nmap / docker_helper) | **Fail with message** | Explicit error to the user |
| **nmap binary** | **Fail with message** | Explicit error; no partial history written (tested: `failed_scans_never_create_history`) |
| **Network** (YARA rule updates) | **Degrade gracefully** | Update fails, the scanner keeps using the local rules |
| **D-Bus** (desktop notifications) | **Degrade gracefully** | Error logged; the plugin call result is still delivered (`dispatcher.rs:270-279`) |

---

## 4. Resilience scenarios

### 4.1 Live scenarios (run 2026-11-10)

All scenarios were run against a debug build (`cargo build --workspace --exclude griffon-gui`) in an isolated sandbox directory. The daemon was started from there with the cleaner plugin as its only plugin. The raw daemon log was kept locally during the run (not committed).

#### S1: Daemon down (service cut off)

- **Action:** start `griffon-cli` with no daemon running.
- **Expected:** clear error, non-zero exit, no hang.
- **Actual:**
  ```
  [CLI](ERROR) Failed to connect to daemon at ./griffon.sock: No such file or directory (os error 2)
  exit code=1
  ```
- **Verdict:** ✅ As expected.

#### S2: Malformed input on the IPC socket

- **Action:** three raw clients connect to the daemon socket and send:
  - (a) 64 random bytes;
  - (b) a valid header declaring a payload of `0xFFFFFFFF` bytes;
  - (c) a valid header followed by non-CBOR bytes.

  Then a normal CLI runs `refresh`.
- **Expected:** each bad connection is rejected without allocating 4 GiB and without crashing. The daemon keeps serving other clients.
- **Actual:**
  - (a) connection reset;
  - (b) and (c) connection closed (EOF).
  - The daemon stayed alive and the following `refresh` returned the plugin table normally. Log:
  ```
  [DAEMON-INTERFACE-NETWORK](WARN) Client disconnected or invalid request: bad magic
  [DAEMON-INTERFACE-NETWORK](WARN) Client disconnected or invalid request: payload too large
  [DAEMON-INTERFACE-NETWORK](WARN) Client disconnected or invalid request: unexpected code at offset 1
  ```
- **Verdict:** ✅ Robust.
  - ⚠️ The client never receives an error frame explaining the rejection.
  - ⚠️ A normal client disconnect is logged with the same `WARN` message (`failed to fill whole buffer`), which adds noise.

#### S3: Plugin process killed (component crash)

- **Action:** `kill -9` the cleaner's runner process, then `refresh` and `call <cleaner-uuid> run_metrics {}` from the CLI.
- **Expected (design intent):** the daemon survives; the plugin is reported as down; calls fail fast instead of hanging.
- **Actual:**
  - The daemon survived and detected the crash immediately:
    ```
    [PLUGIN_MANAGER-NETWORK](INFO) Griffon Cleaner (66092) closed / recv error: failed to fill whole buffer
    [DISPATCHER](ERROR) Plugin runner 66092 closed: failed to fill whole buffer
    ```
  - The call failed fast: `[ERROR] Request 2 failed: send_call failed: Broken pipe (os error 32)`.
  - ❌ `refresh` **still listed the plugin as `Active`**.
  - ❌ The runner stayed a **zombie** (`<defunct>`) until the daemon exited.
  - ❌ No automatic restart.
- **Verdict:** 🟡 Isolation works (no cascading failure, fast failure, clear logs). Recovery and status reporting are missing. See §6, items 1-2.

#### S4: Unexpected load

- **Action:**
  - (a) 100 CLI clients connect at the same moment and each run `refresh`.
  - (b) A script opens 1,000 connections and keeps them idle for 8 s, while a normal client runs `refresh`.
- **Expected:** all requests served; daemon stays responsive and recovers its resources afterwards.
- **Actual:**
  - (a) 100/100 clients got the full plugin table, all exit code 0, in 1.04 s total.
  - (b) The daemon grew from 2 to 1,002 threads (one per connection), RSS 21 MB. The concurrent `refresh` was still answered. After the idle clients disconnected, the thread count went back to 2. The daemon stayed alive.
- **Verdict:** ✅ Handles the load and recovers. ⚠️ Threads per connection are unbounded and there is no idle timeout, so a local user in the `griffon` group could exhaust threads (§6, item 4).

### 4.2 Automated failure-path tests

`cargo test -p griffon_nmap -p griffon_cleaner -p plugin_sdk` on 2026-11-10: **87 passed, 0 failed**. They run in CI (`cargo nextest run --workspace`, with a 70% coverage gate on cleaner and nmap).

| Area | Failure scenarios covered |
|---|---|
| nmap integration (`plugins/griffon_nmap/tests/integration_tests.rs`, fake `nmap`/`docker`/`curl` binaries, 20 s deadline per scenario) | nmap exits non-zero, invalid XML, non-UTF-8 output, nmap missing (`failed_scans_never_create_history`); corrupt or unwritable history (`history_limit_corruption_and_storage_errors`); Docker daemon unavailable (`container_listing_filters_and_errors`); curl / ssh-keyscan errors |
| nmap unit | Target/port boundaries, invalid option combinations, targets kept as separate arguments (injection), malformed payloads rejected without execution |
| cleaner integration | Broken config doesn't modify files (`bad_config_and_requests_do_not_modify_files`); Docker partial failure (`docker_partial_failure_preserves_counts_and_continues`); Docker daemon down (`docker_daemon_failure_preserves_cache_results`) |
| cleaner unit | Path traversal / symlink / out-of-scope deletion refused (`deletion_scope_rejects_root_outside_symlink_and_traversal`); partial deletion failure; missing Docker CLI; bad requests rejected before side effects |
| plugin SDK | Unknown method, invalid params, business error, panic safety |
| scanner | Engine not ready, wrong param types, malformed JSON, zip-bomb ratio, corrupt archive, oversized / media files, quarantine of a missing file, corrupt quarantine manifest, partial batch failure |
| daemon | Unimplemented request, dispatcher channel closed, dropped reply channel produces an error response, stale socket removed |
| all plugins | Contract test: unknown function → no panic or abort (`daemon/plugin_host/tests/plugin_contract.rs`) |

Reproduce with:

```bash
cargo test -p griffon_nmap -p griffon_cleaner -p plugin_sdk
just setup && cargo test -p griffon_scanner   # scanner fixtures (EICAR, large files) are generated, not committed
```

---

## 5. Logging

### 5.1 Current state

Logging uses a custom `shared/logger` crate with one static logger per component:

```
[<COMPONENT>](<LEVEL>) <message>
```

| Question the log should answer | Covered? | How |
|---|---|---|
| **What** happened | ✅ | Message plus the underlying error (`… : Broken pipe (os error 32)`) |
| **Where** (which component) | ✅ | Prefix: `DAEMON-INTERFACE-NETWORK`, `DISPATCHER`, `PLUGIN_MANAGER`, `PLUGIN_MANAGER-NETWORK`, `RUNNER`, `GUI`, `GUI-NETWORK`, `CLI`, `[LIBSCANNER]`, `[LIBDOCKERH]` |
| **Context** to act on | ✅ (partial) | `request_id`, plugin name, UUID, PID, file path, error string. Mapping from client request id to plugin request id is logged by the dispatcher |
| **When** | 🟡 | No timestamp in our format. Under systemd, stdout/stderr go to journald, which timestamps every line (`journalctl -u griffon-daemon`). The file sink (`/var/log/griffon/griffon-daemon.log`) has no timestamps |
| Severity | 🟡 | ERROR → stderr, others → stdout; level fixed at `Debug`, even in release |

There is also a per-plugin audit trail in `/var/log/griffon/plugin-history/history_<uuid>`, in structured key=value form with Unix timestamps:

```
[<unix_ts>][<LEVEL>][<plugin name>][<uuid>] event=plugin_closed name="…" uuid="…" pid=… request_id=…
```

Events: `plugin_enable_failed`, `plugin_closed`, `plugin_call_rejected`, `plugin_result`, … It is shown in the GUI's Logs page.

Example from scenario S3. It identifies the plugin and PID, the cause, the failed request and the user-facing error:

```
[PLUGIN_MANAGER-NETWORK](INFO) Griffon Cleaner (66092) closed / recv error: failed to fill whole buffer
[DISPATCHER](ERROR) Plugin runner 66092 closed: failed to fill whole buffer
[DISPATCHER](ERROR) Send call failed: Broken pipe (os error 32)
[DAEMON-INTERFACE-NETWORK](DEBUG) Response sent for request 2: Error { request_id: 2, message: "send_call failed: Broken pipe (os error 32)" }
```

### 5.2 Logging gaps

- No timestamp in `shared/logger` output (file sink and non-systemd runs).
- Level hard-coded to `Debug` in release. Full request arguments and plugin output are logged at DEBUG and INFO.
- No log rotation for `/var/log/griffon/*`.
- The runner uses raw `eprintln!` for most of its errors, so they don't carry the logger prefix.
- The nmap plugin has no logging. The cleaner uses `println!`. The scanner and docker_helper use the `log` crate, but no logger is initialized inside the plugin, so their output may be lost.
- The GUI frontend logs most errors with `console.error` only.

---

## 6. Known gaps and action plan

Ordered by impact. Items marked 🎯 are the cheapest to implement and would most strengthen this document.

| # | Gap | Proposed fix |
|---|---|---|
| 1 🎯 | Dead plugin still listed `Active`; calls in flight when it dies **hang forever** (`Closed` event carries no request id; `pending_calls` never failed) | On `PluginEvent::Closed`: set `status=false`, fail all `pending_calls` for that PID with `Error{"plugin crashed"}`, `wait()` the child |
| 2 | No automatic restart of crashed plugins | Restart with backoff (e.g. 3 attempts: 1 s / 5 s / 30 s), then mark disabled |
| 3 🎯 | No timeouts: plugin handshake (a hanging `init()` freezes the dispatcher), plugin calls, sockets, `nmap`/`docker` subprocesses | `set_read_timeout` on the handshake and sockets; per-call deadline in the dispatcher; `wait_timeout` for subprocesses |
| 4 | Unbounded threads per connection, no idle timeout (S4b) | Cap concurrent clients; read timeout on idle connections |
| 5 🎯 | No timestamp in logs; level stuck at Debug | Add an RFC 3339 timestamp in `Logger::log`; read the level from `GRIFFON_LOG` / config; default to `Info` in release |
| 6 🎯 | Quarantine `restore`/`delete` and GUI `get_plugin_manifest` accept `../` names | Reject names containing `/`, `\`, `..`; use the existing `sanitize_plugin_name` |
| 7 | Malformed daemon config silently replaced by defaults, then overwritten | Log a `WARN`; back up the broken file as `.bak`; write atomically (temp file + rename) |
| 8 | GUI drops daemon `Error` responses; frontend request timeout is 100 min; "reconnect" button broken (`ReconnectSender` never registered) | Forward errors to the request manager; reject pending requests on disconnect; register the reconnect channel |
| 9 | Missing YARA rules directory reports "ready" with 0 rules | Fail `start` (or flag it as degraded) when 0 rules load |
| 10 | Plugin packages and rule updates aren't authenticated | Signature / SHA-256 check |
| 11 | No tests for the IPC decoder, `plugin_manager`, runner, GUI backend, CLI, installer | Add unit tests reproducing S2/S3; turn the live scenarios into an integration test |
| 12 | Malformed IPC frame closes the connection silently | Send `Error{"invalid request"}` before closing; log clean disconnects at DEBUG |

---

## Appendix: reproducing the live scenarios

```bash
cargo build --workspace --exclude griffon-gui
R=$PWD; S=$(mktemp -d); cd "$S"
mkdir -p .config/griffon
cp "$R"/target/debug/libgriffon_cleaner.so "$R"/plugins/griffon_cleaner/libgriffon_cleaner.toml .config/griffon/

# S1 — daemon down
echo help | "$R"/target/debug/griffon-cli; echo "exit=$?"

# start daemon
GRIFFON_RUNNER="$R"/target/debug/griffon-daemon-runner "$R"/target/debug/griffon-daemon > daemon.log 2>&1 &

# S2 — malformed frames (random bytes / 4 GiB length / bad CBOR)
python3 -c "import socket,os;s=socket.socket(socket.AF_UNIX);s.connect('griffon.sock');s.sendall(os.urandom(64))"
python3 -c "import socket,struct;s=socket.socket(socket.AF_UNIX);s.connect('griffon.sock');s.sendall(struct.pack('>HBBII',0xBEEF,1,8,1,0xFFFFFFFF))"
python3 -c "import socket,struct;p=b'\xff\xfe';s=socket.socket(socket.AF_UNIX);s.connect('griffon.sock');s.sendall(struct.pack('>HBBII',0xBEEF,1,8,1,len(p))+p)"
(echo refresh; sleep 1; echo exit) | "$R"/target/debug/griffon-cli

# S3 — kill the plugin runner
pkill -9 -f 'debug/griffon-daemon-runner .config/griffon/libgriffon_cleaner.so'
(echo refresh; sleep 1; echo 'call 123e4567-e23b-12d3-a456-426614174000 run_metrics {}'; sleep 2; echo exit) | "$R"/target/debug/griffon-cli

# S4 — 100 concurrent clients
for i in $(seq 100); do ( (echo refresh; sleep 1; echo exit) | "$R"/target/debug/griffon-cli >/dev/null; echo $? ) & done | sort | uniq -c
grep -E "WARN|ERROR" daemon.log
```
