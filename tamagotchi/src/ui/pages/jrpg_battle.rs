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

use super::super::helpers::*;
use crate::core::GameState;
use crate::tamagotchi::models::{
    BattleState, CircleType, Enemy, FarmState, LocationType, MapHelper, RestState,
};

use super::super::colors::*;

/// Draw JRPG turn-based battle page
pub fn draw_jrpg_battle_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::tamagotchi::models::{CombatResult, JrpgBattleMenu, JrpgBattleState};

    display.clear(COLOR_BG)?;

    // Get combatants
    let hero = game_state.jrpg_hero_combatant.as_ref();
    let enemy = game_state.jrpg_enemy_combatant.as_ref();

    if hero.is_none() || enemy.is_none() {
        draw_text(
            display,
            "Battle Error!",
            Point::new(100, 224),
            &FONT_10X20,
            Rgb888::RED,
        )?;
        return Ok(());
    }

    let hero = hero.unwrap();
    let enemy = enemy.unwrap();

    // === TOP: Enemy Info ===
    // Enemy name
    draw_text(
        display,
        enemy.name,
        Point::new(20, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Enemy level
    let mut enemy_level_str = String::<16>::new();
    write!(enemy_level_str, "Lv.{}", enemy.level).ok();
    draw_text(
        display,
        &enemy_level_str,
        Point::new(20, 45),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Enemy HP bar
    draw_text(
        display,
        "HP:",
        Point::new(140, 45),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    let enemy_hp_percent = (enemy.hp as u32 * 100) / enemy.max_hp as u32;
    let enemy_hp_color = if enemy_hp_percent > 50 {
        Rgb888::GREEN
    } else if enemy_hp_percent > 25 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };
    draw_bar(
        display,
        Point::new(180, 45),
        150,
        enemy_hp_percent as u8,
        enemy_hp_color,
    )?;

    // Enemy HP value
    let mut enemy_hp_str = String::<32>::new();
    write!(enemy_hp_str, "{}/{}", enemy.hp, enemy.max_hp).ok();
    draw_text(
        display,
        &enemy_hp_str,
        Point::new(180, 65),
        &FONT_9X15,
        enemy_hp_color,
    )?;

    // === CENTER: Battle GIFs ===
    // Draw enemy GIF (left side, moved right a bit)
    draw_monster_gif(display, game_state, Point::new(100, 150), enemy.name)?;

    // Draw hero GIF (right side, moved down a bit)
    draw_hero_gif(display, game_state, Point::new(240, 165))?;

    // === BOTTOM: Hero Info ===
    // Hero name and level
    let mut hero_info = String::<32>::new();
    write!(hero_info, "{} Lv.{}", hero.name, hero.level).ok();
    draw_text(
        display,
        &hero_info,
        Point::new(20, 250),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Hero HP
    draw_text(
        display,
        "HP:",
        Point::new(20, 275),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    let hero_hp_percent = (hero.hp as u32 * 100) / hero.max_hp as u32;
    let hero_hp_color = if hero_hp_percent > 50 {
        Rgb888::GREEN
    } else if hero_hp_percent > 25 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };
    draw_bar(
        display,
        Point::new(60, 275),
        130,
        hero_hp_percent as u8,
        hero_hp_color,
    )?;

    let mut hero_hp_str = String::<32>::new();
    write!(hero_hp_str, "{}/{}", hero.hp, hero.max_hp).ok();
    draw_text(
        display,
        &hero_hp_str,
        Point::new(60, 295),
        &FONT_9X15,
        hero_hp_color,
    )?;

    // Hero SP
    draw_text(
        display,
        "SP:",
        Point::new(200, 275),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    let hero_sp_percent = (hero.sp as u32 * 100) / hero.max_sp as u32;
    draw_bar(
        display,
        Point::new(240, 275),
        110,
        hero_sp_percent as u8,
        Rgb888::CYAN,
    )?;

    let mut hero_sp_str = String::<32>::new();
    write!(hero_sp_str, "{}/{}", hero.sp, hero.max_sp).ok();
    draw_text(
        display,
        &hero_sp_str,
        Point::new(240, 295),
        &FONT_9X15,
        Rgb888::CYAN,
    )?;

    // === Battle Message (if any) ===
    if let Some(msg) = game_state.jrpg_battle_message {
        // Message box background
        Rectangle::new(Point::new(60, 105), Size::new(248, 35))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 60)))
            .draw(display)?;
        Rectangle::new(Point::new(60, 105), Size::new(248, 35))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 2))
            .draw(display)?;

        // Message text (centered)
        let text_x = 184 - ((msg.len() as i32 * 9) / 2); // Center text
        draw_text(
            display,
            msg,
            Point::new(text_x, 115),
            &FONT_9X18_BOLD,
            Rgb888::WHITE,
        )?;
    }

    // === Floating Damage Text Animation ===
    if game_state.jrpg_damage_animation_timer > 0 {
        // Calculate animation progress (0.0 to 1.0)
        let progress = 1.0 - (game_state.jrpg_damage_animation_timer as f32 / 1000.0);

        // Float up by 40 pixels over the animation
        let float_offset = (progress * 40.0) as i32;
        let damage_y = game_state.jrpg_damage_y - float_offset;

        // Fade out alpha (simulate with color brightness)
        let alpha_factor = 1.0 - progress;

        // Color and text based on combat result
        let (damage_color, prefix) = match game_state.jrpg_last_combat_result {
            CombatResult::Critical => {
                let red_value = (255.0 * alpha_factor) as u8;
                let yellow_value = (100.0 * alpha_factor) as u8;
                (Rgb888::new(red_value, yellow_value, 0), "CRIT! ")
            }
            CombatResult::Lucky => {
                let gold_value = (255.0 * alpha_factor) as u8;
                let yellow_value = (215.0 * alpha_factor) as u8;
                (Rgb888::new(gold_value, yellow_value, 0), "LUCKY! ")
            }
            CombatResult::Miss => {
                let gray_value = (150.0 * alpha_factor) as u8;
                (Rgb888::new(gray_value, gray_value, gray_value), "MISS!")
            }
            CombatResult::Normal => {
                let red_value = (255.0 * alpha_factor) as u8;
                (Rgb888::new(red_value, 0, 0), "")
            }
        };

        // Draw damage text
        let mut dmg_str = String::<24>::new();
        if game_state.jrpg_last_combat_result == CombatResult::Miss {
            write!(dmg_str, "{}", prefix).ok();
        } else {
            write!(dmg_str, "{}-{}", prefix, game_state.jrpg_damage_dealt).ok();
        }

        // Draw text centered on damage position
        let text_width = dmg_str.len() as i32 * 10; // FONT_10X20 width
        let text_x = game_state.jrpg_damage_x - (text_width / 2);
        draw_text(
            display,
            &dmg_str,
            Point::new(text_x, damage_y),
            &FONT_10X20,
            damage_color,
        )?;
    }

    // === FPS Display (Top Right Corner) ===
    if game_state.fps > 0 {
        let mut fps_str = String::<16>::new();
        write!(fps_str, "FPS: {}", game_state.fps).ok();
        draw_text(
            display,
            &fps_str,
            Point::new(260, 20),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // === Action Menu (during player turn) ===
    if game_state.jrpg_battle_state == JrpgBattleState::PlayerTurn {
        // Draw action buttons: Attack + 3 Skills in a single row
        if let Some(hero) = &game_state.jrpg_hero_combatant {
            let button_height = 60;
            let spacing_x = 12;
            let start_x = 14;
            let start_y = 360;

            // Attack button (wider)
            let attack_width = 110;
            let skill_width = 66;
            let skill_height = 66;

            // Draw Attack button
            Rectangle::new(
                Point::new(start_x, start_y),
                Size::new(attack_width as u32, button_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 50, 50)))
            .draw(display)?;

            Rectangle::new(
                Point::new(start_x, start_y),
                Size::new(attack_width as u32, button_height as u32),
            )
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(200, 100, 100), 2))
            .draw(display)?;

            let text_x = start_x + (attack_width / 2) - 27; // "Attack" = 6 chars * 9/2
            let text_y = start_y + (button_height / 2);
            draw_text(
                display,
                "Attack",
                Point::new(text_x, text_y),
                &FONT_9X18_BOLD,
                Rgb888::WHITE,
            )?;

            // Draw skill buttons (up to 3)
            for (i, skill) in hero.available_skills.iter().enumerate().take(3) {
                let x = start_x + attack_width + spacing_x + i as i32 * (skill_width + spacing_x);

                // Skill icon GIF (centered in button)
                if let Some(icon_data) = crate::combat::get_skill_icon(skill.id) {
                    // Try to render GIF icon (single frame)
                    if let Ok(gif) = Gif::<Rgb888>::from_slice(icon_data) {
                        // Get GIF dimensions
                        let gif_width = gif.width() as i32;
                        let gif_height = gif.height() as i32;

                        // Center the icon in the button
                        let icon_x = x + (skill_width / 2) - (gif_width / 2);
                        let icon_y = start_y + 8;

                        // Draw first frame (skill icons are single-frame)
                        if let Some(frame) = gif.frames().next() {
                            Image::new(&frame, Point::new(icon_x, icon_y))
                                .draw(display)
                                .ok();
                        }
                    }
                }
            }
        }
    }

    // === Victory Modal ===
    if game_state.jrpg_battle_state == JrpgBattleState::Victory {
        // Semi-transparent overlay
        Rectangle::new(Point::new(0, 0), Size::new(360, 480))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        // Victory panel
        let panel_x = 30;
        let panel_y = 120;
        let panel_width = 300;
        let panel_height = 240;

        // Panel background
        Rectangle::new(
            Point::new(panel_x, panel_y),
            Size::new(panel_width, panel_height),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 60, 30)))
        .draw(display)?;

        // Panel border
        Rectangle::new(
            Point::new(panel_x, panel_y),
            Size::new(panel_width, panel_height),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 200, 100), 3))
        .draw(display)?;

        // Victory title
        draw_text(
            display,
            "VICTORY!",
            Point::new(panel_x + 80, panel_y + 30),
            &FONT_9X18_BOLD,
            Rgb888::new(255, 255, 100),
        )?;

        // Rewards display
        let rewards_y = panel_y + 60;
        draw_text(
            display,
            "Rewards:",
            Point::new(panel_x + 20, rewards_y),
            &FONT_9X15,
            Rgb888::new(100, 255, 100),
        )?;

        let mut exp_str = String::<32>::new();
        write!(exp_str, "+{} EXP", game_state.last_battle_exp).ok();
        draw_text(
            display,
            &exp_str,
            Point::new(panel_x + 40, rewards_y + 25),
            &FONT_9X15,
            Rgb888::new(150, 255, 150),
        )?;

        let mut zeny_str = String::<32>::new();
        write!(zeny_str, "+{} Zeny", game_state.last_battle_zeny).ok();
        draw_text(
            display,
            &zeny_str,
            Point::new(panel_x + 40, rewards_y + 50),
            &FONT_9X15,
            Rgb888::new(255, 255, 100),
        )?;

        // Items dropped header
        let items_y = panel_y + 140;
        draw_text(
            display,
            "Items Dropped:",
            Point::new(panel_x + 20, items_y),
            &FONT_9X15,
            Rgb888::WHITE,
        )?;

        // Display items
        if game_state.last_drops.is_empty() {
            draw_text(
                display,
                "No items dropped",
                Point::new(panel_x + 40, items_y + 30),
                &FONT_9X15,
                Rgb888::new(180, 180, 180),
            )?;
        } else {
            for (i, (_item_id, item_name, quantity)) in game_state.last_drops.iter().enumerate() {
                let item_y = items_y + 30 + (i as i32 * 25);
                let mut item_str = String::<48>::new();
                write!(item_str, "- {} x{}", item_name, quantity).ok();
                draw_text(
                    display,
                    &item_str,
                    Point::new(panel_x + 40, item_y),
                    &FONT_9X15,
                    Rgb888::new(200, 255, 200),
                )?;
            }
        }

        // Continue prompt
        let prompt_y = panel_y + panel_height as i32 - 40;
        draw_text(
            display,
            "Touch to continue",
            Point::new(panel_x + 60, prompt_y),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // === Defeat Modal ===
    if game_state.jrpg_battle_state == JrpgBattleState::Defeat {
        // Semi-transparent overlay
        Rectangle::new(Point::new(0, 0), Size::new(360, 480))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        // Defeat panel
        let panel_x = 30;
        let panel_y = 180;
        let panel_width = 300;
        let panel_height = 120;

        // Panel background
        Rectangle::new(
            Point::new(panel_x, panel_y),
            Size::new(panel_width, panel_height),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 30, 30)))
        .draw(display)?;

        // Panel border
        Rectangle::new(
            Point::new(panel_x, panel_y),
            Size::new(panel_width, panel_height),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(200, 100, 100), 3))
        .draw(display)?;

        // Defeat title
        draw_text(
            display,
            "DEFEAT",
            Point::new(panel_x + 100, panel_y + 40),
            &FONT_9X18_BOLD,
            Rgb888::RED,
        )?;

        // Continue prompt
        draw_text(
            display,
            "Touch to continue",
            Point::new(panel_x + 60, panel_y + 75),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    Ok(())
}
