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
    log::info!("stdgotchi ready! Touch the screen to draw...");

    // Interactive touch drawing loop
    let mut last_touch_count = 0u8;

    loop {
        // Check for touches
        match touch.finger_number(&mut i2c) {
            Ok(count) => {
                if count > 0 {
                    // Read touch coordinates
                    if let Ok(touches) = touch.get_touches(&mut i2c) {
                        for (i, point) in touches.iter().enumerate() {
                            if i == 0 {
                                log::info!("Touch at: x={}, y={}", point.x, point.y);
                            }

                            // Draw a circle at touch point
                            if point.x < LCD_H_RES && point.y < LCD_V_RES {
                                Circle::new(
                                    Point::new(point.x as i32 - 10, point.y as i32 - 10),
                                    20
                                )
                                .into_styled(PrimitiveStyle::with_fill(Rgb888::CYAN))
                                .draw(&mut display)?;
                            }
                        }

                        // Flush after drawing
                        display.flush()?;
                    }

                    // Check for gestures
                    if let Ok(gesture) = touch.read_gesture(&mut i2c) {
                        match gesture {
                            display::Gesture::SwipeUp => {
                                log::info!("Gesture: Swipe Up - Clearing screen");
                                display.clear(Rgb888::BLACK)?;

                                // Redraw title
                                let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::GREEN);
                                Text::new("stdgotchi", Point::new(10, 30), text_style).draw(&mut display)?;
                                Text::new("Swipe to clear", Point::new(10, 50), text_style).draw(&mut display)?;
                                Text::new("Touch to draw", Point::new(10, 70), text_style).draw(&mut display)?;

                                display.flush()?;
                            },
                            display::Gesture::SwipeDown => {
                                log::info!("Gesture: Swipe Down - Fill Red");
                                display.clear(Rgb888::new(100, 0, 0))?;
                                display.flush()?;
                            },
                            display::Gesture::SwipeLeft => {
                                log::info!("Gesture: Swipe Left - Fill Green");
                                display.clear(Rgb888::new(0, 100, 0))?;
                                display.flush()?;
                            },
                            display::Gesture::SwipeRight => {
                                log::info!("Gesture: Swipe Right - Fill Blue");
                                display.clear(Rgb888::new(0, 0, 100))?;
                                display.flush()?;
                            },
                            display::Gesture::DoubleClick => {
                                log::info!("Gesture: Double Click - Reset");
                                // Redraw initial screen
                                display.clear(Rgb888::BLACK)?;
                                let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::GREEN);
                                Text::new("stdgotchi", Point::new(10, 30), text_style).draw(&mut display)?;
                                Text::new("ESP32-S3 AMOLED", Point::new(10, 50), text_style).draw(&mut display)?;
                                Text::new("Touch & Gestures!", Point::new(10, 70), text_style).draw(&mut display)?;

                                Circle::new(Point::new(50, 150), 30)
                                    .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
                                    .draw(&mut display)?;
                                Circle::new(Point::new(100, 150), 30)
                                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                                    .draw(&mut display)?;
                                Circle::new(Point::new(150, 150), 30)
                                    .into_styled(PrimitiveStyle::with_fill(Rgb888::MAGENTA))
                                    .draw(&mut display)?;

                                display.flush()?;
                            },
                            display::Gesture::None => {},
                        }
                    }
                } else if count == 0 && last_touch_count > 0 {
                    log::info!("Touch released");
                }

                last_touch_count = count;
            }
            Err(e) => {
                log::warn!("Failed to read touch: {:?}", e);
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}
