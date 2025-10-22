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
use esp_hal::rng::Rng;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use esp_hal::dma_buffers;
use esp_println::logger::init_logger_from_env;
use log::info;
use static_cell::StaticCell;
use tinybmp::Bmp;

use ft3x68_rs::{FT3168_DEVICE_ADDRESS, Ft3x68Driver};
use sh8601_rs::{ColorMode, DMA_CHUNK_SIZE, ResetDriver, Sh8601Driver, Ws18AmoledDriver};

use axp2101::core::Axp2101;

// Import from our library
use esp32_conways_game_of_life_rs::display::{DISPLAY_SIZE, FB_SIZE};
use esp32_conways_game_of_life_rs::drivers::{Pcf85063, ResetTouchDriver, Tca9554Driver};
use esp32_conways_game_of_life_rs::ecs::resources::*;
use esp32_conways_game_of_life_rs::ecs::systems::*;
use esp32_conways_game_of_life_rs::ui::{voltage_to_battery_percent, GifResource};

// Type aliases for simplifier
static I2C_CELL: StaticCell<AtomicCell<RefCellDevice<'static, I2c<'static, Blocking>>>> =
    StaticCell::new();

const IMAGE_DATA: &[u8] = include_bytes!("./background.bmp");

static I2C_BUS: StaticCell<RefCell<I2c<'static, Blocking>>> = StaticCell::new();

