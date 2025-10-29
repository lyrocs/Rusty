/// FPS display component
///
/// Renders FPS information for debugging and performance monitoring.

use core::fmt::Write;
use embedded_graphics::{
    mono_font::ascii::FONT_9X18_BOLD,
    pixelcolor::Rgb888,
    prelude::*,
};
use heapless::String;

use super::text::draw_text;
use super::super::COLOR_TEXT_DIM;

/// Draw FPS information
pub fn draw_fps_info<D>(display: &mut D, position: Point, fps: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // FPS label
    draw_text(display, "FPS:", position, &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;

    // FPS value
    let mut fps_str = String::<16>::new();
    write!(fps_str, "{}", fps).ok();

    // Color based on FPS (green if 30+, yellow if 20-29, red if <20)
    let fps_color = if fps >= 30 {
        Rgb888::GREEN
    } else if fps >= 20 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    // Position value to the right of the label (FPS: is 4 chars * 9px = 36px + 5px spacing)
    draw_text(
        display,
        &fps_str,
        position + Point::new(45, 0),
        &FONT_9X18_BOLD,
        fps_color,
    )?;

    Ok(())
}
