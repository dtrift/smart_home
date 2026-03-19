use smart_home::{Device, DeviceInfo, Power, Room, SmartHome, Socket, Temperature, Thermometer};

fn main() {
    println!("=== Smart Home Library Demo ===\n");

    // Create kitchen devices
    let kitchen_devices = vec![
        Device::Thermometer(Thermometer::new(
            "Kitchen thermometer".to_string(),
            Temperature::celsius(22.5),
        )),
        Device::Socket(Socket::new(
            "Kettle".to_string(),
            true,
            Power::new(1500.0).unwrap(),
        )),
        Device::Socket(Socket::new(
            "Fridge".to_string(),
            true,
            Power::new(200.0).unwrap(),
        )),
    ];

    // Create living room devices
    let living_room_devices = vec![
        Device::Thermometer(Thermometer::new(
            "Living room thermometer".to_string(),
            Temperature::celsius(24.0),
        )),
        Device::Socket(Socket::new(
            "TV".to_string(),
            true,
            Power::new(120.0).unwrap(),
        )),
        Device::Socket(Socket::new(
            "Floor lamp".to_string(),
            false,
            Power::new(60.0).unwrap(),
        )),
    ];

    // Create bedroom devices
    let bedroom_devices = vec![
        Device::Thermometer(Thermometer::new(
            "Bedroom thermometer".to_string(),
            Temperature::celsius(21.0),
        )),
        Device::Socket(Socket::new(
            "Humidifier".to_string(),
            true,
            Power::new(30.0).unwrap(),
        )),
    ];

    // Create rooms
    let rooms = vec![
        Room::new("Kitchen".to_string(), kitchen_devices),
        Room::new("Living Room".to_string(), living_room_devices),
        Room::new("Bedroom".to_string(), bedroom_devices),
    ];

    // Create smart home
    let mut home = SmartHome::new("My Smart Home".to_string(), rooms);

    // Print initial report
    println!("Initial home state:");
    home.print_report();

    // Turn off TV in living room (room index 1, device index 1)
    println!("Turning off TV in living room...\n");
    let living_room = home.room_mut(1);
    if let Device::Socket(socket) = living_room.device_mut(1) {
        socket.turn_off();
    }

    // Turn on floor lamp in living room (device index 2)
    println!("Turning on floor lamp in living room...\n");
    let living_room = home.room_mut(1);
    if let Device::Socket(socket) = living_room.device_mut(2) {
        socket.turn_on();
    }

    // Turn off kettle in kitchen (room index 0, device index 1)
    println!("Turning off kettle in kitchen...\n");
    let kitchen = home.room_mut(0);
    if let Device::Socket(socket) = kitchen.device_mut(1) {
        socket.turn_off();
    }

    // Print updated report
    println!("Updated home state:");
    home.print_report();

    // Demonstrate getting info about specific devices
    println!("=== Additional Info ===\n");

    // Get kitchen temperature
    let kitchen = home.room(0);
    if let Device::Thermometer(thermometer) = kitchen.device(0) {
        println!(
            "Kitchen temperature: {:.1}°C",
            thermometer.temperature().as_celsius()
        );
    }

    // Get fridge power consumption
    let kitchen = home.room(0);
    if let Device::Socket(socket) = kitchen.device(2) {
        println!("Fridge consumption: {:.1} W", socket.power().watts());
    }

    println!("\n=== DeviceInfo trait demo ===\n");

    // Demonstrate DeviceInfo trait usage
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
