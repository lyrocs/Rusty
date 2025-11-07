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
mod ecs;
mod systems;
mod ui;

use bevy_ecs::prelude::*;
use display::{ColorMode, Ft3x68Driver, Sh8601Driver, FT3168_DEVICE_ADDRESS, LCD_H_RES, LCD_V_RES};
use ecs::resources::{AppState, ButtonResource, DisplayResource, GpioResource, TouchResource};
use esp_idf_svc::hal::gpio::{Gpio0, PinDriver};
use esp_idf_svc::hal::{
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    units::Hertz,
};
use esp_idf_svc::sys::*;
use std::thread;
use std::time::Duration;
use systems::{animation_cleanup_system, animation_init_system, button_system, fps_system, render_system, touch_system};

/// TCA9554 GPIO expander I2C address
const TCA9554_ADDRESS: u8 = 0x20;
/// TCA9554 output register
const REG_OUTPUT: u8 = 0x01;
/// TCA9554 configuration register
const REG_CONFIG: u8 = 0x03;

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

/// Configure GPIO expander to set EXIO4 as input for PWR button
fn configure_pwr_button(i2c: &mut I2cDriver) -> Result<(), Box<dyn std::error::Error>> {
    // Read current configuration
    let mut config = [0u8; 1];
    i2c.write_read(TCA9554_ADDRESS, &[REG_CONFIG], &mut config, 1000)?;

    // Set bit 4 to 1 (input mode for EXIO4)
    let new_config = config[0] | 0b0001_0000;
    i2c.write(TCA9554_ADDRESS, &[REG_CONFIG, new_config], 1000)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize system services
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("=== stdgotchi starting with Bevy ECS ===");
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
    let display = init_display(&mut i2c)?;

    // Initialize touch controller
    log::info!("Initializing FT3168 touch controller...");
    let mut touch = Ft3x68Driver::new(FT3168_DEVICE_ADDRESS);
    touch.initialize(&mut i2c)?;
    touch.set_gesture_mode(&mut i2c, true)?;
    log::info!("Touch controller initialized successfully!");

    // Initialize boot button GPIO
    log::info!("Initializing buttons...");
    let boot_pin = PinDriver::input(peripherals.pins.gpio0)?;

    // Configure PWR button (EXIO4) as input on GPIO expander
    configure_pwr_button(&mut i2c)?;
    log::info!("Buttons initialized successfully!");

    // Create ECS World
    let mut world = World::new();

    // Insert resources
    world.insert_resource(AppState::default());

    // Insert non-send resources (hardware peripherals)
    world.insert_non_send_resource(DisplayResource { display });
    world.insert_non_send_resource(TouchResource {
        touch,
        last_touch_active: false,
    });
    world.insert_non_send_resource(GpioResource { boot_pin });
    world.insert_non_send_resource(ButtonResource {
        boot_last_state: false,
        pwr_last_state: false,
        boot_debounce: 0,
        pwr_debounce: 0,
    });
    world.insert_non_send_resource(i2c);

    // Create schedule and add systems
    // Order: FPS tracking → Input → Animation init → Render → Animation cleanup
    let mut schedule = Schedule::default();
    schedule.add_systems((
        fps_system,
        button_system::<Gpio0>,
        touch_system,
        animation_init_system,
        render_system,
        animation_cleanup_system,
    ));

    log::info!("stdgotchi ready! ECS initialized. Touch the screen to draw...");

    // Main ECS game loop
    loop {
        // Run all systems
        schedule.run(&mut world);

        // Control frame rate (~100 FPS for responsive input)
        thread::sleep(Duration::from_millis(10));
    }
}
