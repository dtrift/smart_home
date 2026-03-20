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
}
