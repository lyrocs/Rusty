use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;

use crate::core::GameState;
use crate::combat::{MvpBattlePhase, MvpBattleState};
use super::super::helpers::*;
use super::super::colors::*;

/// Draw the MVP Battle page (semi-active boss battle)
pub fn draw_mvp_battle_page<D>(
    display: &mut D,
    game_state: &GameState,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    match game_state.mvp_battle_state {
        MvpBattleState::Idle => {
            draw_text(
                display,
                "=== MVP BATTLE ===",
                Point::new(60, 100),
                &FONT_10X20,
                COLOR_TEXT,
            )?;
            draw_text(
                display,
                "Ready to fight!",
                Point::new(100, 200),
                &FONT_9X18_BOLD,
                COLOR_TEXT,
            )?;
        }
        MvpBattleState::Start => {
            // Just transition to Playing
            draw_text(
                display,
                "GET READY!",
                Point::new(100, 200),
                &FONT_10X20,
                COLOR_TEXT,
            )?;
        }
        MvpBattleState::Playing => {
            draw_mvp_battle_playing(display, game_state)?;
        }
        MvpBattleState::Victory => {
            draw_mvp_battle_result(display, game_state, true)?;
        }
        MvpBattleState::Defeat => {
            draw_mvp_battle_result(display, game_state, false)?;
        }
    }

    Ok(())
}

