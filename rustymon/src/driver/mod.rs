pub mod st7789p;

pub use st7789p::ColorMode;

// Display configuration for Waveshare ESP32-C6-Touch-LCD-1.83
pub const LCD_H_RES: u16 = 240;
pub const LCD_V_RES: u16 = 284;

// Type alias for the display driver with specific GPIO pins used on ESP32-C6-Touch-LCD-1.83
// DC = GPIO3, RST = GPIO4
pub type St7789pDriver<'a> = st7789p::St7789pDriver<'a, esp_idf_svc::hal::gpio::Gpio3, esp_idf_svc::hal::gpio::Gpio4>;
