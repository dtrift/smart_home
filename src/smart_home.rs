use crate::room::Room;

/// Smart home containing an array of rooms
#[derive(Debug, Clone, PartialEq)]
pub struct SmartHome {
    name: String,
    rooms: Vec<Room>,
}

impl SmartHome {
    /// Smart home constructor accepting an array of rooms
    ///
    /// # Arguments
    ///
    /// * `name` - Home name
    /// * `rooms` - Array of rooms
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{SmartHome, Room};
    ///
    /// let home = SmartHome::new("My Home".to_string(), vec![]);
    /// assert_eq!(home.name(), "My Home");
    /// ```
    pub fn new(name: String, rooms: Vec<Room>) -> Self {
        Self { name, rooms }
    }

    /// Get a reference to the room at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Room index
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{SmartHome, Room};
    ///
    /// let rooms = vec![Room::new("Living Room".to_string(), vec![])];
    /// let home = SmartHome::new("Home".to_string(), rooms);
    /// let room = home.room(0);
    /// assert_eq!(room.name(), "Living Room");
    /// ```
    pub fn room(&self, index: usize) -> &Room {
        self.rooms.get(index).expect("Room index out of bounds")
    }

    /// Get a mutable reference to the room at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Room index
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{SmartHome, Room, Device, Socket, Power};
    ///
    /// let devices = vec![
    ///     Device::Socket(Socket::new("TV".to_string(), true, Power::new(120.0).unwrap())),
    /// ];
    /// let rooms = vec![Room::new("Living Room".to_string(), devices)];
    /// let mut home = SmartHome::new("Home".to_string(), rooms);
    /// let room = home.room_mut(0);
    /// assert_eq!(room.name(), "Living Room");
    /// ```
    pub fn room_mut(&mut self, index: usize) -> &mut Room {
        self.rooms.get_mut(index).expect("Room index out of bounds")
    }

    /// Print a report of all rooms to stdout
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{SmartHome, Room};
    ///
    /// let home = SmartHome::new("My Home".to_string(), vec![]);
    /// home.print_report();
    /// ```
    pub fn print_report(&self) {
        println!("Smart home '{}':", self.name);
        if self.rooms.is_empty() {
            println!("  (no rooms)");
        } else {
            for room in &self.rooms {
                room.print_report();
            }
        }
        println!();
    }

    /// Returns the home name
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::SmartHome;
    ///
    /// let home = SmartHome::new("My Home".to_string(), vec![]);
    /// assert_eq!(home.name(), "My Home");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of rooms in the home
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{SmartHome, Room};
    ///
    /// let rooms = vec![
    ///     Room::new("Kitchen".to_string(), vec![]),
    ///     Room::new("Living Room".to_string(), vec![]),
    /// ];
    /// let home = SmartHome::new("Home".to_string(), rooms);
    /// assert_eq!(home.room_count(), 2);
    /// ```
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }
}
