//! ST7789P LCD Display Driver for ESP-IDF
//!
//! This module provides an ESP-IDF (std) compatible driver for the ST7789P LCD display controller.
//! It supports RGB565 and RGB888 color modes and uses standard SPI for pixel data transfer.
//!
//! # Hardware Configuration
//! - Display: Waveshare ESP32-C6-Touch-LCD-1.83 (240x284)
//! - Interface: SPI (standard 4-wire SPI)
//! - Color Format: RGB888 (24-bit color) or RGB565 (16-bit color)
//! - Framebuffer: Stored in RAM
//!
//! # Pin Configuration
//! - SCK: GPIO1
//! - MOSI: GPIO2
//! - CS: GPIO5
//! - DC: GPIO3
//! - RST: GPIO4
//! - BL: GPIO6
//!
//! # Features
//! - Hardware-accelerated SPI pixel transfer
//! - Full embedded-graphics DrawTarget support
//! - DMA-based chunked transfers for efficient updates
//!
//! # Example
//! ```no_run
//! use display::{St7789pDriver, ColorMode};
//! use embedded_graphics::prelude::*;
//!
//! let mut display = St7789pDriver::new(
//!     spi_device,
//!     dc_pin,
//!     rst_pin,
//!     240, // width
//!     284, // height
//!     ColorMode::Rgb888
//! )?;
//! display.initialize(ColorMode::Rgb888)?;
//! display.clear(Rgb888::BLACK)?;
//! display.flush()?;
//! ```

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};
use esp_idf_svc::hal::gpio::{Output, PinDriver};
use esp_idf_svc::hal::spi::SpiDeviceDriver;
use std::thread;
use std::time::Duration;

/// ST7789P Command Set
pub mod commands {
    pub const SWRESET: u8 = 0x01;
    pub const SLPIN: u8 = 0x10;
    pub const SLPOUT: u8 = 0x11;
    pub const INVOFF: u8 = 0x20;
    pub const INVON: u8 = 0x21;
    pub const DISPOFF: u8 = 0x28;
    pub const DISPON: u8 = 0x29;
    pub const CASET: u8 = 0x2A;
    pub const RASET: u8 = 0x2B;
    pub const RAMWR: u8 = 0x2C;
    pub const MADCTL: u8 = 0x36;
    pub const COLMOD: u8 = 0x3A;
    pub const PORCTRL: u8 = 0xB2;
    pub const GCTRL: u8 = 0xB7;
    pub const VCOMS: u8 = 0xBB;
    pub const LCMCTRL: u8 = 0xC0;
    pub const VDVVRHEN: u8 = 0xC2;
    pub const VRHS: u8 = 0xC3;
    pub const VDVS: u8 = 0xC4;
    pub const FRCTRL2: u8 = 0xC6;
    pub const PWCTRL1: u8 = 0xD0;
    pub const PVGAMCTRL: u8 = 0xE0;
    pub const NVGAMCTRL: u8 = 0xE1;
}

const DMA_CHUNK_SIZE: usize = 4096;

/// Color modes supported by ST7789P
#[derive(Clone, Copy)]
pub enum ColorMode {
    Rgb565,
    Rgb888,
}

impl ColorMode {
    pub const fn colmod_value(&self) -> u8 {
        match self {
            ColorMode::Rgb565 => 0x55,
            ColorMode::Rgb888 => 0x66,
        }
    }

    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            ColorMode::Rgb565 => 2,
            ColorMode::Rgb888 => 3,
        }
    }
}

/// ST7789P Driver for ESP-IDF with standard SPI support
/// Supports shared SPI bus (using &'static SpiDriver reference)
pub struct St7789pDriver<'a, DC, RST>
where
    DC: esp_idf_svc::hal::gpio::OutputPin,
    RST: esp_idf_svc::hal::gpio::OutputPin,
{
    spi: SpiDeviceDriver<'a, &'a esp_idf_svc::hal::spi::SpiDriver<'a>>,
    dc: PinDriver<'a, DC, Output>,
    rst: PinDriver<'a, RST, Output>,
    framebuffer: Vec<u8>,
    width: u16,
    height: u16,
    color_mode: ColorMode,
    /// Backlight pin (GPIO6) - None if not connected
    backlight: Option<PinDriver<'a, esp_idf_svc::hal::gpio::Gpio6, Output>>,
}

