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

    // Draw farming header if active
    use crate::ui::farming_header::draw_farming_header;
    draw_farming_header(display, game_state)?;

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

    // Draw directional navigation indicators (3 concentric blue circles like waves)
    let exits = MapHelper::exits(map_id);
    for exit in exits {
        let center = match exit.direction {
            "North" => Point::new(164, 20),
            "South" => Point::new(164, 428),
            "West" => Point::new(25, 224),
            "East" => Point::new(343, 224),
            _ => continue,
        };

        // Draw wave circles (outermost to innermost)
        // Outermost circle - lightest blue
        EgCircle::new(center - Point::new(15, 15), 30)
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 150, 255), 2))
            .draw(display)?;

        // Middle circle - medium blue
        EgCircle::new(center - Point::new(10, 10), 20)
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(50, 100, 255), 2))
            .draw(display)?;

        // Inner circle - darkest blue (filled)
        EgCircle::new(center - Point::new(5, 5), 10)
            .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
            .draw(display)?;
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

                        // Draw monster name in white with black background above GIF
                        let label_len = (enemy.name.len() + 5) as i32; // name + " Lv##"
                        let name_x = center.x - (label_len * 9) / 2;
                        let name_y = center.y - 50; // Move up to make room for stars

                        // Draw black background rectangle for name and stars
                        // Note: text y position is at baseline, so background must start higher
                        let name_width = label_len * 9 + 10;
                        let bg_padding = 3;
                        let total_height = 40; // Height for both name and stars
                        Rectangle::new(
                            Point::new(name_x - bg_padding - 5, name_y - 18 - bg_padding),
                            Size::new(
                                (name_width + bg_padding * 2) as u32,
                                (total_height + bg_padding * 2) as u32,
                            ),
                        )
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::BLACK))
                        .draw(display)?;

                        // Draw white text on top with level
                        let mut enemy_label = String::<32>::new();
                        write!(enemy_label, "{} Lv{}", enemy.name, enemy.level).ok();
                        draw_text(
                            display,
                            &enemy_label,
                            Point::new(name_x, name_y),
                            &FONT_9X18_BOLD,
                            Rgb888::WHITE,
                        )?;

                        // Draw difficulty stars below name
                        let difficulty_stars = if enemy.level <= 5 { "★" }
                            else if enemy.level <= 15 { "★★" }
                            else if enemy.level <= 25 { "★★★" }
                            else if enemy.level <= 35 { "★★★★" }
                            else { "★★★★★" };

                        let stars_x = center.x - (difficulty_stars.len() as i32 * 9) / 2;
                        draw_text(
                            display,
                            difficulty_stars,
                            Point::new(stars_x, name_y + 18),
                            &FONT_9X15,
                            Rgb888::YELLOW,
                        )?;

                        // Draw monster idle GIF (0.gif)
                        draw_map_monster_gif(display, game_state, center, enemy.name)?;
                    }
                }

                // Farming Info Panel (above button)
                let info_y = 265;
                let info_x = 20;

                // Calculate farming estimates based on first enemy in map
                let (kills_est, zeny_est, damage_est, rec_level) = if !enemy_ids.is_empty() {
                    if let Some(enemy) = Enemy::from_id(enemy_ids[0]) {
                        use crate::combat::{calculate_farming_rates, calculate_recommended_level};
                        let rates = calculate_farming_rates(&game_state.hero, &enemy);
                        (
                            rates.kills_per_minute as u32,
                            rates.zeny_per_minute as u32,
                            rates.damage_per_minute as u32,
                            calculate_recommended_level(enemy.level),
                        )
                    } else {
                        (45, 120, 5, "1-10")
                    }
                } else {
                    (45, 120, 5, "1-10")
                };

                // Farming estimates text
                let mut est_text = String::<64>::new();
                write!(est_text, "Est. Kills/min: ~{}  Zeny: ~{}", kills_est, zeny_est).ok();
                draw_text(
                    display,
                    &est_text,
                    Point::new(info_x, info_y),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                let mut dmg_text = String::<64>::new();
                write!(dmg_text, "Damage/min: ~{}  Rec. Lv: {}", damage_est, rec_level).ok();
                draw_text(
                    display,
                    &dmg_text,
                    Point::new(info_x, info_y + 20),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                // Battle button (changes based on current farming state)
                let button_y = 305;
                let button_width = 330;
                let button_height = 60;
                let button_x = (368 - button_width) / 2; // Center horizontally

                // Check farming session state
                let has_active_farming = game_state.idle_farm_session.as_ref().map_or(false, |s| s.is_active());
                let farming_on_this_map = game_state.idle_farm_session.as_ref()
                    .map_or(false, |s| s.map_id == map_id && s.is_active());

                if farming_on_this_map {
                    // VIEW BATTLE button (green, highlighted)
                    Rectangle::new(Point::new(button_x, button_y), Size::new(button_width as u32, button_height))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 180, 100)))
                        .draw(display)?;
                    Rectangle::new(Point::new(button_x, button_y), Size::new(button_width as u32, button_height))
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 220, 140), 3))
                        .draw(display)?;

                    draw_text(
                        display,
                        "VIEW BATTLE",
                        Point::new(button_x + 70, button_y + 35),
                        &FONT_10X20,
                        Rgb888::WHITE,
                    )?;
                } else if has_active_farming {
                    // STOP FARMING button (red, warning)
                    Rectangle::new(Point::new(button_x, button_y), Size::new(button_width as u32, button_height))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(180, 40, 40)))
                        .draw(display)?;
                    Rectangle::new(Point::new(button_x, button_y), Size::new(button_width as u32, button_height))
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(220, 60, 60), 3))
                        .draw(display)?;

                    draw_text(
                        display,
                        "STOP FARMING",
                        Point::new(button_x + 50, button_y + 35),
                        &FONT_10X20,
                        Rgb888::WHITE,
                    )?;
                } else {
                    // START FARMING button (normal green)
                    Rectangle::new(Point::new(button_x, button_y), Size::new(button_width as u32, button_height))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 120, 80)))
                        .draw(display)?;
                    Rectangle::new(Point::new(button_x, button_y), Size::new(button_width as u32, button_height))
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(60, 180, 100), 3))
                        .draw(display)?;

                    draw_text(
                        display,
                        "START FARMING",
                        Point::new(button_x + 60, button_y + 35),
                        &FONT_10X20,
                        Rgb888::WHITE,
                    )?;
                }
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
    // Farm duration selection modal (if open)
    else if game_state.farm_selection_open {
        if let Some(enemy) = &game_state.current_enemy {
            draw_farm_duration_selection(display, game_state, enemy)?;
        }
    }

    Ok(())
}