/// Draw the active battle screen
fn draw_mvp_battle_playing<D>(
    display: &mut D,
    game_state: &GameState,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Boss HP bar (top)
    if let Some(enemy) = &game_state.mvp_battle_enemy {
        // Boss name
        draw_text(
            display,
            enemy.name,
            Point::new(10, 20),
            &FONT_9X18_BOLD,
            Rgb888::new(255, 100, 100),
        )?;

        // Boss HP
        let mut boss_hp_str = String::<32>::new();
        write!(boss_hp_str, "HP: {}/{}", enemy.hp, enemy.max_hp).ok();
        draw_text(
            display,
            &boss_hp_str,
            Point::new(10, 45),
            &FONT_9X15,
            COLOR_HP,
        )?;

        // Boss HP bar
        draw_bar(
            display,
            Point::new(10, 58),
            348,
            enemy.hp_percent(),
            COLOR_HP,
        )?;

        // Phase indicator
        let phase_text = match game_state.mvp_battle_phase {
            MvpBattlePhase::Phase1 => "PHASE 1",
            MvpBattlePhase::Phase2 => "PHASE 2 - ENRAGED!",
            MvpBattlePhase::Phase3 => "PHASE 3 - BERSERK!",
        };
        let phase_color = match game_state.mvp_battle_phase {
            MvpBattlePhase::Phase1 => Rgb888::new(100, 200, 100),
            MvpBattlePhase::Phase2 => Rgb888::new(255, 200, 0),
            MvpBattlePhase::Phase3 => Rgb888::new(255, 50, 50),
        };
        draw_text(
            display,
            phase_text,
            Point::new(220, 20),
            &FONT_9X15,
            phase_color,
        )?;
    }

    // Hero HP bar
    let mut hero_hp_str = String::<32>::new();
    write!(
        hero_hp_str,
        "HERO HP: {}/{}",
        game_state.hero.hp, game_state.hero.max_hp
    )
    .ok();
    draw_text(
        display,
        &hero_hp_str,
        Point::new(10, 95),
        &FONT_9X15,
        COLOR_TEXT,
    )?;
    draw_bar(
        display,
        Point::new(10, 108),
        348,
        game_state.hero.hp_percent(),
        Rgb888::new(100, 255, 100),
    )?;

    // Stagger bar with critical window indicator
    draw_text(
        display,
        "STAGGER",
        Point::new(10, 140),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    let stagger_percent = (game_state.mvp_stagger_value as u32 * 100 / game_state.mvp_stagger_max as u32) as u8;
    let stagger_color = if game_state.mvp_critical_window_active {
        Rgb888::new(255, 215, 0) // Gold for critical window
    } else {
        Rgb888::new(150, 150, 255)
    };

    draw_bar(
        display,
        Point::new(10, 153),
        348,
        stagger_percent,
        stagger_color,
    )?;

    // Critical window flash
    if game_state.mvp_critical_window_active {
        draw_text(
            display,
            "*** CRITICAL WINDOW! ***",
            Point::new(50, 175),
            &FONT_9X18_BOLD,
            Rgb888::new(255, 215, 0),
        )?;
    }

    // Combat area message
    draw_text(
        display,
        "Auto-attacking...",
        Point::new(120, 210),
        &FONT_6X10,
        COLOR_TEXT_DIM,
    )?;

    // Skill buttons (3 buttons at bottom)
    let button_y = 350;
    let button_width = 110;
    let button_height = 65;
    let button_spacing = 8;
    let start_x = 11;

    let skills = [
        ("BASH", 0, Rgb888::new(255, 100, 100)),
        ("PROVOKE", 1, Rgb888::new(100, 100, 255)),
        ("POTION", 2, Rgb888::new(100, 255, 100)),
    ];

    for (i, (name, skill_index, color)) in skills.iter().enumerate() {
        let x = start_x + i as i32 * (button_width + button_spacing);
        let cooldown = &game_state.mvp_skill_cooldowns[*skill_index];
        let is_ready = cooldown.is_ready(game_state.last_update_ms);
        let progress = cooldown.progress(game_state.last_update_ms);

        // Button background (gray if on cooldown)
        let bg_color = if is_ready { COLOR_PANEL } else { Rgb888::new(50, 50, 50) };
        Rectangle::new(Point::new(x, button_y), Size::new(button_width as u32, button_height as u32))
            .into_styled(PrimitiveStyle::with_fill(bg_color))
            .draw(display)?;

        // Cooldown progress bar at bottom of button
        if !is_ready {
            let cd_bar_width = (button_width as f32 * progress) as u32;
            Rectangle::new(
                Point::new(x, button_y + button_height - 5),
                Size::new(cd_bar_width, 5),
            )
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(display)?;
        }

        // Button border
        Rectangle::new(Point::new(x, button_y), Size::new(button_width as u32, button_height as u32))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
            .draw(display)?;

        // Skill name
        let text_color = if is_ready { *color } else { COLOR_TEXT_DIM };
        let text_x = if name.len() > 6 { x + 8 } else { x + 20 };
        draw_text(
            display,
            name,
            Point::new(text_x, button_y + 25),
            &FONT_9X15,
            text_color,
        )?;

        // Cooldown time
        if !is_ready {
            let remaining_ms = cooldown.remaining_ms(game_state.last_update_ms);
            let remaining_s = (remaining_ms + 999) / 1000; // Round up
            let mut cd_str = String::<8>::new();
            write!(cd_str, "{}s", remaining_s).ok();
            draw_text(
                display,
                &cd_str,
                Point::new(x + 40, button_y + 45),
                &FONT_9X15,
                Rgb888::new(200, 200, 200),
            )?;
        } else {
            draw_text(
                display,
                "READY",
                Point::new(x + 30, button_y + 45),
                &FONT_6X10,
                Rgb888::new(100, 255, 100),
            )?;
        }
    }

    // Battle time
    let time_s = game_state.mvp_battle_elapsed / 1000;
    let mut time_str = String::<16>::new();
    write!(time_str, "Time: {}s", time_s).ok();
    draw_text(
        display,
        &time_str,
        Point::new(140, 330),
        &FONT_6X10,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Draw the battle result screen
fn draw_mvp_battle_result<D>(
    display: &mut D,
    game_state: &GameState,
    victory: bool,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Title
    if victory {
        draw_text(
            display,
            "=== VICTORY! ===",
            Point::new(70, 40),
            &FONT_10X20,
            Rgb888::new(255, 215, 0),
        )?;
    } else {
        draw_text(
            display,
            "=== DEFEAT ===",
            Point::new(80, 40),
            &FONT_10X20,
            COLOR_HP,
        )?;
    }

    if victory {
        // Rank
        if let Some(rank) = game_state.mvp_battle_rank {
            let mut rank_str = String::<32>::new();
            write!(rank_str, "RANK: {}", rank.display_name()).ok();
            draw_text(
                display,
                &rank_str,
                Point::new(120, 90),
                &FONT_10X20,
                Rgb888::new(255, 215, 0),
            )?;
        }

        // Battle stats
        let time_s = game_state.mvp_battle_elapsed / 1000;
        let mut time_str = String::<32>::new();
        write!(time_str, "Time: {}s", time_s).ok();
        draw_text(
            display,
            &time_str,
            Point::new(100, 130),
            &FONT_9X15,
            COLOR_TEXT,
        )?;

        let mut hits_str = String::<32>::new();
        write!(hits_str, "Perfect Hits: {}", game_state.mvp_perfect_hits).ok();
        draw_text(
            display,
            &hits_str,
            Point::new(70, 155),
            &FONT_9X15,
            COLOR_TEXT,
        )?;

        // Rewards
        draw_text(
            display,
            "REWARDS:",
            Point::new(130, 200),
            &FONT_9X18_BOLD,
            COLOR_TEXT,
        )?;

        let mut exp_str = String::<32>::new();
        write!(exp_str, "EXP: +{}", game_state.last_battle_exp).ok();
        draw_text(
            display,
            &exp_str,
            Point::new(100, 235),
            &FONT_9X15,
            Rgb888::new(100, 255, 255),
        )?;

        let mut zeny_str = String::<32>::new();
        write!(zeny_str, "Zeny: +{}", game_state.last_battle_zeny).ok();
        draw_text(
            display,
            &zeny_str,
            Point::new(100, 260),
            &FONT_9X15,
            Rgb888::new(255, 215, 0),
        )?;

        // Drops
        if !game_state.last_drops.is_empty() {
            draw_text(
                display,
                "DROPS:",
                Point::new(140, 295),
                &FONT_9X15,
                COLOR_TEXT,
            )?;

            for (i, (_item_id, item_name, quantity)) in game_state.last_drops.iter().enumerate() {
                let mut drop_str = String::<32>::new();
                write!(drop_str, "{} x{}", item_name, quantity).ok();
                draw_text(
                    display,
                    &drop_str,
                    Point::new(80, 320 + i as i32 * 20),
                    &FONT_9X15,
                    Rgb888::new(150, 255, 150),
                )?;
            }
        }
    }

    // Instructions
    draw_text(
        display,
        "Touch to continue",
        Point::new(80, 420),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}
