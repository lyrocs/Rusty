// Touch driver implementation for FT3168 capacitive touch controller

use crate::hal::{TouchDriver, pins};
use crate::drivers::gpio_expander::Tca9554Driver;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_idf_svc::hal::i2c::I2cDriver;

const FT3168_ADDRESS: u8 = pins::touch::ADDRESS;

// FT3168 Registers
const REG_DEVICE_MODE: u8 = 0x00;
const REG_GESTURE_ID: u8 = 0x01;
const REG_TD_STATUS: u8 = 0x02;
const REG_TOUCH1_XH: u8 = 0x03;
const REG_TOUCH1_XL: u8 = 0x04;
const REG_TOUCH1_YH: u8 = 0x05;
const REG_TOUCH1_YL: u8 = 0x06;

// FT3168 Power and Configuration Registers
const REG_POWER_MODE: u8 = 0xA5;  // Power mode register
const REG_DEVICE_ID: u8 = 0xA0;    // Device ID register

// Power modes
const POWER_MODE_ACTIVE: u8 = 0x00;
const POWER_MODE_MONITOR: u8 = 0x01;
const POWER_MODE_STANDBY: u8 = 0x02;
const POWER_MODE_HIBERNATE: u8 = 0x03;

/// FT3168 Touch Driver for ESP-IDF
pub struct Ft3168TouchDriver<'d> {
    i2c: &'d mut I2cDriver<'static>,
    address: u8,
    last_touch: Option<(u16, u16)>,
    gesture_mode: bool,
    initialized: bool,
}

impl<'d> Ft3168TouchDriver<'d> {
    /// Create a new touch controller instance (without GPIO expander - reset handled elsewhere)
    pub fn new(
        i2c: &'d mut I2cDriver<'static>,
    ) -> Result<Self> {
        log::info!("Creating FT3168 touch driver (reset handled externally)");
        Ok(Self {
            i2c,
            address: FT3168_ADDRESS,
            last_touch: None,
            gesture_mode: false,
            initialized: false,
        })
    }

    /// Create a new touch controller instance with custom address
    pub fn new_with_address(
        i2c: &'d mut I2cDriver<'static>,
        address: u8,
    ) -> Result<Self> {
        log::info!("Creating FT3168 touch driver at address 0x{:02X} (reset handled externally)", address);
        Ok(Self {
            i2c,
            address,
            last_touch: None,
            gesture_mode: false,
            initialized: false,
        })
    }

    /// Try to detect the FT3168 on common addresses
    pub fn detect_address(i2c: &mut I2cDriver<'static>) -> Result<u8> {
        // Try common FT3168 addresses plus 0x18 (sometimes miswired or alternative config)
        let common_addresses = [0x38, 0x18, 0x39, 0x3A, 0x3B];

        log::info!("Detecting FT3168 touch controller...");

        // First, try to initialize power mode for standard address 0x38
        // This might wake up the controller even if it doesn't ACK simple writes
        log::info!("Attempting to wake up FT3168 at 0x38...");
        let _ = i2c.write(0x38, &[REG_POWER_MODE, POWER_MODE_ACTIVE], 1000);
        thread::sleep(Duration::from_millis(50));

        for &addr in &common_addresses {
            log::info!("Trying FT3168 at address 0x{:02X}...", addr);

            // Try to write power mode first
            let result = i2c.write(addr, &[REG_POWER_MODE, POWER_MODE_ACTIVE], 1000);
            if result.is_ok() {
                thread::sleep(Duration::from_millis(20));

                // Try to read device ID to confirm
                let mut data = [0u8; 1];
                let read_result = i2c.write_read(addr, &[REG_DEVICE_ID], &mut data, 1000);
                if read_result.is_ok() {
                    log::info!("Found FT3168 at address 0x{:02X} (ID: 0x{:02X})", addr, data[0]);
                    return Ok(addr);
                }
            }

            // Also try simple read
            let mut data = [0u8; 1];
            let result = i2c.write_read(addr, &[REG_DEVICE_MODE], &mut data, 1000);
            if result.is_ok() {
                log::info!("Found FT3168 at address 0x{:02X}", addr);
                return Ok(addr);
            }
        }

        anyhow::bail!("FT3168 not found on any common address (0x38, 0x18, 0x39-0x3B)")
    }

