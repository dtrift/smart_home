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
│   ├── thermometer.rs      # Thermometer
│   ├── socket.rs           # Socket
│   ├── device.rs           # Device enumeration, DeviceInfo trait
│   ├── room.rs             # Room
│   ├── smart_home.rs       # Smart home
│   └── types/
│       ├── mod.rs          # Re-exports Power, Temperature
│       ├── power.rs        # Power (watts)
│       └── temperature.rs  # Temperature (Celsius)
```

## Modules

### thermometer (Thermometer)

- Constructor accepting name and initial temperature
- Returns current temperature
- Returns thermometer name

### socket (Socket)

- Constructor accepting name, on/off state and power
- Turn on/off methods
- Ability to check current state
- Returns current power (0 when off)

### device (Device)

- Enumeration containing thermometer or socket
- Outputs device state message

### room (Room)

- Constructor accepting name and array of devices
- Get device reference by index
- Get mutable device reference by index
- Output report of all devices in the room

### smart_home (Smart Home)

- Constructor accepting name and array of rooms
- Get room reference by index
- Get mutable room reference by index
- Output report of all rooms

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
use smart_home::smart_home::SmartHome;
use smart_home::room::Room;
use smart_home::device::{Device, DeviceInfo};
use smart_home::socket::Socket;
use smart_home::thermometer::Thermometer;
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

- **Modular architecture**: Core types live in `src/*.rs`; `Power` and
  `Temperature` live under `src/types/`
- **Simple and clear names**: `Thermometer`, `Socket`, `Device`, `Room`,
  `SmartHome`, `Power`, `Temperature`
- **Re-exports**: `lib.rs` re-exports types and the `DeviceInfo` trait for
  convenient `use smart_home::...` imports
- **Panic on error**: When index is out of bounds, panics with a clear message
- **Documentation**: All public methods are documented with doc comments
