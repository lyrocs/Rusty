//! CST816D Touch Controller Driver for ESP-IDF
//!
//! This module provides an ESP-IDF (std) compatible driver for the CST816D capacitive touch controller.
//! It supports single-touch and gesture recognition.
//!
//! # Hardware Configuration
//! - Touch Controller: CST816D
//! - Interface: I2C (address 0x15)
//! - Max Touch Points: 1
//!
//! # Pin Configuration
//! - SDA: GPIO7
//! - SCL: GPIO8
//! - INT: GPIO11
//!
//! # Features
//! - Single-touch coordinate reading
//! - Gesture recognition (swipe up/down/left/right, single/double-click, long press)
//! - Low power consumption
//!
//! # Example
//! ```no_run
//! use display::{Cst816dDriver, CST816D_DEVICE_ADDRESS};
//!
//! let mut touch = Cst816dDriver::new(CST816D_DEVICE_ADDRESS);
//! touch.initialize(&mut i2c)?;
//!
//! if let Ok(Some(touch_point)) = touch.get_touch(&mut i2c) {
//!     println!("Touch at: x={}, y={}", touch_point.x, touch_point.y);
//! }
//! ```

use esp_idf_svc::hal::i2c::I2cDriver;
use std::error::Error;
use std::fmt;
use std::thread;
use std::time::Duration;

/// CST816D Device I2C Address
pub const CST816D_DEVICE_ADDRESS: u8 = 0x15;

// Register Addresses
const CST816D_REG_GESTURE_ID: u8 = 0x01;
const CST816D_REG_FINGER_NUM: u8 = 0x02;
const CST816D_REG_XPOS_H: u8 = 0x03;
const CST816D_REG_XPOS_L: u8 = 0x04;
const CST816D_REG_YPOS_H: u8 = 0x05;
const CST816D_REG_YPOS_L: u8 = 0x06;
const CST816D_REG_CHIP_ID: u8 = 0xA7;
const CST816D_REG_FW_VERSION: u8 = 0xA9;

/// Gesture types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    None = 0x00,
    SwipeUp = 0x01,
    SwipeDown = 0x02,
    SwipeLeft = 0x03,
    SwipeRight = 0x04,
    SingleClick = 0x05,
    DoubleClick = 0x0B,
    LongPress = 0x0C,
}

impl Gesture {
    fn from_u8(value: u8) -> Self {
        match value {
            0x01 => Gesture::SwipeUp,
            0x02 => Gesture::SwipeDown,
            0x03 => Gesture::SwipeLeft,
            0x04 => Gesture::SwipeRight,
            0x05 => Gesture::SingleClick,
            0x0B => Gesture::DoubleClick,
            0x0C => Gesture::LongPress,
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
}

impl fmt::Display for TouchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TouchError::I2cError(msg) => write!(f, "I2C Error: {}", msg),
        }
    }
}

impl Error for TouchError {}

/// CST816D Touch Controller Driver
pub struct Cst816dDriver {
    address: u8,
}

impl Cst816dDriver {
    /// Create a new CST816D driver instance
    ///
    /// # Arguments
    /// * `address` - Device I2C address (use CST816D_DEVICE_ADDRESS)
    pub fn new(address: u8) -> Self {
        Self { address }
    }

    /// Initialize the touch controller
    pub fn initialize(&mut self, i2c: &mut I2cDriver) -> Result<(), TouchError> {
        log::info!("Initializing CST816D touch controller at address 0x{:02X}...", self.address);

        // Wait for device to be ready
        thread::sleep(Duration::from_millis(50));

        // Try to read chip ID to verify communication
        match self.read_register(i2c, CST816D_REG_CHIP_ID) {
            Ok(chip_id) => {
                log::info!("✓ Touch controller chip ID: 0x{:02X}", chip_id);
            }
            Err(e) => {
                log::error!("✗ Failed to read chip ID: {:?}", e);
                log::error!("Check I2C connections: SDA=GPIO7, SCL=GPIO8");
                return Err(e);
            }
        }

        // Try to read firmware version
        match self.read_register(i2c, CST816D_REG_FW_VERSION) {
            Ok(fw_version) => {
                log::info!("✓ Touch controller firmware version: 0x{:02X}", fw_version);
            }
            Err(e) => {
                log::warn!("Failed to read firmware version: {:?}", e);
            }
        }

        log::info!("✓ CST816D touch controller initialized successfully!");
        Ok(())
    }

    /// Read a single register
    fn read_register(&self, i2c: &mut I2cDriver, register: u8) -> Result<u8, TouchError> {
        let mut buf = [0u8; 1];
        i2c.write_read(self.address, &[register], &mut buf, 1000)
            .map_err(|e| TouchError::I2cError(format!("Read register 0x{:02X} failed: {:?}", register, e)))?;
        Ok(buf[0])
    }

    /// Read multiple registers
    fn read_registers(&self, i2c: &mut I2cDriver, register: u8, buf: &mut [u8]) -> Result<(), TouchError> {
        i2c.write_read(self.address, &[register], buf, 1000)
            .map_err(|e| TouchError::I2cError(format!("Read registers from 0x{:02X} failed: {:?}", register, e)))?;
        Ok(())
    }

    /// Get number of fingers touching the screen
    pub fn finger_number(&self, i2c: &mut I2cDriver) -> Result<u8, TouchError> {
        let num = self.read_register(i2c, CST816D_REG_FINGER_NUM)?;
        // CST816D reports 0 for no touch, 1 for touch
        Ok(num)
    }

    /// Read the current gesture
    pub fn read_gesture(&self, i2c: &mut I2cDriver) -> Result<Gesture, TouchError> {
        let gesture_id = self.read_register(i2c, CST816D_REG_GESTURE_ID)?;
        Ok(Gesture::from_u8(gesture_id))
    }

    /// Get touch point (returns None if no touch detected)
    pub fn get_touch(&self, i2c: &mut I2cDriver) -> Result<Option<TouchPoint>, TouchError> {
        let finger_num = self.finger_number(i2c)?;

        if finger_num == 0 {
            return Ok(None);
        }

        // Read all touch data at once (4 bytes starting from XPOS_H)
        let mut data = [0u8; 4];
        self.read_registers(i2c, CST816D_REG_XPOS_H, &mut data)?;

        let x = ((data[0] as u16 & 0x0F) << 8) | data[1] as u16;
        let y = ((data[2] as u16 & 0x0F) << 8) | data[3] as u16;

        log::debug!("[TOUCH] Detected at x={}, y={} (raw: {:02X} {:02X} {:02X} {:02X})",
                    x, y, data[0], data[1], data[2], data[3]);

        Ok(Some(TouchPoint { x, y }))
    }

    /// Get both touch point and gesture in a single read operation
    pub fn get_touch_and_gesture(&self, i2c: &mut I2cDriver) -> Result<(Option<TouchPoint>, Gesture), TouchError> {
        // Read gesture ID, finger number, and touch coordinates in one go
        let mut data = [0u8; 6];
        self.read_registers(i2c, CST816D_REG_GESTURE_ID, &mut data)?;

        let gesture = Gesture::from_u8(data[0]);
        let finger_num = data[1];

        if finger_num == 0 {
            return Ok((None, gesture));
        }

        let x = ((data[2] as u16 & 0x0F) << 8) | data[3] as u16;
        let y = ((data[4] as u16 & 0x0F) << 8) | data[5] as u16;

        Ok((Some(TouchPoint { x, y }), gesture))
    }
}
