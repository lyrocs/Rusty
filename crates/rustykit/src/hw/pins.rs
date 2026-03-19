/// Pin assignments for Waveshare ESP32-C6-Touch-LCD-1.83

// SPI bus (shared by display and SD card)
pub const SPI_SCK: u8 = 1;
pub const SPI_MOSI: u8 = 2;
pub const SPI_MISO: u8 = 16;

// Display (ST7789P)
pub const LCD_CS: u8 = 5;
pub const LCD_DC: u8 = 3;
pub const LCD_RST: u8 = 4;
pub const LCD_BL: u8 = 6;
pub const LCD_SPI_BAUDRATE: u32 = 40_000_000;

// SD card
pub const SD_CS: u8 = 17;
pub const SD_SPI_BAUDRATE: u32 = 20_000_000;

// Touch controller (CST816D)
pub const TOUCH_SDA: u8 = 7;
pub const TOUCH_SCL: u8 = 8;
pub const TOUCH_INT: u8 = 11;
pub const TOUCH_I2C_ADDR: u8 = 0x15;
pub const TOUCH_I2C_BAUDRATE: u32 = 400_000;

// GPIO expander (TCA9554)
pub const EXIO_I2C_ADDR: u8 = 0x20;
pub const EXIO_SD_CS_PIN: u8 = 7;

// Buttons
pub const BOOT_BUTTON: u8 = 9;
pub const PWR_BUTTON: u8 = 18;

// Display dimensions
pub const LCD_WIDTH: u16 = 240;
pub const LCD_HEIGHT: u16 = 284;
