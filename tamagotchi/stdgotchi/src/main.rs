mod display;

use display::{ColorMode, Sh8601Driver, LCD_H_RES, LCD_V_RES, Ft3x68Driver, FT3168_DEVICE_ADDRESS};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};
use esp_idf_svc::hal::{
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    units::Hertz,
};
use esp_idf_svc::sys::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize system services
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("=== stdgotchi starting ===");
    log::info!("ESP32-S3 with 1.8\" AMOLED Display (SH8601)");

    let peripherals = Peripherals::take()?;

    // Initialize I2C for GPIO expander (reset control)
    log::info!("Initializing I2C...");
    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));

    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio15, // SDA
        peripherals.pins.gpio14, // SCL
        &i2c_config,
    )?;

    // Perform hardware reset via I2C GPIO expander (TCA9554)
    log::info!("Performing display reset...");
    const TCA9554_ADDRESS: u8 = 0x20;
    const REG_OUTPUT: u8 = 0x01;
    const REG_CONFIG: u8 = 0x03;

    // Configure as output
    i2c.write(TCA9554_ADDRESS, &[REG_CONFIG, 0x00], 1000)?;

    // Reset low
    i2c.write(TCA9554_ADDRESS, &[REG_OUTPUT, 0b0000_0010], 1000)?;
    thread::sleep(Duration::from_millis(20));

    // Reset high
    i2c.write(TCA9554_ADDRESS, &[REG_OUTPUT, 0b0000_0111], 1000)?;
    thread::sleep(Duration::from_millis(150));

    // Initialize QSPI bus for display
    log::info!("Initializing QSPI bus...");

    let mut bus_config = spi_bus_config_t::default();
    bus_config.__bindgen_anon_1.data0_io_num = 4;  // SIO0
    bus_config.__bindgen_anon_2.data1_io_num = 5;  // SIO1
    bus_config.__bindgen_anon_3.data2_io_num = 6;  // SIO2
    bus_config.__bindgen_anon_4.data3_io_num = 7;  // SIO3
    bus_config.sclk_io_num = 11; // SCLK
    bus_config.max_transfer_sz = 32768;
    bus_config.flags = SPICOMMON_BUSFLAG_MASTER | SPICOMMON_BUSFLAG_QUAD;

    unsafe {
        let ret = spi_bus_initialize(spi_host_device_t_SPI2_HOST, &bus_config, spi_common_dma_t_SPI_DMA_CH_AUTO);
        if ret != ESP_OK {
            return Err(format!("Failed to initialize SPI bus: {}", ret).into());
        }
    }

    // Initialize display driver
    log::info!("Initializing SH8601 display driver...");
    log::info!("Framebuffer: {}x{}x3 = {} bytes (using PSRAM)",
        LCD_H_RES, LCD_V_RES, LCD_H_RES as usize * LCD_V_RES as usize * 3);

    let mut display = Sh8601Driver::new(spi_host_device_t_SPI2_HOST, 12, LCD_H_RES, LCD_V_RES, ColorMode::Rgb888)?;
    display.initialize(ColorMode::Rgb888)?;

    log::info!("Display initialized successfully!");

    // Initialize touch controller
    log::info!("Initializing FT3168 touch controller...");
    let mut touch = Ft3x68Driver::new(FT3168_DEVICE_ADDRESS);
    touch.initialize(&mut i2c)?;
    touch.set_gesture_mode(&mut i2c, true)?;
    log::info!("Touch controller initialized successfully!");

    // Draw test content
    log::info!("Drawing test content...");

    // Clear screen with black background
    display.clear(Rgb888::BLACK)?;

    // Create text style
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::GREEN);

    // Draw welcome text
    Text::new("stdgotchi", Point::new(10, 30), text_style).draw(&mut display)?;
    Text::new("ESP32-S3 AMOLED", Point::new(10, 50), text_style).draw(&mut display)?;
    Text::new("SH8601 Driver", Point::new(10, 70), text_style).draw(&mut display)?;
    Text::new("368x448 RGB888", Point::new(10, 90), text_style).draw(&mut display)?;

    // Draw some colored circles
    Circle::new(Point::new(50, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
        .draw(&mut display)?;

    Circle::new(Point::new(100, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
        .draw(&mut display)?;

    Circle::new(Point::new(150, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::MAGENTA))
        .draw(&mut display)?;

    // Flush to display
    log::info!("Flushing framebuffer to display...");
    display.flush()?;

    log::info!("Display updated successfully!");
    log::info!("Testing automatic rendering without touch...");

    // TEST: Automatic color cycling WITHOUT touch to isolate I2C conflict
    let mut color_index = 1u8; // Start at 1 (red) not 0 (black)

    loop {
        thread::sleep(Duration::from_secs(2));

        // Cycle through BRIGHT colors (starting with RED, no black)
        let (r, g, b) = match color_index % 4 {
            0 => (255, 0, 0),     // Bright Red
            1 => (0, 255, 0),     // Bright Green
            2 => (0, 0, 255),     // Bright Blue
            _ => (255, 255, 0),   // Bright Yellow
        };
        color_index = color_index.wrapping_add(1);

        log::info!("=== AUTO RENDER {} ===", color_index);
        log::info!("Filling with RGB({}, {}, {})", r, g, b);
        display.fill_test(r, g, b);

        log::info!("Flushing...");
        let flush_start = std::time::Instant::now();
        display.flush()?;
        let flush_time = flush_start.elapsed();
        log::info!("Flush completed in {:?}", flush_time);
        log::info!("=====================");
    }
}
