# QA procedure

## Prepare a bounded check

State the behavior under test, revision, environment, prerequisites and expected
outcome before executing it. Read the tests or scripts to understand their side
effects. Select the smallest relevant suite, then run broader required checks for
a code change. Use the repository toolchain and locked dependencies.

Never use the user's caches, quarantine, Docker resources or system configuration
as disposable fixtures. Functional cleanup and privileged packaging checks belong
in a disposable VM/container with explicitly isolated storage and services.
`ExecutionContext.root_paths` is not sufficient isolation for Cleaner. Do not
assume a container is safe if it mounts host directories or the host Docker socket.
Network tests must use mocks or an explicitly authorized local test environment.

## Existing checks

Commands below run from the repository root unless noted. Inspect the current
[CI workflow](../../.github/workflows/ci.yml) for authoritative merge checks.

| Scope | Checks and prerequisites |
| --- | --- |
| Documentation only | Check local links, referenced paths/commands and `git diff --check`; no application build needed |
| Rust formatting and lints | `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings` (workspace includes Tauri system dependencies) |
| Cleaner / Nmap | `cargo test --locked -p griffon_cleaner -p griffon_nmap --all-targets`; see [unit tests](../plugin-unit-tests.md) |
| Cleaner / Nmap integration only | `cargo test --locked -p griffon_cleaner -p griffon_nmap --test integration_tests`; debug fixtures, mocked system tools; see [integration tests](../plugin-integration-tests.md) |
| Shared protocol / SDK | `cargo test --locked -p ipc_protocol -p plugin_sdk` |
| Daemon | `cargo test --locked -p griffon-daemon` |
| Plugin loading contract | `cargo test --locked -p plugin_host --test plugin_contract`; inspect the suite's library build/setup requirements |
| GUI | In `gui/`, install locked dependencies with `npm ci` if needed; run `npm run lint` and `npm run build` |
| Full Rust suite | `cargo test --workspace --locked`; requires system dependencies and scanner fixtures; CI uses nextest and a separate contract job |
| Packaging / permissions | Follow [packaging tests](../packaging-testing.md) in a disposable environment |

Scanner fixture preparation is defined in the [CI workflow](../../.github/workflows/ci.yml)
and [scanner justfile](../../plugins/griffon_scanner/justfile). Inspect scripts before
running: preparation can download EICAR test data and create large fixture trees.
Do not silently run heavy benchmarks or downloads for an unrelated change.
A passing frontend build does not establish GUI interaction or end-to-end behavior.
Mocked integration tests do not establish behavior of real Docker/Nmap or daemon IPC.

## Critical scenario checklist

These are verification targets, not claims that coverage already exists. Locate
existing tests first and mark unsupported functionality as such.

| Scenario | Required observation |
| --- | --- |
| Daemon unavailable or disconnected | Clear failure in client; record pending-request and recovery behavior |
| Disabled, missing or failed plugin runner | Error reaches the client; assess impact on other plugins and outstanding calls |
| Invalid IPC / plugin arguments | Predictable rejection without unintended file operations or process termination |
| Scan interruption | Determine whether cancellation is supported; test its actual contract, or document the gap |
| Quarantine and restoration | Use harmless temporary files; check contents, metadata, collisions, failures and preservation of unrelated files |
| Cleaner dry-run and selection | Exact before/after fixture comparison; dry-run changes nothing, real deletion affects only authorized selected fixtures |
| Permission failure | Reproduce without elevated privileges; truthful error/partial-success reporting and unaffected files preserved |
| Nmap and Docker failures | Mock missing tools, process errors and malformed output; no real external scan or prune |
| Concurrent requests / history | Correct response association and persistence; no lost or misattributed results |

## Report results

For each check record: command or steps, environment, expected result, actual
result, and **passed / failed / blocked / not run**. Include reproduction evidence
for failures and explain environmental blockers without calling them product bugs.
Clearly separate tests already present, new tests added and proposed future tests.
For fixes, prefer a regression test that fails before and passes after the change
when practical. Do not weaken tests or modify unrelated code to obtain a green run.

## Example request

“Follow docs/ai/qa.md to verify Cleaner dry-run and selected-file deletion using
existing isolated tests. Report what was executed and what remains unverified.”
