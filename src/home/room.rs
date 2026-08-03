use std::collections::HashMap;
use std::fmt;

use crate::devices::Device;
use crate::report::Report;

/// Observer for device additions in a [`Room`].
///
/// Implementations must be [`Send`] + [`Sync`] so that a [`Room`] (and hence a
/// [`crate::SmartHome`]) can be shared across threads — required, for example,
/// to host the home behind an async web server.
pub trait Subscriber: Send + Sync {
    fn on_event(&mut self, device: &Device);
}

impl<F> Subscriber for F
where
    F: FnMut(&Device) + Send + Sync,
{
    fn on_event(&mut self, device: &Device) {
        self(device);
    }
}

/// Room containing smart devices keyed by string identifiers.
pub struct Room {
    name: String,
    devices: HashMap<String, Device>,
    subscribers: Vec<Box<dyn Subscriber>>,
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
        Self {
            name,
            devices,
            subscribers: Vec::new(),
        }
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
        self.notify_subscribers(&device);
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

    /// Subscribes to device-addition events for this room.
    pub fn subscribe<S>(&mut self, subscriber: S)
    where
        S: Subscriber + 'static,
    {
        self.subscribers.push(Box::new(subscriber));
    }

    fn notify_subscribers(&mut self, device: &Device) {
        for subscriber in &mut self.subscribers {
            subscriber.on_event(device);
        }
    }
}

impl Default for Room {
    fn default() -> Self {
        Self::new("Room".to_string(), HashMap::new())
    }
}

impl Clone for Room {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            devices: self.devices.clone(),
            subscribers: Vec::new(),
        }
    }
}

impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.devices == other.devices
    }
}

impl fmt::Debug for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Room")
            .field("name", &self.name)
            .field("devices", &self.devices)
            .field("subscriber_count", &self.subscribers.len())
            .finish()
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
