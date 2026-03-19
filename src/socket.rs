use crate::device::DeviceInfo;
use crate::types::Power;

/// Smart socket for controlling electrical appliances
#[derive(Debug, Clone, PartialEq)]
pub struct Socket {
    name: String,
    is_on: bool,
    power: Power,
}

impl Socket {
    /// Creates a new socket
    ///
    /// # Arguments
    ///
    /// * `name` - Socket name
    /// * `is_on` - Initial state (true - on, false - off)
    /// * `power` - Power of the connected device
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Socket, Power, DeviceInfo};
    ///
    /// let socket = Socket::new("Kettle".to_string(), true, Power::new(1500.0).unwrap());
    /// assert_eq!(socket.name(), "Kettle");
    /// assert!(socket.is_on());
    /// ```
    pub fn new(name: String, is_on: bool, power: Power) -> Self {
        Self { name, is_on, power }
    }

    /// Turn on the socket
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Socket, Power};
    ///
    /// let mut socket = Socket::new("Lamp".to_string(), false, Power::new(60.0).unwrap());
    /// socket.turn_on();
    /// assert!(socket.is_on());
    /// ```
    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    /// Turn off the socket
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Socket, Power};
    ///
    /// let mut socket = Socket::new("TV".to_string(), true, Power::new(120.0).unwrap());
    /// socket.turn_off();
    /// assert!(!socket.is_on());
    /// ```
    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    /// Check current state (on/off)
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Socket, Power};
    ///
    /// let socket = Socket::new("Fridge".to_string(), true, Power::new(200.0).unwrap());
    /// assert!(socket.is_on());
    /// ```
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    /// Returns current power: zero when off, otherwise the configured power
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Socket, Power};
    ///
    /// let socket = Socket::new("Iron".to_string(), true, Power::new(2000.0).unwrap());
    /// assert_eq!(socket.power().watts(), 2000.0);
    ///
    /// let mut socket_off = Socket::new("Iron".to_string(), false, Power::new(2000.0).unwrap());
    /// assert_eq!(socket_off.power().watts(), 0.0);
    /// ```
    pub fn power(&self) -> Power {
        if self.is_on {
            self.power
        } else {
            Power::zero()
        }
    }
}

impl DeviceInfo for Socket {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> String {
        format!(
            "Socket '{}': {} (power: {:.1} W)",
            self.name,
            if self.is_on { "on" } else { "off" },
            self.power().watts()
        )
    }
}
