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

/// Draw the Farm page with enemy and progress
pub fn draw_farm_page<D>(
    display: &mut D,
    game_state: &GameState,
    _battery_mv: u16,
    _battery_pct: u8,
    fps: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    match game_state.farm_state {
        FarmState::Idle => {
            draw_text(
                display,
                "=== AUTO FARM ===",
                Point::new(70, 20),
                &FONT_10X20,
                COLOR_TEXT,
            )?;

            // SP display with color coding
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

            // SP bar
            draw_bar(
                display,
                Point::new(20, 78),
                328,
                game_state.hero.sp_percent(),
                sp_color,
            )?;

            // Check if user has enough SP
            if game_state.hero.sp >= 20 {
                // Enough SP - show normal instructions
                draw_text(
                    display,
                    "Touch screen to",
                    Point::new(90, 200),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;
                draw_text(
                    display,
                    "start farming",
                    Point::new(95, 225),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                draw_text(
                    display,
                    "Cost: 20 SP",
                    Point::new(110, 280),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "Duration: 1 minute",
                    Point::new(90, 300),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
            } else {
                // Not enough SP - show warning
                draw_text(
                    display,
                    "NOT ENOUGH SP!",
                    Point::new(75, 180),
                    &FONT_10X20,
                    COLOR_HP,
                )?;

                let mut needed_str = String::<32>::new();
                write!(needed_str, "Need {} more SP", 20 - game_state.hero.sp).ok();
                draw_text(
                    display,
                    &needed_str,
                    Point::new(90, 215),
                    &FONT_9X18_BOLD,
                    COLOR_HP,
                )?;

                draw_text(
                    display,
                    "Go to Rest page to",
                    Point::new(75, 265),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "recover SP",
                    Point::new(115, 288),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT_DIM,
                )?;
            }

            draw_text(
                display,
                "Press BOOT for Menu",
                Point::new(90, 440),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
        FarmState::Fighting => {
            if let Some(enemy) = &game_state.current_enemy {
                draw_text(
                    display,
                    "=== FIGHTING ===",
                    Point::new(80, 20),
                    &FONT_10X20,
                    COLOR_TEXT,
                )?;

                // Enemy name
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "{} Lv.{}", enemy.name, enemy.level).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(100, 60),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Draw monster GIF animation (left side, closer to middle)
                draw_monster_gif(display, game_state, Point::new(110, 280), enemy.name)?;

                // Draw hero GIF animation (right side, closer to middle)
                draw_hero_gif(display, game_state, Point::new(250, 280))?;

                // Progress bar
                draw_text(
                    display,
                    "Progress",
                    Point::new(135, 330),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;
                draw_bar(
                    display,
                    Point::new(20, 355),
                    328,
                    game_state.farm_progress_percent(),
                    COLOR_EXP,
                )?;

                let mut time_str = String::<32>::new();
                let remaining_sec = (game_state.farm_duration_ms - game_state.farm_progress) / 1000;
                write!(time_str, "{}s remaining", remaining_sec).ok();
                draw_text(
                    display,
                    &time_str,
                    Point::new(120, 375),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                // Potential rewards
                let mut reward_str = String::<32>::new();
                write!(
                    reward_str,
                    "Rewards: EXP {} | Zeny {}",
                    enemy.base_exp, enemy.zeny_reward
                )
                .ok();
                draw_text(
                    display,
                    &reward_str,
                    Point::new(30, 405),
                    &FONT_9X15,
                    COLOR_EXP,
                )?;

                // FPS display at bottom
                draw_fps_info(display, Point::new(10, 425), fps)?;
            }
        }
        FarmState::Victory => {
            draw_text(
                display,
                "=== VICTORY! ===",
                Point::new(80, 20),
                &FONT_10X20,
                COLOR_TEXT,
            )?;

            if let Some(enemy) = &game_state.current_enemy {
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "Defeated {}", enemy.name).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(85, 60),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Draw dying monster GIF animation (centered)
                draw_monster_gif(display, game_state, Point::new(120, 110), enemy.name)?;

                draw_text(
                    display,
                    "Rewards:",
                    Point::new(130, 280),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                let mut exp_str = String::<32>::new();
                write!(exp_str, "+{} EXP", enemy.base_exp).ok();
                draw_text(
                    display,
                    &exp_str,
                    Point::new(115, 310),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                let mut zeny_str = String::<32>::new();
                write!(zeny_str, "+{} Zeny", enemy.zeny_reward).ok();
                draw_text(
                    display,
                    &zeny_str,
                    Point::new(115, 340),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                // Display loot if any
                if !game_state.last_drops.is_empty() {
                    draw_text(
                        display,
                        "Items:",
                        Point::new(140, 380),
                        &FONT_9X18_BOLD,
                        Rgb888::YELLOW,
                    )?;

                    let mut y = 410;
                    for (_, item_name, quantity) in &game_state.last_drops {
                        let mut item_str = String::<48>::new();
                        write!(item_str, "{} x{}", item_name, quantity).ok();
                        draw_text(
                            display,
                            &item_str,
                            Point::new(100, y),
                            &FONT_9X15,
                            Rgb888::YELLOW,
                        )?;
                        y += 20;
                        if y > 450 {
                            break; // Don't overflow screen
                        }
                    }
                } else {
                    draw_text(
                        display,
                        "No items dropped",
                        Point::new(90, 400),
                        &FONT_9X15,
                        COLOR_TEXT_DIM,
                    )?;
                }
            }

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 440),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
        FarmState::Defeat => {
            draw_text(
                display,
                "=== DEFEATED ===",
                Point::new(80, 100),
                &FONT_10X20,
                COLOR_HP,
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

