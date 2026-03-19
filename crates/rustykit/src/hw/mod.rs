pub mod pins;

use esp_idf_svc::hal::{
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::Hertz,
};

/// All initialized hardware handles, ready to be distributed to subsystems.
pub struct Hardware<'a> {
    pub display_spi: SpiDeviceDriver<'a, &'a SpiDriver<'a>>,
    pub sd_spi: SpiDeviceDriver<'a, &'a SpiDriver<'a>>,
    pub dc: PinDriver<'a, esp_idf_svc::hal::gpio::Gpio3, esp_idf_svc::hal::gpio::Output>,
    pub rst: PinDriver<'a, esp_idf_svc::hal::gpio::Gpio4, esp_idf_svc::hal::gpio::Output>,
    pub backlight: PinDriver<'a, esp_idf_svc::hal::gpio::Gpio6, esp_idf_svc::hal::gpio::Output>,
    pub boot_pin: PinDriver<'a, esp_idf_svc::hal::gpio::Gpio9, esp_idf_svc::hal::gpio::Input>,
    pub pwr_pin: PinDriver<'a, esp_idf_svc::hal::gpio::Gpio18, esp_idf_svc::hal::gpio::Input>,
    pub i2c: I2cDriver<'a>,
}

/// Initialize all hardware peripherals.
///
/// The SPI bus is leaked to `'static` so display and SD card can share it.
/// Returns owned handles for all peripherals.
pub fn init_hardware() -> Result<Hardware<'static>, Box<dyn std::error::Error>> {
    let peripherals = Peripherals::take()?;
    let p = peripherals.pins;

    // Shared SPI bus (SCK=GPIO1, MOSI=GPIO2, MISO=GPIO16)
    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        p.gpio1,
        p.gpio2,
        Some(p.gpio16),
        &SpiDriverConfig::new(),
    )?;
    let spi_bus: &'static SpiDriver<'static> = Box::leak(Box::new(spi_driver));

    // Display SPI device (CS=GPIO5, 40 MHz)
    let display_spi = SpiDeviceDriver::new(
        spi_bus,
        Some(p.gpio5),
        &SpiConfig::new().baudrate(Hertz(pins::LCD_SPI_BAUDRATE)),
    )?;

    // SD card SPI device (CS=GPIO17, 20 MHz)
    let sd_spi = SpiDeviceDriver::new(
        spi_bus,
        Some(p.gpio17),
        &SpiConfig::new().baudrate(Hertz(pins::SD_SPI_BAUDRATE)),
    )?;

    let dc = PinDriver::output(p.gpio3)?;
    let rst = PinDriver::output(p.gpio4)?;
    let backlight = PinDriver::output(p.gpio6)?;

    // Buttons
    let mut boot_pin = PinDriver::input(p.gpio9)?;
    boot_pin.set_pull(esp_idf_svc::hal::gpio::Pull::Up)?;
    let mut pwr_pin = PinDriver::input(p.gpio18)?;
    pwr_pin.set_pull(esp_idf_svc::hal::gpio::Pull::Up)?;

    // I2C for touch controller
    let i2c_config = I2cConfig::new().baudrate(Hertz(pins::TOUCH_I2C_BAUDRATE));
    let i2c = I2cDriver::new(peripherals.i2c0, p.gpio7, p.gpio8, &i2c_config)?;

    Ok(Hardware {
        display_spi,
        sd_spi,
        dc,
        rst,
        backlight,
        boot_pin,
        pwr_pin,
        i2c,
    })
}
