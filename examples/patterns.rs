use std::sync::{Arc, Mutex};

use smart_home::{
    Device, DeviceInfo, HomeBuilder, Reporter, Room, Socket, Subscriber, Temperature, Thermometer,
};

#[derive(Default)]
struct MySubscriber {
    added: Arc<Mutex<Vec<String>>>,
}

impl MySubscriber {
    fn with_log(added: Arc<Mutex<Vec<String>>>) -> Self {
        Self { added }
    }
}

impl Subscriber for MySubscriber {
    fn on_event(&mut self, device: &Device) {
        self.added.lock().unwrap().push(device.name().to_string());
    }
}

fn main() {
    let home = HomeBuilder::new()
        .add_room("First room")
        .add_device("Socket_1", Socket::default())
        .add_device("Socket_2", Socket::default())
        .add_device("Thermo_1", Thermometer::default())
        .add_room("Second room")
        .add_device("Socket_3", Socket::default())
        .add_device(
            "Thermo_2",
            Thermometer::new("Thermo_2".to_string(), Temperature::celsius(24.0)),
        )
        .build();

    println!("=== HomeBuilder report ===");
    Reporter::new().add(&home).report();

    let mut room = Room::default();
    let subscriber_log = Arc::new(Mutex::new(Vec::new()));
    room.subscribe(MySubscriber::with_log(Arc::clone(&subscriber_log)));

    let closure_log = Arc::new(Mutex::new(Vec::new()));
    let closure_log_handle = Arc::clone(&closure_log);
    room.subscribe(move |device: &Device| {
        closure_log_handle
            .lock()
            .unwrap()
            .push(format!("closure: {}", device.name()));
    });

    room.insert_device("Socket_4".to_string(), Socket::default().into());
    room.insert_device("Thermo_3".to_string(), Thermometer::default().into());

    println!("\n=== Observer logs ===");
    println!("subscriber: {:?}", subscriber_log.lock().unwrap());
    println!("closure: {:?}", closure_log.lock().unwrap());

    let device = Device::default();
    let socket1 = Socket::default();
    let socket2 = Socket::default();
    let thermo1 = Thermometer::default();
    let thermo2 = Thermometer::new("Thermo".to_string(), Temperature::celsius(21.5));

    println!("\n=== Reporter composite ===");
    Reporter::new()
        .add(&room)
        .add(&device)
        .add(&socket1)
        .add(&socket2)
        .add(&thermo1)
        .add(&thermo2)
        .report();
}
