//! GIF Animation Player for AMOLED Display
//!
//! This module provides GIF decoding and playback functionality for the SH8601 display.
//! It supports animated GIFs with proper frame timing and color palette handling.
//!
//! # Features
//! - GIF decoding with automatic frame extraction
//! - Palette to RGB888 conversion
//! - Frame timing and animation loop support
//! - Centered rendering on display
//!
//! # Example
//! ```no_run
//! use display::GifPlayer;
//!
//! let gif_data = include_bytes!("../../assets/80.gif");
//! let mut player = GifPlayer::new(gif_data)?;
//!
//! // Play one frame centered
//! player.render_frame(&mut display, 0, None)?;
//! display.flush()?;
//!
//! // Animate at fixed position (x=50, y=100)
//! loop {
//!     let delay = player.next_frame(&mut display, Some((50, 100)))?;
//!     display.flush()?;
//!     thread::sleep(delay);
//! }
//! ```

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;
use gif::{ColorOutput, DisposalMethod};
use std::error::Error;
use std::io::Cursor;

use super::Sh8601Driver;

/// Frame information for GIF animation
#[derive(Debug)]
pub struct GifFrame {
    /// Frame pixel data in RGB888 format
    pub pixels: Vec<u8>,
    /// Frame width
    pub width: u16,
    /// Frame height
    pub height: u16,
    /// Frame delay in milliseconds
    pub delay_ms: u16,
    /// X offset from top-left corner
    pub left: u16,
    /// Y offset from top-left corner
    pub top: u16,
    /// Disposal method for this frame
    pub disposal: DisposalMethod,
}

/// GIF animation player
pub struct GifPlayer {
    frames: Vec<GifFrame>,
    gif_width: u16,
    gif_height: u16,
}

impl GifPlayer {
    /// Create a new GIF player from GIF data
    ///
    /// # Arguments
    /// * `gif_data` - Raw GIF file bytes
    pub fn new(gif_data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(ColorOutput::RGBA);

        let mut decoder = options.read_info(Cursor::new(gif_data))?;
        let gif_width = decoder.width();
        let gif_height = decoder.height();

        // Get the proper buffer size needed by the decoder
        let buffer_size = decoder.buffer_size();

        // Calculate canvas size with proper usize casting to prevent overflow
        let canvas_size = (gif_width as usize) * (gif_height as usize) * 4;
        log::info!(
            "Loading GIF: {}x{}, decoder buffer: {} bytes, canvas: {} bytes",
            gif_width, gif_height, buffer_size, canvas_size
        );

        // Pre-allocate a full-size canvas buffer for frame composition
        let mut canvas = vec![0u8; canvas_size];

        let mut frames = Vec::new();

        // Decode all frames
        while let Some(frame) = decoder.read_next_frame()? {
            let delay_ms = frame.delay as u16 * 10; // GIF delay is in 1/100ths of a second
            let left = frame.left;
            let top = frame.top;
            let width = frame.width;
            let height = frame.height;
            let disposal = frame.dispose;
            let interlaced = frame.interlaced;

            if interlaced {
                log::warn!("Frame {} is interlaced! This may cause display issues.", frames.len() + 1);
            }

            // Handle disposal method before compositing new frame
            match disposal {
                DisposalMethod::Background => {
                    // Clear canvas to transparent
                    canvas.fill(0);
                }
                DisposalMethod::Previous => {
                    // Keep previous canvas (do nothing, we'll composite on top)
                }
                _ => {
                    // Any/None - keep previous canvas
                }
            }

            // Composite this frame onto the canvas at (left, top)
            let frame_buffer_size = frame.buffer.len();
            let expected_frame_size = (width as usize) * (height as usize) * 4;

            log::info!(
                "Frame {}: {}x{} at ({},{}) | interlaced:{} | buffer:{} bytes (expected:{})",
                frames.len() + 1, width, height, left, top, interlaced, frame_buffer_size, expected_frame_size
            );

            if frame_buffer_size == 0 {
                log::error!("Frame {} has empty buffer! Skipping...", frames.len() + 1);
                continue;
            }

            if frame_buffer_size != expected_frame_size {
                log::warn!(
                    "Frame {} buffer size mismatch: got {} bytes, expected {} bytes",
                    frames.len() + 1, frame_buffer_size, expected_frame_size
                );
            }

            // Copy frame pixels onto canvas at the correct position
            for y in 0..height {
                for x in 0..width {
                    let frame_idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
                    if frame_idx + 3 < frame.buffer.len() {
                        let canvas_x = left + x;
                        let canvas_y = top + y;
                        if canvas_x < gif_width && canvas_y < gif_height {
                            let canvas_idx = ((canvas_y as usize) * (gif_width as usize) + (canvas_x as usize)) * 4;
                            canvas[canvas_idx] = frame.buffer[frame_idx];         // R
                            canvas[canvas_idx + 1] = frame.buffer[frame_idx + 1]; // G
                            canvas[canvas_idx + 2] = frame.buffer[frame_idx + 2]; // B
                            canvas[canvas_idx + 3] = frame.buffer[frame_idx + 3]; // A
                        }
                    }
                }
            }

            // Store the full canvas for this frame (using gif_width x gif_height)
            frames.push(GifFrame {
                pixels: canvas.clone(),
                width: gif_width,
                height: gif_height,
                delay_ms: if delay_ms > 0 { delay_ms } else { 100 }, // Default 100ms
                left: 0,    // Frame is now the full canvas, no offset
                top: 0,     // Frame is now the full canvas, no offset
                disposal,
            });
        }

        log::info!("Loaded {} frames", frames.len());

        if frames.is_empty() {
            return Err("GIF contains no frames".into());
        }

        Ok(Self {
            frames,
            gif_width,
            gif_height,
        })
    }

