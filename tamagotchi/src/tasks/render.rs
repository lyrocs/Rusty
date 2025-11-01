use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use alloc::sync::Arc;
use bevy_ecs::prelude::*;
use bevy_ecs::system::{IntoSystem, RunSystemOnce};
use log::{info, warn};

use super::channels::{RENDER_CHANNEL, RenderCommand};
use crate::ecs::resources::DisplayResource;
use crate::systems::render::tamagotchi_render_system;

/// Render task - handles display updates
/// Responds to render commands and executes the render system
#[embassy_executor::task]
pub async fn render_task(world: Arc<Mutex<CriticalSectionRawMutex, World>>) {
    info!("[RENDER] Render task started");

    // Statistics tracking
    let mut frame_count: u32 = 0;
    let mut total_render_time_us: u64 = 0;

    loop {
        // Wait for render commands
        match RENDER_CHANNEL.receive().await {
            RenderCommand::Redraw => {
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

                frame_count = frame_count.wrapping_add(1);
                total_render_time_us = total_render_time_us.wrapping_add(render_us);

                // Warn if rendering takes too long (>16ms for 60 FPS)
                if render_us > 16000 {
                    warn!(
                        "[RENDER] Slow frame: {}ms (target: 16ms for 60 FPS)",
                        render_us / 1000
                    );
                }

                // Log stats every 60 frames (~1 second at 60 FPS)
                if frame_count % 60 == 0 {
                    let avg_us = total_render_time_us / 60;
                    info!(
                        "[RENDER] Frame #{}: avg render time {}ms",
                        frame_count,
                        avg_us / 1000
                    );
                    total_render_time_us = 0;
                }

                // Yield to other tasks after rendering
                // The display update is blocking SPI, so we yield to be cooperative
                Timer::after(Duration::from_micros(100)).await;
            }
            RenderCommand::Clear => {
                info!("[RENDER] Clear display requested");

                // Lock world to access display
                let mut world_guard = world.lock().await;

                if let Some(mut display_res) =
                    world_guard.get_non_send_resource_mut::<DisplayResource>()
                {
                    // Clear the display using embedded-graphics DrawTarget trait
                    use embedded_graphics::prelude::*;
                    use embedded_graphics::pixelcolor::Rgb888;

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

                // Yield
                Timer::after(Duration::from_micros(100)).await;
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
