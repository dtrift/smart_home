use std::collections::HashMap;

use crate::devices::Device;
use crate::error::SmartHomeError;
use crate::report::Report;

use super::room::Room;

/// Smart home: rooms stored in a map keyed by string identifiers.
#[derive(Debug, Clone, PartialEq)]
pub struct SmartHome {
    name: String,
    rooms: HashMap<String, Room>,
}

impl SmartHome {
    /// Creates a home with the given name and room map.
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{SmartHome, Room};
    /// use std::collections::HashMap;
    ///
    /// let home = SmartHome::new("My Home".to_string(), HashMap::new());
    /// assert_eq!(home.name(), "My Home");
    /// ```
    pub fn new(name: String, rooms: HashMap<String, Room>) -> Self {
        Self { name, rooms }
    }

    /// Room by key, if present.
    pub fn room(&self, key: &str) -> Option<&Room> {
        self.rooms.get(key)
    }

    /// Mutable room by key, if present.
    pub fn room_mut(&mut self, key: &str) -> Option<&mut Room> {
        self.rooms.get_mut(key)
    }

    /// Inserts or replaces a room under `key`. Returns the previous room, if any.
    pub fn insert_room(&mut self, key: String, room: Room) -> Option<Room> {
        self.rooms.insert(key, room)
    }

    /// Removes a room by key. Returns it if it existed.
    pub fn remove_room(&mut self, key: &str) -> Option<Room> {
        self.rooms.remove(key)
    }

    /// Resolves a device by room key and device key.
    ///
    /// Returns [`SmartHomeError`] indicating whether the room or the device was missing.
    pub fn device(&self, room_name: &str, device_name: &str) -> Result<&Device, SmartHomeError> {
        let room = self
            .rooms
            .get(room_name)
            .ok_or_else(|| SmartHomeError::RoomNotFound {
                room: room_name.to_string(),
            })?;
        room.device(device_name)
            .ok_or_else(|| SmartHomeError::DeviceNotFound {
                room: room_name.to_string(),
                device: device_name.to_string(),
            })
    }

    /// Same as [`SmartHome::device`] but mutable.
    pub fn device_mut(
        &mut self,
        room_name: &str,
        device_name: &str,
    ) -> Result<&mut Device, SmartHomeError> {
        let room = self
            .rooms
            .get_mut(room_name)
            .ok_or_else(|| SmartHomeError::RoomNotFound {
                room: room_name.to_string(),
            })?;
        room.device_mut(device_name)
            .ok_or_else(|| SmartHomeError::DeviceNotFound {
                room: room_name.to_string(),
                device: device_name.to_string(),
            })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Iterator over `(key, room)` pairs (arbitrary order).
    pub fn rooms(&self) -> impl Iterator<Item = (&String, &Room)> {
        self.rooms.iter()
    }
}

impl Report for SmartHome {
    fn report(&self) -> String {
        let mut lines = vec![format!("Smart home '{}':\n", self.name)];
        if self.rooms.is_empty() {
            lines.push("  (no rooms)\n".to_string());
        } else {
            let mut keys: Vec<&String> = self.rooms.keys().collect();
            keys.sort();
            for k in keys {
                if let Some(room) = self.rooms.get(k.as_str()) {
                    lines.push(room.report());
                }
            }
        }
        lines.push('\n'.to_string());
        lines.concat()
    }
}
