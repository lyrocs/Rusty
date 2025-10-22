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
use esp_hal::Blocking;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use esp_hal::dma_buffers;
use esp_println::logger::init_logger_from_env;
use log::info;
use static_cell::StaticCell;

use ft3x68_rs::{FT3168_DEVICE_ADDRESS, Ft3x68Driver};
use sh8601_rs::{ColorMode, DMA_CHUNK_SIZE, ResetDriver, Sh8601Driver, Ws18AmoledDriver};

// Import from our library
use esp32_conways_game_of_life_rs::display::{DISPLAY_SIZE, FB_SIZE};
use esp32_conways_game_of_life_rs::drivers::{ResetTouchDriver, Tca9554Driver};
use esp32_conways_game_of_life_rs::ecs::resources::*;
use esp32_conways_game_of_life_rs::tamagotchi::{GameState};
use esp32_conways_game_of_life_rs::tamagotchi::systems::{
    tamagotchi_button_system,
    tamagotchi_touch_system,
    tamagotchi_update_system,
    tamagotchi_render_system
};

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

    touch.initialize().expect("Failed to initialize touch driver");
    touch.set_gesture_mode(true).expect("Failed to set gesture mode");

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

    // Initialize GPIO expander (not used for this game, but keeping for compatibility)
    esp_println::println!("Initializing GPIO expander...");
    let i2c_gpio_expander = i2c::AtomicDevice::new(i2c_cell);
    let gpio_expander = Tca9554Driver::new(i2c_gpio_expander);

    // Initialize Bevy ECS World
    let mut world = World::default();

    // Insert game state
    world.insert_resource(GameState::default());

    // Insert display and touch as NonSend resources
    world.insert_non_send_resource(DisplayResource { display });
    world.insert_non_send_resource(TouchResource { touch });

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
    schedule.add_systems(tamagotchi_render_system);

    info!("Entering Tamagotchi game loop...");

    // Main game loop
    loop {
        schedule.run(&mut world);

        // Small delay to control frame rate (~60 FPS)
        esp_hal::delay::Delay::new().delay_millis(16);
    }
}
