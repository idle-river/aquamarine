# Aquamarine

Aquamarine is a small persistent key-value store written in Rust. It exposes a
TCP server, an interactive command-line client, and a core library containing
the storage engine and wire protocol.

The project is organized as a Cargo workspace:

| Crate | Purpose |
| --- | --- |
| `aquamarine-core` | In-memory key-value store, write-ahead log (WAL), packet types, and stream helpers |
| `aquamarine-server` | TCP server for serving the key-value store |
| `aquamarine-cli` | Interactive client for connecting to the server |

## Requirements

- Rust 1.85 or newer (the workspace uses the Rust 2024 edition)
- Cargo

The repository also includes a [devenv](https://devenv.sh/) configuration with
Rust, `rust-analyzer`, `clippy`, `rustfmt`, and the repository's formatting
hooks.

## Build and run

Build all workspace crates:

```shell
cargo build
```

Start the server in one terminal:

```shell
cargo run -p aquamarine-server
```

The server listens on `0.0.0.0:2946` and stores its WAL in
`./aquamarine.wal`. Start the CLI in another terminal:

```shell
cargo run -p aquamarine-cli
```

The CLI accepts an optional server address:

```shell
cargo run -p aquamarine-cli -- 127.0.0.1:2946
```

The default address is `127.0.0.1:2946`.

## CLI commands

At the `>` prompt, use:

```text
SET <key> <value>    Store or replace a value
GET <key>            Read a value
DEL <key>            Delete a key
DELETE <key>        Delete a key (alias for DEL)
EXIT                 Close the client
```

For example:

```text
> SET language rust
> GET language
rust
> DEL language
true
> GET language
None
```

Keys and values are whitespace-delimited by the current CLI, so values cannot
contain spaces.

## Public Rust API

Add the core crate as a workspace dependency:

```toml
[dependencies]
aquamarine-core = { path = "crates/core" }
```

### `KVStore`

`KVStore` is a persistent key-value store backed by a WAL:

```rust
use aquamarine_core::KVStore;

let mut store = KVStore::new("./example.wal");

store.set("language".to_owned(), "rust".to_owned())?;
assert_eq!(store.get("language".to_owned()), Some("rust".to_owned()));

store.delete("language".to_owned())?;
assert_eq!(store.get("language".to_owned()), None);
```

The methods are:

- `KVStore::new(path)` opens or creates the WAL and replays it into memory.
- `set(key, value)` appends a set operation and updates the in-memory map.
- `get(key)` returns a cloned value, or `None` when the key is absent.
- `delete(key)` appends a delete operation and removes the key if present.

`set` and `delete` return a boxed error type. `new` currently reports an
unopenable WAL by panicking, because it is intended to initialize a store
during application startup.

### Packet protocol

The public packet types are in `aquamarine_core::packets`:

```rust
use aquamarine_core::packets::{
    ClientPacket, Packet, ServerPacket, read_from_stream, write_to_stream,
};
```

Client requests are represented by:

- `ClientPacket::Set { key, value }`
- `ClientPacket::Get { key }`
- `ClientPacket::Delete { key }`

Server responses are represented by:

- `ServerPacket::SetRes`
- `ServerPacket::GetRes { value }`
- `ServerPacket::DeleteRes { deleted }`
- `ServerPacket::Error { message }`

`Packet` wraps client and server messages and also includes `Ping` and `Pong`.
The asynchronous `write_to_stream` and `read_from_stream` helpers work with
any `tokio::io::AsyncWrite` or `AsyncRead` implementation:

```rust
let request = Packet::Client(ClientPacket::Get {
    key: "language".to_owned(),
});

write_to_stream(&mut stream, request).await?;
let response: Packet = read_from_stream(&mut stream).await?;
```

## Wire format

Each packet is encoded as:

1. A four-byte, big-endian unsigned payload length.
2. A `postcard` serialization of the `Packet` value.

The length prefix allows the receiver to read exactly one packet from a TCP
stream. The serialization buffer used by `write_to_stream` is currently
limited to 1024 bytes, so packets whose encoded representation exceeds that
size cannot be sent by the helper.

## Internals

### Storage engine

`KVStore` keeps the current state in a `HashMap<String, String>` protected by
an `RwLock`. WAL writes are protected by a `Mutex<BufWriter<File>>`. Mutations
are written as serialized `Logline` values before the in-memory map is changed:

```text
Logline::Set    { key, value }
Logline::Delete { key }
```

When a store starts, it reads the WAL from the beginning and replays each
complete `Logline` in order. Later operations overwrite or remove earlier
ones, reconstructing the latest state.

### Server

The server binds to `0.0.0.0:2946`, creates one shared `KVStore`, and accepts
TCP connections in a loop. Each connection is handled by a Tokio task. The
shared store is wrapped in `Arc<RwLock<_>>`; reads use the read lock, while
sets and deletes use the write lock.

The server responds to `Ping` with `Pong` and ignores incoming `Pong` packets.
Unexpected server-to-server or client-to-client packet variants are treated as
unreachable states.

### CLI

The CLI opens one TCP connection, parses commands from standard input, sends
`Packet::Client` requests, and prints the corresponding server response. It
also answers a server `Ping` with `Pong`.

## Development

Run formatting and checks with Cargo:

```shell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets
```

There are currently no automated tests in the workspace. The WAL file is
ignored by Git; remove `aquamarine.wal` when you want to start with an empty
database.

## Current limitations

- There is no authentication or encryption; the TCP protocol should only be
  exposed on a trusted network.
- The server address and WAL path are currently fixed in the server binary.
- The CLI parser expects valid command arguments and does not provide a
  structured help command.
- WAL writes use a buffered writer and do not explicitly flush after each
  mutation, so callers should treat process or system failure during a write as
  a durability risk.
- There is no compaction, snapshotting, or maximum WAL size.
