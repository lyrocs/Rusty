use crate::drivers::Tca9554Driver;
use embedded_hal::digital::OutputPin;

/// Error type for ExioPin that implements digital::Error
#[derive(Debug)]
pub struct ExioPinError;

impl embedded_hal::digital::Error for ExioPinError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

/// Wrapper around a specific pin on the TCA9554 GPIO expander
/// Implements the OutputPin trait for use with SD card CS
pub struct ExioPin<I2C> {
    expander: Tca9554Driver<I2C>,
    pin_number: u8,
}

impl<I2C> ExioPin<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub fn new(mut expander: Tca9554Driver<I2C>, pin_number: u8) -> Result<Self, I2C::Error> {
        // Configure the pin as output
        expander.configure_pin(pin_number, true)?;
        // Set it high initially (CS is active low)
        expander.write_pin(pin_number, true)?;
        Ok(Self {
            expander,
            pin_number,
        })
    }
}

impl<I2C> OutputPin for ExioPin<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.expander.write_pin(self.pin_number, false)
            .map_err(|_| ExioPinError)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.expander.write_pin(self.pin_number, true)
            .map_err(|_| ExioPinError)
    }
}

impl<I2C> embedded_hal::digital::ErrorType for ExioPin<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Error = ExioPinError;
}
