// ESP32-S3 Tamagotchi - STD version with multithreading
//
// Phase 1: Proof of concept demonstrating:
// - ESP-IDF std environment
// - Bevy ECS with std features
// - Multithreading on dual cores
// - Thread-safe hardware access

// Add ESP-IDF app descriptor
// esp_idf_svc::sys::esp_app_desc!();

// Note: link_patches and EspLogger initialization are called in main()

mod drivers;
mod hal;
mod systems;
mod threads;
mod types;

use anyhow::Result;
use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use crossbeam_channel::bounded;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::drivers::{display::create_shared_display, touch::create_shared_touch};
use crate::systems::{
    game::{GameState, game_update_system},
    input::{InputEventReceiver, process_input_system},
    render::{RenderCommandSender, send_render_commands_system},
};
use crate::threads::{input::spawn_input_thread, render::spawn_render_thread};

fn main() -> Result<()> {
    // Initialize ESP-IDF - link_patches is called by esp-idf-svc
    esp_idf_svc::sys::link_patches();

    // Initialize ESP-IDF logging
    esp_idf_svc::log::EspLogger::initialize_default();

    // Feed the watchdog to prevent timeout during initialization
    unsafe {
        esp_idf_svc::sys::esp_task_wdt_reset();
    }

    log::info!("=== ESP32-S3 Tamagotchi STD Version ===");
    log::info!("Phase 1: Proof of Concept");

    // Create shared hardware resources
    log::info!("Initializing hardware drivers...");
    let display = create_shared_display()?;
    let touch = create_shared_touch()?;

    // Create inter-thread communication channels
    let (input_tx, input_rx) = bounded(100);
    let (render_tx, render_rx) = bounded(100);

    // Create running flag for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));

    // Spawn worker threads
    log::info!("Spawning worker threads...");
    let input_handle = spawn_input_thread(touch.clone(), input_tx, running.clone());

    let render_handle = spawn_render_thread(display.clone(), render_rx, running.clone());

    log::info!("Worker threads spawned successfully");

    // Setup Bevy ECS
    log::info!("Initializing Bevy ECS...");
    let mut app = App::new();

    // Add resources
    app.insert_resource(GameState::default());
    app.insert_resource(InputEventReceiver(input_rx));
    app.insert_resource(RenderCommandSender(render_tx));

    // Add systems
    app.add_systems(
        Update,
        (
            process_input_system,
            game_update_system,
            send_render_commands_system,
        )
            .chain(),
    );

    log::info!("Bevy ECS initialized");
    log::info!("Starting main game loop...");

    // Run the game loop for a limited time (proof of concept)
    let loop_duration = Duration::from_secs(60); // Extended to 60 seconds for testing
    let start_time = std::time::Instant::now();

    let mut frame_count = 0u64;
    while start_time.elapsed() < loop_duration {
        // Update ECS systems
        app.update();

        frame_count += 1;
        if frame_count % 20 == 0 {
            log::info!("Main loop frame: {}", frame_count);
            // Feed watchdog periodically
            unsafe {
                esp_idf_svc::sys::esp_task_wdt_reset();
            }
        }

        // Longer sleep to ensure IDLE task can run
        // This is temporary until we properly configure thread affinity
        thread::sleep(Duration::from_millis(50));
    }

    // Graceful shutdown
    log::info!("Shutting down...");
    running.store(false, Ordering::Relaxed);

    input_handle.join().ok();
    render_handle.join().ok();

    log::info!("Shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_structure() {
        // Basic smoke test to ensure modules compile
        assert!(true);
    }
}
