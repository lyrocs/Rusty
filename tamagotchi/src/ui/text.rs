use core::fmt::Write;
use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb888,
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    text::Text,
};
use heapless::String;

/// Write generation count on display
pub fn write_generation<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    generation: usize,
) -> Result<(), D::Error> {
    let mut num_str = String::<20>::new();
    write!(num_str, "Gen: {generation}").unwrap();
    Text::new(
        num_str.as_str(),
        Point::new(8, 400),
        MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE),
    )
    .draw(display)?;
    Ok(())
}

/// Write FPS counter on display
pub fn write_fps<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    fps: usize
) -> Result<(), D::Error> {
    let mut num_str = String::<20>::new();
    write!(num_str, "FPS: {fps}").unwrap();
    Text::new(
        num_str.as_str(),
        Point::new(250, 400),
        MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE),
    )
    .draw(display)?;
    Ok(())
}

/// Display battery voltage and percentage on screen
/// Color changes based on charge level: green (>50%), yellow (20-50%), red (<20%)
pub fn write_battery<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    voltage_mv: u16,
    percent: u8,
) -> Result<(), D::Error> {
    let mut bat_str = String::<32>::new();
    write!(bat_str, "Bat: {}% {}mV", percent, voltage_mv).unwrap();

    let color = if percent >= 50 {
        Rgb888::GREEN
    } else if percent >= 20 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    Text::new(
        bat_str.as_str(),
        Point::new(8, 420),
        MonoTextStyle::new(&FONT_10X20, color),
    )
    .draw(display)?;
    Ok(())
}

/// Display PWR button state on screen with debug info
pub fn write_pwr_button<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    pwr_pressed: bool,
    pwr_low: bool,
    pwr_high: bool,
) -> Result<(), D::Error> {
    let mut pwr_str = String::<40>::new();
    write!(
        pwr_str,
        "PWR: {} (L:{}, H:{})",
        if pwr_pressed { "ON" } else { "OFF" },
        if pwr_low { "1" } else { "0" },
        if pwr_high { "1" } else { "0" }
    )
    .unwrap();

    let color = if pwr_pressed {
        Rgb888::GREEN
    } else {
        Rgb888::RED
    };

    Text::new(
        pwr_str.as_str(),
        Point::new(200, 420),
        MonoTextStyle::new(&FONT_10X20, color),
    )
    .draw(display)?;
    Ok(())
}
