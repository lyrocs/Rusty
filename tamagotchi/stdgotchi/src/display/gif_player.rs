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
use std::time::Duration;

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
    current_frame: usize,
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

        log::info!("Loading GIF: {}x{}", gif_width, gif_height);

        let mut frames = Vec::new();

        // Decode all frames
        while let Some(frame) = decoder.read_next_frame()? {
            let delay_ms = frame.delay as u16 * 10; // GIF delay is in 1/100ths of a second
            let left = frame.left;
            let top = frame.top;
            let width = frame.width;
            let height = frame.height;
            let disposal = frame.dispose;

            // Convert RGBA to RGB888
            let mut pixels = Vec::with_capacity((width * height * 3) as usize);
            for chunk in frame.buffer.chunks(4) {
                pixels.push(chunk[0]); // R
                pixels.push(chunk[1]); // G
                pixels.push(chunk[2]); // B
                // Ignore alpha channel (chunk[3])
            }

            frames.push(GifFrame {
                pixels,
                width,
                height,
                delay_ms: if delay_ms > 0 { delay_ms } else { 100 }, // Default 100ms
                left,
                top,
                disposal,
            });

            log::debug!("Loaded frame {}: {}x{} at ({}, {}), delay={}ms",
                       frames.len(), width, height, left, top, delay_ms);
        }

        log::info!("Loaded {} frames", frames.len());

        if frames.is_empty() {
            return Err("GIF contains no frames".into());
        }

        Ok(Self {
            frames,
            current_frame: 0,
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
                let pixel_idx = ((y * frame.width + x) * 3) as usize;

                if pixel_idx + 2 < frame.pixels.len() {
                    let r = frame.pixels[pixel_idx];
                    let g = frame.pixels[pixel_idx + 1];
                    let b = frame.pixels[pixel_idx + 2];

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

    /// Advance to the next frame and render it
    ///
    /// # Arguments
    /// * `display` - Display driver instance
    /// * `position` - Optional (x, y) position for top-left corner. If None, centers the GIF on screen.
    ///
    /// Returns the delay duration for this frame
    pub fn next_frame(&mut self, display: &mut Sh8601Driver, position: Option<(i32, i32)>) -> Result<Duration, Box<dyn Error>> {
        let frame = &self.frames[self.current_frame];
        let delay = Duration::from_millis(frame.delay_ms as u64);

        // Handle disposal method
        match frame.disposal {
            DisposalMethod::Background => {
                // Clear to background color (black)
                display.clear(Rgb888::BLACK)?;
            }
            DisposalMethod::Previous => {
                // Keep previous frame (don't clear)
            }
            _ => {
                // Any/None - don't dispose
            }
        }

        self.render_frame(display, self.current_frame, position)?;

        // Advance to next frame
        self.current_frame = (self.current_frame + 1) % self.frames.len();

        Ok(delay)
    }

    /// Reset animation to first frame
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.current_frame = 0;
    }

    /// Get current frame index
    #[allow(dead_code)]
    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }
}
