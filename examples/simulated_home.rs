//! Example smart home wired to `socket_sim` / `thermo_sim` processes.
//!
//! Typical local demo (four terminals):
//!
//! ```text
//! cargo run --bin socket_sim -- 127.0.0.1:17001 --watts 200
//! cargo run --bin socket_sim -- 127.0.0.1:17002 --watts 40
//! cargo run --bin thermo_sim -- examples/thermo_kitchen.txt
//! cargo run --bin thermo_sim -- examples/thermo_living.txt
//! cargo run --example simulated_home
//! ```
//!
//! Environment overrides (optional):
//! - `SIM_SOCKET_FRIDGE`, `SIM_SOCKET_DESK` — TCP addresses for outlets
//! - `SIM_THERMO_KITCHEN_BIND`, `SIM_THERMO_LIVING_BIND` — UDP bind addresses for thermometers

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use smart_home::{Device, Power, SmartHome, Socket, Temperature, Thermometer, print_report_value};

fn env_addr(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn main() {
    let fridge_addr = env_addr("SIM_SOCKET_FRIDGE", "127.0.0.1:17001");
    let desk_addr = env_addr("SIM_SOCKET_DESK", "127.0.0.1:17002");
    let kitchen_bind = env_addr("SIM_THERMO_KITCHEN_BIND", "127.0.0.1:19101");
    let living_bind = env_addr("SIM_THERMO_LIVING_BIND", "127.0.0.1:19102");

    let mut errors: Vec<String> = Vec::new();

    let fridge_socket = match fridge_addr.parse() {
        Ok(addr) => match Socket::connect_tcp(
            "Fridge outlet".to_string(),
            addr,
            Power::new(200.0).unwrap(),
        ) {
            Ok(s) => Some(s),
            Err(e) => {
                errors.push(format!("Fridge TCP outlet ({fridge_addr}): {e}"));
                None
            }
        },
        Err(e) => {
            errors.push(format!("Fridge address ({fridge_addr}): {e}"));
            None
        }
    };

    let desk_socket = match desk_addr.parse() {
        Ok(addr) => match Socket::connect_tcp(
            "Desk lamp outlet".to_string(),
            addr,
            Power::new(40.0).unwrap(),
        ) {
            Ok(s) => Some(s),
            Err(e) => {
                errors.push(format!("Desk TCP outlet ({desk_addr}): {e}"));
                None
            }
        },
        Err(e) => {
            errors.push(format!("Desk address ({desk_addr}): {e}"));
            None
        }
    };

    let kitchen_thermo = match kitchen_bind.parse() {
        Ok(addr) => match Thermometer::bind_udp(
            "Kitchen sensor".to_string(),
            addr,
            Temperature::celsius(20.0),
        ) {
            Ok(t) => Some(t),
            Err(e) => {
                errors.push(format!(
                    "Kitchen UDP thermometer bind ({kitchen_bind}): {e}"
                ));
                None
            }
        },
        Err(e) => {
            errors.push(format!("Kitchen bind address ({kitchen_bind}): {e}"));
            None
        }
    };

    let living_thermo = match living_bind.parse() {
        Ok(addr) => match Thermometer::bind_udp(
            "Living room sensor".to_string(),
            addr,
            Temperature::celsius(20.0),
        ) {
            Ok(t) => Some(t),
            Err(e) => {
                errors.push(format!(
                    "Living room UDP thermometer bind ({living_bind}): {e}"
                ));
                None
            }
        },
        Err(e) => {
            errors.push(format!("Living room bind address ({living_bind}): {e}"));
            None
        }
    };

    // Give UDP senders time to deliver at least one datagram.
    thread::sleep(Duration::from_millis(600));

    let kitchen_thermo = kitchen_thermo.and_then(|t| {
        if t.is_udp() && !t.has_udp_reading() {
            errors.push(format!(
                "Kitchen thermometer ({kitchen_bind}): no UDP temperature received yet"
            ));
            None
        } else {
            Some(t)
        }
    });
    let living_thermo = living_thermo.and_then(|t| {
        if t.is_udp() && !t.has_udp_reading() {
            errors.push(format!(
                "Living room thermometer ({living_bind}): no UDP temperature received yet"
            ));
            None
        } else {
            Some(t)
        }
    });

    let mut kitchen_devices: HashMap<String, Device> = HashMap::new();
    if let Some(t) = kitchen_thermo {
        kitchen_devices.insert("kitchen_thermometer".to_string(), t.into());
    }
    if let Some(s) = fridge_socket {
        kitchen_devices.insert("fridge".to_string(), s.into());
    }

    let mut office_devices: HashMap<String, Device> = HashMap::new();
    if let Some(t) = living_thermo {
        office_devices.insert("living_thermometer".to_string(), t.into());
    }
    if let Some(s) = desk_socket {
        office_devices.insert("desk".to_string(), s.into());
    }

    let mut rooms = HashMap::new();
    if !kitchen_devices.is_empty() {
        rooms.insert(
            "Kitchen".to_string(),
            smart_home::Room::new("Kitchen".to_string(), kitchen_devices),
        );
    }
    if !office_devices.is_empty() {
        rooms.insert(
            "Office".to_string(),
            smart_home::Room::new("Office".to_string(), office_devices),
        );
    }

    let home = SmartHome::new("Simulated smart home".to_string(), rooms);

    println!("=== Simulated smart home report ===\n");
    if home.room_count() == 0 {
        println!("No devices were added (all connections/bindings failed).");
    } else {
        print_report_value(&home);
    }

    if errors.is_empty() {
        println!("\nNo transport-level errors while constructing devices.");
    } else {
        println!("\nTransport / data errors:");
        for e in &errors {
            println!("- {e}");
        }
    }
}
