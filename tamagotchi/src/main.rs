#![no_std]
#![no_main]

extern crate alloc;

use esp_bootloader_esp_idf::esp_app_desc;
esp_app_desc!();

use bevy_ecs::prelude::*;
use core::cell::RefCell;
use embedded_hal_bus::i2c;
use embedded_hal_bus::i2c::RefCellDevice;
use embedded_hal_bus::util::AtomicCell;
use embedded_sdmmc::{SdCard, VolumeIdx, VolumeManager};
use esp_hal::Blocking;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use esp_println::logger::init_logger_from_env;
use log::info;
use static_cell::StaticCell;

use axp2101::core::Axp2101;
use ft3x68_rs::{FT3168_DEVICE_ADDRESS, Ft3x68Driver};
use sh8601_rs::{ColorMode, DMA_CHUNK_SIZE, ResetDriver, Sh8601Driver, Ws18AmoledDriver};

// Import from our library
use esp32_conways_game_of_life_rs::core::GameState; // Core game state
use esp32_conways_game_of_life_rs::display::{DISPLAY_SIZE, FB_SIZE};
use esp32_conways_game_of_life_rs::drivers::{ExioPin, Pcf85063, ResetTouchDriver, Tca9554Driver};
use esp32_conways_game_of_life_rs::ecs::resources::*;
use esp32_conways_game_of_life_rs::tamagotchi::systems::{
    tamagotchi_button_system, tamagotchi_render_system, tamagotchi_save_system,
    tamagotchi_touch_system, tamagotchi_update_system,
};
use esp32_conways_game_of_life_rs::ui::voltage_to_battery_percent;
use esp32_conways_game_of_life_rs::utils::DummyTimeSource;

// Type aliases
static I2C_CELL: StaticCell<AtomicCell<RefCellDevice<'static, I2c<'static, Blocking>>>> =
    StaticCell::new();

static I2C_BUS: StaticCell<RefCell<I2c<'static, Blocking>>> = StaticCell::new();

