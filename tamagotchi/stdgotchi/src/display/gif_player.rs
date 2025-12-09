//! GIF Animation Player for AMOLED Display
//!
//! This module provides GIF decoding and playback functionality for the SH8601 display.
//! It supports animated GIFs with proper frame timing and color palette handling.
//!
//! # Features
//! - Streaming GIF decoding with on-demand frame loading
//! - Palette to RGB888 conversion
//! - Frame timing and animation loop support
//! - Centered rendering on display
//! - Memory-efficient: only decodes frames as needed
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
use gif::{ColorOutput, DisposalMethod, Decoder};
use std::error::Error;
use std::io::Cursor;
use std::cell::RefCell;

use super::St7789pDriver;

/// Frame metadata (lightweight, stored for all frames)
#[derive(Debug, Clone)]
struct FrameMetadata {
    /// Frame delay in milliseconds
    delay_ms: u16,
    /// X offset from top-left corner
    left: u16,
    /// Y offset from top-left corner
    top: u16,
    /// Frame width
    width: u16,
    /// Frame height
    height: u16,
    /// Disposal method for this frame
    disposal: DisposalMethod,
}

/// GIF animation player with stateful streaming decoding
///
/// This uses a persistent decoder that maintains state between frames,
/// making sequential playback fast without needing to cache frames.
pub struct GifPlayer {
    /// Original GIF data (stored for decoder resets)
    gif_data: Vec<u8>,
    /// Frame metadata (lightweight)
    frame_metadata: Vec<FrameMetadata>,
    /// Overall GIF dimensions
    gif_width: u16,
    gif_height: u16,
    /// Persistent decoder for sequential playback
    decoder: RefCell<Option<Decoder<Cursor<Vec<u8>>>>>,
    /// Current decoder position (which frame it's at)
    decoder_position: RefCell<usize>,
    /// Reusable canvas buffer for frame composition
    canvas: RefCell<Vec<u8>>,
}

impl GifPlayer {
    /// Create a new GIF player from GIF data
    ///
    /// This only parses metadata, not actual frame data, for memory efficiency.
    ///
    /// # Arguments
    /// * `gif_data` - Raw GIF file bytes
    pub fn new(gif_data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(ColorOutput::RGBA);

        let mut decoder = options.read_info(Cursor::new(gif_data))?;
        let gif_width = decoder.width();
        let gif_height = decoder.height();

        // Calculate canvas size for single frame buffer
        let canvas_size = (gif_width as usize) * (gif_height as usize) * 4;
        log::info!(
            "Loading GIF (streaming): {}x{}, canvas: {} bytes",
            gif_width, gif_height, canvas_size
        );

        let mut frame_metadata = Vec::new();

        // Parse only metadata from all frames
        while let Some(frame) = decoder.read_next_frame()? {
            let delay_ms = frame.delay as u16 * 10; // GIF delay is in 1/100ths of a second

            if frame.interlaced {
                log::warn!("Frame {} is interlaced! This may cause display issues.", frame_metadata.len() + 1);
            }

            frame_metadata.push(FrameMetadata {
                delay_ms: if delay_ms > 0 { delay_ms } else { 100 },
                left: frame.left,
                top: frame.top,
                width: frame.width,
                height: frame.height,
                disposal: frame.dispose,
            });
        }

        log::info!(
            "Loaded GIF metadata: {} frames, {}x{} (stateful streaming: {}KB single buffer)",
            frame_metadata.len(),
            gif_width,
            gif_height,
            canvas_size / 1024
        );

        if frame_metadata.is_empty() {
            return Err("GIF contains no frames".into());
        }

        Ok(Self {
            gif_data: gif_data.to_vec(),
            frame_metadata,
            gif_width,
            gif_height,
            decoder: RefCell::new(None),  // Created on first use
            decoder_position: RefCell::new(0),
            canvas: RefCell::new(vec![0u8; canvas_size]),
        })
    }

    /// Decode a specific frame using stateful decoder
    ///
    /// For sequential playback (frame N -> N+1), this just reads the next frame.
    /// For non-sequential access, resets decoder and reads from beginning.
    fn decode_frame(&self, frame_index: usize) -> Result<(), Box<dyn Error>> {
        if frame_index >= self.frame_metadata.len() {
            return Err(format!("Frame index {} out of bounds", frame_index).into());
        }

        // Check if decoder is initialized
        if self.decoder.borrow().is_none() {
            // First call - initialize decoder
            self.reset_decoder_to_frame(frame_index)?;
            return Ok(());
        }

        let current_position = *self.decoder_position.borrow();

        // Check if we can continue sequentially
        if current_position == frame_index + 1 {
            // Already at this frame, no need to decode
            return Ok(());
        }

        if current_position == frame_index {
            // We're one frame behind, just read next
            self.decode_next_frame()?;
            return Ok(());
        }

        // Non-sequential access or need to restart
        self.reset_decoder_to_frame(frame_index)?;

        Ok(())
    }

    /// Reset decoder and decode up to target frame
    fn reset_decoder_to_frame(&self, target_frame: usize) -> Result<(), Box<dyn Error>> {
        // Create new decoder
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(ColorOutput::RGBA);
        let decoder = options.read_info(Cursor::new(self.gif_data.clone()))?;

        *self.decoder.borrow_mut() = Some(decoder);
        *self.decoder_position.borrow_mut() = 0;

        let mut canvas = self.canvas.borrow_mut();
        canvas.fill(0);

        // Decode frames up to and including target
        for _i in 0..=target_frame {
            self.decode_next_frame_internal(&mut canvas)?;
        }

        Ok(())
    }

