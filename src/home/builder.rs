use std::collections::HashMap;
use std::marker::PhantomData;

use crate::devices::Device;

use super::{Room, SmartHome};

/// Builder state before the first room is added.
pub struct NoRoom;

/// Builder state after at least one room is available.
pub struct WithRoom;

/// Typestate builder for [`SmartHome`].
pub struct HomeBuilder<State = NoRoom> {
    name: String,
    rooms: HashMap<String, Room>,
    current_room: Option<String>,
    _state: PhantomData<State>,
}

impl HomeBuilder<NoRoom> {
    pub fn new() -> Self {
        Self {
            name: "Smart Home".to_string(),
            rooms: HashMap::new(),
            current_room: None,
            _state: PhantomData,
        }
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rooms: HashMap::new(),
            current_room: None,
            _state: PhantomData,
        }
    }

    pub fn add_room(mut self, name: impl Into<String>) -> HomeBuilder<WithRoom> {
        let name = name.into();
        self.rooms
            .insert(name.clone(), Room::new(name.clone(), HashMap::new()));
        HomeBuilder {
            name: self.name,
            rooms: self.rooms,
            current_room: Some(name),
            _state: PhantomData,
        }
    }

    pub fn build(self) -> SmartHome {
        SmartHome::new(self.name, self.rooms)
    }
}

impl Default for HomeBuilder<NoRoom> {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeBuilder<WithRoom> {
    pub fn add_room(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.rooms
            .insert(name.clone(), Room::new(name.clone(), HashMap::new()));
        self.current_room = Some(name);
        self
    }

    pub fn add_device<D>(mut self, key: impl Into<String>, device: D) -> Self
    where
        D: Into<Device>,
    {
        if let Some(current_room) = self.current_room.as_deref()
            && let Some(room) = self.rooms.get_mut(current_room)
        {
            room.insert_device(key.into(), device.into());
        }
        self
    }

    pub fn build(self) -> SmartHome {
        SmartHome::new(self.name, self.rooms)
    }
}
