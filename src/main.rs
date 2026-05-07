use std::collections::HashMap;

use smart_home::{
    Device, DeviceInfo, Power, Room, SmartHome, SmartHomeError, Socket, Temperature, Thermometer,
    print_report_value, room,
};

fn main() {
    println!("=== Smart Home Library Demo ===\n");

    let kitchen = room!(
        "Kitchen",
        "kitchen_thermometer" =>
            Thermometer::new("Kitchen thermometer".to_string(), Temperature::celsius(22.5)),
        "kettle" => Socket::new("Kettle".to_string(), true, Power::new(1500.0).unwrap()),
        "fridge" => Socket::new("Fridge".to_string(), true, Power::new(200.0).unwrap()),
    );

    let living_room = room!(
        "Living Room",
        "living_thermometer" =>
            Thermometer::new("Living room thermometer".to_string(), Temperature::celsius(24.0)),
        "tv" => Socket::new("TV".to_string(), true, Power::new(120.0).unwrap()),
        "floor_lamp" =>
            Socket::new("Floor lamp".to_string(), false, Power::new(60.0).unwrap()),
    );

    let bedroom = room!(
        "Bedroom",
        "bedroom_thermometer" =>
            Thermometer::new("Bedroom thermometer".to_string(), Temperature::celsius(21.0)),
        "humidifier" => Socket::new("Humidifier".to_string(), true, Power::new(30.0).unwrap()),
    );

    let mut rooms = HashMap::new();
    rooms.insert("Kitchen".to_string(), kitchen);
    rooms.insert("Living Room".to_string(), living_room);
    rooms.insert("Bedroom".to_string(), bedroom);

    let mut home = SmartHome::new("My Smart Home".to_string(), rooms);

    println!("Initial home state:");
    print_report_value(&home);

    println!("Dynamic rooms: add 'Office', then remove 'Bedroom'\n");
    let office = room!(
        "Office",
        "office_thermometer" =>
            Thermometer::new("Office thermometer".to_string(), Temperature::celsius(23.0)),
    );
    home.insert_room("Office".to_string(), office);
    let removed_bedroom = home.remove_room("Bedroom");
    println!(
        "Removed bedroom (had {} devices): {:?}\n",
        removed_bedroom
            .as_ref()
            .map(Room::device_count)
            .unwrap_or(0),
        removed_bedroom.as_ref().map(Room::name)
    );

    println!("Dynamic devices: add socket to Kitchen, remove 'kettle' from Kitchen\n");
    if let Some(kitchen) = home.room_mut("Kitchen") {
        kitchen.insert_device(
            "desk_lamp".to_string(),
            Socket::new("Desk lamp".to_string(), true, Power::new(40.0).unwrap()).into(),
        );
        let kettle = kitchen.remove_device("kettle");
        println!("Removed kettle: {:?}\n", kettle.as_ref().map(Device::name));
    }

    println!("Turn off TV, turn on floor lamp (via keys):\n");
    if let Ok(Device::Socket(socket)) = home.device_mut("Living Room", "tv") {
        socket.turn_off();
    }
    if let Ok(Device::Socket(socket)) = home.device_mut("Living Room", "floor_lamp") {
        socket.turn_on();
    }

    println!("Updated home state:");
    print_report_value(&home);

    println!("=== Reports via print_report_value (home / room / device) ===\n");
    print_report_value(&home);
    if let Some(lr) = home.room("Living Room") {
        print_report_value(lr);
    }
    match home.device("Kitchen", "fridge") {
        Ok(dev) => print_report_value(dev),
        Err(e) => println!("{e}"),
    }

    println!("=== SmartHome::device error handling ===\n");
    for (room_key, dev_key) in [
        ("Unknown", "x"),
        ("Kitchen", "no_such_device"),
        ("Kitchen", "fridge"),
    ] {
        match home.device(room_key, dev_key) {
            Ok(d) => println!("OK {room_key}/{dev_key}: {}", d.state()),
            Err(e) => println!("Err {room_key}/{dev_key}: {e} ({e:?})"),
        }
    }

    println!("\n=== SmartHomeError as std::error::Error ===\n");
    let err: SmartHomeError = SmartHomeError::DeviceNotFound {
        room: "Kitchen".to_string(),
        device: "ghost".to_string(),
    };
    println!("display: {err}");
    println!("source: {:?}", std::error::Error::source(&err));

    println!("\n=== DeviceInfo trait demo ===\n");

    let devices: Vec<Box<dyn DeviceInfo>> = vec![
        Box::new(Thermometer::new(
            "Sensor 1".to_string(),
            Temperature::celsius(23.0),
        )),
        Box::new(Socket::new(
            "Socket 1".to_string(),
            true,
            Power::new(100.0).unwrap(),
        )),
    ];

    for device in &devices {
        println!("Device: {}", device.name());
        println!("State: {}", device.state());
        println!();
    }

    println!("=== Demo complete ===");
}
