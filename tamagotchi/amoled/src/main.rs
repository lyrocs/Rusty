use esp_idf_svc::hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
    spi::{SpiDriver, SpiDriverConfig},
};
use esp_idf_svc::sys::{
    link_patches,
    spi_device_handle_t,
    spi_device_interface_config_t,
    spi_bus_add_device,
    spi_device_transmit,
    spi_transaction_t,
    spi_transaction_ext_t,
    spi_bus_config_t,
    spi_bus_initialize,
    spi_host_device_t_SPI2_HOST,
    SPICOMMON_BUSFLAG_MASTER,
};

// DMA channel constants (from esp_idf)
const SPI_DMA_CH_AUTO: u32 = 3;

// SPI line mode constants
const SPI_LINE_MODE_QUAD: u8 = 2;  // 4-line mode
const SPI_LINE_MODE_SINGLE: u8 = 0; // 1-line mode
use log::info;
use sh8601_rs::{ColorMode, DisplaySize, Sh8601Driver};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
};
use core::ptr;

// Display configuration matching the working no_std version
const DISPLAY_SIZE: DisplaySize = DisplaySize::new(368, 448);
const FB_SIZE: usize = sh8601_rs::framebuffer_size(DISPLAY_SIZE, ColorMode::Rgb888);

// Pin definitions for Waveshare ESP32-S3-Touch-AMOLED-1.8
// Note: Using SPI mode with MOSI/MISO instead of full QSPI
// The SH8601 controller supports both standard SPI and QSPI

// TCA9554 GPIO expander address (used for display reset)
const TCA9554_ADDR: u8 = 0x20;
const DISPLAY_RESET_PIN: u8 = 3; // EXIO3 on TCA9554

// TCA9554 Register addresses
const REG_INPUT_PORT: u8 = 0x00;
const REG_OUTPUT_PORT: u8 = 0x01;
const REG_CONFIGURATION: u8 = 0x03;

const QSPI_PIXEL_OPCODE: u8 = 0x32;
const QSPI_CONTROL_OPCODE: u8 = 0x02;
const CMD_RAMWR: u32 = 0x2C;
const CMD_RAMWRC: u32 = 0x3C;
const DMA_CHUNK_SIZE: usize = 16380;

/// QSPI Controller interface using ESP-IDF SPI half-duplex transactions
struct QspiController {
    device_handle: spi_device_handle_t,
}

impl QspiController {
    unsafe fn new(device_handle: spi_device_handle_t) -> Self {
        Self { device_handle }
    }

    unsafe fn half_duplex_write(
        &mut self,
        cmd: u8,
        addr: u32,
        data: &[u8],
        _use_quad: bool,  // Ignored - using standard SPI only
    ) -> Result<(), esp_idf_svc::sys::EspError> {
        use esp_idf_svc::sys::*;

        // Use standard transaction with command/address/data phases
        // All phases use single-line mode
        let mut trans: spi_transaction_t = core::mem::zeroed();

        trans.cmd = cmd as u16;
        trans.addr = addr as u64;
        trans.length = data.len() * 8;

        if !data.is_empty() {
            trans.__bindgen_anon_1.tx_buffer = data.as_ptr() as *const _;
        }

        // No special flags - standard single-line SPI for all phases
        trans.flags = 0;

        let result = spi_device_transmit(self.device_handle, &mut trans as *mut _);
        if result != 0 {
            return Err(esp_idf_svc::sys::EspError::from(result).unwrap());
        }

        Ok(())
    }
}

impl sh8601_rs::ControllerInterface for QspiController {
    type Error = esp_idf_svc::sys::EspError;

    fn send_command(&mut self, cmd: u8) -> Result<(), Self::Error> {
        let address_value = (cmd as u32) << 8;
        unsafe {
            self.half_duplex_write(QSPI_CONTROL_OPCODE, address_value, &[], false)?;
        }
        Ok(())
    }

    fn send_command_with_data(&mut self, cmd: u8, data: &[u8]) -> Result<(), Self::Error> {
        let address_value = (cmd as u32) << 8;
        unsafe {
            self.half_duplex_write(QSPI_CONTROL_OPCODE, address_value, data, false)?;
        }
        Ok(())
    }

