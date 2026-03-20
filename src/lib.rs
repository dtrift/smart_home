pub mod devices;
pub mod home;
pub mod types;

// Re-exports for convenience
pub use devices::{Device, DeviceInfo, Socket, Thermometer};
pub use home::{Room, SmartHome};
pub use types::{Power, Temperature};

#[cfg(test)]
mod tests {
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
        let devices = vec![
            Device::Thermometer(Thermometer::new(
                "T1".to_string(),
                Temperature::celsius(20.0),
            )),
            Device::Socket(Socket::new(
                "S1".to_string(),
                true,
                Power::new(50.0).unwrap(),
            )),
        ];
        let room = Room::new("Kitchen".to_string(), devices);

        assert_eq!(room.name(), "Kitchen");

        // Check device retrieval by index
        let device0 = room.device(0);
        match device0 {
            Device::Thermometer(t) => assert_eq!(t.name(), "T1"),
            _ => panic!("Expected thermometer"),
        }

        let device1 = room.device(1);
        match device1 {
            Device::Socket(s) => assert_eq!(s.name(), "S1"),
            _ => panic!("Expected socket"),
        }
    }

    #[test]
    fn test_room_get_device_mut() {
        let devices = vec![Device::Socket(Socket::new(
            "S1".to_string(),
            true,
            Power::new(50.0).unwrap(),
        ))];
        let mut room = Room::new("Bedroom".to_string(), devices);

        // Modify device through mutable reference
        let device = room.device_mut(0);
        match device {
            Device::Socket(s) => {
                s.turn_off();
            }
            _ => panic!("Expected socket"),
        }

        // Verify changes persisted
        let device = room.device(0);
        match device {
            Device::Socket(s) => assert!(!s.is_on()),
            _ => panic!("Expected socket"),
        }
    }

    #[test]
    #[should_panic(expected = "Device index out of bounds")]
    fn test_room_get_device_out_of_bounds() {
        let room = Room::new("Room".to_string(), vec![]);
        room.device(0);
    }

    #[test]
    fn test_smart_home_get_room() {
        let rooms = vec![
            Room::new("Living Room".to_string(), vec![]),
            Room::new("Kitchen".to_string(), vec![]),
        ];
        let home = SmartHome::new("My Home".to_string(), rooms);

        assert_eq!(home.name(), "My Home");
        assert_eq!(home.room(0).name(), "Living Room");
        assert_eq!(home.room(1).name(), "Kitchen");
    }

    #[test]
    fn test_smart_home_get_room_mut() {
        let rooms = vec![Room::new("Living Room".to_string(), vec![])];
        let mut home = SmartHome::new("My Home".to_string(), rooms);

        let room = home.room_mut(0);
        assert_eq!(room.name(), "Living Room");
    }

    #[test]
    #[should_panic(expected = "Room index out of bounds")]
    fn test_smart_home_get_room_out_of_bounds() {
        let home = SmartHome::new("Home".to_string(), vec![]);
        home.room(0);
    }

    #[test]
    fn test_device_print_state() {
        // Test doesn't verify output, but ensures the method runs without panic
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
}
