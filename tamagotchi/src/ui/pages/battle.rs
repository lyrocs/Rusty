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

/// Draw the Battle (Whac-A-Mole) page
pub fn draw_battle_page<D>(
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
    // Only clear screen when needed (entering battle or state change)
    // During active gameplay, only clear if should_clear is true
    if should_clear || game_state.battle_state != BattleState::Playing {
        display.clear(COLOR_BG)?;
    }

    match game_state.battle_state {
        BattleState::Idle => {
            draw_text(
                display,
                "=== BATTLE ===",
                Point::new(85, 20),
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
            let sp_color = if game_state.hero.sp >= 20 {
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

            if game_state.hero.sp >= 20 {
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
                    "Cost: 20 SP",
                    Point::new(110, 230),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "Duration: 30 seconds",
                    Point::new(75, 250),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
            } else {
                draw_text(
                    display,
                    "NOT ENOUGH SP!",
                    Point::new(75, 150),
                    &FONT_10X20,
                    COLOR_HP,
                )?;
                let mut needed_str = String::<32>::new();
                write!(needed_str, "Need {} more SP", 20 - game_state.hero.sp).ok();
                draw_text(
                    display,
                    &needed_str,
                    Point::new(90, 185),
                    &FONT_9X18_BOLD,
                    COLOR_HP,
                )?;
                draw_text(
                    display,
                    "Go to Rest page to",
                    Point::new(75, 225),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "recover SP",
                    Point::new(115, 248),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT_DIM,
                )?;
            }

            draw_text(
                display,
                "Press BOOT for Menu",
                Point::new(90, 420),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
        BattleState::Playing => {
            if let Some(enemy) = &game_state.battle_enemy {
                // Enemy name and level at top
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "{} Lv.{}", enemy.name, enemy.level).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(100, 60),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Enemy HP bar
                draw_bar(
                    display,
                    Point::new(60, 100),
                    250,
                    enemy.hp_percent(),
                    COLOR_HP,
                )?;

                // No GIF animations during manual battle for better gameplay focus

                // Timer (top right)
                let remaining_sec = (game_state.battle_duration - game_state.battle_elapsed) / 1000;
                let mut time_str = String::<16>::new();
                write!(time_str, "{}s", remaining_sec).ok();
                draw_text(
                    display,
                    &time_str,
                    Point::new(315, 20),
                    &FONT_10X20,
                    Rgb888::YELLOW,
                )?;

                // Score and Combo (top area - no GIF during gameplay for performance)
                let mut score_str = String::<48>::new();
                write!(
                    score_str,
                    "Hits:{} Miss:{} x{}",
                    game_state.battle_score, game_state.battle_missed, game_state.battle_combo
                )
                .ok();
                draw_text(
                    display,
                    &score_str,
                    Point::new(45, 140),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                // Draw all active circles
                for circle in &game_state.battle_circles {
                    if let Some(c) = circle {
                        let color = match c.circle_type {
                            CircleType::GoodTarget => Rgb888::GREEN,
                            CircleType::BadTarget => Rgb888::RED,
                        };

                        // Draw only colored border (no fill)
                        EgCircle::new(
                            Point::new(c.x - c.radius as i32, c.y - c.radius as i32),
                            (c.radius * 2) as u32,
                        )
                        .into_styled(PrimitiveStyle::with_stroke(color, 3))
                        .draw(display)?;
                    }
                }

                // Draw touch indicator cross (shows for 500ms after touch)
                if game_state.battle_last_touch_time > 0 {
                    let time_since_touch = game_state
                        .last_update_ms
                        .saturating_sub(game_state.battle_last_touch_time);
                    if time_since_touch < 500 {
                        let tx = game_state.battle_last_touch_x;
                        let ty = game_state.battle_last_touch_y;
                        let cross_size = 10;

                        // Draw white cross at touch position
                        Line::new(
                            Point::new(tx - cross_size, ty),
                            Point::new(tx + cross_size, ty),
                        )
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 3))
                        .draw(display)?;

                        Line::new(
                            Point::new(tx, ty - cross_size),
                            Point::new(tx, ty + cross_size),
                        )
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 3))
                        .draw(display)?;
                    }
                }

                // Instructions at bottom
                draw_text(
                    display,
                    "Green: Hit  Red: Block",
                    Point::new(60, 395),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                // FPS display at bottom
                draw_fps_info(display, Point::new(10, 415), fps)?;
            }
        }
        BattleState::Victory => {
            draw_text(
                display,
                "=== VICTORY! ===",
                Point::new(75, 60),
                &FONT_10X20,
                COLOR_TEXT,
            )?;

            if let Some(enemy) = &game_state.battle_enemy {
                // Draw dying monster GIF animation (centered)
                draw_monster_gif(display, game_state, Point::new(120, 110), enemy.name)?;

                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "Defeated {}", enemy.name).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(85, 220),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Score
                let mut score_str = String::<32>::new();
                write!(score_str, "Hits: {}", game_state.battle_score).ok();
                draw_text(
                    display,
                    &score_str,
                    Point::new(110, 250),
                    &FONT_9X15,
                    COLOR_TEXT,
                )?;

                draw_text(
                    display,
                    "Rewards:",
                    Point::new(120, 285),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                // Display actual rewards earned (with level penalty and score multiplier)
                let mut exp_str = String::<32>::new();
                write!(exp_str, "+{} EXP", game_state.last_battle_exp).ok();
                draw_text(
                    display,
                    &exp_str,
                    Point::new(105, 310),
                    &FONT_9X15,
                    COLOR_EXP,
                )?;

                let mut zeny_str = String::<32>::new();
                write!(zeny_str, "+{} Zeny", game_state.last_battle_zeny).ok();
                draw_text(
                    display,
                    &zeny_str,
                    Point::new(105, 330),
                    &FONT_9X15,
                    COLOR_EXP,
                )?;

                // Display loot if any
                if !game_state.last_drops.is_empty() {
                    draw_text(
                        display,
                        "Items:",
                        Point::new(140, 360),
                        &FONT_9X15,
                        Rgb888::YELLOW,
                    )?;

                    let mut y = 380;
                    for (_, item_name, quantity) in &game_state.last_drops {
                        let mut item_str = String::<40>::new();
                        write!(item_str, "{} x{}", item_name, quantity).ok();
                        draw_text(
                            display,
                            &item_str,
                            Point::new(80, y),
                            &FONT_9X15,
                            Rgb888::YELLOW,
                        )?;
                        y += 18;
                        if y > 410 {
                            break; // Don't overflow screen
                        }
                    }
                }
            }

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 420),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
        BattleState::Defeat => {
            draw_text(
                display,
                "=== DEFEATED ===",
                Point::new(75, 150),
                &FONT_10X20,
                COLOR_HP,
            )?;

            // Score
            let mut score_str = String::<32>::new();
            write!(score_str, "Hits: {}", game_state.battle_score).ok();
            draw_text(
                display,
                &score_str,
                Point::new(110, 220),
                &FONT_9X18_BOLD,
                COLOR_TEXT,
            )?;

            let mut missed_str = String::<32>::new();
            write!(missed_str, "Missed: {}", game_state.battle_missed).ok();
            draw_text(
                display,
                &missed_str,
                Point::new(95, 250),
                &FONT_9X18_BOLD,
                COLOR_HP,
            )?;

            draw_text(
                display,
                "You were defeated!",
                Point::new(70, 300),
                &FONT_9X18_BOLD,
                COLOR_HP,
            )?;
            draw_text(
                display,
                "No rewards",
                Point::new(110, 330),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 420),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    Ok(())
}