#[main]
fn main() -> ! {
    esp_println::println!("[MAIN] Starting main function");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    esp_println::println!("[MAIN] Config created");

    let peripherals = esp_hal::init(config);
    esp_println::println!("[MAIN] Peripherals initialized");

    esp_println::println!("[MAIN] Starting up...");
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);
    esp_println::println!("[MAIN] PSRAM allocator initialized");

    init_logger_from_env();
    esp_println::println!("[MAIN] Logger initialized");

    let delay = Delay::new();

    info!("Initializing display...");

    // --- DMA Buffers for SPI ---
    #[allow(clippy::manual_div_ceil)]
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(DMA_CHUNK_SIZE);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // SPI Configuration for Waveshare ESP32-S3 1.8inch AMOLED Touch Display
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

    // I2C Configuration for Waveshare ESP32-S3 1.8inch AMOLED Touch Display
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

    // Activate Gesture Mode to detect gestures
    touch
        .set_gesture_mode(true)
        .expect("Failed to set gesture mode");

    // Initialize RTC (PCF85063)
    esp_println::println!("Initializing PCF85063 RTC...");
    let i2c_rtc = i2c::AtomicDevice::new(i2c_cell);
    let mut rtc = Pcf85063::new(i2c_rtc);

    // Read current time from RTC
    match rtc.get_datetime() {
        Ok(dt) => esp_println::println!("RTC initialized. Current time: {:?}", dt),
        Err(_) => esp_println::println!("Warning: Could not read RTC time"),
    }

    // Instantiate and Initialize Display
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

    let bmp = Bmp::from_slice(IMAGE_DATA).expect("Failed to parse BMP image");

    // Initialize AXP2101 PMIC for battery monitoring
    // I2C address: 0x34
    esp_println::println!("Initializing AXP2101 PMIC for battery monitoring...");
    let i2c_pmic = i2c::AtomicDevice::new(i2c_cell);
    let mut pmic = Axp2101::new(i2c_pmic);

    // Read initial battery voltage from AXP2101
    let battery_voltage_mv = pmic.battery_voltage().unwrap_or(0);
    let battery_percent = voltage_to_battery_percent(battery_voltage_mv);

    esp_println::println!(
        "Battery (AXP2101): {}mV ({}%)",
        battery_voltage_mv, battery_percent
    );

    // Initialize RNG
    let rng = Rng::new(peripherals.RNG);

    // Initialize game resources
    let game = GameOfLifeResource::default();

    // Create framebuffer resource
    let fb_res = FrameBufferResource::new();

    // Initialize Bevy ECS World
    let mut world = World::default();
    world.insert_resource(game);
    world.insert_resource(RngResource(rng));
    world.insert_resource(fb_res);
    world.insert_resource(ImageResource { bmp });
    world.insert_resource(GifResource::default());

    // Insert battery resource
    world.insert_resource(BatteryResource {
        voltage_mv: battery_voltage_mv,
        percent: battery_percent,
        last_update_generation: 0,
    });

    // Insert display as NonSend resource
    world.insert_non_send_resource(DisplayResource { display });
    world.insert_non_send_resource(TouchResource { touch });

    // Insert AXP2101 PMIC resource
    world.insert_non_send_resource(Axp2101Resource { pmic });

    // Initialize buttons (GPIO0 - Boot button, EXIO4 - PWR button via TCA9554PWR)
    let boot_button = peripherals.GPIO0;
    let boot_config = InputConfig::default().with_pull(Pull::Up);
    let boot_button = Input::new(boot_button, boot_config);

    // Initialize TCA9554PWR GPIO expander for EXIO4 (PWR button)
    esp_println::println!("Initializing TCA9554PWR GPIO expander for PWR button...");
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

    world.insert_non_send_resource(ButtonResource {
        boot_button,
        gpio_expander,
        boot_last_state: false,
        pwr_last_state: false,
        boot_debounce_counter: 0,
        pwr_debounce_counter: 0,
    });

    // Get initial cycle count and CPU frequency
    let initial_cycles = esp_hal::xtensa_lx::timer::get_cycle_count();
    let cpu_freq_mhz = 240; // ESP32-S3 at max frequency

    world.insert_non_send_resource(RtcResource {
        rtc,
        last_timestamp: None,
        last_cycles: initial_cycles,
        cpu_freq_mhz,
    });

    // Create schedule and add systems
    let mut schedule = Schedule::default();
    schedule.add_systems(button_system);
    schedule.add_systems(render_system);

    info!("Entering Bevy ECS main loop...");

    // Variables for timing statistics
    let mut total_cycles: u64 = 0;
    let mut frame_count: u64 = 0;
    let mut max_cycles: u32 = 0;
    let mut min_cycles: u32 = u32::MAX;

    // Get CPU frequency for time calculations
    let cpu_freq_mhz = 240; // ESP32-S3 running at 240 MHz

    loop {
        // Measure CPU cycles before schedule.run()
        let start = esp_hal::xtensa_lx::timer::get_cycle_count();
        schedule.run(&mut world);
        let end = esp_hal::xtensa_lx::timer::get_cycle_count();

        // Calculate elapsed cycles (handle wraparound)
        let elapsed_cycles = end.wrapping_sub(start);

        // Update statistics
        total_cycles += elapsed_cycles as u64;
        frame_count += 1;
        if elapsed_cycles > max_cycles {
            max_cycles = elapsed_cycles;
        }
        if elapsed_cycles < min_cycles {
            min_cycles = elapsed_cycles;
        }

        // Print timing every 100 frames
        if frame_count % 100 == 0 {
            let avg_cycles = total_cycles / frame_count;
            let avg_time_us = avg_cycles / cpu_freq_mhz;
            let fps = 1_000_000 / avg_time_us;
            let last_time_us = elapsed_cycles as u64 / cpu_freq_mhz;
            let min_time_us = min_cycles as u64 / cpu_freq_mhz;
            let max_time_us = max_cycles as u64 / cpu_freq_mhz;

            // Access game resource through the world
            if let Some(mut game) = world.get_resource_mut::<GameOfLifeResource>() {
                game.fps = fps as usize;
            }
            esp_println::println!(
                "Frame {}: Avg={}us ({}fps), Min={}us, Max={}us, Last={}us",
                frame_count, avg_time_us, fps, min_time_us, max_time_us, last_time_us
            );
        }
    }
}
