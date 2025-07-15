use rppal::i2c::I2c;
use linux_embedded_hal::{
    Delay, SpidevDevice, SysfsPin,
};
use epd_waveshare::epd2in13_v2::{Display2in13, Epd2in13};
pub struct Eink {
    pub i2c: I2c,
    pub display: Display2in13,
    pub epd2in13: Epd2in13<SpidevDevice, SysfsPin, SysfsPin, SysfsPin, Delay>,
    pub spi: SpidevDevice,
    pub delay: Delay,
}