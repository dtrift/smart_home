use crate::devices::Device;

/// Room containing an array of smart devices
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    name: String,
    devices: Vec<Device>,
}

impl Room {
    /// Room constructor accepting an array of devices
    ///
    /// # Arguments
    ///
    /// * `name` - Room name
    /// * `devices` - Array of devices in the room
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Room, Device, Thermometer, Temperature};
    ///
    /// let devices = vec![
    ///     Device::Thermometer(Thermometer::new("Thermometer".to_string(), Temperature::celsius(22.0))),
    /// ];
    /// let room = Room::new("Living Room".to_string(), devices);
    /// assert_eq!(room.name(), "Living Room");
    /// ```
    pub fn new(name: String, devices: Vec<Device>) -> Self {
        Self { name, devices }
    }

    /// Get a reference to the device at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Device index
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Room, Device, Socket, Power, DeviceInfo};
    ///
    /// let devices = vec![
    ///     Device::Socket(Socket::new("TV".to_string(), true, Power::new(120.0).unwrap())),
    /// ];
    /// let room = Room::new("Living Room".to_string(), devices);
    /// let device = room.device(0);
    /// assert_eq!(device.name(), "TV");
    /// ```
    pub fn device(&self, index: usize) -> &Device {
        self.devices.get(index).expect("Device index out of bounds")
    }

    /// Get a mutable reference to the device at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Device index
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Room, Device, Socket, Power};
    ///
    /// let devices = vec![
    ///     Device::Socket(Socket::new("Lamp".to_string(), false, Power::new(60.0).unwrap())),
    /// ];
    /// let mut room = Room::new("Bedroom".to_string(), devices);
    /// let device = room.device_mut(0);
    /// if let Device::Socket(socket) = device {
    ///     socket.turn_on();
    /// }
    /// ```
    pub fn device_mut(&mut self, index: usize) -> &mut Device {
        self.devices
            .get_mut(index)
            .expect("Device index out of bounds")
    }

    /// Print a report of all devices in the room to stdout
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Room, Device, Thermometer, Temperature};
    ///
    /// let devices = vec![
    ///     Device::Thermometer(Thermometer::new("Sensor".to_string(), Temperature::celsius(20.0))),
    /// ];
    /// let room = Room::new("Kitchen".to_string(), devices);
    /// room.print_report();
    /// ```
    pub fn print_report(&self) {
        println!("  Room '{}':", self.name);
        if self.devices.is_empty() {
            println!("    (no devices)");
        } else {
            for device in &self.devices {
                print!("    ");
                device.print_state();
            }
        }
    }

    /// Returns the room name
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Room;
    ///
    /// let room = Room::new("Kitchen".to_string(), vec![]);
    /// assert_eq!(room.name(), "Kitchen");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of devices in the room
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Room, Device, Socket, Power};
    ///
    /// let devices = vec![
    ///     Device::Socket(Socket::new("S1".to_string(), true, Power::new(100.0).unwrap())),
    ///     Device::Socket(Socket::new("S2".to_string(), false, Power::new(50.0).unwrap())),
    /// ];
    /// let room = Room::new("Kitchen".to_string(), devices);
    /// assert_eq!(room.device_count(), 2);
    /// ```
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}
