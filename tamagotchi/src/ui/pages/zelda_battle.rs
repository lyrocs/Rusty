use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle as EgCircle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;

use crate::core::GameState;
use crate::combat::ZeldaBattleState;
use super::super::helpers::*;
use super::super::colors::*;

/// Draw the Zelda Battle page (timing-based action battle)
pub fn draw_zelda_battle_page<D>(
    display: &mut D,
    game_state: &GameState,
    _battery_mv: u16,
    _battery_pct: u8,
    fps: u32,
    should_clear: bool,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear screen when needed
    if should_clear || game_state.zelda_battle_state != ZeldaBattleState::Playing {
        display.clear(COLOR_BG)?;
    }

    match game_state.zelda_battle_state {
        ZeldaBattleState::Idle => {
            draw_text(
                display,
                "=== ZELDA BATTLE ===",
                Point::new(60, 20),
                &FONT_10X20,
                COLOR_TEXT,
            )?;

            // SP display
            let mut sp_str = String::<32>::new();
            write!(
                sp_str,
                "SP: {}/{}",
                game_state.hero.sp, game_state.hero.max_sp
            )
            .ok();
            let sp_color = if game_state.hero.sp >= 5 {
                COLOR_SP
            } else {
                COLOR_HP
            };
            draw_text(
                display,
                &sp_str,
                Point::new(20, 60),
                &FONT_9X18_BOLD,
                sp_color,
            )?;
            draw_bar(
                display,
                Point::new(20, 78),
                328,
                game_state.hero.sp_percent(),
                sp_color,
            )?;

            if game_state.hero.sp >= 5 {
                draw_text(
                    display,
                    "Touch screen to",
                    Point::new(90, 160),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;
                draw_text(
                    display,
                    "start battle!",
                    Point::new(100, 185),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                draw_text(
                    display,
                    "Cost: 5 SP",
                    Point::new(120, 230),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                draw_text(
                    display,
                    "Tap enemies when they",
                    Point::new(75, 280),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "reach the center!",
                    Point::new(90, 300),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
            } else {
                draw_text(
                    display,
                    "Not enough SP!",
                    Point::new(95, 160),
                    &FONT_9X18_BOLD,
                    COLOR_HP,
                )?;
                draw_text(
                    display,
                    "Rest to recover SP",
                    Point::new(85, 200),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
            }

            // Back instruction
            draw_text(
                display,
                "Long press to go back",
                Point::new(70, 440),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }

        ZeldaBattleState::Playing => {
            // Draw game area background
            Rectangle::new(Point::new(0, 50), Size::new(368, 380))
                .into_styled(PrimitiveStyle::with_fill(COLOR_MENU_BG))
                .draw(display)?;

            // Draw hero in center (vertical line to show hero position)
            let hero_x = 184;
            let hero_y = 240;

            // Draw hero GIF in center
            draw_hero_gif(display, game_state, Point::new(hero_x, hero_y))?;

            // Draw hit zone indicator (circle around hero)
            let hit_zone_radius = 50;
            EgCircle::new(
                Point::new(hero_x - hit_zone_radius, hero_y - hit_zone_radius),
                (hit_zone_radius * 2) as u32,
            )
            .into_styled(PrimitiveStyle::with_stroke(
                Rgb888::new(255, 255, 100),
                2,
            ))
            .draw(display)?;

            // Draw all active enemies
            if let Some(ref battle_enemy) = game_state.zelda_battle_enemy {
                for enemy_slot in &game_state.zelda_battle_enemies {
                    if let Some(enemy) = enemy_slot {
                        if !enemy.is_hit {
                            // Draw enemy monster GIF at its position
                            draw_monster_gif(
                                display,
                                game_state,
                                Point::new(enemy.x, enemy.y),
                                battle_enemy.name,
                            )?;

                            // Draw enemy HP bar above it
                            let hp_percent = (enemy.hp as f32 / enemy.max_hp as f32 * 100.0) as u8;
                            let bar_width: u32 = 40;
                            draw_bar(
                                display,
                                Point::new(enemy.x - (bar_width as i32) / 2, enemy.y - 40),
                                bar_width,
                                hp_percent,
                                COLOR_HP,
                            )?;

                            // Highlight if in hit zone
                            if enemy.is_in_hit_zone {
                                EgCircle::new(
                                    Point::new(enemy.x - 20, enemy.y - 20),
                                    40,
                                )
                                .into_styled(PrimitiveStyle::with_stroke(
                                    Rgb888::new(0, 255, 0),
                                    3,
                                ))
                                .draw(display)?;
                            }
                        }
                    }
                }
            }

            // Top bar with stats
            Rectangle::new(Point::new(0, 0), Size::new(368, 50))
                .into_styled(PrimitiveStyle::with_fill(COLOR_BG))
                .draw(display)?;

            // Hero HP bar
            let mut hp_str = String::<32>::new();
            write!(hp_str, "HP:{}/{}", game_state.hero.hp, game_state.hero.max_hp).ok();
            draw_text(display, &hp_str, Point::new(10, 15), &FONT_9X15, COLOR_HP)?;
            draw_bar(
                display,
                Point::new(10, 25),
                100,
                game_state.hero.hp_percent(),
                COLOR_HP,
            )?;

            // Score and combo
            let mut score_str = String::<32>::new();
            write!(score_str, "Score:{}", game_state.zelda_battle_score).ok();
            draw_text(display, &score_str, Point::new(130, 15), &FONT_9X15, COLOR_TEXT)?;

            if game_state.zelda_battle_combo > 0 {
                let mut combo_str = String::<32>::new();
                write!(combo_str, "x{}", game_state.zelda_battle_combo).ok();
                draw_text(display, &combo_str, Point::new(220, 15), &FONT_9X15, Rgb888::new(255, 200, 0))?;
            }

            // Timer
            let elapsed_sec = game_state.zelda_battle_elapsed / 1000;
            let remaining_sec = (game_state.zelda_battle_duration / 1000).saturating_sub(elapsed_sec);
            let mut time_str = String::<16>::new();
            write!(time_str, "{}s", remaining_sec).ok();
            draw_text(display, &time_str, Point::new(300, 15), &FONT_9X18_BOLD, COLOR_TEXT)?;

            // Main enemy HP (bottom bar)
            if let Some(ref enemy) = game_state.zelda_battle_enemy {
                Rectangle::new(Point::new(0, 430), Size::new(368, 50))
                    .into_styled(PrimitiveStyle::with_fill(COLOR_BG))
                    .draw(display)?;

                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "{}", enemy.name).ok();
                draw_text(display, &enemy_str, Point::new(10, 445), &FONT_9X15, COLOR_TEXT)?;

                let enemy_hp_percent = (enemy.hp as f32 / enemy.max_hp as f32 * 100.0) as u8;
                draw_bar(
                    display,
                    Point::new(10, 460),
                    348,
                    enemy_hp_percent,
                    COLOR_HP,
                )?;
            }

            // FPS (debug)
            let mut fps_str = String::<16>::new();
            write!(fps_str, "FPS:{}", fps).ok();
            draw_text(display, &fps_str, Point::new(300, 35), &FONT_9X15, COLOR_TEXT_DIM)?;
        }

        ZeldaBattleState::Victory => {
            draw_text(
                display,
                "=== VICTORY! ===",
                Point::new(80, 100),
                &FONT_10X20,
                Rgb888::new(0, 255, 0),
            )?;

            // Display rewards
            let mut exp_str = String::<32>::new();
            write!(exp_str, "EXP: +{}", game_state.last_battle_exp).ok();
            draw_text(display, &exp_str, Point::new(100, 150), &FONT_9X18_BOLD, COLOR_TEXT)?;

            let mut zeny_str = String::<32>::new();
            write!(zeny_str, "Zeny: +{}", game_state.last_battle_zeny).ok();
            draw_text(display, &zeny_str, Point::new(100, 180), &FONT_9X18_BOLD, Rgb888::new(255, 200, 0))?;

            let mut score_str = String::<32>::new();
            write!(score_str, "Score: {}", game_state.zelda_battle_score).ok();
            draw_text(display, &score_str, Point::new(100, 210), &FONT_9X18_BOLD, COLOR_TEXT)?;

            let mut combo_str = String::<32>::new();
            write!(combo_str, "Max Combo: x{}", game_state.zelda_battle_combo).ok();
            draw_text(display, &combo_str, Point::new(100, 240), &FONT_9X18_BOLD, Rgb888::new(255, 200, 0))?;

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 350),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }

        ZeldaBattleState::Defeat => {
            draw_text(
                display,
                "=== DEFEAT ===",
                Point::new(95, 100),
                &FONT_10X20,
                COLOR_HP,
            )?;

            let mut score_str = String::<32>::new();
            write!(score_str, "Score: {}", game_state.zelda_battle_score).ok();
            draw_text(display, &score_str, Point::new(100, 180), &FONT_9X18_BOLD, COLOR_TEXT)?;

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 350),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    Ok(())
}