    fn send_pixels(&mut self, pixels: &[u8]) -> Result<(), Self::Error> {
        let ramwr_addr_val = CMD_RAMWR << 8;
        let ramwrc_addr_val = CMD_RAMWRC << 8;

        let mut chunks = pixels.chunks(DMA_CHUNK_SIZE).enumerate();

        unsafe {
            while let Some((index, chunk)) = chunks.next() {
                if index == 0 {
                    self.half_duplex_write(QSPI_PIXEL_OPCODE, ramwr_addr_val, chunk, true)?;
                } else {
                    self.half_duplex_write(QSPI_PIXEL_OPCODE, ramwrc_addr_val, chunk, true)?;
                }
            }
        }
        Ok(())
    }
}

/// Reset interface using I2C GPIO expander (TCA9554)
struct I2cResetDriver<'a> {
    i2c: I2cDriver<'a>,
}

impl<'a> I2cResetDriver<'a> {
    fn new(i2c: I2cDriver<'a>) -> Result<Self, esp_idf_svc::sys::EspError> {
        let mut driver = Self { i2c };

        // Configure the reset pin as output on TCA9554
        // (as_output = true clears the bit, since 0 = output, 1 = input)
        driver.configure_pin(DISPLAY_RESET_PIN, true)?;

        Ok(driver)
    }

    fn configure_pin(&mut self, pin: u8, as_output: bool) -> Result<(), esp_idf_svc::sys::EspError> {
        // Read current configuration register
        let mut current = [0u8; 1];
        self.i2c.write_read(TCA9554_ADDR, &[REG_CONFIGURATION], &mut current, 1000)?;

        // Modify the specific bit (0 = output, 1 = input in TCA9554)
        let new_config = if as_output {
            current[0] & !(1 << pin) // Clear bit for output
        } else {
            current[0] | (1 << pin) // Set bit for input
        };

        // Write back configuration
        self.i2c.write(TCA9554_ADDR, &[REG_CONFIGURATION, new_config], 1000)?;

        Ok(())
    }

    fn write_pin(&mut self, pin: u8, state: bool) -> Result<(), esp_idf_svc::sys::EspError> {
        // Read current output port value
        let mut current = [0u8; 1];
        self.i2c.write_read(TCA9554_ADDR, &[REG_OUTPUT_PORT], &mut current, 1000)?;

        // Modify the specific bit
        let new_value = if state {
            current[0] | (1 << pin)
        } else {
            current[0] & !(1 << pin)
        };

        // Write back to output port
        self.i2c.write(TCA9554_ADDR, &[REG_OUTPUT_PORT, new_value], 1000)?;

        Ok(())
    }
}

impl sh8601_rs::ResetInterface for I2cResetDriver<'_> {
    type Error = esp_idf_svc::sys::EspError;

    fn reset(&mut self) -> Result<(), Self::Error> {
        // Perform reset sequence: LOW -> wait -> HIGH
        self.write_pin(DISPLAY_RESET_PIN, false)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.write_pin(DISPLAY_RESET_PIN, true)?;
        std::thread::sleep(std::time::Duration::from_millis(120));
        Ok(())
    }
}

