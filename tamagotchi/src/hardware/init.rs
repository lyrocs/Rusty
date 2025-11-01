use core::cell::RefCell;
use bevy_ecs::prelude::World;
use embedded_hal_bus::i2c;
use embedded_hal_bus::i2c::RefCellDevice;
use embedded_hal_bus::util::AtomicCell;
use embedded_sdmmc::{SdCard, VolumeIdx, VolumeManager};
use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use log::info;
use static_cell::StaticCell;

use axp2101::core::Axp2101;
use ft3x68_rs::{FT3168_DEVICE_ADDRESS, Ft3x68Driver};
use sh8601_rs::{ColorMode, DMA_CHUNK_SIZE, ResetDriver, Sh8601Driver, Ws18AmoledDriver};

use crate::core::GameState;
use crate::display::{DISPLAY_SIZE, FB_SIZE};
use crate::drivers::{ExioPin, Pcf85063, ResetTouchDriver, Tca9554Driver};
use crate::ecs::resources::*;
use crate::ui::voltage_to_battery_percent;
use crate::utils::DummyTimeSource;

// Type aliases for I2C bus sharing
static I2C_CELL: StaticCell<AtomicCell<RefCellDevice<'static, I2c<'static, Blocking>>>> =
    StaticCell::new();

static I2C_BUS: StaticCell<RefCell<I2c<'static, Blocking>>> = StaticCell::new();

/// All hardware resources needed by the application
pub struct HardwareResources {
    pub world: World,
    pub timg0: TimerGroup<'static, esp_hal::peripherals::TIMG0<'static>>,
}