    /// Reset is handled externally via GPIO expander in main.rs
    fn reset(&mut self) -> Result<()> {
        log::debug!("Touch reset is handled externally");
        // Just wait for the controller to be ready
        thread::sleep(Duration::from_millis(300));
        Ok(())
    }

    /// Read a register from the touch controller
    fn read_register(&mut self, reg: u8) -> Result<u8> {
        let mut data = [0u8; 1];
        self.i2c.write_read(self.address, &[reg], &mut data, 1000)?;
        Ok(data[0])
    }

    /// Write a register to the touch controller
    fn write_register(&mut self, reg: u8, value: u8) -> Result<()> {
        self.i2c.write(self.address, &[reg, value], 1000)?;
        Ok(())
    }

    /// Read multiple registers
    fn read_registers(&mut self, reg: u8, buffer: &mut [u8]) -> Result<()> {
        self.i2c.write_read(self.address, &[reg], buffer, 1000)?;
        Ok(())
    }

    /// Initialize the touch controller
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("Initializing FT3168 touch controller");

        // Reset was already handled externally, just wait a bit more
        self.reset()?;

        // Set power mode to Active (0x00) - CRITICAL for FT3168 to respond
        log::info!("Setting FT3168 to Active power mode...");
        self.write_register(REG_POWER_MODE, POWER_MODE_ACTIVE)?;

        // Wait 20ms after setting power mode (per FT3168 datasheet)
        thread::sleep(Duration::from_millis(20));

        // Try to read device ID to verify communication
        match self.read_register(REG_DEVICE_ID) {
            Ok(id) => {
                log::info!("FT3168 Device ID: 0x{:02X}", id);
            }
            Err(e) => {
                log::warn!("Could not read FT3168 device ID: {:?}", e);
            }
        }

        // Check device mode (optional, for verification)
        let mode = self.read_register(REG_DEVICE_MODE)?;
        log::debug!("FT3168 device mode: 0x{:02X}", mode);

        // Set to normal operating mode
        self.write_register(REG_DEVICE_MODE, 0x00)?;

        self.initialized = true;
        log::info!("FT3168 touch initialized successfully");
        Ok(())
    }

    /// Read touch data from the controller
    fn read_touch_data(&mut self) -> Result<Option<(u16, u16)>> {
        if !self.initialized {
            return Ok(None);
        }

        // Read touch status - catch errors gracefully
        let td_status = match self.read_register(REG_TD_STATUS) {
            Ok(status) => status,
            Err(_) => {
                // Touch read can fail if device is busy or not touched
                return Ok(None);
            }
        };

        let touch_points = td_status & 0x0F;

        if touch_points == 0 {
            return Ok(None);
        }

        // Read first touch point coordinates
        let mut touch_data = [0u8; 4];
        if let Err(_) = self.read_registers(REG_TOUCH1_XH, &mut touch_data) {
            // Failed to read touch coordinates
            return Ok(None);
        }

        // Parse coordinates (FT3168 uses 12-bit coordinates)
        let x = (((touch_data[0] & 0x0F) as u16) << 8) | (touch_data[1] as u16);
        let y = (((touch_data[2] & 0x0F) as u16) << 8) | (touch_data[3] as u16);

        Ok(Some((x, y)))
    }
}

impl<'d> TouchDriver for Ft3168TouchDriver<'d> {
    fn read_touch(&mut self) -> Option<(u16, u16)> {
        match self.read_touch_data() {
            Ok(touch) => {
                self.last_touch = touch;
                touch
            }
            Err(e) => {
                log::error!("Touch read error: {:?}", e);
                None
            }
        }
    }

    fn is_touched(&mut self) -> bool {
        self.read_touch().is_some()
    }

    fn set_gesture_mode(&mut self, enabled: bool) -> Result<()> {
        log::info!("Setting gesture mode: {}", enabled);
        self.gesture_mode = enabled;

        if self.initialized {
            // Enable/disable gesture mode in controller
            // Note: Specific register depends on FT3168 firmware
            // This is a simplified implementation
            let mode = if enabled { 0x01 } else { 0x00 };
            self.write_register(REG_DEVICE_MODE, mode)?;
        }

        Ok(())
    }
}

/// Thread-safe touch wrapper
pub type SharedTouch = Arc<Mutex<dyn TouchDriver>>;

/// Create a shared touch instance (placeholder - use hardware initialization in main)
pub fn create_shared_touch() -> Result<SharedTouch> {
    Err(anyhow::anyhow!(
        "Touch must be initialized from main with peripheral access"
    ))
}
