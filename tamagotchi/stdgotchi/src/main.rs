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

mod assets;
mod display;
mod drivers;
mod ecs;
mod game;
mod sdcard;
mod systems;
mod ui;

use bevy_ecs::prelude::*;
use display::{ColorMode, Ft3x68Driver, Sh8601Driver, FT3168_DEVICE_ADDRESS, LCD_H_RES, LCD_V_RES};
use ecs::resources::{AppState, ButtonResource, DisplayResource, GameManager, GpioResource, SharedI2cResource, TouchResource};
use game::WorldMap;
use esp_idf_svc::hal::gpio::{Gpio0, PinDriver};
use esp_idf_svc::hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{SpiBusDriver, SpiDriver, config::DriverConfig, Dma},
    units::Hertz,
};
use esp_idf_svc::sys::*;
use std::thread;
use std::time::Duration;
use systems::{animation_cleanup_system, animation_init_system, autosave_system, AutoSaveState, button_system, fps_system, hero_overview_system, map_navigation_system, menu_system, render_system};

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

/// Initialize SD card with SPI3 and GPIO expander CS pin
fn init_sd_card(
    spi3: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::spi::SPI3> + 'static,
    gpio1: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio1> + 'static,
    gpio2: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio2> + 'static,
    gpio3: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio3> + 'static,
) -> Result<ecs::resources::SdCardWrapper, Box<dyn std::error::Error>> {
    log::info!("Initializing SD card...");

    // Initialize SPI3 for SD card
    // Pins: GPIO1=MOSI, GPIO2=SCK, GPIO3=MISO
    let driver_config = DriverConfig::new().dma(Dma::Auto(4096));
    let spi_driver = SpiDriver::new::<esp_idf_svc::hal::spi::SPI3>(
        spi3,
        gpio2,  // SCK
        gpio1,  // MOSI
        Some(gpio3), // MISO
        &driver_config,
    )?;

    // Wrap SpiDriver in SpiBusDriver to get embedded-hal SpiBus trait
    // Note: Baudrate is configured at the bus level
    let spi_bus = SpiBusDriver::new(spi_driver, &esp_idf_svc::hal::spi::config::Config::new().baudrate(Hertz(400_000)))?;

    log::info!("SPI3 initialized at 400kHz");

    // Create CS pin for SD card (EXIO7 on TCA9554)
    // This doesn't borrow the I2C driver, avoiding lifetime issues
    let cs_pin = drivers::SdCsPin::new()?;

    log::info!("SD card CS pin (EXIO7) configured");

    // Create SD card resource
    let sd_card_resource = sdcard::SdCardResource::new(spi_bus, cs_pin)?;

    log::info!("SD card initialized successfully");

    // Wrap in SdCardWrapper for ECS
    Ok(ecs::resources::SdCardWrapper::new(Box::new(sd_card_resource)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize system services
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("=== stdgotchi starting with Bevy ECS ===");
    log::info!("ESP32-S3 with 1.8\" AMOLED Display (SH8601)");

    let mut peripherals = Peripherals::take()?;

    // Initialize I2C for GPIO expander and touch controller
    log::info!("Initializing I2C...");
    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio15, // SDA
        peripherals.pins.gpio14, // SCL
        &i2c_config,
    )?;

    // Initialize display (uses I2C for reset via GPIO expander)
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

    // NOW leak I2C driver to make it 'static for SD card CS pin sharing
    // All other I2C initialization is complete, so we can dedicate it to SD card
    let i2c_static: &'static mut I2cDriver<'static> = Box::leak(Box::new(i2c));

    // Initialize shared I2C for SD card CS pin
    unsafe {
        drivers::sd_cs_pin::init_sd_i2c(i2c_static);
    }

    // Initialize SD card (CS pin will use shared I2C via static reference)
    log::info!("Initializing storage...");
    let sd_card_wrapper = match init_sd_card(
        peripherals.spi3,
        peripherals.pins.gpio1,
        peripherals.pins.gpio2,
        peripherals.pins.gpio3,
    ) {
        Ok(wrapper) => {
            log::info!("SD card mounted successfully");
            Some(wrapper)
        }
        Err(e) => {
            log::warn!("Failed to initialize SD card: {:?}", e);
            log::warn!("Game saves will not persist. Insert SD card and restart to enable saves.");
            None
        }
    };

    // Load map data
    log::info!("Loading map data...");
    let maps_json = include_str!("../assets/data/maps.json");
    let world_map = WorldMap::from_json(maps_json, "prontera".to_string())
        .expect("Failed to load map data");
    log::info!("Map loaded successfully");

    // Try to load save file if SD card is available
    let mut sd_wrapper_mut = sd_card_wrapper;
    let game_manager = if let Some(ref mut sd_wrapper) = sd_wrapper_mut.as_mut() {
        let filename = sdcard::get_save_path();
        log::info!("Attempting to load save file: {}", filename);

        match sd_wrapper.load_from_file(filename) {
            Ok(json_data) => {
                log::info!("Save file read successfully, parsing JSON...");
                match game::SaveData::from_json(&json_data) {
                    Ok(save_data) => {
                        log::info!("Save file loaded! Hero level: {}, Job: {:?}",
                                  save_data.hero.level, save_data.hero.job);
                        GameManager::from_save_data(save_data, world_map)
                    }
                    Err(e) => {
                        log::error!("Failed to parse save file: {:?}. Starting new game.", e);
                        GameManager::new(world_map)
                    }
                }
            }
            Err(e) => {
                log::info!("Could not load save file: {:?}. Starting new game.", e);
                GameManager::new(world_map)
            }
        }
    } else {
        log::info!("No SD card available. Starting new game.");
        GameManager::new(world_map)
    };

    // Create ECS World
    let mut world = World::new();

    // Insert resources
    let app_state = AppState::default(); // Starts in Menu mode
    world.insert_resource(app_state);

    // Insert autosave state
    world.insert_resource(AutoSaveState::default());

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
    // Insert shared I2C resource - provides access to the static I2C driver
    // Used by SD card CS pin operations and touch controller in systems
    world.insert_non_send_resource(SharedI2cResource);

    // Insert SD card resource (if available)
    if let Some(sd_wrapper) = sd_wrapper_mut {
        world.insert_non_send_resource(sd_wrapper);
    }

    // Insert game manager
    world.insert_non_send_resource(game_manager);

    // Create schedule and add systems
    // Order: FPS tracking → Input → Menu → Map Navigation → Hero Overview → Animation init → Render → Animation cleanup → Auto-save
    let mut schedule = Schedule::default();
    schedule.add_systems((
        fps_system,
        button_system::<Gpio0>,
        menu_system,
        map_navigation_system,
        hero_overview_system,
        animation_init_system,
        render_system,
        animation_cleanup_system,
        autosave_system,
    ));

    log::info!("stdgotchi ready! Starting in Menu mode. Press BOOT button to open menu...");

    // Main ECS game loop
    loop {
        // Run all systems
        schedule.run(&mut world);

        // Control frame rate (~100 FPS for responsive input)
        thread::sleep(Duration::from_millis(10));
    }
}
