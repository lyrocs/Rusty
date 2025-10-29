// QSPI Display driver implementation for SH8601
//
// This implements proper QSPI protocol for the Waveshare 1.8" AMOLED
// Based on sh8601-rs but adapted for std environment with esp-idf-svc

use anyhow::Result;
use esp_idf_svc::hal::spi::SpiDeviceDriver;
use esp_idf_svc::hal::spi::SpiDriver;
use std::thread;
use std::time::Duration;

use crate::drivers::gpio_expander::Tca9554Driver;

// SH8601 Commands
const CMD_SWRESET: u8 = 0x01;  // Software Reset
const CMD_SLPOUT: u8 = 0x11;   // Sleep Out
const CMD_NORON: u8 = 0x13;    // Normal Display Mode On
const CMD_DISPON: u8 = 0x29;   // Display On
const CMD_CASET: u8 = 0x2A;    // Column Address Set
const CMD_RASET: u8 = 0x2B;    // Row Address Set
const CMD_RAMWR: u8 = 0x2C;    // Memory Write
const CMD_PTLAR: u8 = 0x30;    // Partial Area
const CMD_TEON: u8 = 0x35;     // Tearing Effect Line On
const CMD_MADCTL: u8 = 0x36;   // Memory Data Access Control
const CMD_COLMOD: u8 = 0x3A;   // Interface Pixel Format
const CMD_TESCAN: u8 = 0x44;   // Tearing Effect Scan Line

// QSPI Command Format for SH8601
// The SH8601 in QSPI mode uses a special protocol:
// - First byte: Command (0x02 prefix for write commands, 0x32 for quad write)
// - Second byte: The actual command
// - Following bytes: Data (if any)

const QSPI_WRITE_CMD: u8 = 0x02;  // Standard write command prefix
const QSPI_QUAD_WRITE: u8 = 0x32; // Quad write for pixel data

pub struct QspiDisplayDriver<'d> {
    spi: &'d mut SpiDeviceDriver<'static, &'static mut SpiDriver<'static>>,
    gpio_expander: Tca9554Driver<'d>,
    width: u16,
    height: u16,
    initialized: bool,
}

impl<'d> QspiDisplayDriver<'d> {
    pub fn new(
        spi: &'d mut SpiDeviceDriver<'static, &'static mut SpiDriver<'static>>,
        gpio_expander: Tca9554Driver<'d>,
    ) -> Result<Self> {
        log::info!("Creating QSPI display driver for SH8601");
        Ok(Self {
            spi,
            gpio_expander,
            width: 368,
            height: 448,
            initialized: false,
        })
    }

    /// Reset the display using GPIO expander pin 0
    fn reset(&mut self) -> Result<()> {
        log::info!("Resetting display with extended timing...");

        // Configure reset pin as output
        self.gpio_expander.configure_pin(0, false)?;

        // Extended reset sequence for stability
        log::info!("Setting reset HIGH (idle state)...");
        self.gpio_expander.write_pin(0, true)?;
        thread::sleep(Duration::from_millis(20));

        log::info!("Asserting reset LOW...");
        self.gpio_expander.write_pin(0, false)?;
        thread::sleep(Duration::from_millis(50)); // Longer reset pulse

        log::info!("Releasing reset HIGH...");
        self.gpio_expander.write_pin(0, true)?;
        thread::sleep(Duration::from_millis(200)); // Longer boot time

        log::info!("Reset complete, display should be powered up");
        Ok(())
    }

    /// Send command - try standard SPI mode (no QSPI prefix)
    fn send_command(&mut self, cmd: u8) -> Result<()> {
        // Try standard SPI command format
        log::trace!("SPI CMD: 0x{:02X}", cmd);
        let data = [cmd];
        self.spi.write(&data)?;
        thread::sleep(Duration::from_micros(100));
        Ok(())
    }

    /// Send command with data - standard SPI
    fn send_command_with_data(&mut self, cmd: u8, data: &[u8]) -> Result<()> {
        // Send command
        self.send_command(cmd)?;

        // Send data
        if !data.is_empty() {
            log::trace!("SPI DATA: {} bytes for cmd 0x{:02X}", data.len(), cmd);
            self.spi.write(data)?;
            thread::sleep(Duration::from_micros(100));
        }
        Ok(())
    }

