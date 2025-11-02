use alloc::sync::Arc;
use bevy_ecs::prelude::*;
use bevy_ecs::system::{IntoSystem, RunSystemOnce};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Instant, Timer};
use log::{info, warn};
use core::sync::atomic::{AtomicBool, Ordering};

use super::channels::{RENDER_CHANNEL, RenderCommand};
use crate::ecs::resources::DisplayResource;
use crate::systems::render::tamagotchi_render_system;

/// Global flag to prevent render queue buildup
/// When true, a render is currently in progress
pub static IS_RENDERING: AtomicBool = AtomicBool::new(false);

/// Render task - handles display updates
/// Responds to render commands and executes the render system
#[embassy_executor::task]
pub async fn render_task(world: Arc<Mutex<CriticalSectionRawMutex, World>>) {
    info!("[RENDER] Render task started");

    // Statistics tracking
    let mut frame_count: u32 = 0;
    let mut total_render_time_us: u64 = 0;
    let mut queue_saturation_count: u32 = 0;

    loop {
        // Wait for render commands
        match RENDER_CHANNEL.receive().await {
            RenderCommand::Redraw => {
                // Set rendering flag to prevent queue buildup
                IS_RENDERING.store(true, Ordering::Release);

                let start = Instant::now();

                // Lock the world to run the render system
                {
                    let mut world_guard = world.lock().await;

                    // Run the existing tamagotchi render system
                    // This will check needs_redraw and render if necessary
                    let _ = world_guard.run_system_once(tamagotchi_render_system);
                }

                // Track rendering performance
                let render_time = start.elapsed();
                let render_us = render_time.as_micros();

                // Clear rendering flag
                IS_RENDERING.store(false, Ordering::Release);

                frame_count = frame_count.wrapping_add(1);
                total_render_time_us = total_render_time_us.wrapping_add(render_us);

                // Check if queue has backlog (indicates saturation)
                let mut backlog_count = 0;
                while RENDER_CHANNEL.try_receive().is_ok() {
                    backlog_count += 1;
                    // Consume and count excess commands (we already rendered, so discard extras)
                }
                if backlog_count > 0 {
                    queue_saturation_count += 1;
                    warn!(
                        "[RENDER] Queue saturation detected! Discarded {} commands. Total saturations: {}",
                        backlog_count, queue_saturation_count
                    );
                }

                // Warn if rendering takes too long (>16ms for 60 FPS)
                if render_us > 100_000 {
                    warn!(
                        "[RENDER] Slow frame: {}ms (target: 100ms for fixing animation gif)",
                        render_us / 1000
                    );
                }

                // Log stats every 60 frames (~1 second at 60 FPS)
                if frame_count % 60 == 0 {
                    let avg_us = total_render_time_us / 60;
                    info!(
                        "[RENDER] Frame #{}: avg render time {}ms, queue saturations: {}",
                        frame_count,
                        avg_us / 1000,
                        queue_saturation_count
                    );
                    total_render_time_us = 0;
                    queue_saturation_count = 0; // Reset counter every second
                }

                // Yield to other tasks after rendering
                // The display update is blocking SPI (12-15ms), so we yield longer to be cooperative
                Timer::after(Duration::from_millis(1)).await;
            }
            RenderCommand::Clear => {
                info!("[RENDER] Clear display requested");

                // Lock world to access display
                let mut world_guard = world.lock().await;

                if let Some(mut display_res) =
                    world_guard.get_non_send_resource_mut::<DisplayResource>()
                {
                    // Clear the display using embedded-graphics DrawTarget trait
                    use embedded_graphics::pixelcolor::Rgb888;
                    use embedded_graphics::prelude::*;

                    if let Err(e) = display_res.display.clear(Rgb888::BLACK) {
                        warn!("[RENDER] Failed to clear display: {:?}", e);
                    } else {
                        // Flush to actually send to display
                        if let Err(e) = display_res.display.flush() {
                            warn!("[RENDER] Failed to flush display: {:?}", e);
                        }
                    }
                }

                // Release lock
                drop(world_guard);

                // Yield (longer since flush is blocking SPI operation)
                Timer::after(Duration::from_millis(1)).await;
            }
            RenderCommand::SetBrightness(level) => {
                info!("[RENDER] Set brightness: {}", level);

                // Lock world to access display
                let mut world_guard = world.lock().await;

                if let Some(mut display_res) =
                    world_guard.get_non_send_resource_mut::<DisplayResource>()
                {
                    // Note: SH8601 driver may not support brightness control
                    // This is a placeholder for future implementation
                    warn!("[RENDER] Brightness control not implemented in SH8601 driver");
                }

                // Release lock
                drop(world_guard);
            }
        }
    }
}
