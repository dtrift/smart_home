//! Smart home library: rooms, devices (thermometers and sockets), and reports.
//!
//! # Modules
//!
//! - [`devices`] — [`Thermometer`], [`Socket`], [`Device`], [`DeviceInfo`]
//! - [`home`] — [`Room`], [`SmartHome`]
//! - [`types`] — [`Power`], [`Temperature`]
//! - [`report`] — [`Report`]
//! - [`error`] — [`SmartHomeError`]
//!
//! Types are re-exported at the crate root.
//!
//! # Example
//!
//! ```
//! use smart_home::{
//!     room, Device, Room, SmartHome, Socket, Thermometer, Temperature, Power,
//! };
//! use std::collections::HashMap;
//!
//! let r = room!(
//!     "Kitchen",
//!     "t" => Thermometer::new("Kitchen sensor".to_string(), Temperature::celsius(20.0)),
//!     "s" => Socket::new("Kettle".to_string(), false, Power::new(10.0).unwrap()),
//! );
//! let mut rooms = HashMap::new();
//! rooms.insert("Kitchen".to_string(), r);
//! let _home = SmartHome::new("Home".to_string(), rooms);
//! ```

pub mod devices;
pub mod error;
pub mod home;
pub mod report;
pub mod types;

pub use devices::{Device, DeviceInfo, Socket, Thermometer};
pub use error::SmartHomeError;
pub use home::{Room, SmartHome};
pub use report::Report;
pub use types::{Power, Temperature};

/// Prints a report for any type implementing [`Report`].
pub fn print_report_value<T: Report>(item: &T) {
    item.print_report();
}

