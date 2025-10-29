// Low-level QSPI display driver using ESP-IDF sys bindings
//
// This uses raw ESP-IDF C API to configure QSPI mode while staying in std environment.
// esp-idf-svc's Rust wrapper doesn't expose QSPI, so we go directly to the C layer.

use anyhow::Result;
use esp_idf_svc::sys as esp_idf_sys;
use std::ptr;
use std::thread;
use std::time::Duration;

use crate::drivers::gpio_expander::Tca9554Driver;

// SH8601 Commands
const CMD_SWRESET: u8 = 0x01;
const CMD_SLPOUT: u8 = 0x11;
const CMD_NORON: u8 = 0x13;
const CMD_INVON: u8 = 0x21;
const CMD_DISPON: u8 = 0x29;
const CMD_CASET: u8 = 0x2A;
const CMD_RASET: u8 = 0x2B;
const CMD_RAMWR: u8 = 0x2C;
const CMD_MADCTL: u8 = 0x36;
const CMD_COLMOD: u8 = 0x3A;

/// Display driver using raw ESP-IDF SPI with QSPI mode
pub struct RawQspiDriver<'d> {
    spi_device: esp_idf_sys::spi_device_handle_t,
    gpio_expander: Tca9554Driver<'d>,
    width: u16,
    height: u16,
}

impl<'d> RawQspiDriver<'d> {
    /// Create new QSPI driver using raw ESP-IDF API
    pub fn new(gpio_expander: Tca9554Driver<'d>) -> Result<Self> {
        log::info!("Initializing QSPI display using raw ESP-IDF API");

        unsafe {
            // Configure SPI bus with QSPI mode
            // ESP-IDF uses unions for pin assignments, need to initialize them properly
            let bus_config = esp_idf_sys::spi_bus_config_t {
                __bindgen_anon_1: esp_idf_sys::spi_bus_config_t__bindgen_ty_1 { data0_io_num: 4 },  // SIO0 (MOSI)
                __bindgen_anon_2: esp_idf_sys::spi_bus_config_t__bindgen_ty_2 { data1_io_num: 5 },  // SIO1 (MISO)
                sclk_io_num: 11, // SCK
                __bindgen_anon_3: esp_idf_sys::spi_bus_config_t__bindgen_ty_3 { data2_io_num: 6 },  // SIO2 (WP for QSPI)
                __bindgen_anon_4: esp_idf_sys::spi_bus_config_t__bindgen_ty_4 { data3_io_num: 7 },  // SIO3 (HD for QSPI)
                data4_io_num: -1,
                data5_io_num: -1,
                data6_io_num: -1,
                data7_io_num: -1,
                max_transfer_sz: 32768,
                flags: esp_idf_sys::SPICOMMON_BUSFLAG_MASTER
                    | esp_idf_sys::SPICOMMON_BUSFLAG_QUAD,  // Enable QUAD mode!
                isr_cpu_id: esp_idf_sys::esp_intr_cpu_affinity_t_ESP_INTR_CPU_AFFINITY_AUTO,
                intr_flags: 0,
            };

            // Initialize SPI bus
            let ret = esp_idf_sys::spi_bus_initialize(
                esp_idf_sys::spi_host_device_t_SPI2_HOST,
                &bus_config,
                esp_idf_sys::spi_common_dma_t_SPI_DMA_CH_AUTO,
            );

            if ret != esp_idf_sys::ESP_OK {
                anyhow::bail!("Failed to initialize SPI bus: {}", ret);
            }

            log::info!("SPI bus initialized with QSPI mode");

            // Configure SPI device
            let dev_config = esp_idf_sys::spi_device_interface_config_t {
                command_bits: 0,
                address_bits: 0,
                dummy_bits: 0,
                mode: 0, // SPI Mode 0
                duty_cycle_pos: 0,
                cs_ena_pretrans: 0,
                cs_ena_posttrans: 0,
                clock_speed_hz: 40_000_000, // 40 MHz
                input_delay_ns: 0,
                spics_io_num: 12, // CS pin
                flags: esp_idf_sys::SPI_DEVICE_HALFDUPLEX,
                queue_size: 1,
                pre_cb: None,
                post_cb: None,
                ..Default::default()
            };

            let mut spi_device: esp_idf_sys::spi_device_handle_t = ptr::null_mut();
            let ret = esp_idf_sys::spi_bus_add_device(
                esp_idf_sys::spi_host_device_t_SPI2_HOST,
                &dev_config,
                &mut spi_device,
            );

            if ret != esp_idf_sys::ESP_OK {
                anyhow::bail!("Failed to add SPI device: {}", ret);
            }

            log::info!("SPI device configured successfully");

            Ok(Self {
                spi_device,
                gpio_expander,
                width: 368,
                height: 448,
            })
        }
    }

