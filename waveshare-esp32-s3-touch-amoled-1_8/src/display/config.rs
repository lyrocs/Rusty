use sh8601_rs::{ColorMode, DisplaySize, framebuffer_size};

// Display configuration for Waveshare ESP32-S3-Touch-AMOLED-1.8
pub const DISPLAY_SIZE: DisplaySize = DisplaySize::new(368, 448);
pub const FB_SIZE: usize = framebuffer_size(DISPLAY_SIZE, ColorMode::Rgb888);

// LCD buffer dimensions
pub const LCD_H_RES: usize = 368;
pub const LCD_V_RES: usize = 448;
pub const LCD_BUFFER_SIZE: usize = LCD_H_RES * LCD_V_RES;
