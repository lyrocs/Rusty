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

                // Enemy name with efficiency rating
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "{} Lv.{}", enemy.name, enemy.level).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(90, 60),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Efficiency rating (if available)
                if let Some(rating) = game_state.farm_efficiency_rating {
                    let rating_color = match rating {
                        crate::combat::EfficiencyRating::Excellent => Rgb888::new(100, 255, 100),
                        crate::combat::EfficiencyRating::Good => Rgb888::new(150, 255, 150),
                        crate::combat::EfficiencyRating::Fair => Rgb888::new(200, 200, 100),
                        crate::combat::EfficiencyRating::Risky => Rgb888::new(255, 150, 50),
                        crate::combat::EfficiencyRating::Impossible => Rgb888::new(255, 50, 50),
                    };

                    let mut rating_str = String::<16>::new();
                    write!(rating_str, "[{}]", rating.icon()).ok();
                    draw_text(
                        display,
                        &rating_str,
                        Point::new(90, 85),
                        &FONT_9X15,
                        rating_color,
                    )?;
                }

                // Kill counter (if efficiency system is active)
                if game_state.farm_expected_kills > 0 {
                    let mut kills_str = String::<24>::new();
                    write!(
                        kills_str,
                        "Kills: {}/{}",
                        game_state.farm_kills_count,
                        game_state.farm_expected_kills
                    ).ok();
                    draw_text(
                        display,
                        &kills_str,
                        Point::new(220, 85),
                        &FONT_9X15,
                        Rgb888::new(200, 200, 255),
                    )?;
                }

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

                // Expected total rewards (based on efficiency, level penalty, and 1/10 rate)
                if game_state.farm_expected_kills > 0 {
                    let mut reward_str = String::<48>::new();
                    let (total_exp, total_zeny) = crate::combat::calculate_farm_rewards(
                        enemy,
                        game_state.farm_expected_kills,
                        game_state.hero.level,
                    );
                    write!(
                        reward_str,
                        "Expected: ~{} EXP | ~{}z",
                        total_exp, total_zeny
                    )
                    .ok();
                    draw_text(
                        display,
                        &reward_str,
                        Point::new(40, 405),
                        &FONT_9X15,
                        COLOR_EXP,
                    )?;
                } else {
                    // Fallback for old system
                    let mut reward_str = String::<32>::new();
                    let (exp, zeny) = crate::combat::calculate_farm_rewards(
                        enemy,
                        1,
                        game_state.hero.level,
                    );
                    write!(
                        reward_str,
                        "Rewards: EXP {} | Zeny {}",
                        exp, zeny
                    )
                    .ok();
                    draw_text(
                        display,
                        &reward_str,
                        Point::new(30, 405),
                        &FONT_9X15,
                        COLOR_EXP,
                    )?;
                }

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
                // Calculate actual kills (for efficiency system)
                let actual_kills = if game_state.farm_expected_kills > 0 {
                    game_state.farm_kills_count
                } else {
                    1 // Fallback for old system
                };

                // Enemy name (30px higher: 60 - 30 = 30)
                let mut enemy_str = String::<48>::new();
                if actual_kills > 1 {
                    write!(enemy_str, "Defeated {} x{}", enemy.name, actual_kills).ok();
                } else {
                    write!(enemy_str, "Defeated {}", enemy.name).ok();
                }
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(70, 30),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Show efficiency rating if available (30px higher: 85 - 30 = 55)
                if let Some(rating) = game_state.farm_efficiency_rating {
                    let rating_color = match rating {
                        crate::combat::EfficiencyRating::Excellent => Rgb888::new(100, 255, 100),
                        crate::combat::EfficiencyRating::Good => Rgb888::new(150, 255, 150),
                        crate::combat::EfficiencyRating::Fair => Rgb888::new(200, 200, 100),
                        crate::combat::EfficiencyRating::Risky => Rgb888::new(255, 150, 50),
                        crate::combat::EfficiencyRating::Impossible => Rgb888::new(255, 50, 50),
                    };

                    let mut rating_str = String::<32>::new();
                    write!(rating_str, "[{}] {}", rating.icon(), rating.display_name()).ok();
                    draw_text(
                        display,
                        &rating_str,
                        Point::new(110, 55),
                        &FONT_9X15,
                        rating_color,
                    )?;
                }

                // Draw dying monster GIF animation
                // Centered on screen (x=152) and moved down 20px: (110-30)+20 = 100
                draw_monster_gif(display, game_state, Point::new(152, 100), enemy.name)?;

                // Two-column layout for rewards and items
                // Left column: Rewards (30px higher: 280 - 30 = 250)
                draw_text(
                    display,
                    "Rewards:",
                    Point::new(30, 250),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                // Calculate total rewards (with level penalty and 1/10 rate)
                let (total_exp, total_zeny) = crate::combat::calculate_farm_rewards(
                    enemy,
                    actual_kills,
                    game_state.hero.level,
                );

                let mut exp_str = String::<32>::new();
                write!(exp_str, "+{} EXP", total_exp).ok();
                draw_text(
                    display,
                    &exp_str,
                    Point::new(30, 280),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                let mut zeny_str = String::<32>::new();
                write!(zeny_str, "+{} Zeny", total_zeny).ok();
                draw_text(
                    display,
                    &zeny_str,
                    Point::new(30, 310),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                // Right column: Items (same y as Rewards header)
                draw_text(
                    display,
                    "Items:",
                    Point::new(200, 250),
                    &FONT_9X18_BOLD,
                    Rgb888::YELLOW,
                )?;

                // Display loot if any
                if !game_state.last_drops.is_empty() {
                    let mut y = 280;
                    for (_, item_name, quantity) in &game_state.last_drops {
                        let mut item_str = String::<32>::new();
                        write!(item_str, "{} x{}", item_name, quantity).ok();
                        draw_text(
                            display,
                            &item_str,
                            Point::new(200, y),
                            &FONT_9X15,
                            Rgb888::YELLOW,
                        )?;
                        y += 20;
                        if y > 370 {
                            break; // Don't overflow screen
                        }
                    }
                } else {
                    draw_text(
                        display,
                        "None",
                        Point::new(200, 280),
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

