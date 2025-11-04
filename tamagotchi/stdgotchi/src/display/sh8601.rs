//! SH8601 AMOLED Display Driver for ESP-IDF
//!
//! This is an ESP-IDF compatible port of the sh8601-rs driver.
//! Original: https://github.com/theembeddedrustacean/sh8601-rs

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};
use esp_idf_svc::sys::*;
use std::ptr;

/// SH8601 Command Set
pub mod commands {
    pub const SWRESET: u8 = 0x01;
    pub const SLPOUT: u8 = 0x11;
    pub const DISPON: u8 = 0x29;
    pub const CASET: u8 = 0x2A;
    pub const PASET: u8 = 0x2B;
    pub const RAMWR: u8 = 0x2C;
    pub const RAMWRC: u8 = 0x3C;
    pub const MADCTL: u8 = 0x36;
    pub const COLMOD: u8 = 0x3A;
    pub const TESCAN: u8 = 0x44;
    pub const TEON: u8 = 0x35;
    pub const PTLAR: u8 = 0x30;
}

const QSPI_PIXEL_OPCODE: u8 = 0x32;
const QSPI_CONTROL_OPCODE: u8 = 0x02;
const DMA_CHUNK_SIZE: usize = 16380;

/// Color modes supported by SH8601
#[derive(Clone, Copy)]
pub enum ColorMode {
    Rgb565,
    Rgb888,
}

impl ColorMode {
    pub const fn colmod_value(&self) -> u8 {
        match self {
            ColorMode::Rgb565 => 0x55,
            ColorMode::Rgb888 => 0x77,
        }
    }

    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            ColorMode::Rgb565 => 2,
            ColorMode::Rgb888 => 3,
        }
    }
}

/// SH8601 Driver for ESP-IDF with proper QSPI support
pub struct Sh8601Driver {
    spi_device: spi_device_handle_t,
    framebuffer: Vec<u8>,
    width: u16,
    height: u16,
}

impl Sh8601Driver {
    /// Create a new SH8601 driver instance
    ///
    /// # Arguments
    /// * `spi_host` - SPI host (e.g., SPI2_HOST)
    /// * `cs_pin` - Chip select GPIO pin number
    /// * `width` - Display width in pixels
    /// * `height` - Display height in pixels
    /// * `color_mode` - Color mode (RGB565 or RGB888)
    pub fn new(
        spi_host: spi_host_device_t,
        cs_pin: i32,
        width: u16,
        height: u16,
        color_mode: ColorMode,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create framebuffer
        let fb_size = (width as usize) * (height as usize) * color_mode.bytes_per_pixel();
        let framebuffer = vec![0u8; fb_size];

        // Configure SPI device with command and address phases
        let dev_cfg = spi_device_interface_config_t {
            command_bits: 8,           // 8-bit command phase
            address_bits: 24,          // 24-bit address phase
            mode: 0,                   // SPI mode 0
            clock_speed_hz: 40_000_000,
            spics_io_num: cs_pin,
            queue_size: 7,
            flags: SPI_DEVICE_HALFDUPLEX,
            pre_cb: None,
            post_cb: None,
            ..Default::default()
        };

        let mut spi_device: spi_device_handle_t = ptr::null_mut();

        unsafe {
            let ret = spi_bus_add_device(spi_host, &dev_cfg, &mut spi_device);
            if ret != ESP_OK {
                return Err(format!("Failed to add SPI device: {}", ret).into());
            }
        }

        let driver = Self {
            spi_device,
            framebuffer,
            width,
            height,
        };

        Ok(driver)
    }

