//! stdgotchi - ESP32-S3 AMOLED Touch Demo
//!
//! Interactive drawing demo for Waveshare ESP32-S3-Touch-AMOLED-1.8
//!
//! # Features
//! - Touch drawing with multi-touch support
//! - Gesture recognition (swipes and double-tap)
//! - QSPI AMOLED display with RGB888 color
//! - Animated GIF playback
//!
//! # Hardware
//! - Board: ESP32-S3 with PSRAM
//! - Display: 1.8" AMOLED (368x448) via QSPI
//! - Touch: FT3168 capacitive touch via I2C
//!
//! # Controls
//!
//! ## Gestures
//! - Swipe Up: Clear screen
//! - Swipe Down: Play GIF animation
//! - Swipe Left: Fill green
//! - Swipe Right: Fill blue
//! - Double Click: Reset to welcome screen
//! - Touch: Draw cyan circles
//!
//! ## Buttons
//! - BOOT (GPIO0): Shows purple screen when pressed
//! - PWR (EXIO4): Shows yellow screen when pressed

mod display;
mod buttons;

use buttons::{ButtonEvent, Buttons};
use display::{ColorMode, Ft3x68Driver, GifPlayer, Sh8601Driver, FT3168_DEVICE_ADDRESS, LCD_H_RES, LCD_V_RES};
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

/// TCA9554 GPIO expander I2C address
const TCA9554_ADDRESS: u8 = 0x20;
/// TCA9554 output register
const REG_OUTPUT: u8 = 0x01;
/// TCA9554 configuration register
const REG_CONFIG: u8 = 0x03;

/// Embedded GIF animation data
const GIF_DATA: &[u8] = include_bytes!("../assets/80.gif");

/// Initialize the display hardware
fn init_display(i2c: &mut I2cDriver) -> Result<Sh8601Driver, Box<dyn std::error::Error>> {
    log::info!("Performing display reset...");

    // Configure GPIO expander pins as output
    i2c.write(TCA9554_ADDRESS, &[REG_CONFIG, 0x00], 1000)?;

    // Reset low
    i2c.write(TCA9554_ADDRESS, &[REG_OUTPUT, 0b0000_0010], 1000)?;
    thread::sleep(Duration::from_millis(20));

    // Reset high (EXIO0, EXIO1, EXIO2 all high)
    i2c.write(TCA9554_ADDRESS, &[REG_OUTPUT, 0b0000_0111], 1000)?;
    thread::sleep(Duration::from_millis(150));

    // Initialize QSPI bus for display
    log::info!("Initializing QSPI bus...");

    let mut bus_config = spi_bus_config_t::default();
    bus_config.__bindgen_anon_1.data0_io_num = 4; // SIO0
    bus_config.__bindgen_anon_2.data1_io_num = 5; // SIO1
    bus_config.__bindgen_anon_3.data2_io_num = 6; // SIO2
    bus_config.__bindgen_anon_4.data3_io_num = 7; // SIO3
    bus_config.sclk_io_num = 11; // SCLK
    bus_config.max_transfer_sz = 32768;
    bus_config.flags = SPICOMMON_BUSFLAG_MASTER | SPICOMMON_BUSFLAG_QUAD;

    unsafe {
        let ret = spi_bus_initialize(
            spi_host_device_t_SPI2_HOST,
            &bus_config,
            spi_common_dma_t_SPI_DMA_CH_AUTO,
        );
        if ret != ESP_OK {
            return Err(format!("Failed to initialize SPI bus: {}", ret).into());
        }
    }

    // Initialize display driver
    log::info!("Initializing SH8601 display driver...");
    let mut display = Sh8601Driver::new(
        spi_host_device_t_SPI2_HOST,
        12,
        LCD_H_RES,
        LCD_V_RES,
        ColorMode::Rgb888,
    )?;
    display.initialize(ColorMode::Rgb888)?;

    log::info!("Display initialized successfully!");
    Ok(display)
}

