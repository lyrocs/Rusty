use esp_hal::delay::Delay;
use esp_hal::i2c::master::Error as I2cError;
use ft3x68_rs::ResetInterface;

pub struct ResetTouchDriver<I2C> {
    i2c: I2C,
}

impl<I2C> ResetTouchDriver<I2C> {
    pub fn new(i2c: I2C) -> Self {
        ResetTouchDriver { i2c }
    }
}

impl<I2C> ResetInterface for ResetTouchDriver<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Error = I2cError;

    fn reset(&mut self) -> Result<(), Self::Error> {
        esp_println::println!("Resetting touch controller via I2C GPIO expander...");
        let delay = Delay::new();
        self.i2c.write(0x20, &[0x03, 0x00]).unwrap(); // Configure all pins as output
        self.i2c.write(0x20, &[0x01, 0b0000_0000]).unwrap(); // Drive low
        delay.delay_millis(20);
        self.i2c.write(0x20, &[0x01, 0b0000_0100]).unwrap(); // Drive high
        delay.delay_millis(300);
        Ok(())
    }
}
