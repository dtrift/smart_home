use super::device::DeviceInfo;
use crate::types::Temperature;

/// Thermometer for measuring temperature
#[derive(Debug, Clone, PartialEq)]
pub struct Thermometer {
    name: String,
    temperature: Temperature,
}

impl Thermometer {
    /// Creates a new thermometer
    ///
    /// # Arguments
    ///
    /// * `name` - Thermometer name
    /// * `temperature` - Initial temperature
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Thermometer, Temperature, DeviceInfo};
    ///
    /// let thermometer = Thermometer::new("Kitchen".to_string(), Temperature::celsius(22.5));
    /// assert_eq!(thermometer.name(), "Kitchen");
    /// assert_eq!(thermometer.temperature().as_celsius(), 22.5);
    /// ```
    pub fn new(name: String, temperature: Temperature) -> Self {
        Self { name, temperature }
    }

    /// Returns current temperature
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::{Thermometer, Temperature};
    ///
    /// let thermometer = Thermometer::new("Living Room".to_string(), Temperature::celsius(24.0));
    /// let temp = thermometer.temperature();
    /// assert_eq!(temp.as_celsius(), 24.0);
    /// ```
    pub fn temperature(&self) -> Temperature {
        self.temperature
    }
}

impl DeviceInfo for Thermometer {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> String {
        format!(
            "Thermometer '{}': {:.1}°C",
            self.name,
            self.temperature.as_celsius()
        )
    }
}
