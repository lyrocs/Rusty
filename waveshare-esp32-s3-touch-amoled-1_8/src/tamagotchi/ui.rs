use core::fmt::Write;
use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb888,
    mono_font::{MonoTextStyle, ascii::{FONT_10X20, FONT_9X18_BOLD, FONT_9X15}},
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;

use crate::tamagotchi::models::{Hero, GameState, FarmState, RestState};

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
pub fn draw_overview_page<D>(display: &mut D, hero: &Hero, battery_mv: u16, battery_pct: u8, fps: u32, save_msg: Option<&str>) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear background
    display.clear(COLOR_BG)?;

    // Title
    draw_text(display, "=== HERO STATUS ===", Point::new(60, 20), &FONT_10X20, COLOR_TEXT)?;

    // Hero name and job
    let mut name_str = String::<32>::new();
    write!(name_str, "Name: {}", hero.name).ok();
    draw_text(display, &name_str, Point::new(20, 55), &FONT_9X18_BOLD, COLOR_TEXT)?;

    let mut job_str = String::<32>::new();
    write!(job_str, "Job: {}", hero.job).ok();
    draw_text(display, &job_str, Point::new(20, 78), &FONT_9X18_BOLD, COLOR_TEXT)?;

    // Level
    let mut lvl_str = String::<32>::new();
    write!(lvl_str, "Level: {}", hero.level).ok();
    draw_text(display, &lvl_str, Point::new(20, 101), &FONT_9X18_BOLD, COLOR_TEXT)?;

    // HP bar
    draw_text(display, "HP:", Point::new(20, 140), &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;
    let mut hp_str = String::<32>::new();
    write!(hp_str, "{}/{}", hero.hp, hero.max_hp).ok();
    draw_text(display, &hp_str, Point::new(65, 140), &FONT_9X18_BOLD, COLOR_HP)?;
    draw_bar(display, Point::new(20, 155), 328, hero.hp_percent(), COLOR_HP)?;

    // SP bar
    draw_text(display, "SP:", Point::new(20, 185), &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", hero.sp, hero.max_sp).ok();
    draw_text(display, &sp_str, Point::new(65, 185), &FONT_9X18_BOLD, COLOR_SP)?;
    draw_bar(display, Point::new(20, 200), 328, hero.sp_percent(), COLOR_SP)?;

    // EXP bar
    draw_text(display, "EXP:", Point::new(20, 230), &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;
    let mut exp_str = String::<32>::new();
    write!(exp_str, "{}/{}", hero.exp, hero.exp_to_next_level).ok();
    draw_text(display, &exp_str, Point::new(75, 230), &FONT_9X18_BOLD, COLOR_EXP)?;
    draw_bar(display, Point::new(20, 245), 328, hero.exp_percent(), COLOR_EXP)?;

    // Zeny (currency)
    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "Zeny: {} z", hero.zeny).ok();
    draw_text(display, &zeny_str, Point::new(20, 280), &FONT_9X18_BOLD, COLOR_TEXT)?;

    // Battery info
    draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

    // FPS info (right after battery)
    draw_fps_info(display, Point::new(230, 360), fps)?;

    // Save status message (if any)
    if let Some(msg) = save_msg {
        draw_text(display, msg, Point::new(110, 310), &FONT_9X18_BOLD, Rgb888::YELLOW)?;
    }

    // Instructions
    draw_text(display, "Press BOOT for Menu", Point::new(90, 420), &FONT_9X15, COLOR_TEXT_DIM)?;

    Ok(())
}

/// Draw the Farm page with enemy and progress
pub fn draw_farm_page<D>(display: &mut D, game_state: &GameState, battery_mv: u16, battery_pct: u8, fps: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    match game_state.farm_state {
        FarmState::Idle => {
            draw_text(display, "=== AUTO FARM ===", Point::new(70, 20), &FONT_10X20, COLOR_TEXT)?;

            // SP display with color coding
            let mut sp_str = String::<32>::new();
            write!(sp_str, "SP: {}/{}", game_state.hero.sp, game_state.hero.max_sp).ok();
            let sp_color = if game_state.hero.sp >= 20 { COLOR_SP } else { COLOR_HP };
            draw_text(display, &sp_str, Point::new(20, 60), &FONT_9X18_BOLD, sp_color)?;

            // SP bar
            draw_bar(display, Point::new(20, 78), 328, game_state.hero.sp_percent(), sp_color)?;

            // Check if user has enough SP
            if game_state.hero.sp >= 20 {
                // Enough SP - show normal instructions
                draw_text(display, "Touch screen to", Point::new(90, 160), &FONT_9X18_BOLD, COLOR_TEXT)?;
                draw_text(display, "start farming", Point::new(95, 185), &FONT_9X18_BOLD, COLOR_TEXT)?;

                draw_text(display, "Cost: 20 SP", Point::new(110, 230), &FONT_9X15, COLOR_TEXT_DIM)?;
                draw_text(display, "Duration: 1 minute", Point::new(90, 250), &FONT_9X15, COLOR_TEXT_DIM)?;
            } else {
                // Not enough SP - show warning
                draw_text(display, "NOT ENOUGH SP!", Point::new(75, 150), &FONT_10X20, COLOR_HP)?;

                let mut needed_str = String::<32>::new();
                write!(needed_str, "Need {} more SP", 20 - game_state.hero.sp).ok();
                draw_text(display, &needed_str, Point::new(90, 185), &FONT_9X18_BOLD, COLOR_HP)?;

                draw_text(display, "Go to Rest page to", Point::new(75, 225), &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;
                draw_text(display, "recover SP", Point::new(115, 248), &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;

                draw_text(display, "(Press BOOT > Rest)", Point::new(80, 285), &FONT_9X15, Rgb888::YELLOW)?;
            }

            // Battery info
            draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

            // FPS info
            draw_fps_info(display, Point::new(230, 360), fps)?;

            draw_text(display, "Press BOOT for Menu", Point::new(90, 420), &FONT_9X15, COLOR_TEXT_DIM)?;
        }
        FarmState::Fighting => {
            if let Some(enemy) = &game_state.current_enemy {
                draw_text(display, "=== FIGHTING ===", Point::new(80, 20), &FONT_10X20, COLOR_TEXT)?;

                // Hero info
                draw_text(display, "You", Point::new(20, 60), &FONT_9X18_BOLD, COLOR_TEXT)?;
                let mut hero_hp_str = String::<32>::new();
                write!(hero_hp_str, "HP: {}/{}", game_state.hero.hp, game_state.hero.max_hp).ok();
                draw_text(display, &hero_hp_str, Point::new(20, 83), &FONT_9X15, COLOR_HP)?;
                draw_bar(display, Point::new(20, 98), 150, game_state.hero.hp_percent(), COLOR_HP)?;

                // VS indicator
                draw_text(display, "VS", Point::new(165, 75), &FONT_10X20, COLOR_TEXT)?;

                // Enemy info
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "{} Lv.{}", enemy.name, enemy.level).ok();
                draw_text(display, &enemy_str, Point::new(200, 60), &FONT_9X18_BOLD, COLOR_TEXT)?;

                let mut enemy_hp_str = String::<32>::new();
                write!(enemy_hp_str, "HP: {}/{}", enemy.hp, enemy.max_hp).ok();
                draw_text(display, &enemy_hp_str, Point::new(200, 83), &FONT_9X15, COLOR_HP)?;
                draw_bar(display, Point::new(200, 98), 150, enemy.hp_percent(), COLOR_HP)?;

                // Progress bar
                draw_text(display, "Combat Progress", Point::new(85, 170), &FONT_9X18_BOLD, COLOR_TEXT)?;
                draw_bar(display, Point::new(20, 195), 328, game_state.farm_progress_percent(), COLOR_EXP)?;

                let mut time_str = String::<32>::new();
                let remaining_sec = (game_state.farm_duration_ms - game_state.farm_progress) / 1000;
                write!(time_str, "{}s remaining", remaining_sec).ok();
                draw_text(display, &time_str, Point::new(120, 215), &FONT_9X15, COLOR_TEXT_DIM)?;

                // Potential rewards
                draw_text(display, "Rewards:", Point::new(20, 260), &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;
                let mut reward_str = String::<32>::new();
                write!(reward_str, "EXP: {} | Zeny: {}", enemy.exp_reward, enemy.zeny_reward).ok();
                draw_text(display, &reward_str, Point::new(20, 283), &FONT_9X15, COLOR_EXP)?;

                // Battery info
                draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

                // FPS info
                draw_fps_info(display, Point::new(230, 360), fps)?;
            }
        }
        FarmState::Victory => {
            draw_text(display, "=== VICTORY! ===", Point::new(80, 20), &FONT_10X20, COLOR_TEXT)?;

            if let Some(enemy) = &game_state.current_enemy {
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "Defeated {}", enemy.name).ok();
                draw_text(display, &enemy_str, Point::new(85, 100), &FONT_9X18_BOLD, COLOR_TEXT)?;

                draw_text(display, "Rewards:", Point::new(120, 150), &FONT_9X18_BOLD, COLOR_EXP)?;

                let mut exp_str = String::<32>::new();
                write!(exp_str, "+{} EXP", enemy.exp_reward).ok();
                draw_text(display, &exp_str, Point::new(105, 180), &FONT_9X18_BOLD, COLOR_EXP)?;

                let mut zeny_str = String::<32>::new();
                write!(zeny_str, "+{} Zeny", enemy.zeny_reward).ok();
                draw_text(display, &zeny_str, Point::new(105, 210), &FONT_9X18_BOLD, COLOR_EXP)?;
            }

            // Battery info
            draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

            // FPS info
            draw_fps_info(display, Point::new(230, 360), fps)?;

            draw_text(display, "Touch to continue", Point::new(90, 420), &FONT_9X15, COLOR_TEXT_DIM)?;
        }
        FarmState::Defeat => {
            draw_text(display, "=== DEFEATED ===", Point::new(80, 100), &FONT_10X20, COLOR_HP)?;

            // Battery info
            draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

            // FPS info
            draw_fps_info(display, Point::new(230, 360), fps)?;

            draw_text(display, "Touch to continue", Point::new(90, 420), &FONT_9X15, COLOR_TEXT_DIM)?;
        }
    }

    Ok(())
}

/// Draw the Rest/Sit page for SP regeneration
pub fn draw_rest_page<D>(display: &mut D, game_state: &GameState, battery_mv: u16, battery_pct: u8, fps: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    draw_text(display, "=== RESTING ===", Point::new(90, 20), &FONT_10X20, COLOR_TEXT)?;

    // Hero sitting animation (simple representation)
    draw_text(display, "  __", Point::new(150, 80), &FONT_10X20, COLOR_TEXT)?;
    draw_text(display, " /  \\", Point::new(145, 100), &FONT_10X20, COLOR_TEXT)?;
    draw_text(display, "|____|", Point::new(140, 120), &FONT_10X20, COLOR_TEXT)?;
    draw_text(display, "Zzz...", Point::new(200, 90), &FONT_9X15, COLOR_TEXT_DIM)?;

    // SP bar
    draw_text(display, "SP Recovery", Point::new(105, 180), &FONT_9X18_BOLD, COLOR_TEXT)?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", game_state.hero.sp, game_state.hero.max_sp).ok();
    draw_text(display, &sp_str, Point::new(125, 210), &FONT_9X18_BOLD, COLOR_SP)?;
    draw_bar(display, Point::new(20, 230), 328, game_state.hero.sp_percent(), COLOR_SP)?;

    // Regen rate
    let mut regen_str = String::<32>::new();
    write!(regen_str, "+{} SP/sec", game_state.sp_regen_rate).ok();
    draw_text(display, &regen_str, Point::new(120, 255), &FONT_9X15, COLOR_TEXT_DIM)?;

    match game_state.rest_state {
        RestState::Resting => {
            draw_text(display, "Recovering SP...", Point::new(85, 300), &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;
        }
        RestState::FullSP => {
            draw_text(display, "SP Fully Recovered!", Point::new(60, 300), &FONT_9X18_BOLD, COLOR_TEXT)?;
            draw_text(display, "Touch to continue", Point::new(90, 330), &FONT_9X15, COLOR_TEXT_DIM)?;
        }
    }

    // Battery info
    draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

    // FPS info
    draw_fps_info(display, Point::new(230, 360), fps)?;

    draw_text(display, "Press BOOT for Menu", Point::new(90, 420), &FONT_9X15, COLOR_TEXT_DIM)?;

    Ok(())
}

/// Draw the Menu overlay
pub fn draw_menu<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Larger background panel for better spacing
    Rectangle::new(Point::new(40, 60), Size::new(288, 328))
        .into_styled(PrimitiveStyle::with_fill(COLOR_MENU_BG))
        .draw(display)?;

    Rectangle::new(Point::new(40, 60), Size::new(288, 328))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 3))
        .draw(display)?;

    draw_text(display, "=== MENU ===", Point::new(105, 90), &FONT_10X20, COLOR_TEXT)?;

    // Menu items with much more spacing
    let menu_items = ["Overview", "Auto Farm", "Rest", "Save Game"];
    for (i, item) in menu_items.iter().enumerate() {
        let y = 130 + i as i32 * 55; // Increased spacing from 40 to 55
        let color = if i as u8 == game_state.menu_selection {
            COLOR_MENU_SELECT
        } else {
            COLOR_TEXT_DIM
        };

        let mut item_str = String::<32>::new();
        write!(item_str, "{} {}", if i as u8 == game_state.menu_selection { ">" } else { " " }, item).ok();
        draw_text(display, &item_str, Point::new(70, y), &FONT_10X20, color)?;
    }

    draw_text(display, "Touch to select", Point::new(85, 350), &FONT_9X15, COLOR_TEXT_DIM)?;
    draw_text(display, "BOOT to close", Point::new(90, 370), &FONT_9X15, COLOR_TEXT_DIM)?;

    Ok(())
}

/// Helper: Draw FPS information
fn draw_fps_info<D>(
    display: &mut D,
    position: Point,
    fps: u32,
) -> Result<(), D::Error>
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

    draw_text(display, &fps_str, position + Point::new(0, 20), &FONT_9X15, fps_color)?;

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
        Rectangle::new(position + Point::new(1, 1), Size::new(fill_width, height - 2))
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
    draw_text(display, "Battery:", position, &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;

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

    draw_text(display, &bat_str, position + Point::new(0, 20), &FONT_9X15, bat_color)?;

    // Battery bar
    draw_bar(display, position + Point::new(0, 35), 200, percent, bat_color)?;

    Ok(())
}
