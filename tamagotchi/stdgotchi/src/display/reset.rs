use esp_idf_svc::hal::i2c::{I2cDriver, I2cError};
use std::thread;
use std::time::Duration;

/// Reset driver for SH8601 display using TCA9554 I2C GPIO expander
/// The reset pin is connected to EXIO2 (pin 2) on the TCA9554
pub struct DisplayReset<'a> {
    i2c: &'a mut I2cDriver<'a>,
}

impl<'a> DisplayReset<'a> {
    const TCA9554_ADDRESS: u8 = 0x20;  // I2C address from sh8601-rs implementation
    const REG_OUTPUT: u8 = 0x01;
    const REG_CONFIG: u8 = 0x03;

    pub fn new(i2c: &'a mut I2cDriver<'a>) -> Self {
        Self { i2c }
    }

    /// Perform hardware reset sequence according to SH8601 datasheet
    /// Reset low for 20ms, then high for 150ms
    pub fn reset(&mut self) -> Result<(), I2cError> {
        // Configure as output (all pins)
        self.i2c.write(Self::TCA9554_ADDRESS, &[Self::REG_CONFIG, 0x00], 1000)?;

        // Drive reset low (bit 1 = EXIO2)
        self.i2c.write(Self::TCA9554_ADDRESS, &[Self::REG_OUTPUT, 0b0000_0010], 1000)?;
        thread::sleep(Duration::from_millis(20));

        // Drive reset high
        self.i2c.write(Self::TCA9554_ADDRESS, &[Self::REG_OUTPUT, 0b0000_0111], 1000)?;
        thread::sleep(Duration::from_millis(150));

        Ok(())
    }
}
