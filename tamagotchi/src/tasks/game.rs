use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Ticker};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use alloc::sync::Arc;
use log::{info, warn};

use super::channels::{INPUT_CHANNEL, RENDER_CHANNEL, SAVE_CHANNEL, InputEvent, RenderCommand, SaveCommand};
use crate::ecs::resources::*;
use crate::core::GameState;
use crate::systems::{tamagotchi_button_system, tamagotchi_touch_system, tamagotchi_update_system};

/// Main game loop task - runs at 60 FPS
#[embassy_executor::task]
pub async fn game_loop_task(world: Arc<Mutex<CriticalSectionRawMutex, World>>) {
    info!("[GAME] Game loop task started");

    // 60 FPS ticker (~16.67ms per frame)
    let mut ticker = Ticker::every(Duration::from_millis(16));

    let mut frame_count: u32 = 0;
    let mut needs_render = true;
    let mut last_render_time = Instant::now();
    const RENDER_INTERVAL_MS: u64 = 100; // Throttle rendering to max 10 FPS

    loop {
        ticker.next().await;
        frame_count = frame_count.wrapping_add(1);

        // Process any pending input events (non-blocking)
        while let Ok(event) = INPUT_CHANNEL.try_receive() {
            let mut world_guard = world.lock().await;

            match event {
                InputEvent::Touch { x, y } => {
                    // Update touch resource
                    if let Some(mut touch) = world_guard.get_resource_mut::<TouchInput>() {
                        touch.x = x;
                        touch.y = y;
                        touch.is_active = true;
                    }
                    needs_render = true;
                }
                InputEvent::TouchRelease => {
                    if let Some(mut touch) = world_guard.get_resource_mut::<TouchInput>() {
                        touch.is_active = false;
                    }
                    needs_render = true;
                }
                InputEvent::ButtonPress(btn) => {
                    // Handle button press
                    info!("[GAME] Button press: {:?}", btn);
                    needs_render = true;
                }
                InputEvent::ButtonRelease(btn) => {
                    info!("[GAME] Button release: {:?}", btn);
                }
                InputEvent::Gesture(gesture) => {
                    info!("[GAME] Gesture: {:?}", gesture);
                    needs_render = true;
                }
            }
        }

        // Run game systems
        {
            let mut world_guard = world.lock().await;

            // Run button system to process button input (BOOT/PWR buttons)
            let _ = world_guard.run_system_once(tamagotchi_button_system);

            // Run touch system to process touch input
            let _ = world_guard.run_system_once(tamagotchi_touch_system);

            // Run update system for game logic
            let _ = world_guard.run_system_once(tamagotchi_update_system);

            // Check if we need to save, redraw, or shutdown
            let (save_requested, redraw_requested, shutdown_requested) = {
                if let Some(game_state) = world_guard.get_resource::<GameState>() {
                    (game_state.save_requested, game_state.needs_redraw, game_state.shutdown_requested)
                } else {
                    (false, false, false)
                }
            };

            if save_requested {
                let _ = SAVE_CHANNEL.try_send(SaveCommand::SaveGame);
                if let Some(mut state) = world_guard.get_resource_mut::<GameState>() {
                    state.save_requested = false;
                }
            }

            if redraw_requested {
                needs_render = true;
                // Don't clear needs_redraw here - render system will clear it after rendering
            }

            // Handle shutdown request
            if shutdown_requested {
                info!("[SHUTDOWN] Shutdown sequence initiated");

                // Clear the flag to prevent multiple triggers
                if let Some(mut state) = world_guard.get_resource_mut::<GameState>() {
                    state.shutdown_requested = false;
                }

                // Request a save first
                let _ = SAVE_CHANNEL.try_send(SaveCommand::SaveGame);
                info!("[SHUTDOWN] Saving game data...");

                // Release the lock before async operations
                drop(world_guard);

                // Wait for save to complete (async, doesn't block other tasks)
                embassy_time::Timer::after(Duration::from_millis(500)).await;

                // Lock again to access PMIC
                let mut world_guard = world.lock().await;

                info!("[SHUTDOWN] Powering down system...");
                info!("[SHUTDOWN] Using AXP2101 hardware shutdown");
                info!("[SHUTDOWN] RTC will remain powered by battery");
                info!("[SHUTDOWN] Press PWR button to power on again");

                // Perform hardware shutdown via AXP2101 PMIC
                if let Some(mut axp_res) = world_guard.get_non_send_resource_mut::<Axp2101Resource>() {
                    match axp_res.pmic.shutdown() {
                        Ok(_) => {
                            info!("[SHUTDOWN] Good night!");
                            // Give time for the message to be printed
                            embassy_time::Timer::after(Duration::from_millis(100)).await;
                            // Device will shut down here
                            loop {
                                embassy_time::Timer::after(Duration::from_secs(1)).await;
                            }
                        }
                        Err(_) => {
                            info!("[SHUTDOWN] Failed to shutdown");
                            if let Some(mut game_state) = world_guard.get_resource_mut::<GameState>() {
                                game_state.save_status_msg = Some("Shutdown failed!");
                                game_state.needs_redraw = true;
                            }
                        }
                    }
                }
            }
        }

        // Request render if needed, but throttle to RENDER_INTERVAL_MS (prevents queue saturation)
        if needs_render {
            let time_since_last_render = last_render_time.elapsed().as_millis() as u64;

            if time_since_last_render >= RENDER_INTERVAL_MS {
                match RENDER_CHANNEL.try_send(RenderCommand::Redraw) {
                    Ok(_) => {
                        needs_render = false;
                        last_render_time = Instant::now();
                    }
                    Err(_) => {
                        // Queue is full - this is critical as it causes screen freeze
                        warn!("[GAME] Render queue full! Screen may freeze. Frame: {}", frame_count);
                        // Keep needs_render=true to retry on next frame
                    }
                }
            }
            // else: too soon since last render, skip this frame (keeps needs_render=true for next check)
        }

        // Periodic logging (every 60 frames = ~1 second)
        if frame_count % 60 == 0 {
            // info!("[GAME] Frame: {}", frame_count);
        }
    }
}

/// Helper resource for tracking touch input
#[derive(Default, Resource)]
pub struct TouchInput {
    pub x: u16,
    pub y: u16,
    pub is_active: bool,
}
