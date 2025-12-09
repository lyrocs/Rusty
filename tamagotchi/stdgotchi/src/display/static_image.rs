//! Static Image from GIF
//!
//! Loads only the first frame from a GIF for use as a static background image.

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;
use gif::ColorOutput;
use std::error::Error;
use std::io::Cursor;

use super::Sh8601Driver;

/// Static image loaded from first frame of a GIF
pub struct StaticImage {
    pixels: Vec<u8>,
    width: u16,
    height: u16,
}

impl StaticImage {
    /// Create a new static image from GIF data (uses only first frame)
    ///
    /// # Arguments
    /// * `gif_data` - Raw GIF file bytes
    pub fn new(gif_data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(ColorOutput::RGBA);

        let mut decoder = options.read_info(Cursor::new(gif_data))?;
        let gif_width = decoder.width();
        let gif_height = decoder.height();

        // Read only the first frame
        let frame = decoder.read_next_frame()?
            .ok_or("GIF contains no frames")?;

        let frame_width = frame.width;
        let frame_height = frame.height;
        let left = frame.left;
        let top = frame.top;

        // Calculate canvas size with proper usize casting
        let canvas_size = (gif_width as usize) * (gif_height as usize) * 4;
        let mut canvas = vec![0u8; canvas_size];

        // Composite the frame onto the canvas at (left, top)
        for y in 0..frame_height {
            for x in 0..frame_width {
                let frame_idx = ((y as usize) * (frame_width as usize) + (x as usize)) * 4;
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

        Ok(Self {
            pixels: canvas,
            width: gif_width,
            height: gif_height,
        })
    }

    /// Render the image to the display at a specific position
    ///
    /// # Arguments
    /// * `display` - Display driver instance
    /// * `position` - (x, y) position for the image's top-left corner
    pub fn render(&self, display: &mut Sh8601Driver, position: (i32, i32)) -> Result<(), Box<dyn Error>> {
        let display_size = display.size();
        let (base_x, base_y) = position;

        // Draw each pixel
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel_idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;

                if pixel_idx + 3 < self.pixels.len() {
                    let r = self.pixels[pixel_idx];
                    let g = self.pixels[pixel_idx + 1];
                    let b = self.pixels[pixel_idx + 2];
                    let a = self.pixels[pixel_idx + 3];

                    // Skip transparent pixels (alpha < 128)
                    if a < 128 {
                        continue;
                    }

                    let px = base_x + (x as i32);
                    let py = base_y + (y as i32);

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

    /// Render a specific region of the image to the display
    ///
    /// # Arguments
    /// * `display` - Display driver instance
    /// * `position` - (x, y) position for the image's top-left corner
    /// * `region` - (x, y, width, height) region to render in screen coordinates
    pub fn render_region(
        &self,
        display: &mut Sh8601Driver,
        position: (i32, i32),
        region: (i32, i32, u32, u32),
    ) -> Result<(), Box<dyn Error>> {
        let display_size = display.size();
        let (base_x, base_y) = position;
        let (region_x, region_y, region_width, region_height) = region;

        // Calculate the region in image coordinates
        let image_x_start = (region_x - base_x).max(0) as u16;
        let image_y_start = (region_y - base_y).max(0) as u16;
        let image_x_end = ((region_x + region_width as i32 - base_x).min(self.width as i32)) as u16;
        let image_y_end = ((region_y + region_height as i32 - base_y).min(self.height as i32)) as u16;

        // Draw pixels only in the specified region
        for y in image_y_start..image_y_end {
            for x in image_x_start..image_x_end {
                let pixel_idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;

                if pixel_idx + 3 < self.pixels.len() {
                    let r = self.pixels[pixel_idx];
                    let g = self.pixels[pixel_idx + 1];
                    let b = self.pixels[pixel_idx + 2];
                    let a = self.pixels[pixel_idx + 3];

                    // Skip transparent pixels (alpha < 128)
                    if a < 128 {
                        continue;
                    }

                    let px = base_x + (x as i32);
                    let py = base_y + (y as i32);

                    // Check if pixel is within the requested region and display bounds
                    if px >= region_x && px < region_x + region_width as i32 &&
                       py >= region_y && py < region_y + region_height as i32 &&
                       px >= 0 && px < display_size.width as i32 &&
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
