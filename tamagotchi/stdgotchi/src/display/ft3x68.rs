//! FT3x68 Touch Controller Driver for ESP-IDF
//!
//! This is an ESP-IDF compatible port of the ft3x68-rs driver.
//! Original: https://github.com/theembeddedrustacean/ft3x68-rs

use esp_idf_svc::hal::i2c::I2cDriver;
use std::error::Error;
use std::fmt;
use std::thread;
use std::time::Duration;

/// FT3168 Device I2C Address
pub const FT3168_DEVICE_ADDRESS: u8 = 0x38;

/// FT3268 Device I2C Address
pub const FT3268_DEVICE_ADDRESS: u8 = 0x38;

// Register Addresses
const FT3X68_RD_DEVICE_GESTUREID: u8 = 0xD3;
const FT3X68_RD_DEVICE_FINGERNUM: u8 = 0x02;
const FT3X68_RD_DEVICE_X1POSH: u8 = 0x03;
const FT3X68_RD_DEVICE_X1POSL: u8 = 0x04;
const FT3X68_RD_DEVICE_Y1POSH: u8 = 0x05;
const FT3X68_RD_DEVICE_Y1POSL: u8 = 0x06;
const FT3X68_RD_DEVICE_X2POSH: u8 = 0x09;
const FT3X68_RD_DEVICE_X2POSL: u8 = 0x0A;
const FT3X68_RD_DEVICE_Y2POSH: u8 = 0x0B;
const FT3X68_RD_DEVICE_Y2POSL: u8 = 0x0C;
const FT3X68_RD_WR_DEVICE_GESTUREID_MODE: u8 = 0xD0;
const FT3X68_RD_WR_DEVICE_POWER_MODE: u8 = 0xA5;
const FT3X68_RD_WR_DEVICE_PROXIMITY_SENSING_MODE: u8 = 0xB0;
const FT3X68_RD_DEVICE_ID: u8 = 0xA0;

/// TCA9554 GPIO expander address for touch reset control
const TCA9554_ADDRESS: u8 = 0x20;
const REG_OUTPUT: u8 = 0x01;
const REG_CONFIG: u8 = 0x03;

/// Power modes for FT3x68
#[derive(Debug, Clone, Copy)]
pub enum PowerMode {
    Active = 0x00,
    Monitor = 0x01,
    Standby = 0x02,
    Hibernate = 0x03,
}

/// Gesture types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    None = 0x00,
    SwipeLeft = 0x20,
    SwipeRight = 0x21,
    SwipeUp = 0x22,
    SwipeDown = 0x23,
    DoubleClick = 0x24,
}

impl Gesture {
    fn from_u8(value: u8) -> Self {
        match value {
            0x20 => Gesture::SwipeLeft,
            0x21 => Gesture::SwipeRight,
            0x22 => Gesture::SwipeUp,
            0x23 => Gesture::SwipeDown,
            0x24 => Gesture::DoubleClick,
            _ => Gesture::None,
        }
    }
}

/// Touch point coordinates
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
}

/// Touch driver errors
#[derive(Debug)]
pub enum TouchError {
    I2cError(String),
    InvalidData,
}

impl fmt::Display for TouchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TouchError::I2cError(msg) => write!(f, "I2C Error: {}", msg),
            TouchError::InvalidData => write!(f, "Invalid data received"),
        }
    }
}

impl Error for TouchError {}

/// FT3x68 Touch Controller Driver
pub struct Ft3x68Driver {
    address: u8,
}

impl Ft3x68Driver {
    /// Create a new FT3x68 driver instance
    ///
    /// # Arguments
    /// * `address` - Device I2C address (use FT3168_DEVICE_ADDRESS or FT3268_DEVICE_ADDRESS)
    pub fn new(address: u8) -> Self {
        Self { address }
    }

    /// Initialize the touch controller
    ///
    /// This performs a hardware reset via the TCA9554 GPIO expander (EXIO2)
    pub fn initialize(&mut self, i2c: &mut I2cDriver) -> Result<(), TouchError> {
        log::info!("Initializing FT3x68 touch controller...");

        // Perform hardware reset via TCA9554 GPIO expander
        self.reset_via_gpio_expander(i2c)?;

        // Wait for device to be ready
        thread::sleep(Duration::from_millis(300));

        // Read device ID to verify communication
        let device_id = self.read_register(i2c, FT3X68_RD_DEVICE_ID)?;
        log::info!("Touch controller device ID: 0x{:02X}", device_id);

        Ok(())
    }

