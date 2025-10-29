// Pin definitions for ESP32-S3 Waveshare AMOLED 1.8"
//
// Based on the hardware schematic and original no_std implementation

/// Display SPI pins (QSPI mode)
pub mod display {
    pub const SIO0: u8 = 4;
    pub const SIO1: u8 = 5;
    pub const SIO2: u8 = 6;
    pub const SIO3: u8 = 7;
    pub const CS: u8 = 12;
    pub const SCK: u8 = 11;
}

/// Touch I2C pins
pub mod touch {
    pub const SDA: u8 = 15;
    pub const SCL: u8 = 14;
    pub const ADDRESS: u8 = 0x38; // FT3168 I2C address
}

/// SD Card SPI pins
pub mod sd_card {
    pub const SCK: u8 = 2;
    pub const MOSI: u8 = 1;
    pub const MISO: u8 = 3;
    pub const CS_EXIO: u8 = 7; // EXIO7 on TCA9554 GPIO expander
}

/// Button pins
pub mod buttons {
    pub const BOOT: u8 = 0; // GPIO0 - Boot button
    pub const PWR_EXIO: u8 = 4; // EXIO4 on TCA9554 GPIO expander
}

/// I2C shared bus configuration
pub mod i2c {
    pub const SDA: u8 = 15;
    pub const SCL: u8 = 14;
    pub const FREQUENCY_KHZ: u32 = 400;
}

/// SPI bus configurations
pub mod spi {
    /// Display SPI (SPI2)
    pub const DISPLAY_FREQ_MHZ: u32 = 40;

    /// SD Card SPI (SPI3)
    pub const SD_CARD_FREQ_KHZ: u32 = 400; // Start slow for init
    pub const SD_CARD_FREQ_MHZ: u32 = 20;  // Speed up after init
}

/// I2C device addresses
pub mod i2c_addresses {
    pub const TOUCH_FT3168: u8 = 0x38;
    pub const GPIO_EXPANDER_TCA9554: u8 = 0x22;
    pub const PMIC_AXP2101: u8 = 0x34;
    pub const RTC_PCF85063: u8 = 0x51;
}

/// Display specifications
pub mod display_spec {
    pub const WIDTH: u16 = 368;
    pub const HEIGHT: u16 = 448;
    pub const DMA_CHUNK_SIZE: usize = 32768;
}
