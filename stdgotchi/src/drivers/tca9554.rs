/// Simple driver for TCA9554PWR GPIO expander
pub struct Tca9554Driver<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Tca9554Driver<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    const DEFAULT_ADDRESS: u8 = 0x20;

    // Register addresses
    const REG_INPUT_PORT: u8 = 0x00;
    const REG_OUTPUT_PORT: u8 = 0x01;
    #[allow(dead_code)]
    const REG_POLARITY_INVERSION: u8 = 0x02;
    const REG_CONFIGURATION: u8 = 0x03;

    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: Self::DEFAULT_ADDRESS,
        }
    }

    /// Read input port (all 8 pins)
    pub fn read_input_port(&mut self) -> Result<u8, I2C::Error> {
        let mut data = [0u8];
        self.i2c
            .write_read(self.address, &[Self::REG_INPUT_PORT], &mut data)?;
        Ok(data[0])
    }

    /// Read specific pin (0-7)
    pub fn read_pin(&mut self, pin: u8) -> Result<bool, I2C::Error> {
        if pin > 7 {
            // Return a generic error for invalid pin
            return self.read_input_port().map(|_| false);
        }
        let port_value = self.read_input_port()?;
        Ok((port_value & (1 << pin)) != 0)
    }

    /// Write output port (all 8 pins)
    pub fn write_output_port(&mut self, value: u8) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[Self::REG_OUTPUT_PORT, value])
    }

    /// Write specific pin (0-7)
    pub fn write_pin(&mut self, pin: u8, value: bool) -> Result<(), I2C::Error> {
        if pin > 7 {
            return Ok(()); // Ignore invalid pin
        }
        // Read current output port value
        let mut current = [0u8];
        self.i2c
            .write_read(self.address, &[Self::REG_OUTPUT_PORT], &mut current)?;

        // Modify the specific bit
        let new_value = if value {
            current[0] | (1 << pin)
        } else {
            current[0] & !(1 << pin)
        };

        self.write_output_port(new_value)
    }

    /// Configure pin direction (0=input, 1=output)
    pub fn configure_pin(&mut self, pin: u8, as_output: bool) -> Result<(), I2C::Error> {
        if pin > 7 {
            return Ok(()); // Ignore invalid pin
        }

        // Read current configuration
        let mut current = [0u8];
        self.i2c
            .write_read(self.address, &[Self::REG_CONFIGURATION], &mut current)?;

        // Modify the specific bit
        let new_config = if as_output {
            current[0] & !(1 << pin) // Clear bit for output
        } else {
            current[0] | (1 << pin) // Set bit for input
        };

        self.i2c
            .write(self.address, &[Self::REG_CONFIGURATION, new_config])
    }

    /// Configure all pins at once
    pub fn configure_all_pins(&mut self, config: u8) -> Result<(), I2C::Error> {
        self.i2c
            .write(self.address, &[Self::REG_CONFIGURATION, config])
    }
}
