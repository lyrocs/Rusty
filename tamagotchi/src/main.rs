#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

extern crate alloc;

use esp_bootloader_esp_idf::esp_app_desc;
esp_app_desc!();

use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use alloc::sync::Arc;
use esp_hal::clock::{Clock, CpuClock};
use esp_println::logger::init_logger_from_env;
use log::info;

// Import from our library
use esp32_conways_game_of_life_rs::hardware::init_hardware;
use esp32_conways_game_of_life_rs::tasks::{
    game::game_loop_task,
    input::input_task,
    render::render_task,
    storage::storage_task,
};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("[TAMAGOTCHI] Starting Ragnarok Tamagotchi with Embassy...");

    // Configure CPU clock and get peripherals
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize logger
    init_logger_from_env();

    // Initialize all hardware peripherals first
    let hw = init_hardware(peripherals);

    // Initialize Embassy time driver using TIMG0 from hardware init
    esp_hal_embassy::init(hw.timg0.timer0);

    info!("[EMBASSY] Embassy executor initialized");
    info!("[EMBASSY] CPU running at {} MHz", CpuClock::max().mhz());

    // Wrap the ECS World in Arc<Mutex> for safe concurrent access
    let world = Arc::new(Mutex::new(hw.world));

    info!("[EMBASSY] Spawning async tasks...");

    // Spawn all async tasks
    spawner.spawn(input_task(world.clone())).unwrap();
    info!("[EMBASSY] ✓ Input task spawned (100Hz polling)");

    spawner.spawn(game_loop_task(world.clone())).unwrap();
    info!("[EMBASSY] ✓ Game loop task spawned (60 FPS)");

    spawner.spawn(render_task(world.clone())).unwrap();
    info!("[EMBASSY] ✓ Render task spawned");

    spawner.spawn(storage_task(world.clone())).unwrap();
    info!("[EMBASSY] ✓ Storage task spawned");

    info!("[EMBASSY] All tasks spawned successfully!");
    info!("[EMBASSY] Entering main task loop...");

    // Main task heartbeat - keeps the executor alive and provides periodic logging
    let mut heartbeat_counter: u32 = 0;
    loop {
        Timer::after(Duration::from_secs(60)).await;
        heartbeat_counter = heartbeat_counter.wrapping_add(1);
        info!(
            "[EMBASSY] System heartbeat #{} - All tasks running",
            heartbeat_counter
        );
    }
}
