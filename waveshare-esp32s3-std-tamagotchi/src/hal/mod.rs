// Hardware Abstraction Layer for ESP32-S3 Waveshare AMOLED
//
// This module provides thread-safe abstractions for hardware peripherals
// to enable multithreaded operation.

pub mod pins;
pub mod config;

use anyhow::Result;
use embedded_graphics::prelude::*;

/// Display driver trait for thread-safe rendering
pub trait DisplayDriver: Send {
    /// Draw a buffer to the display at the given position
    fn draw_buffer(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<()>;

    /// Clear the display
    fn clear(&mut self) -> Result<()>;

    /// Flush any pending operations
    fn flush(&mut self) -> Result<()>;

    /// Get display dimensions
    fn dimensions(&self) -> (u16, u16);
}

/// Touch driver trait for thread-safe input
pub trait TouchDriver: Send {
    /// Read current touch position (if touched)
    fn read_touch(&mut self) -> Option<(u16, u16)>;

    /// Check if screen is currently touched
    fn is_touched(&mut self) -> bool;

    /// Enable/disable gesture mode
    fn set_gesture_mode(&mut self, enabled: bool) -> Result<()>;
}

/// Storage driver trait for thread-safe file operations
pub trait StorageDriver: Send {
    /// Read file contents
    fn read_file(&mut self, path: &str) -> Result<Vec<u8>>;

    /// Write file contents
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<()>;

    /// Check if file exists
    fn exists(&mut self, path: &str) -> bool;

    /// List files in directory
    fn list_dir(&mut self, path: &str) -> Result<Vec<String>>;
}

/// Button state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

/// Button driver trait for thread-safe input
pub trait ButtonDriver: Send {
    /// Read button state
    fn read_button(&mut self) -> ButtonState;
}

/// Power management driver trait
pub trait PowerDriver: Send {
    /// Get battery voltage in millivolts
    fn battery_voltage(&mut self) -> Result<u16>;

    /// Get battery percentage (0-100)
    fn battery_percent(&mut self) -> Result<u8>;

    /// Check if charging
    fn is_charging(&mut self) -> Result<bool>;
}
