/// Device power in watts
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Power(f32);

impl Power {
    /// Creates new power in watts
    ///
    /// # Arguments
    ///
    /// * `watts` - Power in watts (must be non-negative)
    ///
    /// # Returns
    ///
    /// `Ok(Power)` if power is valid, otherwise `Err` with error message
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Power;
    ///
    /// let power = Power::new(100.0).unwrap();
    /// assert_eq!(power.watts(), 100.0);
    ///
    /// let invalid = Power::new(-10.0);
    /// assert!(invalid.is_err());
    /// ```
    pub fn new(watts: f32) -> Result<Self, String> {
        if watts < 0.0 {
            return Err("Power cannot be negative".to_string());
        }
        Ok(Power(watts))
    }

    /// Creates power without validation (for internal use)
    ///
    /// # Safety
    ///
    /// Should only be used when watts >= 0 is guaranteed
    #[allow(dead_code)]
    pub(crate) fn new_unchecked(watts: f32) -> Self {
        Power(watts)
    }

    /// Returns power in watts
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Power;
    ///
    /// let power = Power::new(1500.0).unwrap();
    /// assert_eq!(power.watts(), 1500.0);
    /// ```
    pub fn watts(&self) -> f32 {
        self.0
    }

    /// Returns zero power
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Power;
    ///
    /// let zero = Power::zero();
    /// assert_eq!(zero.watts(), 0.0);
    /// ```
    pub fn zero() -> Self {
        Power(0.0)
    }
}

impl Default for Power {
    fn default() -> Self {
        Self::zero()
    }
}

/// Temperature in degrees Celsius
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature(f32);

impl Temperature {
    /// Creates new temperature in degrees Celsius
    ///
    /// # Arguments
    ///
    /// * `celsius` - Temperature in degrees Celsius
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Temperature;
    ///
    /// let temp = Temperature::celsius(22.5);
    /// assert_eq!(temp.as_celsius(), 22.5);
    /// ```
    pub fn celsius(celsius: f32) -> Self {
        Temperature(celsius)
    }

    /// Returns temperature in degrees Celsius
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Temperature;
    ///
    /// let temp = Temperature::celsius(20.0);
    /// assert_eq!(temp.as_celsius(), 20.0);
    /// ```
    pub fn as_celsius(&self) -> f32 {
        self.0
    }

    /// Creates temperature from Fahrenheit
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Temperature;
    ///
    /// let temp = Temperature::fahrenheit(68.0);
    /// assert!((temp.as_celsius() - 20.0).abs() < 0.1);
    /// ```
    pub fn fahrenheit(fahrenheit: f32) -> Self {
        Temperature((fahrenheit - 32.0) * 5.0 / 9.0)
    }

    /// Returns temperature in Fahrenheit
    pub fn as_fahrenheit(&self) -> f32 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Temperature(20.0) // Default room temperature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_creation() {
        let power = Power::new(100.0).unwrap();
        assert_eq!(power.watts(), 100.0);
    }

    #[test]
    fn test_power_negative() {
        let result = Power::new(-10.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_power_zero() {
        let power = Power::zero();
        assert_eq!(power.watts(), 0.0);
    }

    #[test]
    fn test_temperature_celsius() {
        let temp = Temperature::celsius(22.5);
        assert_eq!(temp.as_celsius(), 22.5);
    }

    #[test]
    fn test_temperature_fahrenheit_conversion() {
        let temp = Temperature::fahrenheit(32.0);
        assert!((temp.as_celsius() - 0.0).abs() < 0.01);

        let temp2 = Temperature::celsius(0.0);
        assert!((temp2.as_fahrenheit() - 32.0).abs() < 0.01);
    }
}
