// TCA9554 GPIO Expander Driver for ESP-IDF
//
// This 8-bit I2C GPIO expander is used for controlling reset pins

use anyhow::Result;
use esp_idf_svc::hal::i2c::I2cDriver;

const TCA9554_ADDRESS: u8 = 0x22;

// TCA9554 Registers
const REG_INPUT_PORT: u8 = 0x00;
const REG_OUTPUT_PORT: u8 = 0x01;
const REG_POLARITY_INV: u8 = 0x02;
const REG_CONFIGURATION: u8 = 0x03;

/// TCA9554 GPIO Expander Driver
pub struct Tca9554Driver<'d> {
    i2c: &'d mut I2cDriver<'static>,
    address: u8,
}

impl<'d> Tca9554Driver<'d> {
    /// Create a new TCA9554 driver instance
    pub fn new(i2c: &'d mut I2cDriver<'static>) -> Self {
        Self {
            i2c,
            address: TCA9554_ADDRESS,
        }
    }

    /// Create a new TCA9554 driver instance with custom address
    pub fn new_with_address(i2c: &'d mut I2cDriver<'static>, address: u8) -> Self {
        Self {
            i2c,
            address,
        }
    }

    /// Try to detect the device on common TCA9554 addresses
    pub fn detect_address(i2c: &mut I2cDriver<'static>) -> Result<u8> {
        let common_addresses = [0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27];

        for &addr in &common_addresses {
            log::info!("Trying TCA9554 at address 0x{:02X}...", addr);
            let result = i2c.write(addr, &[REG_CONFIGURATION], 100);
            if result.is_ok() {
                log::info!("Found TCA9554 at address 0x{:02X}", addr);
                return Ok(addr);
            }
        }

        anyhow::bail!("TCA9554 not found on any common address")
    }

    /// Configure a pin as input (true) or output (false)
    pub fn configure_pin(&mut self, pin: u8, input: bool) -> Result<()> {
        if pin > 7 {
            anyhow::bail!("Pin number must be 0-7");
        }

        // Read current configuration
        let mut config = [0u8; 1];
        self.i2c
            .write_read(self.address, &[REG_CONFIGURATION], &mut config, 1000)?;

        // Modify the bit for the specified pin
        let new_config = if input {
            config[0] | (1 << pin) // Set bit to 1 for input
        } else {
            config[0] & !(1 << pin) // Clear bit to 0 for output
        };

        // Write back the configuration
        self.i2c
            .write(self.address, &[REG_CONFIGURATION, new_config], 1000)?;

        Ok(())
    }

    /// Set output pin high (true) or low (false)
    pub fn write_pin(&mut self, pin: u8, high: bool) -> Result<()> {
        if pin > 7 {
            anyhow::bail!("Pin number must be 0-7");
        }

        // Read current output state
        let mut output = [0u8; 1];
        self.i2c
            .write_read(self.address, &[REG_OUTPUT_PORT], &mut output, 1000)?;

        // Modify the bit for the specified pin
        let new_output = if high {
            output[0] | (1 << pin) // Set bit to 1 for high
        } else {
            output[0] & !(1 << pin) // Clear bit to 0 for low
        };

        // Write back the output state
        self.i2c
            .write(self.address, &[REG_OUTPUT_PORT, new_output], 1000)?;

        Ok(())
    }

    /// Read input pin state
    pub fn read_pin(&mut self, pin: u8) -> Result<bool> {
        if pin > 7 {
            anyhow::bail!("Pin number must be 0-7");
        }

        let mut input = [0u8; 1];
        self.i2c
            .write_read(self.address, &[REG_INPUT_PORT], &mut input, 1000)?;

        Ok((input[0] & (1 << pin)) != 0)
    }

    /// Set all pins at once
    pub fn write_all(&mut self, value: u8) -> Result<()> {
        self.i2c
            .write(self.address, &[REG_OUTPUT_PORT, value], 1000)?;
        Ok(())
    }

    /// Read all pins at once
    pub fn read_all(&mut self) -> Result<u8> {
        let mut input = [0u8; 1];
        self.i2c
            .write_read(self.address, &[REG_INPUT_PORT], &mut input, 1000)?;
        Ok(input[0])
    }

    /// Configure all pins at once (bit=1 for input, bit=0 for output)
    pub fn configure_all(&mut self, config: u8) -> Result<()> {
        self.i2c
            .write(self.address, &[REG_CONFIGURATION, config], 1000)?;
        Ok(())
    }
}
