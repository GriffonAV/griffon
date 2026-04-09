# Griffon IPC Protocol (Frame + CBOR)

This document describes the IPC protocol used between Griffon components (e.g., plugin manager ↔ runner).
The protocol is a **binary framed transport** with a fixed-size header and a variable-size payload.
Payloads are encoded in **CBOR** using `serde_cbor`.

## Scope / Where this protocol is used

This IPC protocol is **only** used for communication with the **Plugin Manager**.

In practice, it defines how Griffon components exchange messages such as `Hello`, `Call`, and `Result` **over the Plugin Manager communication channel** (for example through a Unix stream like a `socketpair`).

It is **not** intended as a general-purpose protocol for all Griffon modules, nor for external APIs. Any other subsystem (HTTP API, GUI, daemon internal buses, etc.) should use its own dedicated communication format.

## 1) High-level overview

The protocol is built on two layers:

1) **Frame layer (binary)**
- Ensures message boundaries on a raw stream (`Read`/`Write`)
- Validates magic, version, type, payload length
- Carries a `request_id` for correlating responses

2) **Message layer (typed)**
- Maps frames to typed Rust enums/structs (`Message`, `CallPayload`, etc.)
- Serializes/deserializes payloads using CBOR (`serde_cbor`)

## 2) Constants and limits

The frame format includes these constants:

- `MAGIC = 0xBEEF`  
  Used to quickly detect that the stream is aligned on valid protocol data.

- `VERSION = 1`  
  Protocol version. Both sides must use the same version.

- `HEADER_LEN = 12` bytes  
  Fixed header size.

- `MAX_PAYLOAD = 1MB`  
  Hard payload cap to prevent memory abuse or corrupted stream allocations.

## 3) Frame format (binary header)

Each message is transferred as:

- **Header (12 bytes)** + **Payload (N bytes)**

Header layout (big-endian):

| Offset | Size | Field       | Type  | Notes |
|-------:|-----:|-------------|-------|------|
| 0      | 2    | magic       | u16   | Must equal `0xBEEF` |
| 2      | 1    | version     | u8    | Must equal `1` |
| 3      | 1    | type        | u8    | One of `MsgType` |
| 4      | 4    | request_id  | u32   | Correlation id |
| 8      | 4    | payload_len | u32   | Payload length in bytes |

Payload:
- Raw bytes (`payload_len` bytes)
- For most message types, payload is CBOR.

## 4) MsgType values

`MsgType` is a `u8` enum:

- `1`  → `Hello`
- `2`  → `HelloOk`
- `3`  → `Call`
- `4`  → `Result`
- `5`  → `Log` (reserved / probably gonna be remove)
- `6`  → `Heartbeat`
- `7`  → `Error`

Unknown types are rejected as invalid data.

## 5) Request correlation (`request_id`)

The protocol supports request/response matching using `request_id`.

Rules:
- `Call` frames carry a non-zero `request_id` chosen by the caller.
- `Result` and `Error` frames reuse the **same** `request_id` as the corresponding `Call`.
- `Hello`, `HelloOk`, and `Heartbeat` use `request_id = 0` (no correlation).

This allows multiple in-flight calls over a single stream.

## 6) Serialization: CBOR payloads

Payloads are encoded with CBOR:

- Encode: `serde_cbor::to_vec(payload_struct)`
- Decode: `serde_cbor::from_slice::<T>(&bytes)`

CBOR is used for:
- `HelloOkPayload`
- `CallPayload`
- `ResultPayload`
- `ErrorPayload`

Messages without payload:
- `Hello` (empty payload)
- `Heartbeat` (empty payload)

## 7) Message types (typed layer)

The `Message` enum represents the protocol at a higher level:

- `Hello`
- `HelloOk(HelloOkPayload)`
- `Call { request_id, data: CallPayload }`
- `Result { request_id, data: ResultPayload }`
- `Error { request_id, data: ErrorPayload }`
- `Heartbeat`

