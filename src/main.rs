//! stdgotchi - ESP32-C6 LCD Touch Demo
//!
//! Interactive drawing demo for Waveshare ESP32-C6-Touch-LCD-1.83
//!
//! # Features
//! - Touch drawing with single-touch support
//! - Gesture recognition (swipes and double-tap)
//! - SPI LCD display with RGB888 color
//! - Animated GIF playback
//!
//! # Hardware
//! - Board: ESP32-C6 with RISC-V processor
//! - Display: 1.83" LCD (240x284) via SPI
//! - Touch: CST816D capacitive touch via I2C
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
mod input_thread;
mod sdcard;
mod systems;
mod ui;
mod wifi;

use bevy_ecs::prelude::*;
use crossbeam_channel::unbounded;
use display::{ColorMode, Cst816dDriver, St7789pDriver, CST816D_DEVICE_ADDRESS, LCD_H_RES, LCD_V_RES};
use ecs::resources::{AppState, ButtonResource, DisplayResource, GameManager, InputEventChannel, SharedI2cResource};
use game::WorldMap;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::{
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{SpiBusDriver, SpiDeviceDriver, SpiDriver, SpiDriverConfig, config::{Config as SpiConfig, DriverConfig}, Dma},
    units::Hertz,
};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::*;
use std::thread;
use std::time::Duration;
use systems::{animation_cleanup_system, animation_init_system, autosave_system, AutoSaveState, battle_loading_system, battle_result_system, battle_system, button_system, death_detection_system, death_system, dungeon_combat_navigation_system, between_floors_navigation_system, dungeon_defeat_navigation_system, dungeon_list_navigation_system, dungeon_info_navigation_system, expedition_navigation_system, fps_system, home_navigation_system, map_navigation_system, menu_system, monster_navigation_system, render_system, utility_navigation_system};

/// Initialize the shared SPI bus for display and SD card
/// Returns a 'static reference to the SPI driver that can be shared
fn init_shared_spi_bus(
    spi2: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::spi::SPI2> + 'static,
    sck: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio1> + 'static,
    mosi: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio2> + 'static,
    miso: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio16> + 'static,
) -> Result<&'static SpiDriver<'static>, Box<dyn std::error::Error>> {
    log::info!("Initializing shared SPI bus (SCK=GPIO1, MOSI=GPIO2, MISO=GPIO16)...");

    // Initialize SPI2 with MISO for SD card support
    let driver_config = DriverConfig::new().dma(Dma::Auto(4096));
    let spi_driver = SpiDriver::new::<esp_idf_svc::hal::spi::SPI2>(
        spi2,
        sck,
        mosi,
        Some(miso), // MISO for SD card
        &driver_config,
    )?;

    // Leak to get 'static lifetime for bus sharing
    let spi_driver_static: &'static SpiDriver<'static> = Box::leak(Box::new(spi_driver));

    log::info!("Shared SPI bus initialized");
    Ok(spi_driver_static)
}

/// Initialize the display using the shared SPI bus
fn init_display(
    spi_bus: &'static SpiDriver<'static>,
    cs: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio5> + 'static,
    dc: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio3> + 'static,
    rst: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio4> + 'static,
    backlight: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio6> + 'static,
) -> Result<St7789pDriver<'static>, Box<dyn std::error::Error>> {
    log::info!("Initializing display on shared SPI bus...");

    // Configure SPI to 40MHz for fast display updates
    let spi_config = SpiConfig::new().baudrate(Hertz(40_000_000));
    let spi_device = SpiDeviceDriver::new(spi_bus, Some(cs), &spi_config)?;

    log::info!("Display SPI device initialized at 40MHz");

    // Initialize DC and RST pins
    let dc_pin = PinDriver::output(dc)?;
    let rst_pin = PinDriver::output(rst)?;

    // Initialize backlight pin (GPIO6)
    let mut bl_pin = PinDriver::output(backlight)?;
    bl_pin.set_high()?; // Turn backlight on initially
    log::info!("Backlight pin (GPIO6) initialized");

    // Initialize display driver with RGB565 for memory efficiency
    // (Native 65K color display, saves 66KB RAM vs RGB888)
    log::info!("Initializing ST7789P display driver (RGB565 mode)...");
    let mut display = St7789pDriver::new(
        spi_device,
        dc_pin,
        rst_pin,
        LCD_H_RES,
        LCD_V_RES,
        ColorMode::Rgb565,
    )?;
    display.initialize(ColorMode::Rgb565)?;

    // Set backlight pin on display driver for on/off control
    display.set_backlight_pin(bl_pin);

    log::info!("Display initialized successfully!");

    Ok(display)
}