    /// Send command using QSPI
    fn send_command(&mut self, cmd: u8) -> Result<()> {
        unsafe {
            let mut trans = esp_idf_sys::spi_transaction_t {
                flags: esp_idf_sys::SPI_TRANS_USE_TXDATA,
                cmd: 0,
                addr: 0,
                length: 8, // 1 byte = 8 bits
                rxlength: 0,
                user: ptr::null_mut(),
                __bindgen_anon_1: esp_idf_sys::spi_transaction_t__bindgen_ty_1 {
                    tx_data: [cmd, 0, 0, 0],
                },
                __bindgen_anon_2: esp_idf_sys::spi_transaction_t__bindgen_ty_2 {
                    rx_data: [0; 4],
                },
            };

            let ret = esp_idf_sys::spi_device_transmit(self.spi_device, &mut trans);
            if ret != esp_idf_sys::ESP_OK {
                anyhow::bail!("Failed to send command 0x{:02X}: {}", cmd, ret);
            }
        }
        Ok(())
    }

    /// Send command with data using QSPI
    fn send_command_with_data(&mut self, cmd: u8, data: &[u8]) -> Result<()> {
        self.send_command(cmd)?;

        if !data.is_empty() {
            unsafe {
                let mut trans = esp_idf_sys::spi_transaction_t {
                    flags: 0,
                    cmd: 0,
                    addr: 0,
                    length: (data.len() * 8) as _,
                    rxlength: 0,
                    user: ptr::null_mut(),
                    __bindgen_anon_1: esp_idf_sys::spi_transaction_t__bindgen_ty_1 {
                        tx_buffer: data.as_ptr() as *const _,
                    },
                    __bindgen_anon_2: esp_idf_sys::spi_transaction_t__bindgen_ty_2 {
                        rx_buffer: ptr::null_mut(),
                    },
                };

                let ret = esp_idf_sys::spi_device_transmit(self.spi_device, &mut trans);
                if ret != esp_idf_sys::ESP_OK {
                    anyhow::bail!("Failed to send data for command 0x{:02X}: {}", cmd, ret);
                }
            }
        }
        Ok(())
    }

    /// Hardware reset
    fn reset(&mut self) -> Result<()> {
        log::info!("Performing hardware reset...");

        self.gpio_expander.configure_pin(0, false)?;
        self.gpio_expander.write_pin(0, true)?;
        thread::sleep(Duration::from_millis(20));

        self.gpio_expander.write_pin(0, false)?;
        thread::sleep(Duration::from_millis(50));

        self.gpio_expander.write_pin(0, true)?;
        thread::sleep(Duration::from_millis(200));

        Ok(())
    }

    /// Initialize display
    pub fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing SH8601 with raw QSPI...");

        self.reset()?;

        // Software reset
        self.send_command(CMD_SWRESET)?;
        thread::sleep(Duration::from_millis(10));

        // Sleep out
        self.send_command(CMD_SLPOUT)?;
        thread::sleep(Duration::from_millis(120));

        // Color mode RGB888
        self.send_command_with_data(CMD_COLMOD, &[0x77])?;

        // Memory access control
        self.send_command_with_data(CMD_MADCTL, &[0x00])?;

