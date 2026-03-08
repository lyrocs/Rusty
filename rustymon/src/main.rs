mod driver;

use driver::{ColorMode, St7789pDriver, LCD_H_RES, LCD_V_RES};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::Text,
};
use esp_idf_svc::hal::{
    gpio::PinDriver,
    peripherals::Peripherals,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::Hertz,
};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(e) = run() {
        log::error!("Fatal error: {:?}", e);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // SPI bus: SCK=GPIO1, MOSI=GPIO2 (no MISO needed for display)
    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        pins.gpio1,
        pins.gpio2,
        None::<esp_idf_svc::hal::gpio::AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;

    let spi_cfg = SpiConfig::new().baudrate(Hertz(40_000_000));
    // CS=GPIO5
    let spi_device = SpiDeviceDriver::new(&spi_driver, Some(pins.gpio5), &spi_cfg)?;

    let dc = PinDriver::output(pins.gpio3)?;
    let rst = PinDriver::output(pins.gpio4)?;
    let bl = PinDriver::output(pins.gpio6)?;

    let mut display = St7789pDriver::new(
        spi_device,
        dc,
        rst,
        LCD_H_RES,
        LCD_V_RES,
        ColorMode::Rgb888,
    )?;

    display.set_backlight_pin(bl);
    display.initialize(ColorMode::Rgb888)?;
    display.backlight_on()?;

    // Black background
    display.clear(Rgb888::BLACK)?;

    // Draw "Hello world" centered vertically, offset horizontally
    let style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
    Text::new("Hello world !", Point::new(50, 140), style).draw(&mut display)?;

    display.flush()?;

    log::info!("Displaying: Hello world");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