    /// Decode the next frame from current decoder position
    fn decode_next_frame(&self) -> Result<(), Box<dyn Error>> {
        let mut canvas = self.canvas.borrow_mut();
        self.decode_next_frame_internal(&mut canvas)
    }

    /// Internal: decode next frame (canvas must already be borrowed)
    fn decode_next_frame_internal(&self, canvas: &mut [u8]) -> Result<(), Box<dyn Error>> {
        let mut decoder_opt = self.decoder.borrow_mut();
        let decoder = decoder_opt.as_mut()
            .ok_or("Decoder not initialized")?;

        let current_pos = *self.decoder_position.borrow();

        if let Some(frame) = decoder.read_next_frame()? {
            let metadata = &self.frame_metadata[current_pos];

            // Handle disposal method
            match metadata.disposal {
                DisposalMethod::Background => {
                    canvas.fill(0);
                }
                DisposalMethod::Previous => {
                    // Keep canvas as-is
                }
                _ => {
                    // Any/None - keep canvas
                }
            }

            // Composite frame onto canvas
            for y in 0..metadata.height {
                for x in 0..metadata.width {
                    let frame_idx = ((y as usize) * (metadata.width as usize) + (x as usize)) * 4;
                    if frame_idx + 3 < frame.buffer.len() {
                        let canvas_x = metadata.left + x;
                        let canvas_y = metadata.top + y;
                        if canvas_x < self.gif_width && canvas_y < self.gif_height {
                            let canvas_idx = ((canvas_y as usize) * (self.gif_width as usize) + (canvas_x as usize)) * 4;
                            canvas[canvas_idx] = frame.buffer[frame_idx];
                            canvas[canvas_idx + 1] = frame.buffer[frame_idx + 1];
                            canvas[canvas_idx + 2] = frame.buffer[frame_idx + 2];
                            canvas[canvas_idx + 3] = frame.buffer[frame_idx + 3];
                        }
                    }
                }
            }

            *self.decoder_position.borrow_mut() = current_pos + 1;
        }

        Ok(())
    }

    /// Get the total number of frames
    pub fn frame_count(&self) -> usize {
        self.frame_metadata.len()
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
    pub fn render_frame(&self, display: &mut St7789pDriver, frame_index: usize, position: Option<(i32, i32)>) -> Result<(), Box<dyn Error>> {
        self.render_frame_with_flip(display, frame_index, position, false)
    }

    /// Render a specific frame with optional horizontal flip
    ///
    /// The flip is zero-cost - just arithmetic, no memory allocation.
    pub fn render_frame_with_flip(&self, display: &mut St7789pDriver, frame_index: usize, position: Option<(i32, i32)>, flip_horizontal: bool) -> Result<(), Box<dyn Error>> {
        self.render_frame_centered(display, frame_index, position, flip_horizontal, false)
    }

    /// Render a specific frame with center-based positioning and optional horizontal flip
    ///
    /// When `center_positioned` is true, the position represents the center of the image,
    /// not the top-left corner. This ensures animations with different sizes stay centered.
    pub fn render_frame_centered(&self, display: &mut St7789pDriver, frame_index: usize, position: Option<(i32, i32)>, flip_horizontal: bool, center_positioned: bool) -> Result<(), Box<dyn Error>> {
        if frame_index >= self.frame_metadata.len() {
            return Err(format!("Frame index {} out of bounds (max {})", frame_index, self.frame_metadata.len()).into());
        }

        // Decode frame into canvas (fast for sequential access!)
        self.decode_frame(frame_index)?;

        let display_size = display.size();

        // Calculate base position for the overall GIF canvas
        let (base_x, base_y) = if let Some((x, y)) = position {
            if center_positioned {
                // Position is the CENTER of the image - calculate top-left from center
                (x - (self.gif_width as i32 / 2), y - (self.gif_height as i32 / 2))
            } else {
                // Use explicit position as the GIF canvas origin (top-left)
                (x, y)
            }
        } else {
            // Calculate centered position for the overall GIF canvas
            let center_x = (display_size.width as i32 - self.gif_width as i32) / 2;
            let center_y = (display_size.height as i32 - self.gif_height as i32) / 2;
            (center_x, center_y)
        };

        // Borrow canvas for rendering
        let canvas = self.canvas.borrow();

        // Draw each pixel of the frame
        for y in 0..self.gif_height {
            for x in 0..self.gif_width {
                let pixel_idx = ((y as usize) * (self.gif_width as usize) + (x as usize)) * 4;

                if pixel_idx + 3 < canvas.len() {
                    let r = canvas[pixel_idx];
                    let g = canvas[pixel_idx + 1];
                    let b = canvas[pixel_idx + 2];
                    let a = canvas[pixel_idx + 3];

                    // Skip transparent pixels (alpha < 128)
                    if a < 128 {
                        continue;
                    }

                    // Apply horizontal flip if requested (zero-cost: just arithmetic)
                    let px = if flip_horizontal {
                        base_x + (self.gif_width - 1 - x) as i32
                    } else {
                        base_x + x as i32
                    };
                    let py = base_y + y as i32;

                    if px >= 0 && px < display_size.width as i32 &&
                       py >= 0 && py < display_size.height as i32 {
                        let point = Point::new(px, py);
                        // Swap R and B for BGR display format
                        display.draw_iter(core::iter::once(Pixel(point, Rgb888::new(b, g, r))))?;
                    }
                }
            }
        }

        Ok(())
    }

}
