use core::fmt::Write;
use embedded_graphics::{
    image::Image,
    mono_font::ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;
use tinygif::Gif;

use super::super::helpers::*;
use crate::core::GameState;

use super::super::colors::*;

// Background image
const BACKGROUND_GIF: &[u8] = include_bytes!("../../../assets/images/ui/background.gif");

/// Draw the Overview page showing hero stats
pub fn draw_overview_page<D>(
    display: &mut D,
    game_state: &GameState,
    save_msg: Option<&str>,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let hero = &game_state.hero;

    // Clear background
    display.clear(Rgb888::new(0, 0, 0))?;

    // Draw background image (single frame GIF)
    let bg_gif = Gif::<Rgb888>::from_slice(BACKGROUND_GIF).expect("Failed to parse background GIF");
    if let Some(frame) = bg_gif.frames().next() {
        Image::new(&frame, Point::new(0, 0)).draw(display)?;
    }

    let base_y = 20;

    // === HERO NAME & TITLE (centered) ===
    let mut name_str = String::<32>::new();
    write!(name_str, "{}", hero.name).ok();
    draw_text(
        display,
        &name_str,
        Point::new(184 - (name_str.len() as i32 * 5), base_y),
        &FONT_10X20,
        Rgb888::new(255, 230, 150),
    )?;

    let mut job_level_str = String::<32>::new();
    write!(job_level_str, "{} Lv.{}", hero.job, hero.level).ok();
    draw_text(
        display,
        &job_level_str,
        Point::new(184 - (job_level_str.len() as i32 * 4), base_y + 20),
        &FONT_9X15,
        Rgb888::new(180, 180, 200),
    )?;

    // === STATS PANEL (compact card design) ===
    let panel_y = base_y + 50;

    // Draw stats background panel - using only border, no fill to show background
    Rectangle::new(Point::new(30, panel_y), Size::new(308, 115))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 140, 180), 3))
        .draw(display)?;

    // HP Bar (full width)
    draw_text(
        display,
        "HP",
        Point::new(40, panel_y + 18),
        &FONT_9X15,
        COLOR_HP,
    )?;
    let mut hp_str = String::<32>::new();
    write!(hp_str, "{}/{}", hero.hp, hero.max_hp).ok();
    draw_text(
        display,
        &hp_str,
        Point::new(320 - (hp_str.len() as i32 * 9), panel_y + 18),
        &FONT_9X15,
        Rgb888::WHITE,
    )?;
    draw_bar(
        display,
        Point::new(40, panel_y + 30),
        280,
        hero.hp_percent(),
        COLOR_HP,
    )?;

    // SP Bar (full width)
    draw_text(
        display,
        "SP",
        Point::new(40, panel_y + 53),
        &FONT_9X15,
        COLOR_SP,
    )?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", hero.sp, hero.max_sp).ok();
    draw_text(
        display,
        &sp_str,
        Point::new(320 - (sp_str.len() as i32 * 9), panel_y + 53),
        &FONT_9X15,
        Rgb888::WHITE,
    )?;
    draw_bar(
        display,
        Point::new(40, panel_y + 65),
        280,
        hero.sp_percent(),
        COLOR_SP,
    )?;

    // EXP Bar (full width)
    draw_text(
        display,
        "EXP",
        Point::new(40, panel_y + 88),
        &FONT_9X15,
        COLOR_EXP,
    )?;
    let mut exp_str = String::<32>::new();
    write!(exp_str, "{}/{}", hero.exp, hero.exp_to_next_level).ok();
    draw_text(
        display,
        &exp_str,
        Point::new(320 - (exp_str.len() as i32 * 9), panel_y + 88),
        &FONT_9X15,
        Rgb888::WHITE,
    )?;
    draw_bar(
        display,
        Point::new(40, panel_y + 100),
        280,
        hero.exp_percent(),
        COLOR_EXP,
    )?;

    // === ACTION BUTTONS (2x2 grid) ===
    let button_start_y = panel_y + 135;
    let button_spacing = 50;
    let button_width = 155;
    let button_height = 42;

    // Button colors - more vibrant and distinct
    let rest_color = Rgb888::new(80, 120, 180);
    let stats_color = Rgb888::new(180, 80, 120);
    let equip_color = Rgb888::new(120, 180, 80);
    let invent_color = Rgb888::new(180, 140, 60);

    // Row 1: Rest, Stats
    // Rest button (top left)
    Rectangle::new(
        Point::new(20, button_start_y),
        Size::new(button_width, button_height),
    )
    .into_styled(PrimitiveStyle::with_fill(rest_color))
    .draw(display)?;
    Rectangle::new(
        Point::new(20, button_start_y),
        Size::new(button_width, button_height),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(120, 160, 220), 2))
    .draw(display)?;
    draw_text(
        display,
        "Rest",
        Point::new(68, button_start_y + 26),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Stats button (top right)
    Rectangle::new(
        Point::new(193, button_start_y),
        Size::new(button_width, button_height),
    )
    .into_styled(PrimitiveStyle::with_fill(stats_color))
    .draw(display)?;
    Rectangle::new(
        Point::new(193, button_start_y),
        Size::new(button_width, button_height),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(220, 120, 160), 2))
    .draw(display)?;
    draw_text(
        display,
        "Stats",
        Point::new(235, button_start_y + 26),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Row 2: Equipment, Inventory
    // Equipment button (bottom left)
    Rectangle::new(
        Point::new(20, button_start_y + button_spacing),
        Size::new(button_width, button_height),
    )
    .into_styled(PrimitiveStyle::with_fill(equip_color))
    .draw(display)?;
    Rectangle::new(
        Point::new(20, button_start_y + button_spacing),
        Size::new(button_width, button_height),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(160, 220, 120), 2))
    .draw(display)?;
    draw_text(
        display,
        "Equip",
        Point::new(58, button_start_y + button_spacing + 26),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Inventory button (bottom right)
    Rectangle::new(
        Point::new(193, button_start_y + button_spacing),
        Size::new(button_width, button_height),
    )
    .into_styled(PrimitiveStyle::with_fill(invent_color))
    .draw(display)?;
    Rectangle::new(
        Point::new(193, button_start_y + button_spacing),
        Size::new(button_width, button_height),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(220, 180, 100), 2))
    .draw(display)?;
    draw_text(
        display,
        "Invent",
        Point::new(223, button_start_y + button_spacing + 26),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // === HERO CHARACTER SECTION (at bottom) ===
    let hero_section_y = button_start_y + button_spacing + button_height as i32 + 20;

    // Zeny display (above hero)
    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "{} Zeny", hero.zeny).ok();
    draw_text(
        display,
        &zeny_str,
        Point::new(184 - (zeny_str.len() as i32 * 4), hero_section_y),
        &FONT_9X15,
        Rgb888::new(255, 215, 0),
    )?;

    // Hero GIF (centered at bottom, safe from rounded edges)
    draw_hero_gif(display, game_state, Point::new(184, hero_section_y + 100))?;

    // Save status message (if any)
    if let Some(msg) = save_msg {
        draw_text(
            display,
            msg,
            Point::new(184 - (msg.len() as i32 * 5), hero_section_y + 75),
            &FONT_9X18_BOLD,
            Rgb888::new(100, 255, 100),
        )?;
    }

    Ok(())
}
