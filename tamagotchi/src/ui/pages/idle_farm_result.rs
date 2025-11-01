/// IDLE Farming Session Results Screen
///
/// Displays statistics from completed farming session

use core::fmt::Write;
use embedded_graphics::{
    mono_font::ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;

use crate::core::GameState;
use super::super::helpers::*;
use super::super::colors::*;

/// Draw the IDLE farming results page
pub fn draw_idle_farm_result_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    let session = match &game_state.idle_farm_session {
        Some(s) => s,
        None => {
            // No session data - shouldn't happen, but handle gracefully
            draw_text(
                display,
                "No session data",
                Point::new(100, 200),
                &FONT_10X20,
                COLOR_TEXT_DIM,
            )?;
            return Ok(());
        }
    };

    // Determine result type (completed vs died)
    let is_death = session.is_in_cooldown();

    // Title
    let title = if is_death { "=== HERO DIED ===" } else { "=== SESSION END ===" };
    let title_color = if is_death { Rgb888::RED } else { Rgb888::GREEN };

    draw_text(
        display,
        title,
        Point::new(70, 30),
        &FONT_10X20,
        title_color,
    )?;

    // Map name
    use crate::world::MapHelper;
    let map_name = MapHelper::name(session.map_id);
    let mut map_text = String::<32>::new();
    write!(map_text, "Map: {}", map_name).ok();
    draw_text(
        display,
        &map_text,
        Point::new(20, 65),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Duration
    let duration_s = session.duration_ms(game_state.last_update_ms) / 1000;
    let minutes = duration_s / 60;
    let seconds = duration_s % 60;
    let mut time_text = String::<32>::new();
    write!(time_text, "Duration: {}:{:02}", minutes, seconds).ok();
    draw_text(
        display,
        &time_text,
        Point::new(20, 90),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Statistics Section
    draw_text(
        display,
        "=== STATISTICS ===",
        Point::new(80, 125),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Monsters killed
    let mut kills_text = String::<32>::new();
    write!(kills_text, "Monsters Killed: {}", session.monsters_killed).ok();
    draw_text(
        display,
        &kills_text,
        Point::new(20, 155),
        &FONT_9X15,
        COLOR_TEXT,
    )?;

    // Zeny earned
    let mut zeny_text = String::<32>::new();
    write!(zeny_text, "Zeny Earned: +{}", session.zeny_earned).ok();
    draw_text(
        display,
        &zeny_text,
        Point::new(20, 180),
        &FONT_9X15,
        Rgb888::YELLOW,
    )?;

    // EXP gained
    let mut exp_text = String::<32>::new();
    write!(exp_text, "EXP Gained: +{}", session.exp_gained).ok();
    draw_text(
        display,
        &exp_text,
        Point::new(20, 205),
        &FONT_9X15,
        Rgb888::CYAN,
    )?;

    // Items found (placeholder for now)
    let mut items_text = String::<32>::new();
    write!(items_text, "Items Found: {}", session.items_collected).ok();
    draw_text(
        display,
        &items_text,
        Point::new(20, 230),
        &FONT_9X15,
        COLOR_TEXT,
    )?;

    // TODO: Display actual item drops when implemented
    // For now, show placeholder
    if session.items_collected > 0 {
        draw_text(
            display,
            "Item details coming soon...",
            Point::new(20, 255),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }

    // Cooldown message if died
    if is_death {
        let remaining = (session.cooldown_end_ms.saturating_sub(game_state.last_update_ms)) / 1000;

        if remaining > 0 {
            draw_text(
                display,
                "=== COOLDOWN ===",
                Point::new(90, 295),
                &FONT_9X18_BOLD,
                Rgb888::RED,
            )?;

            let mut cooldown_text = String::<32>::new();
            write!(cooldown_text, "Wait {}s to farm again", remaining).ok();
            draw_text(
                display,
                &cooldown_text,
                Point::new(50, 325),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    // Continue button
    let button_y = if is_death && (session.cooldown_end_ms > game_state.last_update_ms) {
        370  // Lower position if cooldown message is shown
    } else {
        320
    };

    Rectangle::new(Point::new(80, button_y), Size::new(200, 50))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 120)))
        .draw(display)?;
    Rectangle::new(Point::new(80, button_y), Size::new(200, 50))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 120, 160), 2))
        .draw(display)?;
    draw_text(
        display,
        "CONTINUE",
        Point::new(125, button_y + 30),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    Ok(())
}
