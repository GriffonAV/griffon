# Griffon Plugin Guide (abi_stable)

## 0) Goal

This document explains how to **create or adapt a plugin** for Griffon.

A Griffon plugin is a **dynamic library** (Rust `cdylib`) exposing a stable ABI via `abi_stable`.
At runtime, Griffon loads the plugin and interacts with it through two exported functions:

- `init()` → returns plugin metadata (name, author, description, available functions…)
- `handle_message(msg)` → receives a command/message and returns a string response

## 1) The Plugin Interface (shared ABI)

The core interface is defined in the `interface` crate.
This is the ABI contract between the host (Griffon) and all plugins.

```rust
use abi_stable::StableAbi;
use abi_stable::library::RootModule;
use abi_stable::std_types::{RResult, RString, RVec, Tuple2};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = PluginRef)))]
pub struct PluginI {
    pub init: extern "C" fn() -> RResult<RVec<Tuple2<RString, RString>>, RString>,
    pub handle_message: extern "C" fn(RString) -> RString,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(default))]
pub struct PluginRoot {
    #[sabi(last_prefix_field)]
    pub plugin: PluginRef,
}

impl RootModule for PluginRoot_Ref {
    abi_stable::declare_root_module_statics! {PluginRoot_Ref}
    const BASE_NAME: &'static str = "Griffon_Plugin";
    const NAME: &'static str = "Griffon_Plugin";
    const VERSION_STRINGS: abi_stable::sabi_types::VersionStrings =
        abi_stable::package_version_strings!();
}
```

### Key points

- `#[repr(C)]` + `StableAbi` ensure ABI compatibility.
- `PluginI` is the function table (vtable) exposed by the plugin.
- `PluginRoot` is the root module exported by the plugin.

> **Always** use ABI-safe types: `RString`, `RVec`, `RResult`.



----

# Griffon Plugin Guide

This document explains **how to create/adapt a Griffon plugin** and **what you must modify** to make it work.
A Griffon plugin is a Rust **dynamic library** (`cdylib`) loaded at runtime by Griffon, exposing a stable ABI via `abi_stable`.

A plugin must provide:
- `init()` → returns metadata describing the plugin (and what it can do)
- `handle_message(msg)` → receives a command and returns a response

> The ABI boundary must use `abi_stable` types (`RString`, `RVec`, `RResult`, …).

## 1) What you MUST keep unchanged (ABI contract)

Your plugin must match the **interface crate** exactly.
You do **not** modify the host-side interface types in your plugin — you only *implement* the functions.

Things that must stay consistent:
- `#[repr(C)]` and `StableAbi` usage in the interface crate
- Function signatures
- ABI-safe types (`RString`, `RVec`, `RResult`, etc.)
- Exported root module (`get_library()`)

If the signatures differ, the plugin will fail to load.

## 2) Create a new plugin crate (what to set in Cargo.toml)

Your plugin is a dynamic library, so you must set:

```toml
[lib]
crate-type = ["cdylib"]
```

### Minimal dependencies :

- `interface` (shared ABI definitions)
- `abi_stable` (ABI-safe glue)

Example:
```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
interface = { workspace = true }
abi_stable = { workspace = true }
```


## 3) Implement `init()` (what to modify)
When a plugin is loaded by Griffon, the host will call `init()` immediately. This makes `init()` the right place to perform **one-time setup**, such as validating the environment, initializing global resources, preparing internal state, or returning metadata (name, author, supported commands). 
However, you should **not** assume that data created inside `init()` will still be available later in `handle_message()`: exported functions are **independent**, and local variables inside `init()` are dropped when the function returns. 

If you need to share state between calls (for example between `init()` and `handle_message()`), 
you must store it in a **persistent structure** such as a global static (e.g., `static` / `lazy_static`) protected by synchronization (`Mutex`, `RwLock`, atomics),
or another explicit shared-state mechanism.

### Goal
Return metadata so the host/UI can display:
- who wrote the plugin
- what it does
- which commands it supports

### What you MUST modify
Inside `init()` you should change:
- `name`
- `author`
- `description`
- list of supported commands (one or many)

### Suggested metadata schema (convention)
Use key/value pairs (`Tuple2<RString, RString>`) like:

- `name`: human-friendly plugin name
- `author`: author/team
- `description`: short description
- `version`: plugin version
- `functions`: list of commands supported (or multiple `function` entries)
- `category`: cleaner / monitor / yara / docker / etc.

Example structure (generic):

```rust
#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    let mut info = RVec::new();

    info.push(Tuple2(RString::from("name"), RString::from("<YOUR_PLUGIN_NAME>")));
    info.push(Tuple2(RString::from("author"), RString::from("<YOU_OR_TEAM>")));
    info.push(Tuple2(RString::from("description"), RString::from("<YOUR_PLUGIN_DESC>")));
    info.push(Tuple2(RString::from("version"), RString::from("0.1.0")));

    info.push(Tuple2(RString::from("function"), RString::from("<FN_1>/<FN_2>/<FN_3>")));


    RResult::ROk(info)
}
```

>The host can use these keys to show plugin cards, list functions, or generate help text.


## 4) Implement `handle_message()` (what to modify)

### Goal
This function receives a command string (ex: `fn:start`) and returns a response string.

### What you MUST modify
You must:
- decide which commands your plugin supports
- parse the incoming message format
- execute the requested action
- return the result

### Recommended conventions (generic)
Incoming message format:
- `fn:<command>`  
- optionally: `fn:<command> <arg1> <arg2> ...`

Responses:
- `RString` with `JSON` format

Generic example:

```rust
#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    let msg = msg.as_str();

    match msg {
        "fn:status" => RString::from("status=ok"),
        "fn:ping"  => ping(),
        "fn:print"   => {
            print();
            RString::from("print done")
        },
        _ => RString::from(format!("ERR unknown_command: {}", msg)),
    }
}
```


## 5) Export the plugin (what you MUST NOT forget)

### Goal
Expose your function pointers to the host loader.

### What you MUST do
Provide the root module export:

```rust
#[export_root_module]
pub fn get_library() -> PluginRoot_Ref {
    PluginRoot {
        plugin: PluginI {
            <FN_1>,
            <FN_2>,
            <FN_3>,
        }
        .leak_into_prefix(),
    }
    .leak_into_prefix()
}
```

## 6) Managing plugin state (what to modify depending on your needs)

Some plugins are **stateless** (just respond immediately).
Others run **continuous work** (watchers, monitors, background scans).

### Stateless plugin
No global variables, just compute and return in `handle_message()`.

### Background/continuous plugin (recommended pattern)
- a global flag (ex: `AtomicBool`) to control the loop
- a thread handle stored (mutex) to join safely on stop
- keep `handle_message()` quick and spawn long tasks

## 7) Common mistakes (what to avoid)

- Using `String`, `Vec`, `Result` across the ABI boundary  
  ✅ Use `RString`, `RVec`, `RResult`.

- Forgetting `crate-type = ["cdylib"]`  
  ✅ Required to produce a loadable `.so`.

- Panicking inside exported functions  
  ✅ Prefer returning `RErr(...)` or `ERR ...` strings.

- Changing the interface signatures  
  ✅ Don’t. Only implement them.

## 8) Checklist (generic)

- [ ] Create `plugins/<my_plugin>/`
- [ ] `Cargo.toml`: `crate-type = ["cdylib"]`
- [ ] Implement `init()` with real metadata
- [ ] Implement `handle_message()` with your commands
- [ ] Export `get_library()` (root module)
- [ ] Build plugin and verify `.so` exists
- [ ] Test with Griffon: list plugin + call at least one command