/// Draw the initial welcome screen
fn draw_welcome_screen(display: &mut Sh8601Driver) -> Result<(), Box<dyn std::error::Error>> {
    display.clear(Rgb888::BLACK)?;

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::GREEN);
    Text::new("stdgotchi", Point::new(10, 30), text_style).draw(display)?;
    Text::new("ESP32-S3 AMOLED", Point::new(10, 50), text_style).draw(display)?;
    Text::new("Touch & Gestures!", Point::new(10, 70), text_style).draw(display)?;
    Text::new("Swipe down for GIF", Point::new(10, 90), text_style).draw(display)?;

    Circle::new(Point::new(50, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
        .draw(display)?;

    Circle::new(Point::new(100, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
        .draw(display)?;

    Circle::new(Point::new(150, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::MAGENTA))
        .draw(display)?;

    display.flush()?;
    Ok(())
}

/// Play the GIF animation at a fixed position
fn play_gif_animation(display: &mut Sh8601Driver) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Loading GIF animation...");

    let mut player = GifPlayer::new(GIF_DATA)?;
    let (width, height) = player.dimensions();
    log::info!("GIF dimensions: {}x{}, frames: {}", width, height, player.frame_count());

    // Calculate centered position for the GIF
    let display_size = display.size();
    let pos_x = (display_size.width as i32 - width as i32) / 2;
    let pos_y = (display_size.height as i32 - height as i32) / 2;

    log::info!("Rendering GIF at position: ({}, {})", pos_x, pos_y);

    // Clear screen before animation
    display.clear(Rgb888::BLACK)?;

    // Play animation loop (3 complete loops) at fixed position
    let total_frames = player.frame_count() * 3;
    for _ in 0..total_frames {
        let delay = player.next_frame(display, Some((pos_x, pos_y)))?;
        display.flush()?;
        thread::sleep(delay);
    }

    log::info!("GIF animation completed");
    Ok(())
}

/// Handle touch events and gestures
fn handle_touch_events(
    touch: &Ft3x68Driver,
    i2c: &mut I2cDriver,
    display: &mut Sh8601Driver,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read touch coordinates
    if let Ok(touches) = touch.get_touches(i2c) {
        for point in touches.iter() {
            // Draw a circle at touch point
            if point.x < LCD_H_RES && point.y < LCD_V_RES {
                Circle::new(Point::new(point.x as i32 - 10, point.y as i32 - 10), 20)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::CYAN))
                    .draw(display)?;
            }
        }
        display.flush()?;
    }

    // Check for gestures
    if let Ok(gesture) = touch.read_gesture(i2c) {
        match gesture {
            display::Gesture::SwipeUp => {
                log::info!("Gesture: Swipe Up - Clearing screen");
                display.clear(Rgb888::BLACK)?;

                let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::GREEN);
                Text::new("stdgotchi", Point::new(10, 30), text_style).draw(display)?;
                Text::new("Swipe to clear", Point::new(10, 50), text_style).draw(display)?;
                Text::new("Touch to draw", Point::new(10, 70), text_style).draw(display)?;

                display.flush()?;
            }
            display::Gesture::SwipeDown => {
                log::info!("Gesture: Swipe Down - Playing GIF animation");
                play_gif_animation(display)?;
                // Return to welcome screen after animation
                draw_welcome_screen(display)?;
            }
            display::Gesture::SwipeLeft => {
                log::info!("Gesture: Swipe Left - Fill Green");
                display.clear(Rgb888::new(0, 100, 0))?;
                display.flush()?;
            }
            display::Gesture::SwipeRight => {
                log::info!("Gesture: Swipe Right - Fill Blue");
                display.clear(Rgb888::new(0, 0, 100))?;
                display.flush()?;
            }
            display::Gesture::DoubleClick => {
                log::info!("Gesture: Double Click - Reset");
                draw_welcome_screen(display)?;
            }
            display::Gesture::None => {}
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize system services
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("=== stdgotchi starting ===");
    log::info!("ESP32-S3 with 1.8\" AMOLED Display (SH8601)");

    let peripherals = Peripherals::take()?;

    // Initialize I2C for GPIO expander and touch controller
    log::info!("Initializing I2C...");
    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio15, // SDA
        peripherals.pins.gpio14, // SCL
        &i2c_config,
    )?;

    // Initialize display
    let mut display = init_display(&mut i2c)?;

    // Initialize touch controller
    log::info!("Initializing FT3168 touch controller...");
    let mut touch = Ft3x68Driver::new(FT3168_DEVICE_ADDRESS);
    touch.initialize(&mut i2c)?;
    touch.set_gesture_mode(&mut i2c, true)?;
    log::info!("Touch controller initialized successfully!");

    // Initialize buttons
    log::info!("Initializing buttons...");
    let mut buttons = Buttons::new(&mut i2c, peripherals.pins.gpio0)?;
    log::info!("Buttons initialized successfully!");

    // Draw welcome screen
    draw_welcome_screen(&mut display)?;

    log::info!("stdgotchi ready! Touch the screen to draw...");

    // Main event loop
    let mut last_touch_count = 0u8;

    loop {
        // Poll buttons
        if let Ok(Some(event)) = buttons.poll(&mut i2c) {
            match event {
                ButtonEvent::BootPress => {
                    log::info!("BOOT button pressed!");
                    display.clear(Rgb888::new(50, 0, 50))?;
                    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
                    Text::new("BOOT pressed", Point::new(10, 30), text_style).draw(&mut display)?;
                    display.flush()?;
                }
                ButtonEvent::BootRelease => {
                    log::info!("BOOT button released!");
                    draw_welcome_screen(&mut display)?;
                }
                ButtonEvent::PowerPress => {
                    log::info!("PWR button pressed!");
                    display.clear(Rgb888::new(50, 50, 0))?;
                    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
                    Text::new("PWR pressed", Point::new(10, 30), text_style).draw(&mut display)?;
                    display.flush()?;
                }
                ButtonEvent::PowerRelease => {
                    log::info!("PWR button released!");
                    draw_welcome_screen(&mut display)?;
                }
            }
        }

        // Poll touch
        match touch.finger_number(&mut i2c) {
            Ok(count) => {
                if count > 0 {
                    handle_touch_events(&touch, &mut i2c, &mut display)?;
                } else if count == 0 && last_touch_count > 0 {
                    log::info!("Touch released");
                }
                last_touch_count = count;
            }
            Err(e) => {
                log::warn!("Failed to read touch: {:?}", e);
            }
        }

        thread::sleep(Duration::from_millis(10)); // Poll at 100Hz for responsive buttons
    }
}