## 8) Payload schemas

### 8.1) HelloOkPayload
Sent after a successful handshake to advertise capabilities.

```rust
pub struct HelloOkPayload {
    pub name: String,
    pub functions: Vec<String>,
}
```

Meaning:
- `name`: component identifier (for example the runner name or the plugin name)
- `functions`: list of callable function names exposed to the host

---

### 8.2) CallPayload
Represents a request to execute a function with arguments.

```rust
pub struct CallPayload {
    pub fn_name: String,
    pub args: Vec<String>,
}
```

Meaning:
- `fn_name`: function identifier (for example `"start"`, `"status"`, `"scan"`)
- `args`: ordered string arguments (interpretation is up to the callee)

---

### 8.3) ResultPayload
Represents the result of a call.

```rust
pub struct ResultPayload {
    pub ok: bool,
    pub output: String,
}
```

Meaning:
- `ok`: `true` if the call succeeded, `false` if it failed at the application level
- `output`: textual output (can be a message, logs, JSON string, etc.)

---

### 8.4) ErrorPayload
Represents a protocol-level error (invalid request, decoding failure, unsupported message, etc.)

```rust
pub struct ErrorPayload {
    pub code: u32,
    pub message: String,
}
```

Meaning:
- `code`: numeric error code (your convention to define)
- `message`: human-readable error description


## 9) Encoding a message into a frame

`Message::into_frame()` converts a `Message` into a `Frame`:

- `Hello` → type `Hello`, `request_id = 0`, empty payload
- `Heartbeat` → type `Heartbeat`, `request_id = 0`, empty payload
- `HelloOk(p)` → type `HelloOk`, `request_id = 0`, CBOR(payload)
- `Call{rid, data}` → type `Call`, `request_id = rid`, CBOR(payload)
- `Result{rid, data}` → type `Result`, `request_id = rid`, CBOR(payload)
- `Error{rid, data}` → type `Error`, `request_id = rid`, CBOR(payload)

## 10) Sending and receiving on a stream

The transport layer is any `Write`/`Read` stream (Unix socket, socketpair, pipe, etc.).

### Sending
`send_message(w, msg)`:
1) Convert `Message` → `Frame`
2) Write header (12 bytes)
3) Write payload (N bytes)

### Receiving
`recv_message(r)`:
1) Read header (12 bytes)
2) Validate magic, version, type
3) Read payload (N bytes)
4) Decode frame → typed `Message`

## 11) Validation and security checks

The frame parser enforces:
- correct `MAGIC`
- correct `VERSION`
- known `MsgType`
- payload length <= `MAX_PAYLOAD`

Invalid data results in `io::ErrorKind::InvalidData`.

This prevents:
- desync due to garbage bytes
- allocating extremely large payload buffers
- accepting unknown frame types

## 12) Handshake flow 

Typical flow when a connection is established between the `runner` and the `plugin manager`:

1) Plugin Manager → `Hello`
2) Runner → `HelloOk { name, functions }`

After this:
- PM can send `Call` frames to request operations
- Runner replies with `Result` or `Error` using the same `request_id`
- Either side may send `Heartbeat` periodically to detect dead connections

## 13) Call / Result flow (example)

Example sequence:

1) Host sends:
- `Call { request_id = 42, fn_name = "status", args = [] }`

2) Peer replies:
- `Result { request_id = 42, ok = true, output = "running=true" }`

If something fails:
- `Error { request_id = 42, code = 1001, message = "unknown function" }`

## 14) Common errors and debugging tips

- **"bad magic"**: stream is desynchronized or not using this protocol
- **"bad version"**: components compiled with different protocol versions
- **"bad type"**: unknown `MsgType` value
- **"payload too large"**: corrupted length field or attacker input
- **CBOR decode error**: wrong payload type for the given `MsgType`, or schema mismatch
