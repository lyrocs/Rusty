/// Global Header Component
///
/// Shows hero info or battle stats in a banner at the top of the screen.
/// - When in battle: shows battle stats (kills, zeny, items) - clickable to view battle
/// - When not in battle: shows hero stats (HP, SP, Level)

use core::fmt::Write;
use embedded_graphics::{
    mono_font::ascii::FONT_9X15,
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;

use crate::core::GameState;
use super::helpers::draw_text;
use super::colors::*;

/// Height of the global header banner
pub const FARMING_HEADER_HEIGHT: i32 = 30;

/// Draw the global header (always visible)
/// Returns true (always draws header now)
pub fn draw_farming_header<D>(display: &mut D, game_state: &GameState) -> Result<bool, D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let hero = &game_state.hero;

    // Check if farming session is active
    let has_active_battle = game_state.idle_farm_session.as_ref().map_or(false, |s| s.is_active());

    // Draw header background
    let bg_color = if has_active_battle {
        Rgb888::new(40, 60, 40) // Green tint when farming
    } else {
        Rgb888::new(30, 40, 60) // Blue tint when idle
    };

    Rectangle::new(
        Point::new(0, 0),
        Size::new(360, FARMING_HEADER_HEIGHT as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(bg_color))
    .draw(display)?;

    // Draw bottom border
    Rectangle::new(
        Point::new(0, FARMING_HEADER_HEIGHT - 2),
        Size::new(360, 2),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 100, 140)))
    .draw(display)?;

    if has_active_battle {
        // Show battle stats
        if let Some(ref session) = game_state.idle_farm_session {
            let mut stats_text = String::<64>::new();
            write!(
                stats_text,
                "K:{} Z:{} I:{}",
                session.monsters_killed,
                session.zeny_earned,
                session.items_collected
            ).ok();

            draw_text(
                display,
                &stats_text,
                Point::new(10, 20),
                &FONT_9X15,
                COLOR_TEXT,
            )?;

            // "FARMING" indicator
            draw_text(
                display,
                "FARMING",
                Point::new(250, 20),
                &FONT_9X15,
                Rgb888::GREEN,
            )?;

            // Tap hint
            draw_text(
                display,
                "tap",
                Point::new(200, 20),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    } else {
        // Show hero stats
        let mut hp_text = String::<32>::new();
        write!(hp_text, "HP:{}/{}", hero.hp, hero.max_hp).ok();
        draw_text(
            display,
            &hp_text,
            Point::new(10, 20),
            &FONT_9X15,
            COLOR_HP,
        )?;

        let mut sp_text = String::<32>::new();
        write!(sp_text, "SP:{}/{}", hero.sp, hero.max_sp).ok();
        draw_text(
            display,
            &sp_text,
            Point::new(120, 20),
            &FONT_9X15,
            COLOR_SP,
        )?;

        let mut lvl_text = String::<32>::new();
        write!(lvl_text, "Lv.{}", hero.level).ok();
        draw_text(
            display,
            &lvl_text,
            Point::new(250, 20),
            &FONT_9X15,
            COLOR_TEXT,
        )?;
    }

    Ok(true)
}

/// Check if a touch point is within the farming header area
pub fn is_farming_header_touched(x: u16, y: u16) -> bool {
    y <= FARMING_HEADER_HEIGHT as u16
}