/// Initialize SD card using the shared SPI bus
fn init_sd_card(
    spi_bus: &'static SpiDriver<'static>,
    cs: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::gpio::Gpio17> + 'static,
) -> Result<ecs::resources::SdCardWrapper, Box<dyn std::error::Error>> {
    log::info!("Initializing SD card on shared SPI bus (CS=GPIO17)...");

    // Configure SPI for SD card at 20MHz (most cards support this)
    // Note: SD spec says init at 400kHz, but modern cards work fine at higher speeds
    let spi_config = SpiConfig::new().baudrate(Hertz(20_000_000)); // 20MHz
    let spi_device = SpiDeviceDriver::new(spi_bus, Some(cs), &spi_config)?;

    log::info!("SD card SPI device created at 20MHz");

    // Initialize SD card
    let sd_resource = sdcard::SdCardResource::new(spi_device)?;
    log::info!("SD card initialized successfully!");

    Ok(ecs::resources::SdCardWrapper::new(Box::new(sd_resource)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize system services
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("=== stdgotchi starting with Bevy ECS ===");
    log::info!("ESP32-C6 with 1.83\" LCD Display (ST7789P)");

    let peripherals = Peripherals::take()?;

    // Initialize I2C for touch controller and sensors
    log::info!("Initializing I2C...");
    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio7, // SDA
        peripherals.pins.gpio8, // SCL
        &i2c_config,
    )?;

    // Initialize shared SPI bus for display and SD card
    log::info!("Initializing shared SPI bus...");
    let spi_bus = init_shared_spi_bus(
        peripherals.spi2,
        peripherals.pins.gpio1,   // SCK
        peripherals.pins.gpio2,   // MOSI
        peripherals.pins.gpio16,  // MISO (for SD card)
    )?;

    // Initialize display on the shared SPI bus
    log::info!("Initializing display...");
    let display = init_display(
        spi_bus,
        peripherals.pins.gpio5,   // Display CS
        peripherals.pins.gpio3,   // DC
        peripherals.pins.gpio4,   // RST
        peripherals.pins.gpio6,   // Backlight
    )?;

    // Initialize touch controller
    log::info!("Initializing CST816D touch controller...");
    let mut touch = Cst816dDriver::new(CST816D_DEVICE_ADDRESS);
    touch.initialize(&mut i2c)?;
    log::info!("Touch controller initialized successfully!");

    // Initialize buttons
    log::info!("Initializing buttons...");
    let boot_pin = PinDriver::input(peripherals.pins.gpio9)?;
    log::info!("Boot button on GPIO9 initialized");

    // PWR button on GPIO18 (per Waveshare documentation)
    let pwr_pin = PinDriver::input(peripherals.pins.gpio18)?;
    log::info!("PWR button on GPIO18 initialized");
    log::info!("Buttons initialized successfully!");

    // Leak I2C driver to make it 'static for input thread sharing
    // The input thread needs access to I2C for touch controller
    log::info!("Setting up shared I2C for input thread...");
    let i2c_static: &'static mut I2cDriver<'static> = Box::leak(Box::new(i2c));

    // Initialize shared I2C for input thread
    unsafe {
        drivers::sd_cs_pin::init_sd_i2c(i2c_static);
    }
    log::info!("Shared I2C configured for input thread");

    // Initialize SD card on the shared SPI bus
    log::info!("Initializing SD card...");
    let mut sd_card_wrapper = match init_sd_card(spi_bus, peripherals.pins.gpio17) {
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

    // WiFi initialization disabled for faster boot
    // TODO: Re-enable when WiFi features are needed
    /*
    // Initialize WiFi (needs to happen after SD card for config loading)
    log::info!("Initializing WiFi...");
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Load WiFi config from SD card or use default
    let wifi_config = if let Some(ref mut sd_wrapper) = sd_card_wrapper {
        match wifi::load_wifi_config(sd_wrapper) {
            Ok(config) => {
                log::info!("WiFi config loaded from SD card");
                config
            }
            Err(e) => {
                log::warn!("Failed to load WiFi config: {:?}", e);
                log::info!("Creating default WiFi config file on SD card...");

                // Try to create default config file
                if let Err(create_err) = wifi::create_default_wifi_config(sd_wrapper) {
                    log::error!("Failed to create default WiFi config: {:?}", create_err);
                }

                log::warn!("Using default WiFi credentials - please edit /sdcard/wifi.json");
                wifi::WifiConfig::default()
            }
        }
    } else {
        log::warn!("No SD card available - using default WiFi credentials");
        wifi::WifiConfig::default()
    };

    let wifi_resource = match wifi::wifi_create(&wifi_config, peripherals.modem, sysloop, nvs) {
        Ok(wifi_conn) => {
            log::info!("WiFi initialized successfully!");
            Some(ecs::resources::WifiResource { wifi: wifi_conn })
        }
        Err(e) => {
            log::error!("WiFi initialization failed: {:?}", e);
            log::warn!("Continuing without WiFi - Pokemon API will not work");
            None
        }
    };
    */
    log::info!("WiFi skipped for faster boot");

    // Load game data (maps, enemies, etc.)
    log::info!("Loading game data...");
    game::init_exp_table(); // Initialize global exp table for Rustymon
    let game_data = game::GameData::load_from_assets()
        .expect("Failed to load game data");
    log::info!("Game data loaded successfully");

    // Clone game data for WorldMap and GameManager
    let game_data_for_map = game_data.clone();

    // Create world map with game data
    let world_map = WorldMap::new(game_data_for_map, 1); // Start at Prontera (ID 1)
    log::info!("World map initialized");

    // Try to load save file if SD card is available
    let game_manager = if let Some(ref mut sd_wrapper) = sd_card_wrapper.as_mut() {
        let filename = sdcard::get_save_path();
        log::info!("Attempting to load save file: {}", filename);

        match sd_wrapper.load_from_file(filename) {
            Ok(json_data) => {
                log::info!("Save file read successfully, parsing JSON...");
                match game::SaveData::from_json(&json_data) {
                    Ok(save_data) => {
                        log::info!("Save file loaded! Play time: {} seconds", save_data.play_time_seconds);
                        GameManager::from_save_data(save_data, world_map, game_data)
                    }
                    Err(e) => {
                        log::error!("Failed to parse save file: {:?}. Starting new game.", e);
                        GameManager::new(world_map, game_data)
                    }
                }
            }
            Err(e) => {
                log::info!("Could not load save file: {:?}. Starting new game.", e);
                GameManager::new(world_map, game_data)
            }
        }
    } else {
        log::info!("No SD card available. Starting new game.");
        GameManager::new(world_map, game_data)
    };

    // Create input event channel for Core 0 → Core 1 communication
    let (input_sender, input_receiver) = unbounded();

    // Spawn input polling thread
    // This thread handles all input (touch, buttons) at high frequency
    // FreeRTOS scheduler will distribute threads across both cores automatically
    log::info!("Starting dual-threaded mode: Input thread + Game thread on dual-core ESP32-C6");
    let _input_thread = input_thread::spawn_input_thread(
        boot_pin,
        pwr_pin,
        touch,
        input_sender,
    );
    log::info!("[MAIN] Input thread spawned - will run on separate core for maximum responsiveness");

    // Create ECS World
    let mut world = World::new();

    // Insert resources
    let app_state = AppState::default(); // Starts in Menu mode
    world.insert_resource(app_state);

    // Insert autosave state
    world.insert_resource(AutoSaveState::default());

    // Insert input event channel
    world.insert_resource(InputEventChannel {
        receiver: input_receiver,
    });

    // Insert pending input events resource (for forwarding events from button_system)
    world.insert_resource(ecs::resources::PendingInputEvents::default());

    // Insert non-send resources (hardware peripherals)
    world.insert_non_send_resource(DisplayResource { display });
    // Note: TouchResource and GpioResource are now owned by the input thread
    // Input events come through the InputEventChannel instead

    // Insert button resource for button state tracking
    world.insert_non_send_resource(ButtonResource {
        boot_last_state: false,
        pwr_last_state: false,
        boot_debounce: 0,
        pwr_debounce: 0,
    });

    // Insert shared I2C resource - provides access to the static I2C driver
    // Used by SD card CS pin operations
    world.insert_non_send_resource(SharedI2cResource);

    // WiFi resource disabled - see WiFi initialization section above
    /*
    // Insert WiFi resource (if available) - keeps WiFi connection alive
    if let Some(wifi_res) = wifi_resource {
        world.insert_non_send_resource(wifi_res);
        log::info!("WiFi resource inserted into ECS world - connection will stay active");
    }
    */

    // Insert SD card resource (if available)
    if let Some(sd_wrapper) = sd_card_wrapper {
        world.insert_non_send_resource(sd_wrapper);
    }

    // Insert game manager
    world.insert_non_send_resource(game_manager);

    // Create schedule and add systems
    // Order: FPS tracking → Button handler (MUST run first to consume PWR events) → Input handlers → Render → Auto-save
    // Note: Input now comes from the input thread via channel, consumed by mode-specific systems
    let mut schedule = Schedule::default();

    // Button system MUST run first (sequentially) to consume PWR button events
    schedule.add_systems(button_system);

    // All other systems run after button_system
    // Add systems in groups (max 16 per tuple)
    schedule.add_systems((
        fps_system,
        home_navigation_system, // Handle home screen navigation (default start screen)
        menu_system,
        map_navigation_system,
        monster_navigation_system, // Handle monster list/detail navigation
        utility_navigation_system, // Handle inventory/collection navigation
        expedition_navigation_system, // Handle expedition map/team/result navigation
        dungeon_list_navigation_system, // Handle dungeon list -> dungeon info
        dungeon_info_navigation_system, // Handle dungeon info -> start combat
        dungeon_combat_navigation_system, // Handle dungeon combat -> between floors
        between_floors_navigation_system, // Handle between floors -> next combat or exit
        dungeon_defeat_navigation_system, // Handle defeat screen retry/quit
        battle_loading_system, // Creates battle page after loading screen shown
        battle_system,
        battle_result_system, // Handle battle result screen
        death_detection_system, // Check for death in battle
    ));

    // Second batch of systems
    schedule.add_systems((
        death_system, // Handle death screen and respawn
        animation_init_system,
        render_system,
        animation_cleanup_system,
        autosave_system,
    ));

    log::info!("stdgotchi ready! Dual-threaded mode active.");
    log::info!("Input thread: GPIO at 100Hz, Touch/I2C at 20Hz (reduced bus contention)");
    log::info!("Main thread: Game logic and rendering at ~60 FPS");

    // Main ECS game loop
    loop {
        // Run all systems
        schedule.run(&mut world);

        // Control frame rate (~60 FPS)
        // Input is handled separately at 200Hz in the input thread
        thread::sleep(Duration::from_millis(16));
    }
}
