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