    /// Initialize the display
    pub fn initialize(&mut self, color_mode: ColorMode) -> Result<(), Box<dyn std::error::Error>> {
        use std::thread;
        use std::time::Duration;

        // Software reset
        self.send_command(commands::SWRESET)?;
        thread::sleep(Duration::from_millis(10));

        // Sleep out
        self.send_command(commands::SLPOUT)?;
        thread::sleep(Duration::from_millis(120));

        // Set pixel format
        self.send_command_with_data(commands::COLMOD, &[color_mode.colmod_value()])?;
        thread::sleep(Duration::from_millis(5));

        // Set MADCTL
        self.send_command_with_data(commands::MADCTL, &[0x00])?;

        // Configure tearing effect scan line
        self.send_command_with_data(commands::TESCAN, &[0x01, 0xC5])?;

        // Enable tearing effect
        self.send_command_with_data(commands::TEON, &[0x00])?;

        // Display on
        self.send_command(commands::DISPON)?;
        thread::sleep(Duration::from_millis(120));

        // Partial area row set
        self.send_command_with_data(commands::PTLAR, &[0x00, 0x80, 0x00, 0x02])?;
        thread::sleep(Duration::from_millis(10));

        Ok(())
    }

    fn send_command(&mut self, cmd: u8) -> Result<(), Box<dyn std::error::Error>> {
        // Use spi_device_transmit for proper half-duplex with cmd/addr
        unsafe {
            let mut trans: spi_transaction_t = std::mem::zeroed();
            trans.cmd = QSPI_CONTROL_OPCODE as u16;
            trans.addr = (cmd as u64) << 8;
            trans.length = 0;
            trans.rxlength = 0;

            let ret = spi_device_transmit(self.spi_device, &mut trans as *mut _);
            if ret != ESP_OK {
                return Err(format!("send_command failed: {}", ret).into());
            }
        }
        Ok(())
    }

    fn send_command_with_data(&mut self, cmd: u8, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let mut trans: spi_transaction_t = std::mem::zeroed();
            trans.cmd = QSPI_CONTROL_OPCODE as u16;
            trans.addr = (cmd as u64) << 8;
            trans.length = data.len() * 8;
            trans.__bindgen_anon_1.tx_buffer = data.as_ptr() as *const _;

            let ret = spi_device_transmit(self.spi_device, &mut trans as *mut _);
            if ret != ESP_OK {
                return Err(format!("send_command_with_data failed: {}", ret).into());
            }
        }
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

        // Page address set
        self.send_command_with_data(
            commands::PASET,
            &[
                (y_start >> 8) as u8,
                (y_start & 0xFF) as u8,
                (y_end >> 8) as u8,
                (y_end & 0xFF) as u8,
            ],
        )?;

        Ok(())
    }

    /// Flush framebuffer to display using QSPI
    pub fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_window(0, 0, self.width - 1, self.height - 1)?;

        // Send pixels in QSPI quad mode
        let mut first = true;
        for chunk in self.framebuffer.chunks(DMA_CHUNK_SIZE) {
            let cmd = if first {
                first = false;
                commands::RAMWR
            } else {
                commands::RAMWRC
            };

            unsafe {
                let mut trans: spi_transaction_t = std::mem::zeroed();
                trans.flags = SPI_TRANS_MODE_QIO; // Quad mode for data
                trans.cmd = QSPI_PIXEL_OPCODE as u16;
                trans.addr = (cmd as u64) << 8;
                trans.length = chunk.len() * 8;
                trans.__bindgen_anon_1.tx_buffer = chunk.as_ptr() as *const _;

                let ret = spi_device_transmit(self.spi_device, &mut trans as *mut _);
                if ret != ESP_OK {
                    return Err(format!("flush pixel transmission failed: {}", ret).into());
                }
            }
        }

        Ok(())
    }
}

impl Drop for Sh8601Driver {
    fn drop(&mut self) {
        unsafe {
            if !self.spi_device.is_null() {
                spi_bus_remove_device(self.spi_device);
            }
        }
    }
}

impl DrawTarget for Sh8601Driver {
    type Color = Rgb888;
    type Error = Box<dyn std::error::Error>;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0 && coord.x < self.width as i32 && coord.y >= 0 && coord.y < self.height as i32 {
                let x = coord.x as usize;
                let y = coord.y as usize;
                let offset = (y * self.width as usize + x) * 3;

                if offset + 2 < self.framebuffer.len() {
                    self.framebuffer[offset] = color.r();
                    self.framebuffer[offset + 1] = color.g();
                    self.framebuffer[offset + 2] = color.b();
                }
            }
        }
        Ok(())
    }
}

impl OriginDimensions for Sh8601Driver {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}
