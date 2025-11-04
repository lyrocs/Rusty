pub mod sh8601;
pub mod reset;

pub use sh8601::{Sh8601Driver, ColorMode};

// Display configuration for Waveshare ESP32-S3-Touch-AMOLED-1.8
pub const LCD_H_RES: u16 = 368;
pub const LCD_V_RES: u16 = 448;
