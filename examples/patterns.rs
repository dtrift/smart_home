use std::cell::RefCell;
use std::rc::Rc;

use smart_home::{
    Device, DeviceInfo, HomeBuilder, Reporter, Room, Socket, Subscriber, Temperature, Thermometer,
};

#[derive(Default)]
struct MySubscriber {
    added: Rc<RefCell<Vec<String>>>,
}

impl MySubscriber {
    fn with_log(added: Rc<RefCell<Vec<String>>>) -> Self {
        Self { added }
    }
}

impl Subscriber for MySubscriber {
    fn on_event(&mut self, device: &Device) {
        self.added.borrow_mut().push(device.name().to_string());
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
    let subscriber_log = Rc::new(RefCell::new(Vec::new()));
    room.subscribe(MySubscriber::with_log(Rc::clone(&subscriber_log)));

    let closure_log = Rc::new(RefCell::new(Vec::new()));
    let closure_log_handle = Rc::clone(&closure_log);
    room.subscribe(move |device: &Device| {
        closure_log_handle
            .borrow_mut()
            .push(format!("closure: {}", device.name()));
    });

    room.insert_device("Socket_4".to_string(), Socket::default().into());
    room.insert_device("Thermo_3".to_string(), Thermometer::default().into());

    println!("\n=== Observer logs ===");
    println!("subscriber: {:?}", subscriber_log.borrow());
    println!("closure: {:?}", closure_log.borrow());

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
