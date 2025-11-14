//! Rustymon Detail Page
//!
//! Shows detailed stats of a single Rustymon and allows team management

use crate::display::Sh8601Driver;
use crate::game::{Rustymon, RustymonTeam, GameData};
use crate::game::element_system::get_element_color;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, PrimitiveStyleBuilder},
    text::Text,
};
use std::error::Error;

/// Actions from detail page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustymonDetailAction {
    AddToTeam,
    RemoveFromTeam,
    ToggleSkill(usize), // Toggle skill at given slot (0-5)
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
}

impl RustymonDetailPage {
    /// Create new Rustymon detail page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
        }
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

    /// Toggle skill on/off for a Rustymon
    /// Returns true if successful, false if unable (e.g., max 3 enabled)
    pub fn toggle_skill(rustymon: &mut Rustymon, skill_index: usize) -> bool {
        if skill_index >= rustymon.skills.learned_skills.len() {
            return false;
        }

        let skill_id = rustymon.skills.learned_skills[skill_index];

        // Check if skill is currently enabled
        let current_slot = rustymon.skills.enabled_skills
            .iter()
            .position(|&s| s == Some(skill_id));

        if let Some(slot) = current_slot {
            // Skill is enabled, disable it
            rustymon.skills.disable_skill(slot);
            log::info!("Disabled skill ID {} from slot {}", skill_id, slot);
            true
        } else {
            // Skill is not enabled, try to enable it
            // Find first empty slot
            if let Some(empty_slot) = rustymon.skills.enabled_skills.iter().position(|s| s.is_none()) {
                rustymon.skills.enable_skill(skill_id, empty_slot);
                log::info!("Enabled skill ID {} in slot {}", skill_id, empty_slot);
                true
            } else {
                // All 3 slots are full
                log::warn!("Cannot enable skill: all 3 slots are full");
                false
            }
        }
    }

    /// Draw Rustymon detail screen
    pub fn draw_rustymon_detail(
        &mut self,
        display: &mut Sh8601Driver,
        rustymon: &Rustymon,
        rustymon_team: &RustymonTeam,
        game_data: &GameData,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.touch_areas.clear();

        let element_color = get_element_color(rustymon.element);

        // Draw header with element color
        Rectangle::new(Point::new(0, 0), Size::new(368, 60))
            .into_styled(PrimitiveStyle::with_fill(element_color))
            .draw(display)?;

        // Draw name
        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::BLACK);
        let mut name_str = heapless::String::<32>::new();
        write!(name_str, "{}", rustymon.name).ok();
        Text::new(&name_str, Point::new(10, 25), name_style).draw(display)?;

        // Draw level
        let mut level_str = heapless::String::<16>::new();
        write!(level_str, "Lv {}", rustymon.level).ok();
        Text::new(&level_str, Point::new(10, 50), name_style).draw(display)?;

        // Draw element name
        let elem_str = rustymon.element.as_str();
        Text::new(elem_str, Point::new(250, 50), name_style).draw(display)?;

        // Stats section - compact layout
        let stat_label_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 200));
        let stat_value_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 255));

        let mut y = 80;
        let left_label_x = 20;
        let left_value_x = left_label_x + 60; // Compact spacing
        let right_label_x = 194;
        let right_value_x = right_label_x + 60; // Compact spacing
        let line_height = 22;

        // HP (full width)
        let mut label_str = heapless::String::<16>::new();
        write!(label_str, "HP:").ok();
        Text::new(&label_str, Point::new(left_label_x, y), stat_label_style).draw(display)?;

        let mut value_str = heapless::String::<24>::new();
        write!(value_str, "{}/{}", rustymon.current_hp, rustymon.max_hp).ok();
        Text::new(&value_str, Point::new(left_value_x, y), stat_value_style).draw(display)?;

        // HP bar
        self.draw_hp_bar(
            display,
            (left_label_x + 120, y - 15),
            rustymon.current_hp,
            rustymon.max_hp,
            130,
        )?;

        y += line_height + 8;

        // EXP (full width)
        label_str.clear();
        write!(label_str, "EXP:").ok();
        Text::new(&label_str, Point::new(left_label_x, y), stat_label_style).draw(display)?;

        value_str.clear();
        write!(value_str, "{}/{}", rustymon.exp, rustymon.exp_to_next).ok();
        Text::new(&value_str, Point::new(left_value_x, y), stat_value_style).draw(display)?;

        // EXP bar
        self.draw_exp_bar(
            display,
            (left_label_x + 120, y - 15),
            rustymon.exp,
            rustymon.exp_to_next,
            130,
        )?;

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
        y = left_y.max(right_y) + 5;

        // Skills Section (full width, where Combat Stats used to be)
        y += 10;
        Text::new("Skills", Point::new(left_label_x, y), section_style).draw(display)?;
        y += line_height;

        // Draw skills (max 6 skills, show learned count)
        let learned_count = rustymon.skills.learned_skills.len();
        let mut skills_header = heapless::String::<32>::new();
        write!(skills_header, "Learned: {}/6", learned_count).ok();
        Text::new(&skills_header, Point::new(left_label_x, y), stat_label_style).draw(display)?;
        y += line_height - 3;

        // Draw each learned skill
        for (idx, &skill_id) in rustymon.skills.learned_skills.iter().enumerate() {
            if let Some(skill) = game_data.get_skill(skill_id) {
                let is_enabled = rustymon.skills.enabled_skills.iter().any(|&s| s == Some(skill_id));

                // Determine skill color (passive vs active)
                let skill_color = if skill.is_passive() {
                    Rgb888::new(200, 150, 255) // Purple for passive
                } else {
                    Rgb888::new(150, 200, 255) // Blue for active
                };

                // Skill name
                let mut skill_name = heapless::String::<24>::new();
                write!(skill_name, "{}", &skill.name[..skill.name.len().min(18)]).ok();
                let name_style = MonoTextStyle::new(&FONT_10X20, skill_color);
                Text::new(&skill_name, Point::new(left_label_x, y), name_style).draw(display)?;

                // Enabled indicator / Toggle button
                let button_x = 250;
                let button_y = y - 18;
                let button_w = 80;
                let button_h = 22;

                if is_enabled {
                    // Green "ON" button
                    Rectangle::new(Point::new(button_x, button_y), Size::new(button_w, button_h))
                        .into_styled(
                            PrimitiveStyleBuilder::new()
                                .fill_color(Rgb888::new(40, 100, 40))
                                .stroke_color(Rgb888::new(80, 200, 80))
                                .stroke_width(1)
                                .build(),
                        )
                        .draw(display)?;
                    let btn_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
                    Text::new("ON", Point::new(button_x + 25, y - 3), btn_text_style).draw(display)?;
                } else {
                    // Gray "OFF" button
                    Rectangle::new(Point::new(button_x, button_y), Size::new(button_w, button_h))
                        .into_styled(
                            PrimitiveStyleBuilder::new()
                                .fill_color(Rgb888::new(60, 60, 60))
                                .stroke_color(Rgb888::new(120, 120, 120))
                                .stroke_width(1)
                                .build(),
                        )
                        .draw(display)?;
                    let btn_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));
                    Text::new("OFF", Point::new(button_x + 20, y - 3), btn_text_style).draw(display)?;
                }

                // Add touch area for toggle
                self.touch_areas.push(TouchArea {
                    bounds: (button_x, button_y, button_w, button_h),
                    action: RustymonDetailAction::ToggleSkill(idx),
                });

                y += line_height - 2;
            }
        }

        // If no skills learned, show message
        if learned_count == 0 {
            let no_skills_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(120, 120, 120));
            Text::new("No skills learned yet", Point::new(left_label_x, y), no_skills_style).draw(display)?;
        }

        // Draw buttons at bottom
        let in_team = rustymon_team.is_in_team(&rustymon.id);

        if in_team {
            // Remove from team button
            Rectangle::new(Point::new(120, 420), Size::new(140, 30))
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(Rgb888::new(80, 40, 40))
                        .stroke_color(Rgb888::new(160, 80, 80))
                        .stroke_width(2)
                        .build(),
                )
                .draw(display)?;

            let btn_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            Text::new("Remove", Point::new(140, 440), btn_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (120, 420, 140, 30),
                action: RustymonDetailAction::RemoveFromTeam,
            });
        } else {
            // Add to team button
            Rectangle::new(Point::new(120, 420), Size::new(140, 30))
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(Rgb888::new(40, 80, 40))
                        .stroke_color(Rgb888::new(80, 160, 80))
                        .stroke_width(2)
                        .build(),
                )
                .draw(display)?;

            let btn_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            Text::new("Add to Team", Point::new(125, 440), btn_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (120, 420, 140, 30),
                action: RustymonDetailAction::AddToTeam,
            });
        }

        // Back button
        Rectangle::new(Point::new(10, 420), Size::new(100, 30))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb888::new(60, 60, 80))
                    .stroke_color(Rgb888::new(120, 120, 160))
                    .stroke_width(2)
                    .build(),
            )
            .draw(display)?;

        let back_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("Back", Point::new(30, 440), back_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (10, 420, 100, 30),
            action: RustymonDetailAction::Close,
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

        Rectangle::new(Point::new(x, y), Size::new(fill_width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 150, 255)))
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
