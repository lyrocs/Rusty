//! Rustymon Detail Page
//!
//! Shows detailed stats of a single Rustymon and allows team management

use crate::display::Sh8601Driver;
use crate::game::{Rustymon, RustymonTeam, GameData};
use crate::game::element_system::get_element_color;
use crate::ui::page::Page;
use crate::ui::sprite::AnimatedSprite;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, PrimitiveStyleBuilder},
    text::Text,
};
use std::error::Error;
use std::time::Duration;

/// Actions from detail page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustymonDetailAction {
    AddToTeam,
    RemoveFromTeam,
    OpenSkills,
    Close,
}

/// Touch area
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: RustymonDetailAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Rustymon Detail page
pub struct RustymonDetailPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
    idle_sprite: Option<AnimatedSprite>,
}

impl RustymonDetailPage {
    /// Create new Rustymon detail page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
            idle_sprite: None,
        }
    }

    /// Clear idle animation (called when changing rustymon)
    pub fn clear_idle_animation(&mut self) {
        self.idle_sprite = None;
    }

    /// Check if idle animation is loaded
    pub fn has_idle_animation(&self) -> bool {
        self.idle_sprite.is_some()
    }

    /// Load idle animation for a specific monster
    pub fn load_idle_animation(&mut self, species_id: u32) -> Result<(), Box<dyn Error>> {
        // Load idle animation based on species_id
        let idle_data: &[u8] = match species_id {
            1002 => include_bytes!("../../../assets/images/poring/6.gif"),
            1004 => include_bytes!("../../../assets/images/hornet/6.gif"),
            1007 => include_bytes!("../../../assets/images/fabre/6.gif"),
            1051 => include_bytes!("../../../assets/images/thief_bug/6.gif"),
            _ => {
                log::warn!("No idle animation for species_id {}, using fabre as default", species_id);
                include_bytes!("../../../assets/images/fabre/6.gif")
            }
        };

        // Create sprite at center-right of stats area (between header and buttons)
        let sprite = AnimatedSprite::new(
            idle_data,
            (280, 250), // Position on right side
            Duration::from_millis(100),
            None, // Infinite loop
        )?;

        self.idle_sprite = Some(sprite);
        Ok(())
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<RustymonDetailAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Rustymon detail action: {:?}", area.action);
                return Some(area.action);
            }
        }
        None
    }

    /// Draw Rustymon detail screen
    pub fn draw_rustymon_detail(
        &mut self,
        display: &mut Sh8601Driver,
        rustymon: &Rustymon,
        rustymon_team: &RustymonTeam,
        _game_data: &GameData,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.touch_areas.clear();

        let element_color = get_element_color(rustymon.element);

        // Draw header with element color (reduced height)
        Rectangle::new(Point::new(0, 0), Size::new(368, 45))
            .into_styled(PrimitiveStyle::with_fill(element_color))
            .draw(display)?;

        // Draw name (moved away from corner)
        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::BLACK);
        let mut name_str = heapless::String::<32>::new();
        write!(name_str, "{}", rustymon.name).ok();
        Text::new(&name_str, Point::new(20, 25), name_style).draw(display)?;

        // Draw level and element on same line
        let mut level_str = heapless::String::<16>::new();
        write!(level_str, "Lv {}", rustymon.level).ok();
        Text::new(&level_str, Point::new(250, 25), name_style).draw(display)?;

        // Stats section - compact layout
        let stat_label_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 200));
        let stat_value_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 255));

        let mut y = 65;
        let left_label_x = 20;
        let left_value_x = left_label_x + 60; // Compact spacing
        let right_label_x = 194;
        let right_value_x = right_label_x + 60; // Compact spacing
        let line_height = 22;

        // HP (bar first, then values)
        let mut label_str = heapless::String::<16>::new();
        write!(label_str, "HP:").ok();
        Text::new(&label_str, Point::new(left_label_x, y), stat_label_style).draw(display)?;

        // HP bar
        self.draw_hp_bar(
            display,
            (left_label_x + 35, y - 15),
            rustymon.current_hp,
            rustymon.max_hp,
            130,
        )?;

        // HP values (after bar)
        let mut value_str = heapless::String::<24>::new();
        write!(value_str, "{}/{}", rustymon.current_hp, rustymon.max_hp).ok();
        Text::new(&value_str, Point::new(left_label_x + 175, y), stat_value_style).draw(display)?;

        y += line_height + 8;

        // EXP (bar first, then values)
        label_str.clear();
        write!(label_str, "EXP:").ok();
        Text::new(&label_str, Point::new(left_label_x, y), stat_label_style).draw(display)?;

        // EXP bar
        self.draw_exp_bar(
            display,
            (left_label_x + 35, y - 15),
            rustymon.exp,
            rustymon.exp_to_next,
            130,
        )?;

        // EXP values (after bar)
        value_str.clear();
        write!(value_str, "{}/{}", rustymon.exp, rustymon.exp_to_next).ok();
        Text::new(&value_str, Point::new(left_label_x + 175, y), stat_value_style).draw(display)?;

        y += line_height + 12;

        // Two-column layout for stats
        let section_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 100));

        // LEFT COLUMN: Base Stats
        let mut left_y = y;
        Text::new("Base Stats", Point::new(left_label_x, left_y), section_style).draw(display)?;
        left_y += line_height;

        // STR
        label_str.clear();
        write!(label_str, "STR:").ok();
        Text::new(&label_str, Point::new(left_label_x, left_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.str).ok();
        Text::new(&value_str, Point::new(left_value_x, left_y), stat_value_style).draw(display)?;
        left_y += line_height;

        // DEX
        label_str.clear();
        write!(label_str, "DEX:").ok();
        Text::new(&label_str, Point::new(left_label_x, left_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.dex).ok();
        Text::new(&value_str, Point::new(left_value_x, left_y), stat_value_style).draw(display)?;
        left_y += line_height;

        // VIT
        label_str.clear();
        write!(label_str, "VIT:").ok();
        Text::new(&label_str, Point::new(left_label_x, left_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.vit).ok();
        Text::new(&value_str, Point::new(left_value_x, left_y), stat_value_style).draw(display)?;
        left_y += line_height;

        // INT
        label_str.clear();
        write!(label_str, "INT:").ok();
        Text::new(&label_str, Point::new(left_label_x, left_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.int).ok();
        Text::new(&value_str, Point::new(left_value_x, left_y), stat_value_style).draw(display)?;
        left_y += line_height;

        // LUK
        label_str.clear();
        write!(label_str, "LUK:").ok();
        Text::new(&label_str, Point::new(left_label_x, left_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.luk).ok();
        Text::new(&value_str, Point::new(left_value_x, left_y), stat_value_style).draw(display)?;

        // RIGHT COLUMN: Combat Stats
        let mut right_y = y;
        Text::new("Combat Stats", Point::new(right_label_x, right_y), section_style).draw(display)?;
        right_y += line_height;

        // ATK
        label_str.clear();
        write!(label_str, "ATK:").ok();
        Text::new(&label_str, Point::new(right_label_x, right_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.atk).ok();
        Text::new(&value_str, Point::new(right_value_x, right_y), stat_value_style).draw(display)?;
        right_y += line_height;

        // DEF
        label_str.clear();
        write!(label_str, "DEF:").ok();
        Text::new(&label_str, Point::new(right_label_x, right_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.def).ok();
        Text::new(&value_str, Point::new(right_value_x, right_y), stat_value_style).draw(display)?;
        right_y += line_height;

        // HIT
        label_str.clear();
        write!(label_str, "HIT:").ok();
        Text::new(&label_str, Point::new(right_label_x, right_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.hit).ok();
        Text::new(&value_str, Point::new(right_value_x, right_y), stat_value_style).draw(display)?;
        right_y += line_height;

        // FLEE
        label_str.clear();
        write!(label_str, "FLEE:").ok();
        Text::new(&label_str, Point::new(right_label_x, right_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{}", rustymon.flee).ok();
        Text::new(&value_str, Point::new(right_value_x, right_y), stat_value_style).draw(display)?;
        right_y += line_height;

        // CRIT
        label_str.clear();
        write!(label_str, "CRIT:").ok();
        Text::new(&label_str, Point::new(right_label_x, right_y), stat_label_style).draw(display)?;
        value_str.clear();
        write!(value_str, "{:.1}%", rustymon.crit_rate).ok();
        Text::new(&value_str, Point::new(right_value_x, right_y), stat_value_style).draw(display)?;

        // Continue from the taller column
        let _ = left_y.max(right_y);

        // Update and draw idle sprite animation
        if let Some(ref mut sprite) = self.idle_sprite {
            sprite.update();
            if let Err(e) = sprite.draw(display) {
                log::warn!("Failed to draw idle sprite: {:?}", e);
            }
        }

        // Draw buttons at bottom (moved up to avoid border)
        let button_y = 380;
        let in_team = rustymon_team.is_in_team(&rustymon.id);

        if in_team {
            // Leave team button
            Rectangle::new(Point::new(20, button_y), Size::new(160, 35))
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(Rgb888::new(80, 40, 40))
                        .stroke_color(Rgb888::new(160, 80, 80))
                        .stroke_width(2)
                        .build(),
                )
                .draw(display)?;

            let btn_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            Text::new("Leave team", Point::new(30, button_y + 23), btn_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (20, button_y, 160, 35),
                action: RustymonDetailAction::RemoveFromTeam,
            });
        } else {
            // Add to team button
            Rectangle::new(Point::new(20, button_y), Size::new(160, 35))
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(Rgb888::new(40, 80, 40))
                        .stroke_color(Rgb888::new(80, 160, 80))
                        .stroke_width(2)
                        .build(),
                )
                .draw(display)?;

            let btn_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            Text::new("Add to Team", Point::new(30, button_y + 23), btn_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (20, button_y, 160, 35),
                action: RustymonDetailAction::AddToTeam,
            });
        }

        // Skills button
        Rectangle::new(Point::new(190, button_y), Size::new(160, 35))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb888::new(60, 60, 100))
                    .stroke_color(Rgb888::new(120, 120, 200))
                    .stroke_width(2)
                    .build(),
            )
            .draw(display)?;

        let skills_btn_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("Skills", Point::new(237, button_y + 23), skills_btn_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (190, button_y, 160, 35),
            action: RustymonDetailAction::OpenSkills,
        });

        display.flush()?;
        Ok(())
    }

    /// Draw HP bar
    fn draw_hp_bar(
        &self,
        display: &mut Sh8601Driver,
        position: (i32, i32),
        current: u32,
        max: u32,
        width: u32,
    ) -> Result<(), Box<dyn Error>> {
        let (x, y) = position;
        let height = 12;

        // Background
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
            .draw(display)?;

        // Fill
        let fill_width = if max > 0 {
            ((current as f32 / max as f32) * width as f32) as u32
        } else {
            0
        };

        let fill_color = if current as f32 / max as f32 > 0.5 {
            Rgb888::new(100, 255, 100)
        } else if current as f32 / max as f32 > 0.2 {
            Rgb888::new(255, 200, 100)
        } else {
            Rgb888::new(255, 100, 100)
        };

        Rectangle::new(Point::new(x, y), Size::new(fill_width, height))
            .into_styled(PrimitiveStyle::with_fill(fill_color))
            .draw(display)?;

        // Border
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
            .draw(display)?;

        Ok(())
    }

    /// Draw EXP bar
    fn draw_exp_bar(
        &self,
        display: &mut Sh8601Driver,
        position: (i32, i32),
        current: u32,
        max: u32,
        width: u32,
    ) -> Result<(), Box<dyn Error>> {
        let (x, y) = position;
        let height = 12;

        // Background
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
            .draw(display)?;

        // Fill
        let fill_width = if max > 0 {
            ((current as f32 / max as f32) * width as f32) as u32
        } else {
            0
        };

        // Yellow/gold color like battle page
        Rectangle::new(Point::new(x, y), Size::new(fill_width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(255, 215, 0)))
            .draw(display)?;

        // Border
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
            .draw(display)?;

        Ok(())
    }
}

impl Default for RustymonDetailPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for RustymonDetailPage {
    fn update(&mut self) -> bool {
        true // Stay active until explicitly closed
    }

    fn draw(
        &mut self,
        _display: &mut Sh8601Driver,
        _full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // This page requires external data
        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering Rustymon detail page");
        self.needs_full_redraw = true;
        // Reset idle sprite animation when entering
        if let Some(ref mut sprite) = self.idle_sprite {
            sprite.reset_animation();
        }
    }

    fn on_exit(&mut self) {
        log::info!("Exiting Rustymon detail page");
    }

    fn mark_dirty(&mut self) {
        self.needs_full_redraw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.needs_full_redraw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