#[main]
fn main() -> ! {
    esp_println::println!("[TAMAGOTCHI] Starting Ragnarok Tamagotchi...");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);
    init_logger_from_env();

    let delay = Delay::new();
    info!("Initializing display...");

    // --- DMA Buffers for SPI ---
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(DMA_CHUNK_SIZE);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // SPI Configuration
    let lcd_spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40_u32))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sio0(peripherals.GPIO4)
    .with_sio1(peripherals.GPIO5)
    .with_sio2(peripherals.GPIO6)
    .with_sio3(peripherals.GPIO7)
    .with_cs(peripherals.GPIO12)
    .with_sck(peripherals.GPIO11)
    .with_dma(peripherals.DMA_CH0)
    .with_buffers(dma_rx_buf, dma_tx_buf);

    // I2C Configuration
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO15)
    .with_scl(peripherals.GPIO14);

    let i2c_bus = I2C_BUS.init(RefCell::new(i2c));
    let i2c_device = RefCellDevice::new(i2c_bus);
    let ws_driver = Ws18AmoledDriver::new(lcd_spi);
    let reset = ResetDriver::new(i2c_device);

    let i2c_touch = RefCellDevice::new(i2c_bus);
    let i2c_cell = I2C_CELL.init(AtomicCell::new(i2c_touch));

    let reset_touch = ResetTouchDriver::new(i2c::AtomicDevice::new(i2c_cell));
    let mut touch = Ft3x68Driver::new(
        i2c::AtomicDevice::new(i2c_cell),
        FT3168_DEVICE_ADDRESS,
        reset_touch,
        delay,
    );

    touch
        .initialize()
        .expect("Failed to initialize touch driver");
    touch
        .set_gesture_mode(true)
        .expect("Failed to set gesture mode");

    // Initialize Display
    esp_println::println!("Initializing SH8601 Display...");
    let display_res = Sh8601Driver::new_heap::<_, FB_SIZE>(
        ws_driver,
        reset,
        ColorMode::Rgb888,
        DISPLAY_SIZE,
        delay,
    );

    let display = match display_res {
        Ok(d) => {
            esp_println::println!("Display initialized successfully.");
            d
        }
        Err(e) => {
            esp_println::println!("Error initializing display: {:?}", e);
            panic!("Failed to initialize display");
        }
    };

    // Initialize button (GPIO0 - Boot button)
    let boot_button = peripherals.GPIO0;
    let boot_config = InputConfig::default().with_pull(Pull::Up);
    let boot_button = Input::new(boot_button, boot_config);

    // Initialize GPIO expander for PWR button (EXIO4)
    esp_println::println!("Initializing GPIO expander for PWR button...");
    let i2c_gpio_expander = i2c::AtomicDevice::new(i2c_cell);
    let mut gpio_expander = Tca9554Driver::new(i2c_gpio_expander);

    // Configure EXIO4 (pin 4) as input for PWR button
    gpio_expander.configure_pin(4, false).unwrap_or_else(|e| {
        esp_println::println!("Warning: Could not configure EXIO4 as input: {:?}", e);
    });

    // Read initial state
    match gpio_expander.read_pin(4) {
        Ok(state) => esp_println::println!(
            "EXIO4 (PWR button) initial state: {}",
            if state { "HIGH" } else { "LOW" }
        ),
        Err(e) => esp_println::println!("Warning: Could not read EXIO4 initial state: {:?}", e),
    }

    // Initialize AXP2101 PMIC for battery monitoring
    esp_println::println!("Initializing AXP2101 PMIC...");
    let i2c_pmic = i2c::AtomicDevice::new(i2c_cell);
    let mut pmic = Axp2101::new(i2c_pmic);

    // Read initial battery voltage
    let battery_voltage_mv = pmic.battery_voltage().unwrap_or(0);
    let battery_percent = voltage_to_battery_percent(battery_voltage_mv);
    esp_println::println!("Battery: {}mV ({}%)", battery_voltage_mv, battery_percent);

    // Initialize RTC (for potential future use)
    let i2c_rtc = i2c::AtomicDevice::new(i2c_cell);
    let rtc = Pcf85063::new(i2c_rtc);

    // Initialize SD Card
    // SD card pins: MOSI=GPIO1, MISO=GPIO3, SCK=GPIO2, CS=EXIO7
    esp_println::println!("Initializing SD card...");

    let sd_spi = Spi::new(
        peripherals.SPI3,
        SpiConfig::default()
            .with_frequency(Rate::from_khz(400)) // Start slow for initialization
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO2)
    .with_mosi(peripherals.GPIO1)
    .with_miso(peripherals.GPIO3);

    // Create GPIO expander instance for SD card CS pin (EXIO7)
    let i2c_sd_cs = i2c::AtomicDevice::new(i2c_cell);
    let sd_cs_expander = Tca9554Driver::new(i2c_sd_cs);
    let sd_cs_pin = ExioPin::new(sd_cs_expander, 7).expect("Failed to configure SD CS pin");

    // Wrap SPI with ExclusiveDevice for CS control
    use embedded_hal_bus::spi::ExclusiveDevice;
    let sd_spi_device = ExclusiveDevice::new(sd_spi, sd_cs_pin, Delay::new()).unwrap();

    // Create SD card and volume manager
    let sd_card = SdCard::new(sd_spi_device, Delay::new());
    let time_source = DummyTimeSource;
    let mut volume_mgr = VolumeManager::new(sd_card, time_source);

    esp_println::println!("SD card initialized successfully");

    // Initialize Bevy ECS World
    let mut world = World::default();

    // Try to load saved hero data from SD card
    let loaded_hero = load_hero_from_sd(&mut volume_mgr);

    // Insert game state with loaded or default hero
    let mut game_state = GameState::default();
    if let Some(mut hero) = loaded_hero {
        // Try to load inventory
        load_inventory_from_sd(&mut volume_mgr, &mut hero);

        esp_println::println!(
            "Loaded saved hero: Level {} {} with {} EXP and {} items",
            hero.level,
            hero.job,
            hero.exp,
            hero.inventory.len()
        );
        game_state.hero = hero;
    } else {
        esp_println::println!(
            "No save file found - Starting {} Level {}",
            game_state.hero.job,
            game_state.hero.level
        );
    }

    // Initialize quest system (auto-start achievements and daily quests)
    esp32_conways_game_of_life_rs::tamagotchi::quest_system::initialize_quest_system(&mut game_state);

    world.insert_resource(game_state);

    // Insert SD card resource
    let sd_resource = SdCardResource { volume_mgr };
    world.insert_non_send_resource(sd_resource);

    // Insert battery resource
    world.insert_resource(BatteryResource {
        voltage_mv: battery_voltage_mv,
        percent: battery_percent,
        last_update_generation: 0,
    });

    // Insert display and touch as NonSend resources
    world.insert_non_send_resource(DisplayResource { display });
    world.insert_non_send_resource(TouchResource {
        touch,
        last_touch_state: false,
    });

    // Insert AXP2101 PMIC resource
    world.insert_non_send_resource(Axp2101Resource { pmic });

    // Insert RTC resource (for potential future use)
    let initial_cycles = esp_hal::xtensa_lx::timer::get_cycle_count();
    world.insert_non_send_resource(RtcResource {
        rtc,
        last_timestamp: None,
        last_cycles: initial_cycles,
        cpu_freq_mhz: 240,
    });

    // Insert button resource
    world.insert_non_send_resource(ButtonResource {
        boot_button,
        gpio_expander,
        boot_last_state: false,
        pwr_last_state: false,
        boot_debounce_counter: 0,
        pwr_debounce_counter: 0,
    });

    // Create schedule and add systems
    let mut schedule = Schedule::default();
    schedule.add_systems(tamagotchi_button_system);
    schedule.add_systems(tamagotchi_touch_system);
    schedule.add_systems(tamagotchi_update_system);
    schedule.add_systems(update_battery_system);
    schedule.add_systems(tamagotchi_save_system);
    schedule.add_systems(tamagotchi_render_system);

    info!("Entering Tamagotchi game loop...");

    // Main game loop
    loop {
        schedule.run(&mut world);

        // Small delay to control frame rate (~120 FPS)
        esp_hal::delay::Delay::new().delay_millis(8);
    }
}

