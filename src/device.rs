use crate::socket::Socket;
use crate::thermometer::Thermometer;

/// Trait for all smart home devices
pub trait DeviceInfo {
    /// Returns the device name
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Thermometer, DeviceInfo, Temperature};
    ///
    /// let thermometer = Thermometer::new("Kitchen".to_string(), Temperature::celsius(22.5));
    /// assert_eq!(thermometer.name(), "Kitchen");
    /// ```
    fn name(&self) -> &str;

    /// Returns string representation of device state
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Socket, DeviceInfo, Power};
    ///
    /// let socket = Socket::new("Kettle".to_string(), true, Power::new(1500.0).unwrap());
    /// let state = socket.state();
    /// assert!(state.contains("on"));
    /// ```
    fn state(&self) -> String;

    /// Prints device information to stdout
    fn print_info(&self) {
        println!("{}", self.state());
    }
}

/// Device (enumeration)
#[derive(Debug, Clone, PartialEq)]
pub enum Device {
    Thermometer(Thermometer),
    Socket(Socket),
}

impl Device {
    /// Prints device state message to stdout
    pub fn print_state(&self) {
        match self {
            Device::Thermometer(t) => t.print_info(),
            Device::Socket(s) => s.print_info(),
        }
    }
}

impl DeviceInfo for Device {
    fn name(&self) -> &str {
        match self {
            Device::Thermometer(t) => t.name(),
            Device::Socket(s) => s.name(),
        }
    }

    fn state(&self) -> String {
        match self {
            Device::Thermometer(t) => t.state(),
            Device::Socket(s) => s.state(),
        }
    }
}
