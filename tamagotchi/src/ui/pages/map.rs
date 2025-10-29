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
use super::super::helpers::*;

use super::super::colors::*;

/// Draw the Map/Navigation page
pub fn draw_map_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    let map_id = game_state.current_location;

    // Draw map background image if available
    if let Some(map_bg_data) = crate::tamagotchi::models::get_map_background(map_id) {
        let gif = Gif::<Rgb888>::from_slice(map_bg_data).expect("Failed to parse map GIF");

        // Get GIF dimensions
        let gif_width = gif.width() as i32;
        let gif_height = gif.height() as i32;

        // Center the background on screen (368x448 display)
        let top_left = Point::new((368 - gif_width) / 2, (448 - gif_height) / 2);

        // Render first (and only) frame of the map GIF
        if let Some(frame) = gif.frames().next() {
            Image::new(&frame, top_left).draw(display)?;
        }
    }

    let location_type = MapHelper::location_type(map_id);

    // Title with location name
    let mut title = String::<32>::new();
    write!(title, "=== {} ===", MapHelper::name(map_id)).ok();
    draw_text(display, &title, Point::new(60, 20), &FONT_10X20, COLOR_TEXT)?;

    // Draw directional navigation indicators (blue circles at borders)
    let exits = MapHelper::exits(map_id);
    for exit in exits {
        match exit.direction {
            "North" => {
                // Top circle
                EgCircle::new(Point::new(164, 5), 30)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                    .draw(display)?;
            }
            "South" => {
                // Bottom circle
                EgCircle::new(Point::new(164, 413), 30)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                    .draw(display)?;
            }
            "West" => {
                // Left circle
                EgCircle::new(Point::new(10, 209), 30)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                    .draw(display)?;
            }
            "East" => {
                // Right circle
                EgCircle::new(Point::new(328, 209), 30)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                    .draw(display)?;
            }
            _ => {}
        }
    }

    // Center area for info and actions
    match location_type {
        LocationType::City => {
            // Show NPC actions as buttons (similar to menu)
            let npcs = MapHelper::npcs(map_id);
            if !npcs.is_empty() {
                for (i, npc) in npcs.iter().enumerate() {
                    let row = i / 2;
                    let col = i % 2;
                    let x = 59 + col as i32 * 130; // Centered buttons
                    let y = 100 + row as i32 * 75;

                    // Draw action button
                    Rectangle::new(Point::new(x, y), Size::new(120, 60))
                        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
                        .draw(display)?;
                    Rectangle::new(Point::new(x, y), Size::new(120, 60))
                        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
                        .draw(display)?;

                    // Text (word wrap for long names)
                    if npc.len() > 10 {
                        // Split long text
                        if let Some(space_idx) = npc.find(' ') {
                            let (first, second) = npc.split_at(space_idx);
                            draw_text(
                                display,
                                first,
                                Point::new(x + 10, y + 20),
                                &FONT_9X15,
                                COLOR_TEXT,
                            )?;
                            draw_text(
                                display,
                                second.trim(),
                                Point::new(x + 10, y + 38),
                                &FONT_9X15,
                                COLOR_TEXT_DIM,
                            )?;
                        } else {
                            draw_text(
                                display,
                                npc,
                                Point::new(x + 10, y + 25),
                                &FONT_9X15,
                                COLOR_TEXT,
                            )?;
                        }
                    } else {
                        draw_text(
                            display,
                            npc,
                            Point::new(x + 10, y + 25),
                            &FONT_9X15,
                            COLOR_TEXT,
                        )?;
                    }
                }
            }
        }
        LocationType::Field => {
            // Show monster GIF animations on the map
            let enemy_ids = MapHelper::enemies(map_id);
            if !enemy_ids.is_empty() {
                // Display up to 4 monsters with their GIF animations
                for (i, &enemy_id) in enemy_ids.iter().enumerate().take(4) {
                    if let Some(enemy) = Enemy::from_id(enemy_id) {
                        // Calculate position for monsters (2x2 grid in center)
                        let col = i % 2;
                        let row = i / 2;
                        let x = 90 + col as i32 * 100;
                        let y = 140 + row as i32 * 100;
                        let center = Point::new(x, y);

                        // Draw monster name in black with white background above GIF
                        let name_x = center.x - (enemy.name.len() as i32 * 9) / 2;
                        let name_y = center.y - 40;

                        // Draw white background rectangle for name
                        // Note: text y position is at baseline, so background must start higher
                        let name_width = enemy.name.len() as i32 * 9;
                        let bg_padding = 3;
                        let font_height = 18; // FONT_9X18_BOLD height
                        Rectangle::new(
                            Point::new(name_x - bg_padding, name_y - font_height - bg_padding + 2),
                            Size::new(
                                (name_width + bg_padding * 2) as u32,
                                (font_height + bg_padding * 2) as u32,
                            ),
                        )
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
                        .draw(display)?;

                        // Draw black text on top
                        draw_text(
                            display,
                            enemy.name,
                            Point::new(name_x, name_y),
                            &FONT_9X18_BOLD,
                            Rgb888::BLACK,
                        )?;

                        // Draw monster idle GIF (0.gif)
                        draw_map_monster_gif(display, game_state, center, enemy.name)?;
                    }
                }

                // Action buttons (centered, higher to leave space for bottom navigation)
                // Auto Farm button
                Rectangle::new(Point::new(84, 280), Size::new(200, 50))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50)))
                    .draw(display)?;
                Rectangle::new(Point::new(84, 280), Size::new(200, 50))
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 3))
                    .draw(display)?;
                draw_text(
                    display,
                    "AUTO FARM",
                    Point::new(115, 300),
                    &FONT_9X18_BOLD,
                    Rgb888::WHITE,
                )?;

                // Battle button
                Rectangle::new(Point::new(84, 335), Size::new(200, 50))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
                    .draw(display)?;
                Rectangle::new(Point::new(84, 335), Size::new(200, 50))
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 3))
                    .draw(display)?;
                draw_text(
                    display,
                    "BATTLE",
                    Point::new(140, 355),
                    &FONT_9X18_BOLD,
                    Rgb888::WHITE,
                )?;
            }
        }
    }

    // Status message (for SP/HP warnings)
    if let Some(msg) = game_state.save_status_msg {
        draw_text(display, msg, Point::new(60, 390), &FONT_10X20, Rgb888::RED)?;
    }

    // Equipment selection menu overlay (if open)
    if game_state.equipment_selection_open {
        draw_equipment_selection(display, game_state)?;
    }
    // Refine popup overlay (if open)
    else if game_state.refine_popup_open {
        if let Some(slot) = game_state.refine_slot {
            draw_refine_popup(display, game_state, slot)?;
        }
    }

    Ok(())
}

