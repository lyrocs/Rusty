pub mod st7789p;
pub mod cst816d;
pub mod gif_player;
pub mod static_image;
pub mod raw_anim;

pub use st7789p::ColorMode;
pub use raw_anim::{RawAnimMeta, RawAnimPlayer, StreamingRawAnim, render_rgb565_frame};
pub use cst816d::{Cst816dDriver, CST816D_DEVICE_ADDRESS};
pub use gif_player::{GifPlayer, GifMeta, SharedCanvas, DynamicGifMeta, count_gif_frames};
pub use static_image::StaticImage;

// Display configuration for Waveshare ESP32-C6-Touch-LCD-1.83
pub const LCD_H_RES: u16 = 240;
pub const LCD_V_RES: u16 = 284;

// Type alias for the display driver with specific GPIO pins used on ESP32-C6-Touch-LCD-1.83
// DC = GPIO3, RST = GPIO4
pub type St7789pDriver<'a> = st7789p::St7789pDriver<'a, esp_idf_svc::hal::gpio::Gpio3, esp_idf_svc::hal::gpio::Gpio4>;
