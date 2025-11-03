use core::fmt::Write;
use embedded_graphics::{
    mono_font::ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;

use super::super::helpers::*;
use super::battle_overview_zones::*;
use crate::combat::Enemy;
use crate::core::GameState;

use super::super::colors::*;

/// Draw the Battle Overview page (live combat visualization) with zone-based rendering
/// Only re-renders zones when their data changes
pub fn draw_battle_overview_page<D>(
    display: &mut D,
    game_state: &mut GameState,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Check if there's an active farming session
    let session = match &game_state.idle_farm_session {
        Some(session) if session.is_active() => session,
        _ => {
            // No active session - full clear and show message
            display.clear(COLOR_BG)?;
            draw_text(
                display,
                "No active battle!",
                Point::new(80, 224),
                &FONT_10X20,
                Rgb888::RED,
            )?;
            return Ok(());
        }
    };

    // Get enemy info
    let enemy = match Enemy::from_id(session.enemy_id) {
        Some(e) => e,
        None => {
            display.clear(COLOR_BG)?;
            draw_text(
                display,
                "Enemy not found!",
                Point::new(80, 224),
                &FONT_10X20,
                Rgb888::RED,
            )?;
            return Ok(());
        }
    };

    // Force full redraw on first render or page change
    if game_state.battle_overview_needs_full_redraw {
        // Draw static elements that don't change
        display.clear(COLOR_BG)?;

        // Draw farming header at top
        use crate::ui::farming_header::draw_farming_header;
        draw_farming_header(display, game_state)?;

        // Draw STOP FARMING button
        let button_y = 410;
        let button_width = 330;
        let button_height = 60;
        let button_x = (368 - button_width) / 2;

        Rectangle::new(
            Point::new(button_x, button_y),
            Size::new(button_width as u32, button_height),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(180, 40, 40)))
        .draw(display)?;

        Rectangle::new(
            Point::new(button_x, button_y),
            Size::new(button_width as u32, button_height),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(220, 60, 60), 3))
        .draw(display)?;

        draw_text(
            display,
            "STOP FARMING",
            Point::new(button_x + 50, button_y + 35),
            &FONT_10X20,
            Rgb888::WHITE,
        )?;

        // Draw all zones on first render
        draw_enemy_info_zone(
            display,
            &enemy,
            session.current_enemy_hp,
            session.enemy_max_hp,
        )?;
        draw_hero_info_zone(
            display,
            &game_state.hero.name,
            game_state.hero.level,
            session.current_hp,
            game_state.hero.max_hp,
        )?;
        draw_stats_panel_zone(display, &session)?;

        // Update previous values
        game_state.battle_overview_prev_enemy_hp = session.current_enemy_hp;
        game_state.battle_overview_prev_hero_hp = session.current_hp;
        game_state.battle_overview_prev_kills = session.monsters_killed;
        game_state.battle_overview_prev_zeny = session.zeny_earned;
        game_state.battle_overview_prev_exp = session.exp_gained;
        game_state.battle_overview_prev_items = session.items_collected;
        game_state.battle_overview_needs_full_redraw = false;
    } else {
        // Incremental rendering - only redraw zones that changed

        // Check enemy HP zone
        if game_state.battle_overview_prev_enemy_hp != session.current_enemy_hp {
            draw_enemy_info_zone(
                display,
                &enemy,
                session.current_enemy_hp,
                session.enemy_max_hp,
            )?;
            game_state.battle_overview_prev_enemy_hp = session.current_enemy_hp;
        }

        // Check hero HP zone
        if game_state.battle_overview_prev_hero_hp != session.current_hp {
            draw_hero_info_zone(
                display,
                &game_state.hero.name,
                game_state.hero.level,
                session.current_hp,
                game_state.hero.max_hp,
            )?;
            game_state.battle_overview_prev_hero_hp = session.current_hp;
        }

        // Check stats panel zone
        let stats_changed = game_state.battle_overview_prev_kills != session.monsters_killed
            || game_state.battle_overview_prev_zeny != session.zeny_earned
            || game_state.battle_overview_prev_exp != session.exp_gained
            || game_state.battle_overview_prev_items != session.items_collected;

        if stats_changed {
            draw_stats_panel_zone(display, &session)?;
            game_state.battle_overview_prev_kills = session.monsters_killed;
            game_state.battle_overview_prev_zeny = session.zeny_earned;
            game_state.battle_overview_prev_exp = session.exp_gained;
            game_state.battle_overview_prev_items = session.items_collected;
        }
    }

    // === CENTER: Battle Animations (always re-rendered for smooth animation) ===
    let battle_center_y = 200;

    // Enemy GIF (left side) - position changes during spawn animation
    let enemy_x = if session.enemy_spawning {
        session.enemy_spawn_position_x // Animated position during walk-in
    } else {
        90 // Normal battle position
    };

    // clean monster zone (largest to handle all animations)
    Rectangle::new(Point::new(0, 120), Size::new(200, 150))
        .into_styled(PrimitiveStyle::with_fill(COLOR_BG))
        .draw(display)?;
    // Draw enemy at appropriate position
    if session.enemy_dying {
        // Show death animation with DEFEATED message
        draw_monster_gif_with_animation(
            display,
            game_state,
            Point::new(90, 200), // Always at normal position for death
            enemy.name,
            game_state.monster_animation,
        )?;
    } else {
        // Show monster GIF with animation (walk-in during spawn, normal otherwise)
        draw_monster_gif_with_animation(
            display,
            game_state,
            Point::new(enemy_x, 200),
            enemy.name,
            game_state.monster_animation,
        )?;
    }

    // clean hero zone (largest to handle all animations)
    Rectangle::new(Point::new(180, 120), Size::new(120, 150))
        .into_styled(PrimitiveStyle::with_fill(COLOR_BG))
        .draw(display)?;

    // Hero GIF (right side) with animation (controlled by update system)
    // draw_hero_gif(display, game_state, Point::new(240, battle_center_y + 15))?;
    draw_hero_gif_with_animation(
        display,
        game_state,
        Point::new(240, 250),
        game_state.hero_animation,
    )?;

    // === DAMAGE/MISS DISPLAY WITH ANIMATIONS ===
    let current_time = game_state.last_update_ms;
    const DAMAGE_DISPLAY_DURATION_MS: u32 = 1000; // Show damage for 1 second
    const FADE_START_MS: u32 = 800; // Start fading after 800ms

    // Show damage/miss on enemy (when hero damage is applied)
    let hero_damage_elapsed = current_time.saturating_sub(session.hero_damage_apply_ms);
    if hero_damage_elapsed < DAMAGE_DISPLAY_DURATION_MS
        && !session.enemy_spawning
        && !session.enemy_dying
        && !session.hero_attack_pending
    {
        // Calculate animation progress using integer math (0 to 1000 for precision)
        // Avoid division by zero
        let progress_1000 = if DAMAGE_DISPLAY_DURATION_MS > 0 {
            (hero_damage_elapsed * 1000) / DAMAGE_DISPLAY_DURATION_MS
        } else {
            1000
        };

        // Float upward: starts at -30, moves up by 40 pixels over duration
        // Use integer math: (progress_1000 * 40) / 1000 = pixels to move
        let float_offset = (progress_1000 * 40) / 1000;
        let animated_y = battle_center_y - 30 - float_offset as i32;

        // Simple fade: just hide after fade start time (avoids expensive alpha blending)
        let should_show = hero_damage_elapsed < FADE_START_MS;

        if should_show {
            if session.hero_attack_missed {
                // MISS text
                draw_text(
                    display,
                    "MISS",
                    Point::new(70, animated_y),
                    &FONT_9X18_BOLD,
                    Rgb888::new(150, 150, 150),
                )?;
            } else if session.last_hero_damage > 0 {
                let mut damage_text = String::<16>::new();
                if session.last_skill_used {
                    // Show damage number on enemy
                    write!(damage_text, "-{}", session.last_hero_damage).ok();
                    draw_text(
                        display,
                        &damage_text,
                        Point::new(60, animated_y),
                        &FONT_9X18_BOLD,
                        Rgb888::new(255, 200, 0),
                    )?;

                    // Show skill name above hero animation
                    draw_text(
                        display,
                        "BASH",
                        Point::new(200, 150),
                        &FONT_9X18_BOLD,
                        Rgb888::new(255, 255, 255),
                    )?;
                } else {
                    write!(damage_text, "-{}", session.last_hero_damage).ok();
                    draw_text(
                        display,
                        &damage_text,
                        Point::new(60, animated_y),
                        &FONT_9X18_BOLD,
                        Rgb888::RED,
                    )?;
                }
            }
        }
    }

    // Show damage/miss on hero (when enemy damage is applied)
    let enemy_damage_elapsed = current_time.saturating_sub(session.enemy_damage_apply_ms);
    if enemy_damage_elapsed < DAMAGE_DISPLAY_DURATION_MS
        && !session.enemy_spawning
        && !session.enemy_dying
        && !session.enemy_attack_pending
    {
        // Calculate animation progress using integer math (0 to 1000 for precision)
        let progress_1000 = if DAMAGE_DISPLAY_DURATION_MS > 0 {
            (enemy_damage_elapsed * 1000) / DAMAGE_DISPLAY_DURATION_MS
        } else {
            1000
        };

        // Float upward: starts at -15, moves up by 40 pixels over duration
        let float_offset = (progress_1000 * 40) / 1000;
        let animated_y = battle_center_y - 15 - float_offset as i32;

        // Simple fade: just hide after fade start time
        let should_show = enemy_damage_elapsed < FADE_START_MS;

        if should_show {
            if session.enemy_attack_missed {
                // MISS text
                draw_text(
                    display,
                    "MISS",
                    Point::new(220, animated_y),
                    &FONT_9X18_BOLD,
                    Rgb888::new(150, 150, 150),
                )?;
            } else if session.last_enemy_damage > 0 {
                let mut damage_text = String::<16>::new();
                write!(damage_text, "-{}", session.last_enemy_damage).ok();
                draw_text(
                    display,
                    &damage_text,
                    Point::new(220, animated_y),
                    &FONT_9X18_BOLD,
                    Rgb888::new(255, 100, 100),
                )?;
            }
        }
    }

    Ok(())
}