impl<'a, DC, RST> St7789pDriver<'a, DC, RST>
where
    DC: esp_idf_svc::hal::gpio::OutputPin,
    RST: esp_idf_svc::hal::gpio::OutputPin,
{
    /// Create a new ST7789P driver instance
    ///
    /// # Arguments
    /// * `spi` - SPI device driver
    /// * `dc` - Data/Command pin driver
    /// * `rst` - Reset pin driver
    /// * `width` - Display width in pixels
    /// * `height` - Display height in pixels
    /// * `color_mode` - Color mode (RGB565 or RGB888)
    pub fn new(
        spi: SpiDeviceDriver<'a, &'a esp_idf_svc::hal::spi::SpiDriver<'a>>,
        dc: PinDriver<'a, DC, Output>,
        rst: PinDriver<'a, RST, Output>,
        width: u16,
        height: u16,
        color_mode: ColorMode,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create framebuffer
        let fb_size = (width as usize) * (height as usize) * color_mode.bytes_per_pixel();
        let framebuffer = vec![0u8; fb_size];

        let driver = Self {
            spi,
            dc,
            rst,
            framebuffer,
            width,
            height,
            color_mode,
            backlight: None,
        };

        Ok(driver)
    }

    /// Set the backlight pin for controlling display brightness
    pub fn set_backlight_pin(&mut self, backlight: PinDriver<'a, esp_idf_svc::hal::gpio::Gpio6, Output>) {
        self.backlight = Some(backlight);
    }

    /// Turn backlight on
    pub fn backlight_on(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut bl) = self.backlight {
            bl.set_high()?;
            log::info!("Backlight ON");
        }
        Ok(())
    }

    /// Turn backlight off
    pub fn backlight_off(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut bl) = self.backlight {
            bl.set_low()?;
            log::info!("Backlight OFF");
        }
        Ok(())
    }

    /// Initialize the display
    pub fn initialize(&mut self, color_mode: ColorMode) -> Result<(), Box<dyn std::error::Error>> {
        // Hardware reset
        self.rst.set_high()?;
        thread::sleep(Duration::from_millis(5));
        self.rst.set_low()?;
        thread::sleep(Duration::from_millis(20));
        self.rst.set_high()?;
        thread::sleep(Duration::from_millis(150));

        // Software reset
        self.send_command(commands::SWRESET)?;
        thread::sleep(Duration::from_millis(150));

        // Sleep out
        self.send_command(commands::SLPOUT)?;
        thread::sleep(Duration::from_millis(120));

        // Memory Data Access Control
        // Bit 3 = BGR order (instead of RGB)
        // 0x08 = BGR color order, normal orientation
        self.send_command_with_data(commands::MADCTL, &[0x08])?;

        // Interface Pixel Format
        self.send_command_with_data(commands::COLMOD, &[color_mode.colmod_value()])?;

        // Porch Setting
        self.send_command_with_data(commands::PORCTRL, &[0x0C, 0x0C, 0x00, 0x33, 0x33])?;

        // Gate Control
        self.send_command_with_data(commands::GCTRL, &[0x35])?;

        // VCOM Setting
        self.send_command_with_data(commands::VCOMS, &[0x19])?;

        // LCM Control
        self.send_command_with_data(commands::LCMCTRL, &[0x2C])?;

        // VDV and VRH Command Enable
        self.send_command_with_data(commands::VDVVRHEN, &[0x01])?;

        // VRH Set
        self.send_command_with_data(commands::VRHS, &[0x12])?;

        // VDV Set
        self.send_command_with_data(commands::VDVS, &[0x20])?;

        // Frame Rate Control
        self.send_command_with_data(commands::FRCTRL2, &[0x0F])?;

        // Power Control 1
        self.send_command_with_data(commands::PWCTRL1, &[0xA4, 0xA1])?;

        // Positive Voltage Gamma Control
        self.send_command_with_data(
            commands::PVGAMCTRL,
            &[
                0xD0, 0x04, 0x0D, 0x11, 0x13, 0x2B, 0x3F, 0x54, 0x4C, 0x18, 0x0D, 0x0B, 0x1F, 0x23,
            ],
        )?;

        // Negative Voltage Gamma Control
        self.send_command_with_data(
            commands::NVGAMCTRL,
            &[
                0xD0, 0x04, 0x0C, 0x11, 0x13, 0x2C, 0x3F, 0x44, 0x51, 0x2F, 0x1F, 0x1F, 0x20, 0x23,
            ],
        )?;

        // Display Inversion ON (needed for correct colors on this LCD)
        self.send_command(commands::INVON)?;

        // Display on
        self.send_command(commands::DISPON)?;
        thread::sleep(Duration::from_millis(120));

        Ok(())
    }

    fn send_command(&mut self, cmd: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.dc.set_low()?;
        self.spi.write(&[cmd])?;
        Ok(())
    }

    fn send_command_with_data(&mut self, cmd: u8, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(cmd)?;
        self.dc.set_high()?;
        self.spi.write(data)?;
        Ok(())
    }

    fn set_window(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Column address set
        self.send_command_with_data(
            commands::CASET,
            &[
                (x_start >> 8) as u8,
                (x_start & 0xFF) as u8,
                (x_end >> 8) as u8,
                (x_end & 0xFF) as u8,
            ],
        )?;

        // Row address set
        self.send_command_with_data(
            commands::RASET,
            &[
                (y_start >> 8) as u8,
                (y_start & 0xFF) as u8,
                (y_end >> 8) as u8,
                (y_end & 0xFF) as u8,
            ],
        )?;

        Ok(())
    }

    /// Flush framebuffer to display using SPI
    pub fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_window(0, 0, self.width - 1, self.height - 1)?;

        // Send RAMWR command
        self.send_command(commands::RAMWR)?;

        // Send pixel data
        self.dc.set_high()?;
        for chunk in self.framebuffer.chunks(DMA_CHUNK_SIZE) {
            self.spi.write(chunk)?;
        }

        Ok(())
    }

    /// Turn display on
    pub fn display_on(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Exit sleep mode
        self.send_command(commands::SLPOUT)?;
        thread::sleep(Duration::from_millis(120));

        // Turn display on
        self.send_command(commands::DISPON)?;
        thread::sleep(Duration::from_millis(10));

        // Turn backlight on
        self.backlight_on()?;

        Ok(())
    }

    /// Turn display off
    pub fn display_off(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Turn backlight off first
        self.backlight_off()?;

        // Turn display off
        self.send_command(commands::DISPOFF)?;
        thread::sleep(Duration::from_millis(10));

        // Enter sleep mode
        self.send_command(commands::SLPIN)?;
        thread::sleep(Duration::from_millis(120));

        Ok(())
    }

    /// Set a single pixel directly in the framebuffer (fast path)
    /// Coordinates are in display space (0,0 is top-left)
    /// Converts RGB888 to framebuffer color mode automatically
    #[inline]
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Rgb888) {
        if x < self.width && y < self.height {
            match self.color_mode {
                ColorMode::Rgb888 => {
                    let offset = (y as usize * self.width as usize + x as usize) * 3;
                    if offset + 2 < self.framebuffer.len() {
                        self.framebuffer[offset] = color.r();
                        self.framebuffer[offset + 1] = color.g();
                        self.framebuffer[offset + 2] = color.b();
                    }
                }
                ColorMode::Rgb565 => {
                    // Convert RGB888 to RGB565
                    let r = (color.r() >> 3) as u16;
                    let g = (color.g() >> 2) as u16;
                    let b = (color.b() >> 3) as u16;
                    let rgb565 = (r << 11) | (g << 5) | b;

                    let offset = (y as usize * self.width as usize + x as usize) * 2;
                    if offset + 1 < self.framebuffer.len() {
                        // Big-endian (MSB first) - ST7789P expects this format
                        self.framebuffer[offset] = (rgb565 >> 8) as u8;
                        self.framebuffer[offset + 1] = (rgb565 & 0xFF) as u8;
                    }
                }
            }
        }
    }

    /// Set a single pixel from RGB565 value (fast path)
    #[inline]
    pub fn set_pixel_rgb565(&mut self, x: u16, y: u16, rgb565: u16) {
        if x < self.width && y < self.height {
            match self.color_mode {
                ColorMode::Rgb565 => {
                    // Direct write to RGB565 framebuffer (zero-cost!)
                    let offset = (y as usize * self.width as usize + x as usize) * 2;
                    if offset + 1 < self.framebuffer.len() {
                        // Big-endian (MSB first) - ST7789P expects this format
                        self.framebuffer[offset] = (rgb565 >> 8) as u8;
                        self.framebuffer[offset + 1] = (rgb565 & 0xFF) as u8;
                    }
                }
                ColorMode::Rgb888 => {
                    // Convert RGB565 to RGB888
                    let r = ((rgb565 >> 11) & 0x1F) as u8;
                    let g = ((rgb565 >> 5) & 0x3F) as u8;
                    let b = (rgb565 & 0x1F) as u8;

                    // Expand to 8-bit
                    let r = (r << 3) | (r >> 2);
                    let g = (g << 2) | (g >> 4);
                    let b = (b << 3) | (b >> 2);

                    let offset = (y as usize * self.width as usize + x as usize) * 3;
                    if offset + 2 < self.framebuffer.len() {
                        self.framebuffer[offset] = r;
                        self.framebuffer[offset + 1] = g;
                        self.framebuffer[offset + 2] = b;
                    }
                }
            }
        }
    }

    /// Blit RGB565 data directly to framebuffer (fast bulk transfer)
    /// Returns the number of pixels written
    pub fn blit_rgb565(
        &mut self,
        x: i32,
        y: i32,
        width: u16,
        height: u16,
        data: &[u8],
        flip_h: bool,
        alpha_mask: Option<&[u8]>,
    ) -> usize {
        let w = width as usize;
        let h = height as usize;
        let mut pixels_written = 0;

        for py in 0..h {
            let draw_y = y + py as i32;
            if draw_y < 0 || draw_y >= self.height as i32 {
                continue;
            }

            for px in 0..w {
                let draw_x = x + px as i32;
                if draw_x < 0 || draw_x >= self.width as i32 {
                    continue;
                }

                let src_x = if flip_h { w - 1 - px } else { px };
                let pixel_idx = py * w + src_x;

                // Check transparency
                if let Some(mask) = alpha_mask {
                    let byte_idx = pixel_idx / 8;
                    let bit_idx = pixel_idx % 8;
                    if byte_idx < mask.len() && (mask[byte_idx] & (1 << bit_idx)) == 0 {
                        continue; // Skip transparent pixel
                    }
                }

                // Read RGB565 (little-endian)
                let data_idx = pixel_idx * 2;
                if data_idx + 1 >= data.len() {
                    continue;
                }
                let rgb565 = u16::from_le_bytes([data[data_idx], data[data_idx + 1]]);

                self.set_pixel_rgb565(draw_x as u16, draw_y as u16, rgb565);
                pixels_written += 1;
            }
        }

        pixels_written
    }
}

impl<DC, RST> DrawTarget for St7789pDriver<'_, DC, RST>
where
    DC: esp_idf_svc::hal::gpio::OutputPin,
    RST: esp_idf_svc::hal::gpio::OutputPin,
{
    type Color = Rgb888;
    type Error = Box<dyn std::error::Error>;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0 && coord.x < self.width as i32 && coord.y >= 0 && coord.y < self.height as i32 {
                // Use set_pixel which handles both RGB888 and RGB565 modes
                self.set_pixel(coord.x as u16, coord.y as u16, color);
            }
        }
        Ok(())
    }
}

impl<DC, RST> OriginDimensions for St7789pDriver<'_, DC, RST>
where
    DC: esp_idf_svc::hal::gpio::OutputPin,
    RST: esp_idf_svc::hal::gpio::OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}
