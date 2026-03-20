# Smart Home

A library for smart home management in Rust.

## Project Structure

The project is organized as a modular structure:

```text
smart_home/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # Main library file (re-exports)
│   ├── main.rs             # Usage example
│   ├── devices/
│   │   ├── mod.rs          # Re-exports device types
│   │   ├── device.rs       # Device enum, DeviceInfo trait
│   │   ├── socket.rs       # Socket
│   │   └── thermometer.rs  # Thermometer
│   ├── home/
│   │   ├── mod.rs          # Re-exports Room, SmartHome
│   │   ├── room.rs         # Room
│   │   └── smart_home.rs   # Smart home
│   └── types/
│       ├── mod.rs          # Re-exports Power, Temperature
│       ├── power.rs        # Power (watts)
│       └── temperature.rs  # Temperature (Celsius)
```

## Modules

### devices

- **`thermometer`**: constructor, current temperature, name
- **`socket`**: constructor, on/off, state, power (0 when off)
- **`device`**: `Device` enum (thermometer or socket), `DeviceInfo` trait,
  state output

### home

- **`room`**: constructor, device by index (ref / mut), room report
- **`smart_home`**: constructor, room by index (ref / mut), home report

### types (`Power`, `Temperature`)

- **`Power`**: non-negative power in watts; used by `Socket`
- **`Temperature`**: Celsius (with Fahrenheit helpers); used by `Thermometer`

## Running

```bash
# Build the project
cargo build

# Run the example
cargo run

# Run tests
cargo test

# Build API documentation (rustdoc); use `cargo doc --open` to view in browser
cargo doc

# Check code
cargo clippy

# Format code
cargo fmt
```

## Library Usage

Thanks to type re-exports from `lib.rs`, you can import types directly:

```rust
use smart_home::{SmartHome, Room, Device, Socket, Thermometer, Temperature, Power};

fn main() {
    // Create devices
    let devices = vec![
        Device::Thermometer(Thermometer::new(
            "Thermometer".to_string(),
            Temperature::celsius(22.5),
        )),
        Device::Socket(Socket::new(
            "Socket".to_string(),
            true,
            Power::new(100.0).unwrap(),
        )),
    ];

    // Create room
    let room = Room::new("Living Room".to_string(), devices);

    // Create smart home
    let home = SmartHome::new("My Home".to_string(), vec![room]);

    // Print report
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

The library contains 10+ unit tests that verify:

- Thermometer creation and operation
- Socket on/off switching
- Device retrieval by index
- Panic when index is out of bounds
- Room and home operations

## Implementation Details

- **Modular architecture**: `devices/` and `home/` group domain types;
  `Power` and `Temperature` live under `src/types/`
- **Simple and clear names**: `Thermometer`, `Socket`, `Device`, `Room`,
  `SmartHome`, `Power`, `Temperature`
- **Re-exports**: `lib.rs` re-exports types and the `DeviceInfo` trait for
  convenient `use smart_home::...` imports
- **Panic on error**: When index is out of bounds, panics with a clear message
- **Documentation**: All public methods are documented with doc comments
