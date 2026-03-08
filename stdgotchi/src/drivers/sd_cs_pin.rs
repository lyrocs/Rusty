/// SD Card CS pin using TCA9554 GPIO expander
/// This version stores state but requires I2C driver to be passed for operations
use embedded_hal::digital::OutputPin;
use esp_idf_svc::hal::i2c::I2cDriver;

/// TCA9554 GPIO expander I2C address
const TCA9554_ADDRESS: u8 = 0x20;
/// TCA9554 output register
const REG_OUTPUT: u8 = 0x01;
/// TCA9554 configuration register
const REG_CONFIG: u8 = 0x03;

// Global I2C driver for SD card CS pin access
// Using a raw pointer wrapped in an unsafe cell-like structure
static mut SD_I2C: Option<&'static mut I2cDriver<'static>> = None;

/// SD Card CS pin (EXIO7 on TCA9554)
/// Uses shared I2C driver access
pub struct SdCsPin {
    pin_number: u8,
    pin_mask: u8,
}

/// Initialize the shared I2C driver for SD CS pin
/// SAFETY: This must be called exactly once before any SdCsPin operations
pub unsafe fn init_sd_i2c(i2c: &'static mut I2cDriver<'static>) {
    SD_I2C = Some(i2c);
}

/// Get mutable access to the shared I2C driver
/// SAFETY: Caller must ensure no other mutable references exist
pub unsafe fn get_shared_i2c() -> Option<&'static mut I2cDriver<'static>> {
    SD_I2C.as_mut().map(|r| &mut **r)
}

/// Helper to access I2C driver safely
fn with_i2c<F, R>(f: F) -> Result<R, &'static str>
where
    F: FnOnce(&mut I2cDriver) -> Result<R, &'static str>,
{
    unsafe {
        if let Some(i2c) = SD_I2C.as_mut() {
            f(i2c)
        } else {
            Err("I2C not initialized for SD card")
        }
    }
}

impl SdCsPin {
    /// Create a new SD CS pin (EXIO7)
    /// Configures the pin as output on the TCA9554
    pub fn new() -> Result<Self, &'static str> {
        let pin_number = 7;
        let pin_mask = 1 << pin_number;

        // Configure EXIO7 as output using shared I2C
        with_i2c(|i2c| {
            // Read current config
            let mut config = [0u8; 1];
            i2c.write_read(TCA9554_ADDRESS, &[REG_CONFIG], &mut config, 1000)
                .map_err(|_| "Failed to read TCA9554 config")?;

            // Set pin as output (clear bit)
            let new_config = config[0] & !pin_mask;
            i2c.write(TCA9554_ADDRESS, &[REG_CONFIG, new_config], 1000)
                .map_err(|_| "Failed to configure EXIO7 as output")?;

            Ok(())
        })?;

        // Start with CS high (inactive)
        let mut pin = Self {
            pin_number,
            pin_mask,
        };
        pin.set_high().map_err(|_| "Failed to set initial state")?;

        Ok(pin)
    }
}

impl OutputPin for SdCsPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        with_i2c(|i2c| {
            // Read current output
            let mut output = [0u8; 1];
            i2c.write_read(TCA9554_ADDRESS, &[REG_OUTPUT], &mut output, 1000)
                .map_err(|_| "Failed to read output register")?;

            // Clear the bit (set low)
            let new_output = output[0] & !self.pin_mask;
            i2c.write(TCA9554_ADDRESS, &[REG_OUTPUT, new_output], 1000)
                .map_err(|_| "Failed to set pin low")?;

            Ok(())
        }).map_err(|_| SdCsPinError)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        with_i2c(|i2c| {
            // Read current output
            let mut output = [0u8; 1];
            i2c.write_read(TCA9554_ADDRESS, &[REG_OUTPUT], &mut output, 1000)
                .map_err(|_| "Failed to read output register")?;

            // Set the bit (set high)
            let new_output = output[0] | self.pin_mask;
            i2c.write(TCA9554_ADDRESS, &[REG_OUTPUT, new_output], 1000)
                .map_err(|_| "Failed to set pin high")?;

            Ok(())
        }).map_err(|_| SdCsPinError)
    }
}

/// Error type for SD CS pin
#[derive(Debug)]
pub struct SdCsPinError;

impl embedded_hal::digital::Error for SdCsPinError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl embedded_hal::digital::ErrorType for SdCsPin {
    type Error = SdCsPinError;
}