    /// Get the total number of frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get the GIF dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.gif_width, self.gif_height)
    }

    /// Render a specific frame to the display
    ///
    /// # Arguments
    /// * `display` - Display driver instance
    /// * `frame_index` - Frame index to render
    /// * `position` - Optional (x, y) position for top-left corner. If None, centers the GIF on screen.
    pub fn render_frame(&self, display: &mut Sh8601Driver, frame_index: usize, position: Option<(i32, i32)>) -> Result<(), Box<dyn Error>> {
        if frame_index >= self.frames.len() {
            return Err(format!("Frame index {} out of bounds (max {})", frame_index, self.frames.len()).into());
        }

        let frame = &self.frames[frame_index];
        let display_size = display.size();

        // Calculate base position for the overall GIF canvas
        let (base_x, base_y) = if let Some((x, y)) = position {
            // Use explicit position as the GIF canvas origin
            (x, y)
        } else {
            // Calculate centered position for the overall GIF canvas
            let center_x = (display_size.width as i32 - self.gif_width as i32) / 2;
            let center_y = (display_size.height as i32 - self.gif_height as i32) / 2;
            (center_x, center_y)
        };

        // Calculate frame position within the GIF canvas
        // frame.left and frame.top are the frame's offset within the GIF
        let frame_offset_x = base_x + frame.left as i32;
        let frame_offset_y = base_y + frame.top as i32;

        // Draw each pixel of the frame
        for y in 0..frame.height {
            for x in 0..frame.width {
                let pixel_idx = ((y * frame.width + x) * 4) as usize;

                if pixel_idx + 3 < frame.pixels.len() {
                    let r = frame.pixels[pixel_idx];
                    let g = frame.pixels[pixel_idx + 1];
                    let b = frame.pixels[pixel_idx + 2];
                    let a = frame.pixels[pixel_idx + 3];

                    // Skip transparent pixels (alpha < 128)
                    if a < 128 {
                        continue;
                    }

                    let px = frame_offset_x + x as i32;
                    let py = frame_offset_y + y as i32;

                    if px >= 0 && px < display_size.width as i32 &&
                       py >= 0 && py < display_size.height as i32 {
                        let point = Point::new(px, py);
                        display.draw_iter(core::iter::once(Pixel(point, Rgb888::new(r, g, b))))?;
                    }
                }
            }
        }

        Ok(())
    }

}
