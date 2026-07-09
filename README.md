# Smart Home

A library for smart home management in Rust, plus an FFI sub-project that
exposes a smart socket through a C ABI with static and dynamic linking demos.

## Project Structure

The repository is a Cargo workspace. The root package is the `smart_home`
library; an FFI sub-project lives under [`ffi/`](ffi).

```text
smart_home/
├── Cargo.toml                # root package + [workspace] members
├── README.md
├── src/
│   ├── lib.rs                # Main library file (re-exports, room! macro)
│   ├── main.rs               # Usage example
│   ├── error.rs              # SmartHomeError
│   ├── report.rs             # Report / ReportItems / Reporter traits
│   ├── wire.rs               # TCP/UDP framing helpers
│   ├── devices/
│   │   ├── mod.rs            # Re-exports device types
│   │   ├── device.rs         # Device enum, DeviceInfo trait
│   │   ├── socket.rs         # Socket (local + TCP)
│   │   └── thermometer.rs    # Thermometer (local + UDP)
│   ├── home/
│   │   ├── mod.rs            # Re-exports Room, SmartHome, HomeBuilder
│   │   ├── room.rs           # Room
│   │   ├── smart_home.rs     # Smart home
│   │   └── builder.rs        # HomeBuilder (type-state builder)
│   ├── types/
│   │   ├── mod.rs            # Re-exports Power, Temperature
│   │   ├── power.rs          # Power (watts)
│   │   └── temperature.rs    # Temperature (Celsius)
│   └── bin/
│       ├── socket_sim.rs     # Simulated smart socket (TCP server)
│       └── thermo_sim.rs     # Simulated thermometer (UDP server)
├── examples/
│   ├── patterns.rs           # Design-patterns demo
│   └── simulated_home.rs     # Home with simulated devices
├── tests/
│   └── integration.rs        # Integration tests (public API only)
└── ffi/                      # FFI sub-project (C ABI smart socket)
    ├── smart_socket_ffi/     # C ABI library
    ├── app_static/           # statically-linked demo
    └── app_dynamic/          # dynamically-linked demo
```

## Modules

### devices

- **`thermometer`**: constructor, current temperature, name
- **`socket`**: constructor, on/off, state, power (0 when off)
- **`device`**: `Device` enum (thermometer or socket), `DeviceInfo` trait,
  state output

### home

- **`room`**: constructor, device by key (ref / mut), insert/remove device,
  device count, room report, subscriber notifications
- **`smart_home`**: constructor, room by key (ref / mut), insert/remove room,
  device lookup by `(room, device)` returning `Result`, home report
- **`builder`**: `HomeBuilder` type-state builder (room/device then `build()`)

### types (`Power`, `Temperature`)

- **`Power`**: non-negative power in watts; used by `Socket`
- **`Temperature`**: Celsius (with Fahrenheit helpers); used by `Thermometer`

## Running

```bash
# Build the whole workspace (smart_home + FFI crates)
cargo build --workspace

# Run the smart_home demo
cargo run

# Run all tests (unit, integration, doctests) across the workspace
cargo test --workspace --all-features

# Build API documentation (rustdoc); use `cargo doc --open` to view in browser
cargo doc

# Lint (fail on warnings; same flags as CI)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format check (same as CI; run `cargo fmt --all` to apply formatting)
cargo fmt --all -- --check
```

CI (see [`.github/workflows/ci.yml`](.github/workflows/ci.yml)) runs the same
three steps: `cargo fmt --all -- --check`, `cargo clippy --workspace ...`, and
`cargo test --workspace --all-features`.

## Library Usage

Thanks to type re-exports from `lib.rs`, you can import types directly:

```rust
use std::collections::HashMap;

use smart_home::{Device, Power, Room, SmartHome, Socket, Temperature, Thermometer};

fn main() {
    // Create devices
    let mut devices = HashMap::new();
    devices.insert(
        "thermometer".to_string(),
        Device::Thermometer(Thermometer::new(
            "Thermometer".to_string(),
            Temperature::celsius(22.5),
        )),
    );
    devices.insert(
        "socket".to_string(),
        Device::Socket(Socket::new(
            "Socket".to_string(),
            true,
            Power::new(100.0).unwrap(),
        )),
    );

    // Create room
    let room = Room::new("Living Room".to_string(), devices);

    // Create smart home
    let mut rooms = HashMap::new();
    rooms.insert("Living Room".to_string(), room);
    let home = SmartHome::new("My Home".to_string(), rooms);

    // Print report
    home.print_report();
}
```

Or use the `room!` macro for brevity:

```rust
use smart_home::{room, Power, SmartHome, Socket, Temperature, Thermometer};
use std::collections::HashMap;

fn main() {
    let kitchen = room!(
        "Kitchen",
        "t" => Thermometer::new("Sensor".to_string(), Temperature::celsius(22.5)),
        "s" => Socket::new("Kettle".to_string(), true, Power::new(1500.0).unwrap()),
    );
    let mut rooms = HashMap::new();
    rooms.insert("Kitchen".to_string(), kitchen);
    let home = SmartHome::new("My Home".to_string(), rooms);
    home.print_report();
}
```

Or use full module paths:

```rust
use smart_home::home::smart_home::SmartHome;
use smart_home::home::room::Room;
use smart_home::devices::device::{Device, DeviceInfo};
use smart_home::devices::socket::Socket;
use smart_home::devices::thermometer::Thermometer;
use smart_home::types::{Power, Temperature};
```