/// System to update battery information periodically
fn update_battery_system(
    mut axp_res: NonSendMut<Axp2101Resource>,
    mut battery_res: ResMut<BatteryResource>,
    mut game_state: ResMut<GameState>,
) {
    // Update battery every 10 seconds (much less frequent)
    if game_state.last_update_ms % 10000 < 100 {
        if let Ok(voltage_mv) = axp_res.pmic.battery_voltage() {
            let new_percent = voltage_to_battery_percent(voltage_mv);
            // Only update and mark for redraw if values actually changed
            if battery_res.voltage_mv != voltage_mv || battery_res.percent != new_percent {
                battery_res.voltage_mv = voltage_mv;
                battery_res.percent = new_percent;
                game_state.needs_redraw = true;
            }
        }
    }
}

/// Load hero data from SD card
fn load_hero_from_sd<D, T>(
    volume_mgr: &mut VolumeManager<D, T, 4, 4, 1>,
) -> Option<esp32_conways_game_of_life_rs::tamagotchi::models::Hero>
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    D::Error: core::fmt::Debug,
{
    use embedded_sdmmc::Mode;

    esp_println::println!("[LOAD] Attempting to load hero from SD card...");

    // Open volume
    let mut volume = volume_mgr.open_volume(VolumeIdx(0)).ok()?;

    // Open root directory
    let mut root_dir = volume.open_root_dir().ok()?;

    // Try to open save file
    let mut file = root_dir.open_file_in_dir("HERO.SAV", Mode::ReadOnly).ok()?;

    // Read file contents
    let mut buffer = [0u8; 128];
    let bytes_read = file.read(&mut buffer).ok()?;

    esp_println::println!("[LOAD] Read {} bytes from HERO.SAV", bytes_read);

    // Parse save data
    let save_str = core::str::from_utf8(&buffer[..bytes_read]).ok()?;
    esp_println::println!("[LOAD] Save data: {}", save_str);

    esp32_conways_game_of_life_rs::tamagotchi::models::Hero::from_save_string(save_str)
}

/// Load inventory data from SD card
fn load_inventory_from_sd<D, T>(
    volume_mgr: &mut VolumeManager<D, T, 4, 4, 1>,
    hero: &mut esp32_conways_game_of_life_rs::tamagotchi::models::Hero,
) where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    D::Error: core::fmt::Debug,
{
    use embedded_sdmmc::Mode;

    esp_println::println!("[LOAD] Attempting to load inventory from SD card...");

    // Open volume
    let Ok(mut volume) = volume_mgr.open_volume(VolumeIdx(0)) else {
        esp_println::println!("[LOAD] Failed to open volume for inventory");
        return;
    };

    // Open root directory
    let Ok(mut root_dir) = volume.open_root_dir() else {
        esp_println::println!("[LOAD] Failed to open root directory for inventory");
        return;
    };

    // Try to open inventory file
    let Ok(mut file) = root_dir.open_file_in_dir("ITEMS.SAV", Mode::ReadOnly) else {
        esp_println::println!("[LOAD] No ITEMS.SAV found (this is OK for new games)");
        return;
    };

    // Read file contents
    let mut buffer = [0u8; 512];
    let Ok(bytes_read) = file.read(&mut buffer) else {
        esp_println::println!("[LOAD] Failed to read ITEMS.SAV");
        return;
    };

    esp_println::println!("[LOAD] Read {} bytes from ITEMS.SAV", bytes_read);

    // Parse inventory data
    if let Ok(save_str) = core::str::from_utf8(&buffer[..bytes_read]) {
        esp_println::println!("[LOAD] Inventory data: {}", save_str);
        hero.inventory_from_save_string(save_str);
    } else {
        esp_println::println!("[LOAD] Failed to parse ITEMS.SAV");
    }

    // Resources will be cleaned up automatically when they go out of scope
}