        // Brightness
        self.send_command_with_data(0x51, &[0xFF])?;
        self.send_command_with_data(0x53, &[0x2C])?;

        // Display inversion on
        self.send_command(CMD_INVON)?;

        // Normal display mode
        self.send_command(CMD_NORON)?;

        // Display on
        self.send_command(CMD_DISPON)?;
        thread::sleep(Duration::from_millis(120));

        log::info!("Display initialized with raw QSPI!");
        Ok(())
    }

    /// Set drawing window
    pub fn set_window(&mut self, x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        let x_end = x + width - 1;
        let y_end = y + height - 1;

        self.send_command_with_data(CMD_CASET, &[
            (x >> 8) as u8,
            (x & 0xFF) as u8,
            (x_end >> 8) as u8,
            (x_end & 0xFF) as u8,
        ])?;

        self.send_command_with_data(CMD_RASET, &[
            (y >> 8) as u8,
            (y & 0xFF) as u8,
            (y_end >> 8) as u8,
            (y_end & 0xFF) as u8,
        ])?;

        Ok(())
    }

    /// Draw buffer
    pub fn draw_buffer(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<()> {
        self.set_window(x, y, width, height)?;

        self.send_command(CMD_RAMWR)?;

        // Send pixel data in chunks
        const CHUNK_SIZE: usize = 4096;
        for chunk in buffer.chunks(CHUNK_SIZE) {
            unsafe {
                let mut trans = esp_idf_sys::spi_transaction_t {
                    flags: 0,
                    cmd: 0,
                    addr: 0,
                    length: (chunk.len() * 8) as _,
                    rxlength: 0,
                    user: ptr::null_mut(),
                    __bindgen_anon_1: esp_idf_sys::spi_transaction_t__bindgen_ty_1 {
                        tx_buffer: chunk.as_ptr() as *const _,
                    },
                    __bindgen_anon_2: esp_idf_sys::spi_transaction_t__bindgen_ty_2 {
                        rx_buffer: ptr::null_mut(),
                    },
                };

                let ret = esp_idf_sys::spi_device_transmit(self.spi_device, &mut trans);
                if ret != esp_idf_sys::ESP_OK {
                    anyhow::bail!("Failed to send pixel data: {}", ret);
                }
            }
        }

        Ok(())
    }

    /// Clear display
    pub fn clear(&mut self) -> Result<()> {
        self.set_window(0, 0, self.width, self.height)?;

        let buffer_size = (self.width as usize) * (self.height as usize) * 3;
        let black = vec![0u8; buffer_size];

        self.send_command(CMD_RAMWR)?;

        const CHUNK_SIZE: usize = 4096;
        for chunk in black.chunks(CHUNK_SIZE) {
            unsafe {
                let mut trans = esp_idf_sys::spi_transaction_t {
                    flags: 0,
                    cmd: 0,
                    addr: 0,
                    length: (chunk.len() * 8) as _,
                    rxlength: 0,
                    user: ptr::null_mut(),
                    __bindgen_anon_1: esp_idf_sys::spi_transaction_t__bindgen_ty_1 {
                        tx_buffer: chunk.as_ptr() as *const _,
                    },
                    __bindgen_anon_2: esp_idf_sys::spi_transaction_t__bindgen_ty_2 {
                        rx_buffer: ptr::null_mut(),
                    },
                };

                esp_idf_sys::spi_device_transmit(self.spi_device, &mut trans);
            }
        }

        Ok(())
    }
}

// SAFETY: ESP-IDF SPI driver is thread-safe when properly initialized
unsafe impl<'d> Send for RawQspiDriver<'d> {}

impl<'d> Drop for RawQspiDriver<'d> {
    fn drop(&mut self) {
        unsafe {
            esp_idf_sys::spi_bus_remove_device(self.spi_device);
            esp_idf_sys::spi_bus_free(esp_idf_sys::spi_host_device_t_SPI2_HOST);
        }
    }
}
