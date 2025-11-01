use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;

use super::super::helpers::*;
use crate::core::GameState;
use crate::combat::Enemy;

use super::super::colors::*;

/// Draw the Battle Overview page (live combat visualization)
pub fn draw_battle_overview_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    // Draw farming header at top
    use crate::ui::farming_header::draw_farming_header;
    draw_farming_header(display, game_state)?;

    // Check if there's an active farming session
    let session = match &game_state.idle_farm_session {
        Some(session) if session.is_active() => session,
        _ => {
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

    // === TOP: Enemy Info ===
    let enemy_name_y = 45;

    // Enemy name with level
    let mut enemy_label = String::<32>::new();
    write!(enemy_label, "{} Lv{}", enemy.name, enemy.level).ok();
    draw_text(
        display,
        &enemy_label,
        Point::new(20, enemy_name_y),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Enemy HP bar
    draw_text(
        display,
        "HP:",
        Point::new(20, enemy_name_y + 25),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    let enemy_hp_percent = if session.enemy_max_hp > 0 {
        (session.current_enemy_hp as u32 * 100) / session.enemy_max_hp as u32
    } else {
        0
    };

    let enemy_hp_color = if enemy_hp_percent > 50 {
        Rgb888::GREEN
    } else if enemy_hp_percent > 25 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    draw_bar(
        display,
        Point::new(60, enemy_name_y + 25),
        150,
        enemy_hp_percent as u8,
        enemy_hp_color,
    )?;

    // Enemy HP value
    let mut enemy_hp_str = String::<32>::new();
    write!(enemy_hp_str, "{}/{}", session.current_enemy_hp, session.enemy_max_hp).ok();
    draw_text(
        display,
        &enemy_hp_str,
        Point::new(220, enemy_name_y + 25),
        &FONT_9X15,
        enemy_hp_color,
    )?;

    // === CENTER: Battle Animations ===
    let battle_center_y = 150;

    // Enemy GIF (left side)
    if session.enemy_spawning {
        // Show "Spawning..." text during spawn delay
        draw_text(
            display,
            "Spawning...",
            Point::new(50, battle_center_y + 40),
            &FONT_9X18_BOLD,
            Rgb888::YELLOW,
        )?;
    } else if session.current_enemy_hp == 0 {
        // Show death message
        draw_text(
            display,
            "DEFEATED!",
            Point::new(50, battle_center_y + 40),
            &FONT_9X18_BOLD,
            Rgb888::GREEN,
        )?;
    } else {
        // Show monster GIF
        draw_monster_gif(
            display,
            game_state,
            Point::new(90, battle_center_y),
            enemy.name,
        )?;
    }

    // Hero GIF (right side)
    draw_hero_gif(display, game_state, Point::new(240, battle_center_y + 15))?;

    // === BOTTOM LEFT: Hero Info ===
    let hero_info_y = 280;

    // Hero name and level
    let mut hero_label = String::<32>::new();
    write!(hero_label, "{} Lv{}", game_state.hero.name, game_state.hero.level).ok();
    draw_text(
        display,
        &hero_label,
        Point::new(20, hero_info_y),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Hero HP
    draw_text(
        display,
        "HP:",
        Point::new(20, hero_info_y + 25),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    let hero_hp_percent = if game_state.hero.max_hp > 0 {
        (session.current_hp as u32 * 100) / game_state.hero.max_hp as u32
    } else {
        0
    };

    let hero_hp_color = if hero_hp_percent > 50 {
        Rgb888::GREEN
    } else if hero_hp_percent > 25 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    draw_bar(
        display,
        Point::new(60, hero_info_y + 25),
        130,
        hero_hp_percent as u8,
        hero_hp_color,
    )?;

    let mut hero_hp_str = String::<32>::new();
    write!(hero_hp_str, "{}/{}", session.current_hp, game_state.hero.max_hp).ok();
    draw_text(
        display,
        &hero_hp_str,
        Point::new(60, hero_info_y + 45),
        &FONT_9X15,
        hero_hp_color,
    )?;

    // === RIGHT SIDE: Session Stats Panel ===
    let stats_panel_x = 200;
    let stats_panel_y = 280;
    let stats_panel_width = 158;
    let stats_panel_height = 120;

    // Background panel
    Rectangle::new(
        Point::new(stats_panel_x, stats_panel_y),
        Size::new(stats_panel_width, stats_panel_height),
    )
    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
    .draw(display)?;

    Rectangle::new(
        Point::new(stats_panel_x, stats_panel_y),
        Size::new(stats_panel_width, stats_panel_height),
    )
    .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 2))
    .draw(display)?;

    // Session stats header
    draw_text(
        display,
        "Session Stats",
        Point::new(stats_panel_x + 10, stats_panel_y + 20),
        &FONT_9X15,
        COLOR_TEXT,
    )?;

    // Kills
    let mut kills_str = String::<32>::new();
    write!(kills_str, "Kills: {}", session.monsters_killed).ok();
    draw_text(
        display,
        &kills_str,
        Point::new(stats_panel_x + 10, stats_panel_y + 40),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Zeny
    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "Zeny: {}", session.zeny_earned).ok();
    draw_text(
        display,
        &zeny_str,
        Point::new(stats_panel_x + 10, stats_panel_y + 60),
        &FONT_9X15,
        Rgb888::YELLOW,
    )?;

    // Exp
    let mut exp_str = String::<32>::new();
    write!(exp_str, "Exp: {}", session.exp_gained).ok();
    draw_text(
        display,
        &exp_str,
        Point::new(stats_panel_x + 10, stats_panel_y + 80),
        &FONT_9X15,
        Rgb888::CYAN,
    )?;

    // Items
    let mut items_str = String::<32>::new();
    write!(items_str, "Items: {}", session.items_collected).ok();
    draw_text(
        display,
        &items_str,
        Point::new(stats_panel_x + 10, stats_panel_y + 100),
        &FONT_9X15,
        Rgb888::new(200, 150, 255),
    )?;

    // === BOTTOM: STOP FARMING Button ===
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

    Ok(())
}
