use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle as EgCircle, Line, PrimitiveStyle, Rectangle},
    text::Text,
    image::Image,
};
use heapless::String;
use tinygif::Gif;

use crate::tamagotchi::models::{
    BattleState, CircleType, Enemy, FarmState, GameState, Hero, LocationType, MapHelper, RestState,
};

// Color palette inspired by Ragnarok Online
pub const COLOR_BG: Rgb888 = Rgb888::new(40, 40, 60);
pub const COLOR_PANEL: Rgb888 = Rgb888::new(60, 60, 80);
pub const COLOR_TEXT: Rgb888 = Rgb888::new(255, 255, 255);
pub const COLOR_TEXT_DIM: Rgb888 = Rgb888::new(180, 180, 200);
pub const COLOR_HP: Rgb888 = Rgb888::new(220, 50, 50);
pub const COLOR_SP: Rgb888 = Rgb888::new(50, 120, 220);
pub const COLOR_EXP: Rgb888 = Rgb888::new(255, 200, 50);
pub const COLOR_MENU_BG: Rgb888 = Rgb888::new(30, 30, 50);
pub const COLOR_MENU_SELECT: Rgb888 = Rgb888::new(100, 150, 255);

/// Draw the Overview page showing hero stats
pub fn draw_overview_page<D>(
    display: &mut D,
    hero: &Hero,
    battery_mv: u16,
    battery_pct: u8,
    fps: u32,
    save_msg: Option<&str>,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear background
    display.clear(COLOR_BG)?;

    // Title
    draw_text(
        display,
        "=== HERO STATUS ===",
        Point::new(60, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Hero name and job
    let mut name_str = String::<32>::new();
    write!(name_str, "Name: {}", hero.name).ok();
    draw_text(
        display,
        &name_str,
        Point::new(20, 55),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    let mut job_str = String::<32>::new();
    write!(job_str, "Job: {}", hero.job).ok();
    draw_text(
        display,
        &job_str,
        Point::new(20, 78),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Level
    let mut lvl_str = String::<32>::new();
    write!(lvl_str, "Level: {}", hero.level).ok();
    draw_text(
        display,
        &lvl_str,
        Point::new(20, 101),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // HP bar
    draw_text(
        display,
        "HP:",
        Point::new(20, 140),
        &FONT_9X18_BOLD,
        COLOR_TEXT_DIM,
    )?;
    let mut hp_str = String::<32>::new();
    write!(hp_str, "{}/{}", hero.hp, hero.max_hp).ok();
    draw_text(
        display,
        &hp_str,
        Point::new(65, 140),
        &FONT_9X18_BOLD,
        COLOR_HP,
    )?;
    draw_bar(
        display,
        Point::new(20, 155),
        328,
        hero.hp_percent(),
        COLOR_HP,
    )?;

    // SP bar
    draw_text(
        display,
        "SP:",
        Point::new(20, 185),
        &FONT_9X18_BOLD,
        COLOR_TEXT_DIM,
    )?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", hero.sp, hero.max_sp).ok();
    draw_text(
        display,
        &sp_str,
        Point::new(65, 185),
        &FONT_9X18_BOLD,
        COLOR_SP,
    )?;
    draw_bar(
        display,
        Point::new(20, 200),
        328,
        hero.sp_percent(),
        COLOR_SP,
    )?;

    // EXP bar
    draw_text(
        display,
        "EXP:",
        Point::new(20, 230),
        &FONT_9X18_BOLD,
        COLOR_TEXT_DIM,
    )?;
    let mut exp_str = String::<32>::new();
    write!(exp_str, "{}/{}", hero.exp, hero.exp_to_next_level).ok();
    draw_text(
        display,
        &exp_str,
        Point::new(75, 230),
        &FONT_9X18_BOLD,
        COLOR_EXP,
    )?;
    draw_bar(
        display,
        Point::new(20, 245),
        328,
        hero.exp_percent(),
        COLOR_EXP,
    )?;

    // Zeny (currency)
    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "Zeny: {} z", hero.zeny).ok();
    draw_text(
        display,
        &zeny_str,
        Point::new(20, 280),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Battery info
    draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

    // FPS info (right after battery)
    draw_fps_info(display, Point::new(230, 360), fps)?;

    // Save status message (if any)
    if let Some(msg) = save_msg {
        draw_text(
            display,
            msg,
            Point::new(110, 310),
            &FONT_9X18_BOLD,
            Rgb888::YELLOW,
        )?;
    }

    // Instructions
    draw_text(
        display,
        "Press BOOT for Menu",
        Point::new(90, 420),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Draw the Farm page with enemy and progress
pub fn draw_farm_page<D>(
    display: &mut D,
    game_state: &GameState,
    _battery_mv: u16,
    _battery_pct: u8,
    _fps: u32,
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

                // Enemy HP bar
                let mut enemy_hp_str = String::<32>::new();
                write!(enemy_hp_str, "HP: {}/{}", enemy.hp, enemy.max_hp).ok();
                draw_text(
                    display,
                    &enemy_hp_str,
                    Point::new(105, 85),
                    &FONT_9X15,
                    COLOR_HP,
                )?;
                draw_bar(
                    display,
                    Point::new(60, 100),
                    250,
                    enemy.hp_percent(),
                    COLOR_HP,
                )?;

                // Draw hero GIF animation (left side)
                draw_hero_gif(display, game_state, Point::new(60, 200))?;

                // Draw monster GIF animation with attacked state (right side)
                draw_monster_attacked_gif(display, game_state, Point::new(180, 200), enemy.name)?;

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
                    Point::new(30, 420),
                    &FONT_9X15,
                    COLOR_EXP,
                )?;
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

/// Draw the Rest/Sit page for HP and SP regeneration
pub fn draw_rest_page<D>(
    display: &mut D,
    game_state: &GameState,
    battery_mv: u16,
    battery_pct: u8,
    fps: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    draw_text(
        display,
        "=== RESTING ===",
        Point::new(90, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Hero resting GIF animation (16.gif)
    draw_hero_gif(display, game_state, Point::new(120, 100))?;

    // HP bar
    draw_text(
        display,
        "HP Recovery",
        Point::new(105, 160),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;
    let mut hp_str = String::<32>::new();
    write!(hp_str, "{}/{}", game_state.hero.hp, game_state.hero.max_hp).ok();
    draw_text(
        display,
        &hp_str,
        Point::new(125, 180),
        &FONT_9X18_BOLD,
        COLOR_HP,
    )?;
    draw_bar(
        display,
        Point::new(20, 195),
        328,
        game_state.hero.hp_percent(),
        COLOR_HP,
    )?;

    // HP Regen rate
    draw_text(
        display,
        "+10 HP/sec",
        Point::new(120, 215),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // SP bar
    draw_text(
        display,
        "SP Recovery",
        Point::new(105, 245),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", game_state.hero.sp, game_state.hero.max_sp).ok();
    draw_text(
        display,
        &sp_str,
        Point::new(125, 265),
        &FONT_9X18_BOLD,
        COLOR_SP,
    )?;
    draw_bar(
        display,
        Point::new(20, 280),
        328,
        game_state.hero.sp_percent(),
        COLOR_SP,
    )?;

    // SP Regen rate
    let mut sp_regen_str = String::<32>::new();
    write!(sp_regen_str, "+{} SP/sec", game_state.sp_regen_rate).ok();
    draw_text(
        display,
        &sp_regen_str,
        Point::new(120, 300),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    match game_state.rest_state {
        RestState::Resting => {
            draw_text(
                display,
                "Recovering HP & SP...",
                Point::new(65, 330),
                &FONT_9X18_BOLD,
                COLOR_TEXT_DIM,
            )?;
        }
        RestState::FullSP => {
            draw_text(
                display,
                "Fully Recovered!",
                Point::new(75, 330),
                &FONT_9X18_BOLD,
                COLOR_TEXT,
            )?;
            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 355),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    // Battery info
    draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

    // FPS info
    draw_fps_info(display, Point::new(230, 360), fps)?;

    draw_text(
        display,
        "Press BOOT for Menu",
        Point::new(90, 420),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Draw the Battle (Whac-A-Mole) page
pub fn draw_battle_page<D>(
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
                            c.radius * 2,
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
                    Point::new(60, 410),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                // FPS debug display
                draw_fps_info(display, Point::new(10, 430), fps)?;
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

                let mut exp_str = String::<32>::new();
                write!(exp_str, "+{} EXP", enemy.base_exp).ok();
                draw_text(
                    display,
                    &exp_str,
                    Point::new(105, 310),
                    &FONT_9X15,
                    COLOR_EXP,
                )?;

                let mut zeny_str = String::<32>::new();
                write!(zeny_str, "+{} Zeny", enemy.zeny_reward).ok();
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

/// Draw the Map/Navigation page
pub fn draw_map_page<D>(
    display: &mut D,
    game_state: &GameState,
    battery_mv: u16,
    battery_pct: u8,
    fps: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    let map_id = game_state.current_location;
    let location_type = MapHelper::location_type(map_id);

    // Title with location name
    let mut title = String::<32>::new();
    write!(title, "=== {} ===", MapHelper::name(map_id)).ok();
    draw_text(display, &title, Point::new(60, 20), &FONT_10X20, COLOR_TEXT)?;

    // Location type indicator
    let type_str = match location_type {
        LocationType::City => "[CITY]",
        LocationType::Field => "[FIELD]",
    };
    let type_color = match location_type {
        LocationType::City => Rgb888::YELLOW,
        LocationType::Field => Rgb888::RED,
    };
    draw_text(
        display,
        type_str,
        Point::new(145, 50),
        &FONT_9X18_BOLD,
        type_color,
    )?;

    // Draw directional navigation buttons on screen borders (large buttons)
    let exits = MapHelper::exits(map_id);
    for exit in exits {
        match exit.direction {
            "North" => {
                // Top button - almost full width
                Rectangle::new(Point::new(10, 0), Size::new(348, 40))
                    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
                    .draw(display)?;
                Rectangle::new(Point::new(10, 0), Size::new(348, 40))
                    .into_styled(PrimitiveStyle::with_stroke(COLOR_MENU_SELECT, 3))
                    .draw(display)?;
                draw_text(
                    display,
                    "NORTH",
                    Point::new(154, 18),
                    &FONT_10X20,
                    COLOR_TEXT,
                )?;
            }
            "South" => {
                // Bottom button - almost full width
                Rectangle::new(Point::new(10, 408), Size::new(348, 40))
                    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
                    .draw(display)?;
                Rectangle::new(Point::new(10, 408), Size::new(348, 40))
                    .into_styled(PrimitiveStyle::with_stroke(COLOR_MENU_SELECT, 3))
                    .draw(display)?;
                draw_text(
                    display,
                    "SOUTH",
                    Point::new(154, 426),
                    &FONT_10X20,
                    COLOR_TEXT,
                )?;
            }
            "West" => {
                // Left button - almost full height
                Rectangle::new(Point::new(0, 45), Size::new(50, 358))
                    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
                    .draw(display)?;
                Rectangle::new(Point::new(0, 45), Size::new(50, 358))
                    .into_styled(PrimitiveStyle::with_stroke(COLOR_MENU_SELECT, 3))
                    .draw(display)?;
                // Rotated text effect with vertical letters
                draw_text(display, "W", Point::new(17, 170), &FONT_10X20, COLOR_TEXT)?;
                draw_text(display, "E", Point::new(17, 200), &FONT_10X20, COLOR_TEXT)?;
                draw_text(display, "S", Point::new(17, 230), &FONT_10X20, COLOR_TEXT)?;
                draw_text(display, "T", Point::new(17, 260), &FONT_10X20, COLOR_TEXT)?;
            }
            "East" => {
                // Right button - almost full height
                Rectangle::new(Point::new(318, 45), Size::new(50, 358))
                    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
                    .draw(display)?;
                Rectangle::new(Point::new(318, 45), Size::new(50, 358))
                    .into_styled(PrimitiveStyle::with_stroke(COLOR_MENU_SELECT, 3))
                    .draw(display)?;
                // Rotated text effect with vertical letters
                draw_text(display, "E", Point::new(335, 170), &FONT_10X20, COLOR_TEXT)?;
                draw_text(display, "A", Point::new(335, 200), &FONT_10X20, COLOR_TEXT)?;
                draw_text(display, "S", Point::new(335, 230), &FONT_10X20, COLOR_TEXT)?;
                draw_text(display, "T", Point::new(335, 260), &FONT_10X20, COLOR_TEXT)?;
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
            // Show monster info and actions (from JSON)
            let enemy_ids = MapHelper::enemies(map_id);
            if !enemy_ids.is_empty() {
                // Monster list
                draw_text(
                    display,
                    "Monsters:",
                    Point::new(120, 85),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;
                for (i, &enemy_id) in enemy_ids.iter().enumerate() {
                    if let Some(enemy) = Enemy::from_id(enemy_id) {
                        let y = 110 + i as i32 * 22;
                        let mut monster_str = String::<32>::new();
                        write!(monster_str, "{} (Lv {})", enemy.name, enemy.level).ok();
                        draw_text(
                            display,
                            &monster_str,
                            Point::new(130, y),
                            &FONT_9X15,
                            COLOR_TEXT_DIM,
                        )?;
                    }
                }

                // Action buttons (centered, large)
                // Auto Farm button
                Rectangle::new(Point::new(84, 210), Size::new(200, 60))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50)))
                    .draw(display)?;
                Rectangle::new(Point::new(84, 210), Size::new(200, 60))
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 3))
                    .draw(display)?;
                draw_text(
                    display,
                    "AUTO FARM",
                    Point::new(115, 235),
                    &FONT_9X18_BOLD,
                    Rgb888::WHITE,
                )?;

                // Battle button
                Rectangle::new(Point::new(84, 280), Size::new(200, 60))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
                    .draw(display)?;
                Rectangle::new(Point::new(84, 280), Size::new(200, 60))
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 3))
                    .draw(display)?;
                draw_text(
                    display,
                    "BATTLE",
                    Point::new(140, 305),
                    &FONT_9X18_BOLD,
                    Rgb888::WHITE,
                )?;
            }
        }
    }

    // Battery and FPS at bottom
    draw_battery_info(display, Point::new(10, 355), battery_mv, battery_pct)?;
    draw_fps_info(display, Point::new(240, 365), fps)?;

    // Status message (for SP/HP warnings)
    if let Some(msg) = game_state.save_status_msg {
        draw_text(
            display,
            msg,
            Point::new(60, 390),
            &FONT_10X20,
            Rgb888::RED,
        )?;
    }

    Ok(())
}

/// Draw the Menu overlay
pub fn draw_menu<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Full-width background panel with padding
    Rectangle::new(Point::new(10, 40), Size::new(348, 368))
        .into_styled(PrimitiveStyle::with_fill(COLOR_MENU_BG))
        .draw(display)?;

    Rectangle::new(Point::new(10, 40), Size::new(348, 368))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 3))
        .draw(display)?;

    draw_text(
        display,
        "=== MENU ===",
        Point::new(115, 70),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Menu items in 2 columns x 3 rows (6 items)
    // Farm and Battle are now accessed via Map page
    // Button size: 150x70 with 10px spacing
    let menu_items = ["Overview", "Rest", "Map", "Inventory", "Settings", "Save"];

    for (i, item) in menu_items.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;

        // Calculate button position
        let x = 24 + col as i32 * 160; // 24px left margin, 160px spacing (150 button + 10 gap)
        let y = 110 + row as i32 * 80; // 110px top, 80px spacing (70 button + 10 gap)

        let is_selected = i as u8 == game_state.menu_selection;

        // Draw button background
        let button_color = if is_selected {
            COLOR_MENU_SELECT
        } else {
            COLOR_PANEL
        };

        Rectangle::new(Point::new(x, y), Size::new(150, 70))
            .into_styled(PrimitiveStyle::with_fill(button_color))
            .draw(display)?;

        // Draw button border (thicker if selected)
        let border_width = if is_selected { 3 } else { 2 };
        Rectangle::new(Point::new(x, y), Size::new(150, 70))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, border_width))
            .draw(display)?;

        // Draw text centered in button
        let text_color = if is_selected {
            COLOR_TEXT
        } else {
            COLOR_TEXT_DIM
        };

        // Calculate text centering (rough approximation)
        let text_len = item.len() as i32;
        let text_x = x + (150 - text_len * 9) / 2; // 9px per char for FONT_9X18_BOLD
        let text_y = y + 30; // Center vertically in 70px button

        draw_text(
            display,
            item,
            Point::new(text_x, text_y),
            &FONT_9X18_BOLD,
            text_color,
        )?;
    }

    draw_text(
        display,
        "Touch button to select",
        Point::new(75, 360),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    draw_text(
        display,
        "BOOT to close menu",
        Point::new(80, 385),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Draw inventory page showing all collected items
pub fn draw_inventory<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear display
    display.clear(COLOR_BG)?;

    // Header
    draw_text(
        display,
        "=== INVENTORY ===",
        Point::new(85, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Draw item list
    let inventory = &game_state.hero.inventory;

    if inventory.is_empty() {
        draw_text(
            display,
            "No items yet!",
            Point::new(110, 200),
            &FONT_10X20,
            COLOR_TEXT_DIM,
        )?;
        draw_text(
            display,
            "Defeat enemies to earn items",
            Point::new(40, 230),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    } else {
        // Draw items in a scrollable list (show first 15 items)
        let mut y = 60;
        for (i, item) in inventory.iter().take(15).enumerate() {
            let mut item_str = String::<64>::new();
            write!(item_str, "{} x{}", item.name, item.quantity).ok();

            let text_color = if i % 2 == 0 {
                COLOR_TEXT
            } else {
                Rgb888::new(200, 200, 200)
            };

            draw_text(
                display,
                &item_str,
                Point::new(20, y),
                &FONT_9X15,
                text_color,
            )?;

            y += 20;
        }

        // Show count if there are more items
        if inventory.len() > 15 {
            let mut count_str = String::<32>::new();
            write!(count_str, "...and {} more", inventory.len() - 15).ok();
            draw_text(
                display,
                &count_str,
                Point::new(90, y + 10),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    // Footer
    draw_text(
        display,
        "Touch to go back",
        Point::new(90, 440),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Draw the settings page with brightness slider
pub fn draw_settings_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear display
    display.clear(COLOR_BG)?;

    draw_text(
        display,
        "=== SETTINGS ===",
        Point::new(85, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Brightness section
    draw_text(
        display,
        "Brightness",
        Point::new(130, 100),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Brightness value display
    let mut brightness_str = String::<16>::new();
    write!(brightness_str, "{}%", (game_state.brightness as u32 * 100) / 255).ok();
    draw_text(
        display,
        &brightness_str,
        Point::new(155, 130),
        &FONT_9X18_BOLD,
        Rgb888::YELLOW,
    )?;

    // Slider track (background bar)
    Rectangle::new(Point::new(40, 180), Size::new(280, 20))
        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
        .draw(display)?;

    Rectangle::new(Point::new(40, 180), Size::new(280, 20))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    // Slider filled portion (represents current brightness)
    let filled_width = ((game_state.brightness as u32 * 280) / 255) as u32;
    if filled_width > 0 {
        Rectangle::new(Point::new(40, 180), Size::new(filled_width, 20))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::YELLOW))
            .draw(display)?;
    }

    // Slider handle (indicator)
    let handle_x = 40 + ((game_state.brightness as i32 * 280) / 255);
    EgCircle::new(Point::new(handle_x - 8, 172), 16)
        .into_styled(PrimitiveStyle::with_fill(COLOR_MENU_SELECT))
        .draw(display)?;

    EgCircle::new(Point::new(handle_x - 8, 172), 16)
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 3))
        .draw(display)?;

    // Instructions
    draw_text(
        display,
        "Touch slider to adjust",
        Point::new(70, 250),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Brightness range labels
    draw_text(
        display,
        "0%",
        Point::new(35, 210),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    draw_text(
        display,
        "100%",
        Point::new(290, 210),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Footer
    draw_text(
        display,
        "Touch bottom to go back",
        Point::new(65, 440),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Helper: Draw monster GIF animation
fn draw_monster_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let gif_data = game_state.monster_animation.gif_data(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame
    let frame_index = game_state.monster_animation_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Helper: Draw hero GIF animation
fn draw_hero_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let gif_data = game_state.hero_animation.gif_data();
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse hero GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame
    let frame_index = game_state.hero_animation_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Helper: Draw monster attacked GIF animation (24.gif when hero attacks)
fn draw_monster_attacked_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::tamagotchi::models::{MonsterAttackedAnimation, get_monster_attacked_gif};

    if game_state.monster_attacked_animation == MonsterAttackedAnimation::Normal {
        // No attacked animation, draw normal monster
        return draw_monster_gif(display, game_state, center_position, monster_name);
    }

    // Draw attacked animation (24.gif) for specific monster
    let gif_data = get_monster_attacked_gif(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse monster attacked GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame
    let frame_index = game_state.monster_attacked_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Helper: Draw FPS information
fn draw_fps_info<D>(display: &mut D, position: Point, fps: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // FPS label
    draw_text(display, "FPS:", position, &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;

    // FPS value
    let mut fps_str = String::<16>::new();
    write!(fps_str, "{}", fps).ok();

    // Color based on FPS (green if 30+, yellow if 20-29, red if <20)
    let fps_color = if fps >= 30 {
        Rgb888::GREEN
    } else if fps >= 20 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    draw_text(
        display,
        &fps_str,
        position + Point::new(0, 20),
        &FONT_9X15,
        fps_color,
    )?;

    Ok(())
}

/// Helper: Draw text
fn draw_text<D>(
    display: &mut D,
    text: &str,
    position: Point,
    font: &embedded_graphics::mono_font::MonoFont,
    color: Rgb888,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    Text::new(text, position, MonoTextStyle::new(font, color)).draw(display)?;
    Ok(())
}

/// Helper: Draw a horizontal progress bar
fn draw_bar<D>(
    display: &mut D,
    position: Point,
    width: u32,
    percent: u8,
    color: Rgb888,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let percent = percent.min(100);
    let height = 10;

    // Background
    Rectangle::new(position, Size::new(width, height))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;

    // Fill
    let fill_width = (width as u32 * percent as u32) / 100;
    if fill_width > 0 {
        Rectangle::new(
            position + Point::new(1, 1),
            Size::new(fill_width, height - 2),
        )
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display)?;
    }

    Ok(())
}

/// Helper: Draw battery information (voltage and percentage)
fn draw_battery_info<D>(
    display: &mut D,
    position: Point,
    voltage_mv: u16,
    percent: u8,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Battery label
    draw_text(
        display,
        "Battery:",
        position,
        &FONT_9X18_BOLD,
        COLOR_TEXT_DIM,
    )?;

    // Battery percentage and voltage
    let mut bat_str = String::<32>::new();
    write!(bat_str, "{}% | {}mV", percent, voltage_mv).ok();

    // Color based on battery level
    let bat_color = if percent >= 50 {
        Rgb888::GREEN
    } else if percent >= 20 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    draw_text(
        display,
        &bat_str,
        position + Point::new(0, 20),
        &FONT_9X15,
        bat_color,
    )?;

    // Battery bar
    draw_bar(
        display,
        position + Point::new(0, 35),
        200,
        percent,
        bat_color,
    )?;

    Ok(())
}
