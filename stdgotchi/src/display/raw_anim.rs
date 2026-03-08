//! Raw Animation Format Loader
//!
//! Streams pre-decoded RGB565 frames directly from SD card for fast animation playback.
//! Designed for memory-constrained ESP32-C6 (no PSRAM).
//!
//! File Format (.raw):
//!   Header (8 bytes):
//!     - u16: width
//!     - u16: height
//!     - u16: frame_count
//!     - u16: flags (bit 0: has_transparency)
//!
//!   Frame Table (6 bytes per frame):
//!     - u32: file offset to frame data
//!     - u16: delay in ms
//!
//!   Frame Data:
//!     - width * height * 2 bytes: RGB565 pixels (little-endian)
//!     - If has_transparency: ceil(width * height / 8) bytes: bitmask

use crate::display::St7789pDriver;

/// Header size in bytes
const HEADER_SIZE: usize = 8;
/// Frame table entry size in bytes
const FRAME_ENTRY_SIZE: usize = 6;
/// Maximum frame size to keep in memory (64x64 sprite = 8KB + mask)
const MAX_SINGLE_FRAME_SIZE: usize = 64 * 64 * 2 + 64 * 64 / 8;

/// Raw animation metadata (loaded from header)
#[derive(Debug, Clone)]
pub struct RawAnimMeta {
    pub width: u16,
    pub height: u16,
    pub frame_count: u16,
    pub has_transparency: bool,
    /// File offsets for each frame
    pub frame_offsets: Vec<u32>,
    /// Delay in ms for each frame
    pub frame_delays: Vec<u16>,
}

impl RawAnimMeta {
    /// Parse raw animation header from file data
    /// Only needs the header + frame table portion of the file
    pub fn from_header(data: &[u8]) -> Option<Self> {
        if data.len() < HEADER_SIZE {
            return None;
        }

        let width = u16::from_le_bytes([data[0], data[1]]);
        let height = u16::from_le_bytes([data[2], data[3]]);
        let frame_count = u16::from_le_bytes([data[4], data[5]]);
        let flags = u16::from_le_bytes([data[6], data[7]]);
        let has_transparency = (flags & 1) != 0;

        let table_size = frame_count as usize * FRAME_ENTRY_SIZE;
        if data.len() < HEADER_SIZE + table_size {
            return None;
        }

        let mut frame_offsets = Vec::with_capacity(frame_count as usize);
        let mut frame_delays = Vec::with_capacity(frame_count as usize);

        for i in 0..frame_count as usize {
            let entry_start = HEADER_SIZE + i * FRAME_ENTRY_SIZE;
            let offset = u32::from_le_bytes([
                data[entry_start],
                data[entry_start + 1],
                data[entry_start + 2],
                data[entry_start + 3],
            ]);
            let delay = u16::from_le_bytes([
                data[entry_start + 4],
                data[entry_start + 5],
            ]);
            frame_offsets.push(offset);
            frame_delays.push(delay);
        }

        Some(Self {
            width,
            height,
            frame_count,
            has_transparency,
            frame_offsets,
            frame_delays,
        })
    }

    /// Calculate the size of frame data (RGB565 + optional transparency mask)
    pub fn frame_data_size(&self) -> usize {
        let pixel_size = self.width as usize * self.height as usize * 2;
        if self.has_transparency {
            let mask_size = (self.width as usize * self.height as usize + 7) / 8;
            pixel_size + mask_size
        } else {
            pixel_size
        }
    }

    /// Calculate header + frame table size
    pub fn header_size(&self) -> usize {
        HEADER_SIZE + self.frame_count as usize * FRAME_ENTRY_SIZE
    }
}

/// Streaming raw animation player
/// Holds metadata and a reusable single-frame buffer
pub struct RawAnimPlayer {
    /// Animation metadata
    pub meta: RawAnimMeta,
    /// SD card file path for loading frames
    file_path: String,
    /// Single frame buffer for current frame (RGB565 + optional alpha mask)
    frame_buffer: Vec<u8>,
    /// Current frame index
    current_frame: usize,
    /// Frame timer for auto-advance
    frame_timer: f32,
    /// Whether current frame is loaded in buffer
    frame_loaded: bool,
}

impl RawAnimPlayer {
    /// Create a new streaming player from metadata and file path
    /// Only loads metadata + header, not frame data
    pub fn from_metadata(meta: RawAnimMeta, file_path: String) -> Self {
        let frame_data_size = meta.frame_data_size();

        Self {
            meta,
            file_path,
            frame_buffer: vec![0u8; frame_data_size],
            current_frame: 0,
            frame_timer: 0.0,
            frame_loaded: false,
        }
    }

