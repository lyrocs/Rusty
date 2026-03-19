//! CST816D Touch Controller Driver
//!
//! Extracted from rustymon. Supports single-touch and gesture recognition.

use esp_idf_svc::hal::i2c::I2cDriver;
use std::error::Error;
use std::fmt;
use std::thread;
use std::time::Duration;

pub const CST816D_DEVICE_ADDRESS: u8 = 0x15;

const CST816D_REG_GESTURE_ID: u8 = 0x01;
const CST816D_REG_FINGER_NUM: u8 = 0x02;
const CST816D_REG_XPOS_H: u8 = 0x03;
const CST816D_REG_CHIP_ID: u8 = 0xA7;
const CST816D_REG_FW_VERSION: u8 = 0xA9;

/// Hardware gesture types reported by CST816D.
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
    pub fn from_u8(value: u8) -> Self {
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

/// Touch point coordinates.
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
}

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

/// CST816D Touch Controller Driver.
pub struct Cst816dDriver {
    address: u8,
}

impl Cst816dDriver {
    pub fn new(address: u8) -> Self {
        Self { address }
    }

    pub fn initialize(&mut self, i2c: &mut I2cDriver) -> Result<(), TouchError> {
        log::info!("Initializing CST816D touch controller...");
        thread::sleep(Duration::from_millis(50));

        match self.read_register(i2c, CST816D_REG_CHIP_ID) {
            Ok(chip_id) => log::info!("Touch chip ID: 0x{:02X}", chip_id),
            Err(e) => {
                log::error!("Failed to read touch chip ID: {:?}", e);
                return Err(e);
            }
        }

        if let Ok(fw) = self.read_register(i2c, CST816D_REG_FW_VERSION) {
            log::info!("Touch FW version: 0x{:02X}", fw);
        }

        log::info!("CST816D initialized");
        Ok(())
    }

    fn read_register(&self, i2c: &mut I2cDriver, register: u8) -> Result<u8, TouchError> {
        let mut buf = [0u8; 1];
        i2c.write_read(self.address, &[register], &mut buf, 1000)
            .map_err(|e| TouchError::I2cError(format!("Read 0x{:02X}: {:?}", register, e)))?;
        Ok(buf[0])
    }

    fn read_registers(
        &self,
        i2c: &mut I2cDriver,
        register: u8,
        buf: &mut [u8],
    ) -> Result<(), TouchError> {
        i2c.write_read(self.address, &[register], buf, 1000)
            .map_err(|e| TouchError::I2cError(format!("Read 0x{:02X}: {:?}", register, e)))?;
        Ok(())
    }

    pub fn finger_number(&self, i2c: &mut I2cDriver) -> Result<u8, TouchError> {
        self.read_register(i2c, CST816D_REG_FINGER_NUM)
    }

    pub fn get_touch(&self, i2c: &mut I2cDriver) -> Result<Option<TouchPoint>, TouchError> {
        let finger_num = self.finger_number(i2c)?;
        if finger_num == 0 {
            return Ok(None);
        }

        let mut data = [0u8; 4];
        self.read_registers(i2c, CST816D_REG_XPOS_H, &mut data)?;

        let x = ((data[0] as u16 & 0x0F) << 8) | data[1] as u16;
        let y = ((data[2] as u16 & 0x0F) << 8) | data[3] as u16;

        Ok(Some(TouchPoint { x, y }))
    }

    pub fn get_touch_and_gesture(
        &self,
        i2c: &mut I2cDriver,
    ) -> Result<(Option<TouchPoint>, Gesture), TouchError> {
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