## Usage Example

The example in `main.rs` demonstrates:

1. Creating a smart home with multiple rooms
2. Adding various devices (thermometers and sockets)
3. Printing initial home state report
4. Controlling devices (turning sockets on/off)
5. Printing updated report

## Testing

**Unit tests** live next to the library code (`src/lib.rs`, `src/types/*`,
`src/devices/*`, `src/home/*`, `src/wire.rs`). They cover thermometer/socket
behavior, rooms, smart home accessors, `DeviceInfo`, type helpers, and the
TCP/UDP wire framing.

**Integration tests** in [`tests/integration.rs`](tests/integration.rs) link
against the crate as a user would: only the public `smart_home::*` API.
Run them with `cargo test` or `cargo test --test integration`.

**FFI tests** live in [`ffi/smart_socket_ffi/src/lib.rs`](ffi/smart_socket_ffi/src/lib.rs)
and cover the C ABI: on/off cycle, rated power, null-handle safety, and cookie
rejection of foreign/corrupted handles.

## FFI Workspace (C ABI smart socket)

The workspace's FFI sub-project (under [`ffi/`](ffi)) exposes a smart socket
through a C ABI and demonstrates the two linking strategies.

```text
ffi/
├── smart_socket_ffi/     # C ABI library
│   ├── Cargo.toml        # crate-type = ["rlib", "staticlib", "cdylib"]
│   ├── src/lib.rs        # SmartSocket + #[no_mangle] extern "C" functions
│   └── smart_socket.h    # C header for the same API
├── app_static/           # links the library statically (rlib)
│   └── src/main.rs
└── app_dynamic/          # loads the cdylib at runtime (libloading)
    └── src/main.rs
```

### Library: `smart_socket_ffi`

A smart socket with an on/off switch and a rated power (watts). When off, the
reported power is `0`; when on, it equals the rated power.

The crate produces **three build artifacts** (see `crate-type` in
[`ffi/smart_socket_ffi/Cargo.toml`](ffi/smart_socket_ffi/Cargo.toml)):

| Artifact    | File                                         | Used by                                 |
|-------------|----------------------------------------------|-----------------------------------------|
| `rlib`      | `libsmart_socket_ffi.rlib`                   | Rust consumers (`app_static`)           |
| `staticlib` | `libsmart_socket_ffi.a`                      | C/C++ static linking                    |
| `cdylib`    | `libsmart_socket_ffi.so` / `.dylib` / `.dll` | Runtime dynamic loading (`app_dynamic`) |

C ABI (see [`ffi/smart_socket_ffi/smart_socket.h`](ffi/smart_socket_ffi/smart_socket.h)):

```c
SmartSocket *smart_socket_new(bool is_on, float power_watts);
void         smart_socket_free(SmartSocket *ptr);
void         smart_socket_turn_on(SmartSocket *ptr);
void         smart_socket_turn_off(SmartSocket *ptr);
bool         smart_socket_is_on(const SmartSocket *ptr);
float        smart_socket_power(const SmartSocket *ptr);
float        smart_socket_rated_power(const SmartSocket *ptr);
const char  *smart_socket_version(void);
```

Every entry point checks the handle for `NULL` **and** a prefix cookie
(`"SSOK"`): foreign, corrupted, or already-freed handles are rejected as
no-ops / defaults instead of dereferencing garbage. `smart_socket_free` also
stamps out the cookie, giving a best-effort guard against double-free. (This
cannot protect against pointers to unmapped memory — reading such a pointer
still segfaults — but it catches the common misuse cases.)

### Static linking: `app_static`

Depends on `smart_socket_ffi` as a Cargo path dependency. The `rlib` artifact
is linked into the binary at compile time — the library code lives inside the
executable, with no runtime dependency on a shared object.

### Dynamic linking: `app_dynamic`

Does **not** link `smart_socket_ffi` at compile time. Instead it uses
[`libloading`](https://crates.io/crates/libloading) to load the `cdylib`
artifact (`libsmart_socket_ffi.so`) at startup and resolves the C ABI symbols
through function pointers — the Rust equivalent of C's `dlopen`/`dlsym`.

### Running the FFI examples

```bash
# Build everything (library + both apps + all artifacts)
cargo build --workspace

# Run the statically-linked app
cargo run -p app_static

# Run the dynamically-linked app (loads the cdylib at runtime)
cargo run -p app_dynamic
```

## Implementation Details

- **Modular architecture**: `devices/` and `home/` group domain types;
  `Power` and `Temperature` live under `src/types/`
- **Simple and clear names**: `Thermometer`, `Socket`, `Device`, `Room`,
  `SmartHome`, `Power`, `Temperature`
- **Re-exports**: `lib.rs` re-exports types and the `DeviceInfo` trait for
  convenient `use smart_home::...` imports
- **No panics on lookup**: `room.device(key)` / `home.room(key)` return `Option`,
  and `home.device(room, device)` returns `Result<_, SmartHomeError>` — callers
  handle missing rooms/devices explicitly
- **FFI hardening**: the `smart_socket_ffi` crate validates a prefix cookie on
  every C ABI entry point, rejecting foreign/corrupted/freed handles
  (best-effort) in addition to `NULL`
- **Documentation**: All public methods are documented with doc comments
