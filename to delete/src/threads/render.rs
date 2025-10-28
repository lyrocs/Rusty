// Render thread implementation
//
// This thread runs on Core 0 and processes render commands to update the display

use crate::drivers::display::{SharedDisplay, FrameBuffer};
use crate::types::{RenderCommand, Color};
use crossbeam_channel::Receiver;
use std::thread::{self, JoinHandle};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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

            // Create double buffers
            let mut front_buffer = FrameBuffer::new(240, 280);
            let mut back_buffer = FrameBuffer::new(240, 280);

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
                    // Swap buffers
                    std::mem::swap(&mut front_buffer, &mut back_buffer);

                    // Send to display
                    if let Some(mut display_driver) = display.try_lock() {
                        display_driver.draw_buffer(
                            &front_buffer.data,
                            0,
                            0,
                            front_buffer.width,
                            front_buffer.height
                        ).ok();
                    }
                }

                // Yield to avoid busy-waiting
                thread::yield_now();
            }

            log::info!("Render thread stopped");
        })
        .expect("Failed to spawn render thread")
}
