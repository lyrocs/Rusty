use core::fmt::Write;
use embedded_graphics::{
    image::Image,
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle as EgCircle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;
use tinygif::Gif;

use crate::core::GameState;
use crate::tamagotchi::models::{BattleState, CircleType, Enemy, FarmState, LocationType, MapHelper, RestState};
use super::super::colors::*;

use super::super::helpers::*;

/// Draw the settings page with brightness slider
pub fn draw_settings_page<D>(
    display: &mut D,
    game_state: &GameState,
    battery_mv: u16,
    battery_pct: u8,
    fps: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear display
    display.clear(COLOR_BG)?;

    draw_text(
        display,
        "=== SETTINGS ===",
        Point::new(85, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Brightness section
    draw_text(
        display,
        "Brightness",
        Point::new(130, 100),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Brightness value display
    let mut brightness_str = String::<16>::new();
    write!(
        brightness_str,
        "{}%",
        (game_state.brightness as u32 * 100) / 255
    )
    .ok();
    draw_text(
        display,
        &brightness_str,
        Point::new(155, 130),
        &FONT_9X18_BOLD,
        Rgb888::YELLOW,
    )?;

    // Slider track (background bar)
    Rectangle::new(Point::new(40, 180), Size::new(280, 20))
        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
        .draw(display)?;

    Rectangle::new(Point::new(40, 180), Size::new(280, 20))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    // Slider filled portion (represents current brightness)
    let filled_width = ((game_state.brightness as u32 * 280) / 255) as u32;
    if filled_width > 0 {
        Rectangle::new(Point::new(40, 180), Size::new(filled_width, 20))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::YELLOW))
            .draw(display)?;
    }

    // Slider handle (indicator)
    let handle_x = 40 + ((game_state.brightness as i32 * 280) / 255);
    EgCircle::new(Point::new(handle_x - 8, 172), 16)
        .into_styled(PrimitiveStyle::with_fill(COLOR_MENU_SELECT))
        .draw(display)?;

    EgCircle::new(Point::new(handle_x - 8, 172), 16)
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 3))
        .draw(display)?;

    // Instructions
    draw_text(
        display,
        "Touch slider to adjust",
        Point::new(70, 250),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Brightness range labels
    draw_text(
        display,
        "0%",
        Point::new(35, 210),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    draw_text(
        display,
        "100%",
        Point::new(290, 210),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Shutdown button
    let shutdown_btn_x = 95;
    let shutdown_btn_y = 280;
    let shutdown_btn_w = 170;
    let shutdown_btn_h = 50;

    Rectangle::new(
        Point::new(shutdown_btn_x, shutdown_btn_y),
        Size::new(shutdown_btn_w, shutdown_btn_h),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(180, 50, 50)))
    .draw(display)?;

    Rectangle::new(
        Point::new(shutdown_btn_x, shutdown_btn_y),
        Size::new(shutdown_btn_w, shutdown_btn_h),
    )
    .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
    .draw(display)?;

    draw_text(
        display,
        "SHUTDOWN",
        Point::new(shutdown_btn_x + 25, shutdown_btn_y + 30),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // System Info section at bottom (Battery and FPS)
    draw_text(
        display,
        "System Info",
        Point::new(125, 360),
        &FONT_9X18_BOLD,
        COLOR_TEXT_DIM,
    )?;

    // Battery info
    draw_battery_info(display, Point::new(20, 380), battery_mv, battery_pct)?;

    // FPS info (right side)
    draw_fps_info(display, Point::new(230, 380), fps)?;

    // Footer
    draw_text(
        display,
        "Touch bottom to go back",
        Point::new(65, 420),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Draw shutdown confirmation modal if showing
    if game_state.show_shutdown_confirm {
        draw_shutdown_confirm_modal(display)?;
    }

    Ok(())
}

/// Draw shutdown confirmation modal
fn draw_shutdown_confirm_modal<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let modal_x = 40;
    let modal_y = 150;
    let modal_w = 280;
    let modal_h = 150;

    // Semi-transparent overlay effect (dark background)
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    // Modal background
    Rectangle::new(Point::new(modal_x, modal_y), Size::new(modal_w, modal_h))
        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
        .draw(display)?;

    Rectangle::new(Point::new(modal_x, modal_y), Size::new(modal_w, modal_h))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 3))
        .draw(display)?;

    // Title
    draw_text(
        display,
        "Confirm Shutdown",
        Point::new(modal_x + 40, modal_y + 30),
        &FONT_10X20,
        Rgb888::RED,
    )?;

    // Message
    draw_text(
        display,
        "Save and power off?",
        Point::new(modal_x + 30, modal_y + 60),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Confirm button (red)
    let confirm_x = modal_x + 20;
    let confirm_y = modal_y + 90;
    let btn_w = 110;
    let btn_h = 40;

    Rectangle::new(Point::new(confirm_x, confirm_y), Size::new(btn_w, btn_h))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(200, 50, 50)))
        .draw(display)?;

    Rectangle::new(Point::new(confirm_x, confirm_y), Size::new(btn_w, btn_h))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    draw_text(
        display,
        "CONFIRM",
        Point::new(confirm_x + 15, confirm_y + 25),
        &FONT_9X18_BOLD,
        Rgb888::WHITE,
    )?;

    // Cancel button (gray)
    let cancel_x = modal_x + 150;
    let cancel_y = modal_y + 90;

    Rectangle::new(Point::new(cancel_x, cancel_y), Size::new(btn_w, btn_h))
        .into_styled(PrimitiveStyle::with_fill(COLOR_MENU_SELECT))
        .draw(display)?;

    Rectangle::new(Point::new(cancel_x, cancel_y), Size::new(btn_w, btn_h))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    draw_text(
        display,
        "CANCEL",
        Point::new(cancel_x + 20, cancel_y + 25),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    Ok(())
}