    /// Send pixel data - standard SPI
    fn send_pixels(&mut self, pixels: &[u8]) -> Result<()> {
        // Send memory write command
        self.send_command(CMD_RAMWR)?;

        // Send pixel data in chunks (RGB888 format)
        const CHUNK_SIZE: usize = 4096;
        log::info!("Sending {} bytes of pixel data in chunks of {}...", pixels.len(), CHUNK_SIZE);

        for (i, chunk) in pixels.chunks(CHUNK_SIZE).enumerate() {
            if i % 10 == 0 {
                log::debug!("Sending chunk {} ({} bytes)...", i, chunk.len());
            }
            self.spi.write(chunk)?;
        }

        log::info!("Pixel data transfer complete");
        Ok(())
    }

    /// Initialize the display
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("Initializing SH8601 display in QSPI mode...");

        // Hardware reset
        self.reset()?;

        // Software reset
        log::info!("Software reset...");
        self.send_command(CMD_SWRESET)?;
        thread::sleep(Duration::from_millis(10));

        // Sleep out
        log::info!("Sleep out...");
        self.send_command(CMD_SLPOUT)?;
        thread::sleep(Duration::from_millis(120));

        // Set color mode to RGB888 (24-bit)
        log::info!("Setting color mode to RGB888...");
        self.send_command_with_data(CMD_COLMOD, &[0x77])?;

        // Memory data access control (orientation)
        log::info!("Setting memory access control...");
        self.send_command_with_data(CMD_MADCTL, &[0x00])?;

        // Set tearing effect scan line
        log::info!("Configuring tearing effect...");
        self.send_command_with_data(CMD_TESCAN, &[0x01, 0xC5])?;

        // Enable tearing effect line
        self.send_command_with_data(CMD_TEON, &[0x00])?;

        // Display on
        log::info!("Display on...");
        self.send_command(CMD_DISPON)?;
        thread::sleep(Duration::from_millis(120));

        // Try setting brightness/backlight control
        // Command 0x51 - Write Display Brightness
        log::info!("Setting brightness to MAX...");
        self.send_command_with_data(0x51, &[0xFF])?; // Max brightness

        // Command 0x53 - Write CTRL Display
        log::info!("Enabling display control...");
        self.send_command_with_data(0x53, &[0x2C])?; // Enable backlight, display on

        // Inversion ON (some AMOLEDs need this)
        log::info!("Display inversion ON...");
        self.send_command(0x21)?; // Display Inversion ON

        // Set partial area (full screen)
        log::info!("Setting partial area...");
        self.send_command_with_data(CMD_PTLAR, &[0x00, 0x80, 0x01, 0xBF])?;
        thread::sleep(Duration::from_millis(10));

        // Normal display mode
        self.send_command(CMD_NORON)?;

        self.initialized = true;
        log::info!("SH8601 display initialized successfully!");
        Ok(())
    }

    /// Set drawing window
    pub fn set_window(&mut self, x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        let x_end = x + width - 1;
        let y_end = y + height - 1;

        // Column address set
        self.send_command_with_data(CMD_CASET, &[
            (x >> 8) as u8,
            (x & 0xFF) as u8,
            (x_end >> 8) as u8,
            (x_end & 0xFF) as u8,
        ])?;

        // Row address set
        self.send_command_with_data(CMD_RASET, &[
            (y >> 8) as u8,
            (y & 0xFF) as u8,
            (y_end >> 8) as u8,
            (y_end & 0xFF) as u8,
        ])?;

        Ok(())
    }

    /// Draw a buffer to the display
    pub fn draw_buffer(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        if !self.initialized {
            log::warn!("Display not initialized");
            return Ok(());
        }

        log::debug!("Drawing buffer at ({}, {}), size {}x{}", x, y, width, height);

        // Set the drawing window
        self.set_window(x, y, width, height)?;

        // Send pixel data
        self.send_pixels(buffer)?;

        Ok(())
    }

    /// Clear the display to black
    pub fn clear(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("Clearing display...");

        // Set full screen window
        self.set_window(0, 0, self.width, self.height)?;

        // Create black buffer and send it
        let buffer_size = (self.width as usize) * (self.height as usize) * 3;
        let black_buffer = vec![0u8; buffer_size];
        self.send_pixels(&black_buffer)?;

        Ok(())
    }
}