//! Integration tests: use only the public `smart_home` API (as a downstream crate would).

use std::collections::HashMap;

use smart_home::{
    Device, DeviceInfo, Power, Report, Room, SmartHome, Socket, Temperature, Thermometer, room,
};

#[test]
fn full_home_workflow() {
    let kitchen = room!(
        "Kitchen",
        "kitchen_sensor" => Thermometer::new("Kitchen".to_string(), Temperature::celsius(22.5)),
        "kettle" => Socket::new("Kettle".to_string(), true, Power::new(1500.0).unwrap()),
    );

    let living_room = room!(
        "Living Room",
        "tv" => Socket::new("TV".to_string(), true, Power::new(120.0).unwrap()),
    );

    let mut rooms = HashMap::new();
    rooms.insert("Kitchen".to_string(), kitchen);
    rooms.insert("Living Room".to_string(), living_room);

    let mut home = SmartHome::new("Test Home".to_string(), rooms);

    assert_eq!(home.room_count(), 2);
    assert_eq!(home.room("Kitchen").unwrap().device_count(), 2);
    assert_eq!(home.room("Living Room").unwrap().device_count(), 1);

    if let Ok(Device::Socket(socket)) = home.device_mut("Kitchen", "kettle") {
        socket.turn_off();
    }

    if let Ok(Device::Socket(socket)) = home.device("Kitchen", "kettle") {
        assert!(!socket.is_on());
        assert_eq!(socket.power().watts(), 0.0);
    } else {
        panic!("expected kettle");
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
    let mut map = HashMap::new();
    for i in 0..10 {
        map.insert(
            format!("s{i}"),
            Device::Socket(Socket::new(
                format!("Socket {i}"),
                true,
                Power::new(100.0 + i as f32).unwrap(),
            )),
        );
    }

    let room = Room::new("Large Room".to_string(), map);
    assert_eq!(room.device_count(), 10);

    let last_device = room.device("s9").expect("s9");
    assert_eq!(last_device.name(), "Socket 9");
}

#[test]
fn empty_home_and_rooms() {
    let home = SmartHome::new("Empty Home".to_string(), HashMap::new());
    assert_eq!(home.room_count(), 0);

    let room = Room::new("Empty Room".to_string(), HashMap::new());
    assert_eq!(room.device_count(), 0);
}

#[test]
fn report_contains_expected_sections() {
    let r = room!(
        "Kitchen",
        "t" => Thermometer::new("T".to_string(), Temperature::celsius(20.0)),
    );
    let s = r.report();
    assert!(s.contains("Kitchen"));
    assert!(s.contains("20.0"));

    let mut rooms = HashMap::new();
    rooms.insert("Kitchen".to_string(), r);
    let home = SmartHome::new("H".to_string(), rooms);
    assert!(home.report().contains("Smart home 'H'"));
}