fn main() {
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting display POC...");

    let peripherals = Peripherals::take().unwrap();

    // Initialize I2C for reset control
    info!("Initializing I2C...");
    let i2c_config = I2cConfig::new()
        .baudrate(400.kHz().into());

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio15,
        peripherals.pins.gpio14,
        &i2c_config,
    ).expect("Failed to initialize I2C");

    // Initialize reset driver
    info!("Initializing reset driver...");
    let reset = I2cResetDriver::new(i2c).expect("Failed to initialize reset driver");

    // Initialize SPI for QSPI communication using low-level ESP-IDF API
    info!("Initializing SPI/QSPI...");

    use esp_idf_svc::sys::*;

    // Configure SPI bus with standard SPI (MOSI only, no quad lines)
    let mut bus_config: spi_bus_config_t = unsafe { core::mem::zeroed() };
    unsafe {
        // Standard SPI: only MOSI, MISO, and CLK
        *(&mut bus_config.__bindgen_anon_1 as *mut _ as *mut i32) = 4; // mosi_io_num (SIO0)
        *(&mut bus_config.__bindgen_anon_2 as *mut _ as *mut i32) = 5; // miso_io_num (SIO1)
        *(&mut bus_config.__bindgen_anon_3 as *mut _ as *mut i32) = -1; // quadwp_io_num (not used)
        *(&mut bus_config.__bindgen_anon_4 as *mut _ as *mut i32) = -1; // quadhd_io_num (not used)
    }
    bus_config.sclk_io_num = 11;
    bus_config.data4_io_num = -1;
    bus_config.data5_io_num = -1;
    bus_config.data6_io_num = -1;
    bus_config.data7_io_num = -1;
    bus_config.max_transfer_sz = DMA_CHUNK_SIZE as i32;
    bus_config.flags = SPICOMMON_BUSFLAG_MASTER;
    bus_config.isr_cpu_id = 0;
    bus_config.intr_flags = 0;

    // Initialize SPI bus
    unsafe {
        let result = spi_bus_initialize(spi_host_device_t_SPI2_HOST, &bus_config, SPI_DMA_CH_AUTO);
        if result != 0 {
            panic!("Failed to initialize SPI bus: {}", result);
        }
    }

    // Configure SPI device with command and address support
    // Use SPI_DEVICE_HALFDUPLEX flag to enable half-duplex mode
    let dev_config = spi_device_interface_config_t {
        command_bits: 8,
        address_bits: 24,
        dummy_bits: 0,
        mode: 0,
        clock_source: 0,
        duty_cycle_pos: 0,
        cs_ena_pretrans: 0,
        cs_ena_posttrans: 0,
        clock_speed_hz: 40_000_000,
        input_delay_ns: 0,
        spics_io_num: 12,
        flags: (1 << 0), // SPI_DEVICE_HALFDUPLEX
        queue_size: 1,
        pre_cb: None,
        post_cb: None,
        sample_point: 0, // SPI_DEVICE_SAMPLE_DEFAULT
    };

    let mut device_handle: spi_device_handle_t = ptr::null_mut();

    unsafe {
        let result = spi_bus_add_device(spi_host_device_t_SPI2_HOST, &dev_config, &mut device_handle);
        if result != 0 {
            panic!("Failed to add SPI device: {}", result);
        }
    }

    let controller = unsafe { QspiController::new(device_handle) };

    // Initialize display
    info!("Initializing SH8601 display...");
    let mut display = Sh8601Driver::new_heap::<_, FB_SIZE>(
        controller,
        reset,
        ColorMode::Rgb888,
        DISPLAY_SIZE,
        FreeRtos,
    ).expect("Failed to initialize display");

    info!("Display initialized successfully!");

    // Draw test pattern
    info!("Drawing test pattern...");

    // Clear screen to black
    let clear_style = PrimitiveStyle::with_fill(Rgb888::BLACK);
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(clear_style)
        .draw(&mut display)
        .ok();

    // Draw red rectangle
    let red_style = PrimitiveStyle::with_fill(Rgb888::RED);
    Rectangle::new(Point::new(50, 50), Size::new(100, 100))
        .into_styled(red_style)
        .draw(&mut display)
        .ok();

    // Draw green circle
    let green_style = PrimitiveStyle::with_fill(Rgb888::GREEN);
    Circle::new(Point::new(200, 100), 50)
        .into_styled(green_style)
        .draw(&mut display)
        .ok();

    // Draw blue rectangle
    let blue_style = PrimitiveStyle::with_fill(Rgb888::BLUE);
    Rectangle::new(Point::new(50, 200), Size::new(268, 100))
        .into_styled(blue_style)
        .draw(&mut display)
        .ok();

    // Draw white text
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
    Text::new("ESP32-S3 AMOLED", Point::new(10, 350), text_style)
        .draw(&mut display)
        .ok();

    Text::new("Display Working!", Point::new(10, 370), text_style)
        .draw(&mut display)
        .ok();

    // Flush to display
    info!("Flushing to display...");
    display.flush().expect("Failed to flush display");

    info!("Test pattern displayed successfully!");

    loop {
        FreeRtos::delay_ms(1000);
    }
}
