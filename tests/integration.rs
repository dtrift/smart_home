//! Integration tests: use only the public `smart_home` API (as a downstream crate would).

use smart_home::{Device, DeviceInfo, Power, Room, SmartHome, Socket, Temperature, Thermometer};

#[test]
fn full_home_workflow() {
    let kitchen_devices = vec![
        Device::Thermometer(Thermometer::new(
            "Kitchen".to_string(),
            Temperature::celsius(22.5),
        )),
        Device::Socket(Socket::new(
            "Kettle".to_string(),
            true,
            Power::new(1500.0).unwrap(),
        )),
    ];

    let living_room_devices = vec![Device::Socket(Socket::new(
        "TV".to_string(),
        true,
        Power::new(120.0).unwrap(),
    ))];

    let rooms = vec![
        Room::new("Kitchen".to_string(), kitchen_devices),
        Room::new("Living Room".to_string(), living_room_devices),
    ];

    let mut home = SmartHome::new("Test Home".to_string(), rooms);

    assert_eq!(home.room_count(), 2);
    assert_eq!(home.room(0).device_count(), 2);
    assert_eq!(home.room(1).device_count(), 1);

    let kitchen = home.room_mut(0);
    if let Device::Socket(socket) = kitchen.device_mut(1) {
        socket.turn_off();
    }

    let kitchen = home.room(0);
    if let Device::Socket(socket) = kitchen.device(1) {
        assert!(!socket.is_on());
        assert_eq!(socket.power().watts(), 0.0);
    }
}

#[test]
fn clone_and_equality() {
    let socket1 = Socket::new("Test".to_string(), true, Power::new(100.0).unwrap());
    let socket2 = socket1.clone();

    assert_eq!(socket1, socket2);

    let device1 = Device::Socket(socket1);
    let device2 = device1.clone();

    assert_eq!(device1, device2);
}

#[test]
fn room_with_multiple_devices() {
    let devices: Vec<Device> = (0..10)
        .map(|i| {
            Device::Socket(Socket::new(
                format!("Socket {}", i),
                true,
                Power::new(100.0 + i as f32).unwrap(),
            ))
        })
        .collect();

    let room = Room::new("Large Room".to_string(), devices);
    assert_eq!(room.device_count(), 10);

    let last_device = room.device(9);
    assert_eq!(last_device.name(), "Socket 9");
}

#[test]
fn empty_home_and_rooms() {
    let home = SmartHome::new("Empty Home".to_string(), vec![]);
    assert_eq!(home.room_count(), 0);

    let room = Room::new("Empty Room".to_string(), vec![]);
    assert_eq!(room.device_count(), 0);
}
