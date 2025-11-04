pub mod sh8601;
pub mod reset;
pub mod ft3x68;

pub use sh8601::{Sh8601Driver, ColorMode};
pub use ft3x68::{Ft3x68Driver, FT3168_DEVICE_ADDRESS, Gesture};

// Display configuration for Waveshare ESP32-S3-Touch-AMOLED-1.8
pub const LCD_H_RES: u16 = 368;
pub const LCD_V_RES: u16 = 448;