/// Initialize all hardware peripherals and resources
pub fn init_hardware(peripherals: esp_hal::peripherals::Peripherals) -> HardwareResources {
    info!("[INIT] Starting hardware initialization...");

    // Initialize PSRAM allocator
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let delay = Delay::new();
    info!("[INIT] Initializing display...");

    // --- DMA Buffers for SPI ---
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(DMA_CHUNK_SIZE);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // SPI Configuration for Display
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
    info!("[INIT] Initializing SH8601 Display...");
    let display_res = Sh8601Driver::new_heap::<_, FB_SIZE>(
        ws_driver,
        reset,
        ColorMode::Rgb888,
        DISPLAY_SIZE,
        delay,
    );

    let display = match display_res {
        Ok(d) => {
            info!("[INIT] Display initialized successfully.");
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
    info!("[INIT] Initializing GPIO expander for PWR button...");
    let i2c_gpio_expander = i2c::AtomicDevice::new(i2c_cell);
    let mut gpio_expander = Tca9554Driver::new(i2c_gpio_expander);

    // Configure EXIO4 (pin 4) as input for PWR button
    gpio_expander.configure_pin(4, false).unwrap_or_else(|e| {
        esp_println::println!("Warning: Could not configure EXIO4 as input: {:?}", e);
    });

    // Read initial state
    match gpio_expander.read_pin(4) {
        Ok(state) => info!(
            "EXIO4 (PWR button) initial state: {}",
            if state { "HIGH" } else { "LOW" }
        ),
        Err(e) => esp_println::println!("Warning: Could not read EXIO4 initial state: {:?}", e),
    }

    // Initialize AXP2101 PMIC for battery monitoring
    info!("[INIT] Initializing AXP2101 PMIC...");
    let i2c_pmic = i2c::AtomicDevice::new(i2c_cell);
    let mut pmic = Axp2101::new(i2c_pmic);

    // Read initial battery voltage
    let battery_voltage_mv = pmic.battery_voltage().unwrap_or(0);
    let battery_percent = voltage_to_battery_percent(battery_voltage_mv);
    info!("Battery: {}mV ({}%)", battery_voltage_mv, battery_percent);

    // Initialize RTC (for potential future use)
    let i2c_rtc = i2c::AtomicDevice::new(i2c_cell);
    let rtc = Pcf85063::new(i2c_rtc);

    // Initialize SD Card
    info!("[INIT] Initializing SD card...");

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

    info!("[INIT] SD card initialized successfully");

    // Initialize Bevy ECS World
    let mut world = World::default();

    // Try to load saved hero data from SD card
    let loaded_hero = load_hero_from_sd(&mut volume_mgr);

    // Insert game state with loaded or default hero
    let mut game_state = GameState::default();
    let has_saved_hero = loaded_hero.is_some();

    if let Some(mut hero) = loaded_hero {
        // Try to load inventory
        load_inventory_from_sd(&mut volume_mgr, &mut hero);

        // Try to load equipment with card data
        load_equipment_from_sd(&mut volume_mgr, &mut hero);

        info!(
            "Loaded saved hero: Level {} {} with {} EXP and {} items",
            hero.level,
            hero.job,
            hero.exp,
            hero.inventory.len()
        );
        game_state.hero = hero;
    } else {
        info!(
            "No save file found - Starting {} Level {}",
            game_state.hero.job, game_state.hero.level
        );
    }

    // Try to load quest data
    load_quests_from_sd(&mut volume_mgr, &mut game_state);

    // Initialize quest system only if no saved quests were loaded (new game)
    if game_state.active_quests.is_empty() && has_saved_hero {
        info!("[QUEST] No saved quests found, initializing quest system");
        crate::quest::initialize_quest_system(&mut game_state);
    } else if game_state.active_quests.is_empty() && !has_saved_hero {
        // Brand new game - initialize quests
        info!("[QUEST] New game, initializing quest system");
        crate::quest::initialize_quest_system(&mut game_state);
    } else {
        info!("[QUEST] Loaded saved quest progress");
    }

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

    // Insert TouchInput resource for Embassy tasks
    world.insert_resource(crate::tasks::game::TouchInput::default());

    // Get timer for Embassy
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    info!("[INIT] Hardware initialization complete!");

    HardwareResources { world, timg0 }
}

/// Load hero data from SD card
fn load_hero_from_sd<D, T>(
    volume_mgr: &mut VolumeManager<D, T, 4, 4, 1>,
) -> Option<crate::hero::Hero>
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    D::Error: core::fmt::Debug,
{
    use embedded_sdmmc::Mode;

    info!("[LOAD] Attempting to load hero from SD card...");

    // Open volume
    let mut volume = volume_mgr.open_volume(VolumeIdx(0)).ok()?;

    // Open root directory
    let mut root_dir = volume.open_root_dir().ok()?;

    // Try to open save file
    let mut file = root_dir.open_file_in_dir("HERO.SAV", Mode::ReadOnly).ok()?;

    // Read file contents
    let mut buffer = [0u8; 128];
    let bytes_read = file.read(&mut buffer).ok()?;

    info!("[LOAD] Read {} bytes from HERO.SAV", bytes_read);

    // Parse save data
    let save_str = core::str::from_utf8(&buffer[..bytes_read]).ok()?;
    info!("[LOAD] Save data: {}", save_str);

    crate::hero::Hero::from_save_string(save_str)
}

/// Load inventory data from SD card
fn load_inventory_from_sd<D, T>(
    volume_mgr: &mut VolumeManager<D, T, 4, 4, 1>,
    hero: &mut crate::hero::Hero,
) where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    D::Error: core::fmt::Debug,
{
    use embedded_sdmmc::Mode;

    info!("[LOAD] Attempting to load inventory from SD card...");

    // Open volume
    let Ok(mut volume) = volume_mgr.open_volume(VolumeIdx(0)) else {
        info!("[LOAD] Failed to open volume for inventory");
        return;
    };

    // Open root directory
    let Ok(mut root_dir) = volume.open_root_dir() else {
        info!("[LOAD] Failed to open root directory for inventory");
        return;
    };

    // Try to open inventory file
    let Ok(mut file) = root_dir.open_file_in_dir("ITEMS.SAV", Mode::ReadOnly) else {
        info!("[LOAD] No ITEMS.SAV found (this is OK for new games)");
        return;
    };

    // Read file contents
    let mut buffer = [0u8; 512];
    let Ok(bytes_read) = file.read(&mut buffer) else {
        info!("[LOAD] Failed to read ITEMS.SAV");
        return;
    };

    info!("[LOAD] Read {} bytes from ITEMS.SAV", bytes_read);

    // Parse inventory data
    if let Ok(save_str) = core::str::from_utf8(&buffer[..bytes_read]) {
        info!("[LOAD] Inventory data: {}", save_str);
        hero.inventory_from_save_string(save_str);
    } else {
        info!("[LOAD] Failed to parse ITEMS.SAV");
    }
}

/// Load equipment data from SD card
fn load_equipment_from_sd<D, T>(
    volume_mgr: &mut VolumeManager<D, T, 4, 4, 1>,
    hero: &mut crate::hero::Hero,
) where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    D::Error: core::fmt::Debug,
{
    use embedded_sdmmc::Mode;

    info!("[LOAD] Attempting to load equipment from SD card...");

    // Open volume
    let Ok(mut volume) = volume_mgr.open_volume(VolumeIdx(0)) else {
        info!("[LOAD] Failed to open volume for equipment");
        return;
    };

    // Open root directory
    let Ok(mut root_dir) = volume.open_root_dir() else {
        info!("[LOAD] Failed to open root directory for equipment");
        return;
    };

    // Try to open equipment file
    let Ok(mut file) = root_dir.open_file_in_dir("EQUIP.SAV", Mode::ReadOnly) else {
        info!("[LOAD] No EQUIP.SAV found (this is OK for old saves)");
        return;
    };

    // Read file contents
    let mut buffer = [0u8; 256];
    let Ok(bytes_read) = file.read(&mut buffer) else {
        info!("[LOAD] Failed to read EQUIP.SAV");
        return;
    };

    info!("[LOAD] Read {} bytes from EQUIP.SAV", bytes_read);

    // Parse equipment data
    if let Ok(save_str) = core::str::from_utf8(&buffer[..bytes_read]) {
        info!("[LOAD] Equipment data: {}", save_str);
        hero.equipment_from_save_string(save_str);
    } else {
        info!("[LOAD] Failed to parse EQUIP.SAV");
    }
}

/// Load quest data from SD card
fn load_quests_from_sd<D, T>(
    volume_mgr: &mut VolumeManager<D, T, 4, 4, 1>,
    game_state: &mut crate::core::GameState,
) where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    D::Error: core::fmt::Debug,
{
    use embedded_sdmmc::Mode;

    info!("[LOAD] Attempting to load quests from SD card...");

    // Open volume
    let Ok(mut volume) = volume_mgr.open_volume(VolumeIdx(0)) else {
        info!("[LOAD] Failed to open volume for quests");
        return;
    };

    // Open root directory
    let Ok(mut root_dir) = volume.open_root_dir() else {
        info!("[LOAD] Failed to open root directory for quests");
        return;
    };

    // Try to open quest file
    let Ok(mut file) = root_dir.open_file_in_dir("QUESTS.SAV", Mode::ReadOnly) else {
        info!("[LOAD] No QUESTS.SAV found (this is OK for new games)");
        return;
    };

    // Read file contents
    let mut buffer = [0u8; 1024];
    let Ok(bytes_read) = file.read(&mut buffer) else {
        info!("[LOAD] Failed to read QUESTS.SAV");
        return;
    };

    info!("[LOAD] Read {} bytes from QUESTS.SAV", bytes_read);

    // Parse quest data
    if let Ok(save_str) = core::str::from_utf8(&buffer[..bytes_read]) {
        info!("[LOAD] Quest data: {}", save_str);
        game_state.quests_from_save_string(save_str);
    } else {
        info!("[LOAD] Failed to parse QUESTS.SAV");
    }
}