    /// Create a new player from raw file data (loads full file into player)
    /// DEPRECATED: Use from_metadata for streaming. This loads entire file.
    pub fn from_file_data(data: Vec<u8>) -> Option<Self> {
        let meta = RawAnimMeta::from_header(&data)?;
        let file_path = String::from("<embedded>");
        let frame_data_size = meta.frame_data_size();

        // Store full file as frame_buffer for compatibility
        Some(Self {
            meta,
            file_path,
            frame_buffer: data,
            current_frame: 0,
            frame_timer: 0.0,
            frame_loaded: true,
        })
    }

    /// Load a specific frame from SD card into the frame buffer
    /// Returns Ok(()) if loaded successfully
    pub fn load_frame(&mut self, frame_idx: usize, sd_card: &mut crate::ecs::resources::SdCardWrapper) -> Result<(), String> {
        if frame_idx >= self.frame_count() {
            return Err(format!("Frame index {} out of range", frame_idx));
        }

        let offset = self.meta.frame_offsets[frame_idx] as usize;
        let frame_size = self.meta.frame_data_size();

        // Load frame range from SD card
        let frame_data = sd_card.load_binary_range(&self.file_path, offset, frame_size)
            .map_err(|e| format!("Failed to load frame {}: {:?}", frame_idx, e))?;

        // Copy into frame buffer
        if frame_data.len() == frame_size {
            self.frame_buffer[..frame_size].copy_from_slice(&frame_data);
            self.frame_loaded = true;
            Ok(())
        } else {
            Err(format!("Frame data size mismatch: expected {}, got {}", frame_size, frame_data.len()))
        }
    }

    /// Get frame count
    pub fn frame_count(&self) -> usize {
        self.meta.frame_count as usize
    }

    /// Get current frame index
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Set current frame
    pub fn set_frame(&mut self, frame: usize) {
        self.current_frame = frame.min(self.frame_count().saturating_sub(1));
        self.frame_timer = 0.0;
        // Only mark for reload in streaming mode (not full-file mode)
        if self.file_path != "<embedded>" {
            self.frame_loaded = false;
        }
    }

    /// Update animation timer, returns true if frame changed
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.meta.frame_count <= 1 {
            return false;
        }

        let delay_ms = self.meta.frame_delays.get(self.current_frame).copied().unwrap_or(100);
        let delay_secs = delay_ms as f32 / 1000.0;

