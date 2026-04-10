use std::collections::HashMap;

use crate::devices::Device;
use crate::report::Report;

/// Room containing smart devices keyed by string identifiers.
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    name: String,
    devices: HashMap<String, Device>,
}

impl Room {
    /// Creates a room with the given display name and device map.
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Room, Device, Thermometer, Temperature};
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(
    ///     "t1".to_string(),
    ///     Device::Thermometer(Thermometer::new("Thermometer".to_string(), Temperature::celsius(22.0))),
    /// );
    /// let room = Room::new("Living Room".to_string(), map);
    /// assert_eq!(room.name(), "Living Room");
    /// ```
    pub fn new(name: String, devices: HashMap<String, Device>) -> Self {
        Self { name, devices }
    }

    /// Reference to a device by key, if present.
    pub fn device(&self, key: &str) -> Option<&Device> {
        self.devices.get(key)
    }

    /// Mutable reference to a device by key, if present.
    pub fn device_mut(&mut self, key: &str) -> Option<&mut Device> {
        self.devices.get_mut(key)
    }

    /// Inserts or replaces a device under `key`. Returns the previous device, if any.
    pub fn insert_device(&mut self, key: String, device: Device) -> Option<Device> {
        self.devices.insert(key, device)
    }

    /// Removes a device by key. Returns it if it existed.
    pub fn remove_device(&mut self, key: &str) -> Option<Device> {
        self.devices.remove(key)
    }

    /// Room display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of devices in the room.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Iterator over `(key, device)` pairs (arbitrary order).
    pub fn devices(&self) -> impl Iterator<Item = (&String, &Device)> {
        self.devices.iter()
    }
}

impl Report for Room {
    fn report(&self) -> String {
        let mut lines = vec![format!("  Room '{}':\n", self.name)];
        if self.devices.is_empty() {
            lines.push("    (no devices)\n".to_string());
        } else {
            let mut keys: Vec<&String> = self.devices.keys().collect();
            keys.sort();
            for k in keys {
                if let Some(d) = self.devices.get(k.as_str()) {
                    lines.push(format!("    {}", d.report()));
                }
            }
        }
        lines.concat()
    }
}
