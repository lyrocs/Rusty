// Display driver implementation using ESP-IDF HAL
//
// SH8601 AMOLED display driver for Waveshare 1.8" AMOLED

use crate::hal::{DisplayDriver, pins};
use crate::drivers::gpio_expander::Tca9554Driver;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_idf_svc::hal::{
    delay::Delay,
    spi::{SpiDeviceDriver, SpiDriver},
};

pub const DISPLAY_WIDTH: u16 = pins::display_spec::WIDTH;
pub const DISPLAY_HEIGHT: u16 = pins::display_spec::HEIGHT;

/// Frame buffer for double buffering
pub struct FrameBuffer {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

impl FrameBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize) * 3; // RGB888
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, r: u8, g: u8, b: u8) {
        if x < self.width && y < self.height {
            let idx = ((y as usize * self.width as usize) + x as usize) * 3;
            self.data[idx] = r;
            self.data[idx + 1] = g;
            self.data[idx + 2] = b;
        }
    }

    pub fn draw_rect(&mut self, x: u16, y: u16, width: u16, height: u16, r: u8, g: u8, b: u8) {
        for dy in 0..height {
            for dx in 0..width {
                self.set_pixel(x + dx, y + dy, r, g, b);
            }
        }
    }
}

/// SH8601 Display Driver for ESP-IDF
pub struct Sh8601DisplayDriver<'d> {
    spi: &'d mut SpiDeviceDriver<'static, &'static mut SpiDriver<'static>>,
    gpio_expander: Tca9554Driver<'d>,
    width: u16,
    height: u16,
    initialized: bool,
}

impl<'d> Sh8601DisplayDriver<'d> {
    /// Create a new display driver instance
    pub fn new(
        spi: &'d mut SpiDeviceDriver<'static, &'static mut SpiDriver<'static>>,
        gpio_expander: Tca9554Driver<'d>,
    ) -> Result<Self> {
        log::info!("Creating SH8601 display driver");
        Ok(Self {
            spi,
            gpio_expander,
            width: DISPLAY_WIDTH,
            height: DISPLAY_HEIGHT,
            initialized: false,
        })
    }

    /// Reset the display via GPIO expander
    fn reset(&mut self) -> Result<()> {
        log::debug!("Resetting display");

        // Configure reset pin as output (pin 0)
        self.gpio_expander.configure_pin(0, false)?;

        // Reset sequence: low -> wait -> high -> wait
        self.gpio_expander.write_pin(0, false)?;
        thread::sleep(Duration::from_millis(10));

        self.gpio_expander.write_pin(0, true)?;
        thread::sleep(Duration::from_millis(120));

        Ok(())
    }

    /// Send command to display
    fn send_command(&mut self, cmd: u8) -> Result<()> {
        // For SH8601, command/data is controlled by D/C pin or first bit
        // In QSPI mode with Waveshare driver, we send command as single byte
        let data = [cmd];
        self.spi.write(&data)?;
        Ok(())
    }

    /// Send data to display
    fn send_data(&mut self, data: &[u8]) -> Result<()> {
        self.spi.write(data)?;
        Ok(())
    }

    /// Initialize the display hardware
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("Initializing SH8601 AMOLED display");

        // Reset the display
        self.reset()?;

        // SH8601 initialization sequence
        // Note: These commands are based on typical AMOLED controller initialization
        // You may need to adjust based on the actual SH8601 datasheet

        // Sleep Out
        self.send_command(0x11)?;
        thread::sleep(Duration::from_millis(120));

        // Display Inversion On (typical for AMOLED)
        self.send_command(0x21)?;

        // Pixel Format Set - 24bit/pixel (RGB888)
        self.send_command(0x3A)?;
        self.send_data(&[0x77])?; // 24-bit color

        // Memory Data Access Control
        self.send_command(0x36)?;
        self.send_data(&[0x00])?; // Normal orientation

        // Column Address Set (0 to WIDTH-1)
        self.send_command(0x2A)?;
        self.send_data(&[
            0x00, 0x00, // Start column
            ((self.width - 1) >> 8) as u8,
            ((self.width - 1) & 0xFF) as u8, // End column
        ])?;

        // Row Address Set (0 to HEIGHT-1)
        self.send_command(0x2B)?;
        self.send_data(&[
            0x00, 0x00, // Start row
            ((self.height - 1) >> 8) as u8,
            ((self.height - 1) & 0xFF) as u8, // End row
        ])?;

        // Display On
        self.send_command(0x29)?;
        thread::sleep(Duration::from_millis(20));

        self.initialized = true;
        log::info!("SH8601 display initialized successfully");
        Ok(())
    }

    /// Set the drawing window
    fn set_window(&mut self, x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        let x_end = x + width - 1;
        let y_end = y + height - 1;

        // Column Address Set
        self.send_command(0x2A)?;
        self.send_data(&[
            (x >> 8) as u8,
            (x & 0xFF) as u8,
            (x_end >> 8) as u8,
            (x_end & 0xFF) as u8,
        ])?;

        // Row Address Set
        self.send_command(0x2B)?;
        self.send_data(&[
            (y >> 8) as u8,
            (y & 0xFF) as u8,
            (y_end >> 8) as u8,
            (y_end & 0xFF) as u8,
        ])?;

        // Memory Write
        self.send_command(0x2C)?;

        Ok(())
    }

}

impl<'d> DisplayDriver for Sh8601DisplayDriver<'d> {
    fn draw_buffer(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        if !self.initialized {
            log::warn!("Display not initialized, skipping draw");
            return Ok(());
        }

        log::trace!("Drawing buffer at ({}, {}), size {}x{}", x, y, width, height);

        // Set drawing window
        self.set_window(x, y, width, height)?;

        // Send pixel data (RGB888 format)
        self.send_data(buffer)?;

        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        log::trace!("Clearing display");

        // Set full screen window
        self.set_window(0, 0, self.width, self.height)?;

        // Send black pixels (all zeros)
        let black_buffer = vec![0u8; (self.width as usize * self.height as usize * 3)];
        self.send_data(&black_buffer)?;

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        // SPI transfer is immediate, no flush needed
        Ok(())
    }

    fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}

/// Thread-safe display wrapper
pub type SharedDisplay = Arc<Mutex<dyn DisplayDriver>>;

/// Create a shared display instance (placeholder - use create_display_with_hardware)
pub fn create_shared_display() -> Result<SharedDisplay> {
    Err(anyhow::anyhow!(
        "Display must be initialized from main with peripheral access"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_creation() {
        let fb = FrameBuffer::new(240, 280);
        assert_eq!(fb.width, 240);
        assert_eq!(fb.height, 280);
        assert_eq!(fb.data.len(), 240 * 280 * 3);
    }

    #[test]
    fn test_framebuffer_set_pixel() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.set_pixel(10, 20, 255, 128, 64);
        let idx = (20 * 100 + 10) * 3;
        assert_eq!(fb.data[idx], 255);
        assert_eq!(fb.data[idx + 1], 128);
        assert_eq!(fb.data[idx + 2], 64);
    }
}
