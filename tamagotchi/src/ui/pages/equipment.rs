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

/// Draw the Equipment page
pub fn draw_equipment_page<D>(
    display: &mut D,
    game_state: &GameState,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let hero = &game_state.hero;

    // Clear background
    display.clear(COLOR_BG)?;

    // Title
    draw_text(
        display,
        "=== EQUIPMENT ===",
        Point::new(60, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Zeny display
    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "Zeny: {}", hero.zeny).ok();
    draw_text(
        display,
        &zeny_str,
        Point::new(220, 50),
        &FONT_9X15,
        COLOR_EXP,
    )?;

    // Equipment display (3 slots, stacked vertically)
    let start_y = 80;
    let spacing = 100;

    // WEAPON
    draw_equipment_slot(
        display,
        &hero.equipped_weapon,
        Point::new(20, start_y),
        "WEAPON",
    )?;

    // ARMOR
    draw_equipment_slot(
        display,
        &hero.equipped_armor,
        Point::new(20, start_y + spacing),
        "ARMOR",
    )?;

    // ACCESSORY
    draw_equipment_slot(
        display,
        &hero.equipped_accessory,
        Point::new(20, start_y + (spacing * 2)),
        "ACCESSORY",
    )?;

    // Back button
    Rectangle::new(Point::new(100, 400), Size::new(160, 40))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 100, 150)))
        .draw(display)?;
    draw_text(
        display,
        "Back",
        Point::new(155, 418),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Draw refine popup if open
    if game_state.refine_popup_open {
        if let Some(slot) = game_state.refine_slot {
            draw_refine_popup(display, game_state, slot)?;
        }
    }

    Ok(())
}