/// Builds a [`Room`] from a name and `key => socket_or_thermometer` pairs.
///
/// Uses `=>` so device expressions may contain commas (e.g. `Thermometer::new(..., ...)`).
#[macro_export]
macro_rules! room {
    ($name:expr $(, $key:expr => $dev:expr)* $(,)?) => {{
        let mut map = ::std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $crate::Device::from($dev));
        )*
        $crate::Room::new($name.to_string(), map)
    }};
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_thermometer() {
        let thermometer = Thermometer::new("Thermometer1".to_string(), Temperature::celsius(23.5));
        assert_eq!(thermometer.name(), "Thermometer1");
        assert!((thermometer.temperature().as_celsius() - 23.5).abs() < 0.01);
    }

    #[test]
    fn test_socket_on() {
        let mut socket = Socket::new("Socket1".to_string(), false, Power::new(100.0).unwrap());
        assert_eq!(socket.name(), "Socket1");
        assert!(!socket.is_on());
        assert!((socket.power().watts() - 0.0).abs() < 0.01);

        socket.turn_on();
        assert!(socket.is_on());
        assert!((socket.power().watts() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_socket_off() {
        let mut socket = Socket::new("Socket2".to_string(), true, Power::new(150.0).unwrap());
        assert!(socket.is_on());
        assert!((socket.power().watts() - 150.0).abs() < 0.01);

        socket.turn_off();
        assert!(!socket.is_on());
        assert!((socket.power().watts() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_room_get_device() {
        let room = room!(
            "Kitchen",
            "t1" => Thermometer::new("T1".to_string(), Temperature::celsius(20.0)),
            "s1" => Socket::new("S1".to_string(), true, Power::new(50.0).unwrap()),
        );

        assert_eq!(room.name(), "Kitchen");

        let device0 = room.device("t1").expect("t1");
        match device0 {
            Device::Thermometer(t) => assert_eq!(t.name(), "T1"),
            _ => panic!("Expected thermometer"),
        }

        let device1 = room.device("s1").expect("s1");
        match device1 {
            Device::Socket(s) => assert_eq!(s.name(), "S1"),
            _ => panic!("Expected socket"),
        }
    }

    #[test]
    fn test_room_get_device_mut() {
        let mut room = room!(
            "Bedroom",
            "s1" => Socket::new("S1".to_string(), true, Power::new(50.0).unwrap()),
        );

        let device = room.device_mut("s1").expect("s1");
        match device {
            Device::Socket(s) => {
                s.turn_off();
            }
            _ => panic!("Expected socket"),
        }

        let device = room.device("s1").expect("s1");
        match device {
            Device::Socket(s) => assert!(!s.is_on()),
            _ => panic!("Expected socket"),
        }
    }

    #[test]
    fn test_room_get_device_missing() {
        let room = Room::new("Room".to_string(), HashMap::new());
        assert!(room.device("x").is_none());
    }

    #[test]
    fn test_smart_home_get_room() {
        let mut rooms = HashMap::new();
        rooms.insert(
            "Living Room".to_string(),
            Room::new("Living Room".to_string(), HashMap::new()),
        );
        rooms.insert(
            "Kitchen".to_string(),
            Room::new("Kitchen".to_string(), HashMap::new()),
        );
        let home = SmartHome::new("My Home".to_string(), rooms);

        assert_eq!(home.name(), "My Home");
        assert_eq!(home.room("Living Room").expect("lr").name(), "Living Room");
        assert_eq!(home.room("Kitchen").expect("k").name(), "Kitchen");
    }

    #[test]
    fn test_smart_home_get_room_mut() {
        let mut rooms = HashMap::new();
        let devices = HashMap::new();
        rooms.insert(
            "Living Room".to_string(),
            Room::new("Living Room".to_string(), devices),
        );
        let mut home = SmartHome::new("My Home".to_string(), rooms);

        let room = home.room_mut("Living Room").expect("lr");
        assert_eq!(room.name(), "Living Room");
    }

    #[test]
    fn test_smart_home_get_room_missing() {
        let home = SmartHome::new("Home".to_string(), HashMap::new());
        assert!(home.room("nope").is_none());
    }

    #[test]
    fn test_device_resolve() {
        let mut rooms = HashMap::new();
        rooms.insert(
            "Kitchen".to_string(),
            room!(
                "Kitchen",
                "kettle" => Socket::new("K".to_string(), true, Power::new(1.0).unwrap()),
            ),
        );
        let home = SmartHome::new("H".to_string(), rooms);
        assert!(home.device("Kitchen", "kettle").is_ok());
        assert_eq!(
            home.device("Bad", "kettle").unwrap_err(),
            SmartHomeError::RoomNotFound {
                room: "Bad".to_string()
            }
        );
        assert_eq!(
            home.device("Kitchen", "missing").unwrap_err(),
            SmartHomeError::DeviceNotFound {
                room: "Kitchen".to_string(),
                device: "missing".to_string()
            }
        );
    }

    #[test]
    fn test_device_print_state() {
        let thermometer = Device::Thermometer(Thermometer::new(
            "T1".to_string(),
            Temperature::celsius(22.5),
        ));
        thermometer.print_state();

        let socket = Device::Socket(Socket::new(
            "S1".to_string(),
            true,
            Power::new(100.0).unwrap(),
        ));
        socket.print_state();
    }

    #[test]
    fn test_device_info_trait() {
        let thermometer = Thermometer::new("Kitchen".to_string(), Temperature::celsius(22.0));
        assert_eq!(thermometer.name(), "Kitchen");
        let state = thermometer.state();
        assert!(state.contains("22.0"));

        let socket = Socket::new("TV".to_string(), true, Power::new(120.0).unwrap());
        assert_eq!(socket.name(), "TV");
        let state = socket.state();
        assert!(state.contains("on"));
    }

    #[test]
    fn test_power_validation() {
        let valid_power = Power::new(100.0);
        assert!(valid_power.is_ok());

        let invalid_power = Power::new(-10.0);
        assert!(invalid_power.is_err());
    }

    #[test]
    fn test_temperature_conversions() {
        let temp_c = Temperature::celsius(0.0);
        assert!((temp_c.as_fahrenheit() - 32.0).abs() < 0.01);

        let temp_f = Temperature::fahrenheit(32.0);
        assert!((temp_f.as_celsius() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_from_socket_thermometer_to_device() {
        let t: Device = Thermometer::new("x".to_string(), Temperature::celsius(1.0)).into();
        assert!(matches!(t, Device::Thermometer(_)));
        let s: Device = Socket::new("y".to_string(), false, Power::new(1.0).unwrap()).into();
        assert!(matches!(s, Device::Socket(_)));
    }

    #[test]
    fn test_report_trait() {
        let room = room!(
            "R",
            "a" => Thermometer::new("T".to_string(), Temperature::celsius(1.0)),
        );
        assert!(room.report().contains("Room 'R'"));
        let mut rooms = HashMap::new();
        rooms.insert("R".to_string(), room);
        let home = SmartHome::new("H".to_string(), rooms);
        assert!(home.report().contains("Smart home 'H'"));
    }
}