    /// Hardware reset via TCA9554 GPIO expander (EXIO2 pin)
    fn reset_via_gpio_expander(&mut self, i2c: &mut I2cDriver) -> Result<(), TouchError> {
        log::info!("Resetting touch controller via GPIO expander...");

        // Configure all pins as output
        i2c.write(TCA9554_ADDRESS, &[REG_CONFIG, 0x00], 1000)
            .map_err(|e| TouchError::I2cError(format!("GPIO config failed: {:?}", e)))?;

        // Drive EXIO2 low (bit 2 = 0) while keeping display reset (EXIO1, bit 1) high
        i2c.write(TCA9554_ADDRESS, &[REG_OUTPUT, 0b0000_0011], 1000)
            .map_err(|e| TouchError::I2cError(format!("GPIO low failed: {:?}", e)))?;
        thread::sleep(Duration::from_millis(20));

        // Drive EXIO2 high (bit 2 = 1) while keeping display reset (EXIO1, bit 1) high
        // Final state: EXIO0=1, EXIO1=1 (display reset), EXIO2=1 (touch reset)
        i2c.write(TCA9554_ADDRESS, &[REG_OUTPUT, 0b0000_0111], 1000)
            .map_err(|e| TouchError::I2cError(format!("GPIO high failed: {:?}", e)))?;
        thread::sleep(Duration::from_millis(300));

        Ok(())
    }

    /// Read a single register
    fn read_register(&self, i2c: &mut I2cDriver, register: u8) -> Result<u8, TouchError> {
        let mut buf = [0u8; 1];
        i2c.write_read(self.address, &[register], &mut buf, 1000)
            .map_err(|e| TouchError::I2cError(format!("Read register 0x{:02X} failed: {:?}", register, e)))?;
        Ok(buf[0])
    }

    /// Write a single register
    fn write_register(&self, i2c: &mut I2cDriver, register: u8, value: u8) -> Result<(), TouchError> {
        i2c.write(self.address, &[register, value], 1000)
            .map_err(|e| TouchError::I2cError(format!("Write register 0x{:02X} failed: {:?}", register, e)))?;
        Ok(())
    }

    /// Set power mode
    pub fn set_power_mode(&self, i2c: &mut I2cDriver, mode: PowerMode) -> Result<(), TouchError> {
        self.write_register(i2c, FT3X68_RD_WR_DEVICE_POWER_MODE, mode as u8)
    }

    /// Enable or disable gesture recognition
    pub fn set_gesture_mode(&self, i2c: &mut I2cDriver, enable: bool) -> Result<(), TouchError> {
        let value = if enable { 0x01 } else { 0x00 };
        self.write_register(i2c, FT3X68_RD_WR_DEVICE_GESTUREID_MODE, value)
    }

    /// Enable or disable proximity sensing
    pub fn set_proximity_sensing(&self, i2c: &mut I2cDriver, enable: bool) -> Result<(), TouchError> {
        let value = if enable { 0x01 } else { 0x00 };
        self.write_register(i2c, FT3X68_RD_WR_DEVICE_PROXIMITY_SENSING_MODE, value)
    }

    /// Get number of fingers touching the screen
    pub fn finger_number(&self, i2c: &mut I2cDriver) -> Result<u8, TouchError> {
        self.read_register(i2c, FT3X68_RD_DEVICE_FINGERNUM)
    }

    /// Read the current gesture
    pub fn read_gesture(&self, i2c: &mut I2cDriver) -> Result<Gesture, TouchError> {
        let gesture_id = self.read_register(i2c, FT3X68_RD_DEVICE_GESTUREID)?;
        Ok(Gesture::from_u8(gesture_id))
    }

    /// Get touch points (supports up to 2 touches)
    pub fn get_touches(&self, i2c: &mut I2cDriver) -> Result<Vec<TouchPoint>, TouchError> {
        let finger_num = self.finger_number(i2c)?;

        if finger_num == 0 {
            return Ok(Vec::new());
        }

        let mut touches = Vec::new();

        // Read first touch point
        if finger_num >= 1 {
            let x_high = self.read_register(i2c, FT3X68_RD_DEVICE_X1POSH)?;
            let x_low = self.read_register(i2c, FT3X68_RD_DEVICE_X1POSL)?;
            let y_high = self.read_register(i2c, FT3X68_RD_DEVICE_Y1POSH)?;
            let y_low = self.read_register(i2c, FT3X68_RD_DEVICE_Y1POSL)?;

            let x = ((x_high as u16 & 0x0F) << 8) | x_low as u16;
            let y = ((y_high as u16 & 0x0F) << 8) | y_low as u16;

            touches.push(TouchPoint { x, y });
        }

        // Read second touch point
        if finger_num >= 2 {
            let x_high = self.read_register(i2c, FT3X68_RD_DEVICE_X2POSH)?;
            let x_low = self.read_register(i2c, FT3X68_RD_DEVICE_X2POSL)?;
            let y_high = self.read_register(i2c, FT3X68_RD_DEVICE_Y2POSH)?;
            let y_low = self.read_register(i2c, FT3X68_RD_DEVICE_Y2POSL)?;

            let x = ((x_high as u16 & 0x0F) << 8) | x_low as u16;
            let y = ((y_high as u16 & 0x0F) << 8) | y_low as u16;

            touches.push(TouchPoint { x, y });
        }

        Ok(touches)
    }
}