        self.frame_timer += delta_time;
        if self.frame_timer >= delay_secs {
            self.frame_timer = 0.0;
            self.current_frame = (self.current_frame + 1) % self.frame_count();
            // Only mark for reload in streaming mode (not full-file mode)
            if self.file_path != "<embedded>" {
                self.frame_loaded = false;
            }
            return true;
        }
        false
    }

    /// Check if current frame needs to be loaded from SD card
    pub fn needs_frame_load(&self) -> bool {
        !self.frame_loaded
    }

    /// Render current loaded frame to display at given position (centered)
    /// For streaming mode: ensure load_frame() was called first
    /// Returns number of pixels rendered, or 0 if no frame loaded
    pub fn render(
        &self,
        display: &mut St7789pDriver,
        center_x: i32,
        center_y: i32,
        flip_h: bool,
    ) -> usize {
        if !self.frame_loaded {
            return 0; // No frame loaded in streaming mode
        }

        let width = self.meta.width;
        let height = self.meta.height;
        let pixel_count = width as usize * height as usize;

        // Calculate top-left position from center
        let x = center_x - (width as i32 / 2);
        let y = center_y - (height as i32 / 2);

        // For streaming: frame data is at buffer start (offset 0)
        // For from_file_data: check if this is old mode (buffer size >> frame size)
        let is_full_file = self.frame_buffer.len() > self.meta.frame_data_size() * 2;

        let (frame_data, alpha_mask) = if is_full_file {
            // Old mode: buffer contains full file
            let offset = self.meta.frame_offsets[self.current_frame] as usize;
            let frame_data = &self.frame_buffer[offset..offset + pixel_count * 2];
            let alpha_mask = if self.meta.has_transparency {
                let mask_offset = offset + pixel_count * 2;
                let mask_size = (pixel_count + 7) / 8;
                Some(&self.frame_buffer[mask_offset..mask_offset + mask_size])
            } else {
                None
            };
            (frame_data, alpha_mask)
        } else {
            // Streaming mode: buffer contains single frame at offset 0
            let frame_data = &self.frame_buffer[0..pixel_count * 2];
            let alpha_mask = if self.meta.has_transparency {
                let mask_offset = pixel_count * 2;
                let mask_size = (pixel_count + 7) / 8;
                Some(&self.frame_buffer[mask_offset..mask_offset + mask_size])
            } else {
                None
            };
            (frame_data, alpha_mask)
        };

        // Use optimized blit method
        display.blit_rgb565(x, y, width, height, frame_data, flip_h, alpha_mask)
    }

    /// Render specific frame to display (DEPRECATED - use render() with load_frame())
    /// This only works with from_file_data() mode (full file in memory)
    pub fn render_frame(
        &self,
        display: &mut St7789pDriver,
        frame: usize,
        center_x: i32,
        center_y: i32,
        flip_h: bool,
    ) -> usize {
        // Only works if buffer contains full file
        if self.frame_buffer.len() <= self.meta.frame_data_size() * 2 {
            return 0; // Streaming mode - use render() instead
        }

        let frame = frame.min(self.frame_count().saturating_sub(1));
        let offset = self.meta.frame_offsets[frame] as usize;
        let width = self.meta.width;
        let height = self.meta.height;
        let pixel_count = width as usize * height as usize;

        // Calculate top-left position from center
        let x = center_x - (width as i32 / 2);
        let y = center_y - (height as i32 / 2);

        // Get frame data slice
        let frame_data = &self.frame_buffer[offset..offset + pixel_count * 2];

        // Optional transparency mask
        let alpha_mask = if self.meta.has_transparency {
            let mask_offset = offset + pixel_count * 2;
            let mask_size = (pixel_count + 7) / 8;
            Some(&self.frame_buffer[mask_offset..mask_offset + mask_size])
        } else {
            None
        };

        // Use optimized blit method
        display.blit_rgb565(x, y, width, height, frame_data, flip_h, alpha_mask)
    }
}

/// Lightweight animation reference that streams frames from SD card
/// Only stores metadata, loads frame data on demand
pub struct StreamingRawAnim {
    /// Animation metadata
    pub meta: RawAnimMeta,
    /// Path to the raw file on SD card
    pub path: String,
    /// Current frame index
    current_frame: usize,
    /// Frame timer
    frame_timer: f32,
}

impl StreamingRawAnim {
    /// Create from metadata and path (doesn't load frame data yet)
    pub fn new(meta: RawAnimMeta, path: String) -> Self {
        Self {
            meta,
            path,
            current_frame: 0,
            frame_timer: 0.0,
        }
    }

    /// Get frame count
    pub fn frame_count(&self) -> usize {
        self.meta.frame_count as usize
    }

    /// Get current frame index
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Set current frame
    pub fn set_frame(&mut self, frame: usize) {
        self.current_frame = frame.min(self.frame_count().saturating_sub(1));
        self.frame_timer = 0.0;
    }

    /// Update animation timer, returns true if frame changed
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.meta.frame_count <= 1 {
            return false;
        }

        let delay_ms = self.meta.frame_delays.get(self.current_frame).copied().unwrap_or(100);
        let delay_secs = delay_ms as f32 / 1000.0;

        self.frame_timer += delta_time;
        if self.frame_timer >= delay_secs {
            self.frame_timer = 0.0;
            self.current_frame = (self.current_frame + 1) % self.frame_count();
            return true;
        }
        false
    }

    /// Get the file offset and size for current frame
    pub fn current_frame_info(&self) -> (u32, usize) {
        let offset = self.meta.frame_offsets[self.current_frame];
        let size = self.meta.frame_data_size();
        (offset, size)
    }
}

/// Render RGB565 frame data directly to display
/// This is the core rendering function - call after loading frame from SD
/// Returns number of pixels rendered
pub fn render_rgb565_frame(
    display: &mut St7789pDriver,
    frame_data: &[u8],
    width: u16,
    height: u16,
    center_x: i32,
    center_y: i32,
    flip_h: bool,
    alpha_mask: Option<&[u8]>,
) -> usize {
    // Calculate top-left position from center
    let x = center_x - (width as i32 / 2);
    let y = center_y - (height as i32 / 2);

    // Use optimized blit method
    display.blit_rgb565(x, y, width, height, frame_data, flip_h, alpha_mask)
}
