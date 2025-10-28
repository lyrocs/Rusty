// Power management driver for AXP2101 PMIC

use crate::hal::PowerDriver;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;

/// AXP2101 Power Management Driver
pub struct Axp2101PowerDriver {
    last_voltage: u16,
}

impl Axp2101PowerDriver {
    pub fn new() -> Self {
        Self {
            last_voltage: 3700, // Default voltage
        }
    }

    /// Convert voltage to battery percentage
    fn voltage_to_percent(voltage_mv: u16) -> u8 {
        // Simple linear mapping from 3.0V (0%) to 4.2V (100%)
        const MIN_VOLTAGE: u16 = 3000;
        const MAX_VOLTAGE: u16 = 4200;

        if voltage_mv <= MIN_VOLTAGE {
            0
        } else if voltage_mv >= MAX_VOLTAGE {
            100
        } else {
            ((voltage_mv - MIN_VOLTAGE) as u32 * 100 / (MAX_VOLTAGE - MIN_VOLTAGE) as u32) as u8
        }
    }
}

impl PowerDriver for Axp2101PowerDriver {
    fn battery_voltage(&mut self) -> Result<u16> {
        // TODO: Implement actual I2C reading from AXP2101
        Ok(self.last_voltage)
    }

    fn battery_percent(&mut self) -> Result<u8> {
        let voltage = self.battery_voltage()?;
        Ok(Self::voltage_to_percent(voltage))
    }

    fn is_charging(&mut self) -> Result<bool> {
        // TODO: Implement actual charging status reading
        Ok(false)
    }
}

/// Thread-safe power driver wrapper
pub type SharedPower = Arc<Mutex<dyn PowerDriver>>;

/// Create a shared power driver instance
pub fn create_shared_power() -> Result<SharedPower> {
    let power = Axp2101PowerDriver::new();
    Ok(Arc::new(Mutex::new(power)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voltage_to_percent() {
        assert_eq!(Axp2101PowerDriver::voltage_to_percent(3000), 0);
        assert_eq!(Axp2101PowerDriver::voltage_to_percent(4200), 100);
        assert_eq!(Axp2101PowerDriver::voltage_to_percent(3600), 50);
    }
}
