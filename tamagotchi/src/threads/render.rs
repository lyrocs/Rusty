// Render thread implementation
//
// This thread runs on Core 0 and processes render commands to update the display

use crate::drivers::display::FrameBuffer;
use crate::drivers::display_hal::RawQspiDriver;
use crate::types::{RenderCommand, Color};
use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

pub type SharedDisplay = Arc<Mutex<RawQspiDriver<'static>>>;

/// Spawn the render thread
pub fn spawn_render_thread(
    display: SharedDisplay,
    rx: Receiver<RenderCommand>,
    running: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("render".to_string())
        .stack_size(8192)
        .spawn(move || {
            log::info!("Render thread started");

            // Create double buffers (368x448 for Waveshare 1.8" AMOLED)
            let mut front_buffer = FrameBuffer::new(368, 448);
            let mut back_buffer = FrameBuffer::new(368, 448);

            // Simple test: clear to solid red color
            log::info!("Testing display with solid red screen...");

            // Fill buffer with red (255, 0, 0)
            for pixel in back_buffer.data.chunks_exact_mut(3) {
                pixel[0] = 255; // R
                pixel[1] = 0;   // G
                pixel[2] = 0;   // B
            }

            // Display the red screen using QSPI
            if let Some(mut display_driver) = display.try_lock() {
                log::info!("Sending red buffer to display using QSPI...");
                log::info!("Buffer size: {} bytes for {}x{}", back_buffer.data.len(), back_buffer.width, back_buffer.height);

                match display_driver.draw_buffer(
                    &back_buffer.data,
                    0,
                    0,
                    back_buffer.width,
                    back_buffer.height
                ) {
                    Ok(_) => log::info!("🔴 RED SCREEN sent successfully via QSPI - display should be RED now!"),
                    Err(e) => log::error!("Failed to send red screen: {:?}", e),
                }
            } else {
                log::error!("Could not lock display driver!");
            }

            let mut frame_count = 0u64;

            while running.load(Ordering::Relaxed) {
                // Process all pending render commands
                let mut should_present = false;

                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        RenderCommand::Clear => {
                            back_buffer.clear();
                        }
                        RenderCommand::DrawSprite { sprite_id, x, y, frame } => {
                            log::trace!("Drawing sprite {} at ({}, {}) frame {}", sprite_id, x, y, frame);
                            // TODO: Implement sprite rendering from GIF cache
                        }
                        RenderCommand::DrawRect { x, y, width, height, color } => {
                            back_buffer.draw_rect(x, y, width, height, color.r, color.g, color.b);
                        }
                        RenderCommand::DrawText { text, x, y, color } => {
                            log::trace!("Drawing text '{}' at ({}, {})", text, x, y);
                            // TODO: Implement text rendering
                        }
                        RenderCommand::Present => {
                            should_present = true;
                        }
                    }
                }

                // Present the frame if requested
                if should_present {
                    frame_count += 1;

                    // Swap buffers
                    std::mem::swap(&mut front_buffer, &mut back_buffer);

                    // Send to display via QSPI
                    if let Some(mut display_driver) = display.try_lock() {
                        display_driver.draw_buffer(
                            &front_buffer.data,
                            0,
                            0,
                            front_buffer.width,
                            front_buffer.height
                        ).ok();
                    }

                    if frame_count % 60 == 0 {
                        log::info!("Rendered {} frames", frame_count);
                    }
                }

                // Sleep longer to avoid starving the watchdog
                // The ESP32 IDLE task needs time to run
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            log::info!("Render thread stopped");
        })
        .expect("Failed to spawn render thread")
}
