use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb888,
    primitives::Rectangle,
    image::GetPixel,
};
use sh8601_rs::{ColorMode, ResetDriver};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;

/// Restores background in a specific rectangular area
pub fn restore_background_area<D, I>(
    display: &mut D,
    background: &I,
    area: Rectangle,
) where
    D: DrawTarget<Color = Rgb888>,
    I: GetPixel<Color = Rgb888>,
{
    for pixel in area.points() {
        if let Some(color) = background.pixel(pixel) {
            embedded_graphics::Pixel(pixel, color).draw(display).ok();
        }
    }
}

/// Flushes a partial display region
pub fn flush_region(
    display: &mut sh8601_rs::Sh8601Driver<
        sh8601_rs::Ws18AmoledDriver,
        ResetDriver<RefCellDevice<'static, I2c<'static, Blocking>>>,
    >,
    x_start: u16,
    x_end: u16,
    y_start: u16,
    y_end: u16,
) {
    display
        .partial_flush(x_start, x_end, y_start, y_end, ColorMode::Rgb888)
        .ok();
}
