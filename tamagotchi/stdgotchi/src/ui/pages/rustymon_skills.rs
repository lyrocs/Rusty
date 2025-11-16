//! Rustymon Skills Page
//!
//! Shows detailed skills of a single Rustymon with enable/disable controls

use crate::display::Sh8601Driver;
use crate::game::{Rustymon, GameData};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, PrimitiveStyleBuilder},
    text::Text,
};
use std::error::Error;

/// Actions from skills page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustymonSkillsAction {
    ToggleSkill(usize), // Toggle skill at given slot (0-5)
    Close,
}

/// Touch area
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: RustymonSkillsAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Rustymon Skills page
pub struct RustymonSkillsPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
    scroll_offset: usize, // For scrolling through skills
}

impl RustymonSkillsPage {
    /// Create new Rustymon skills page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
            scroll_offset: 0,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<RustymonSkillsAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Rustymon skills action: {:?}", area.action);
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

    /// Draw Rustymon skills screen
    pub fn draw_skills(
        &mut self,
        display: &mut Sh8601Driver,
        rustymon: &Rustymon,
        game_data: &GameData,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.touch_areas.clear();

        // Draw header
        let header_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let mut header_str = heapless::String::<48>::new();
        write!(header_str, "{}'s Skills", rustymon.name).ok();
        Text::new(&header_str, Point::new(10, 25), header_style).draw(display)?;

        // Draw enabled count
        let enabled_count = rustymon.skills.enabled_skills.iter().filter(|s| s.is_some()).count();
        let info_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));
        let mut enabled_str = heapless::String::<32>::new();
        write!(enabled_str, "Enabled: {}/3", enabled_count).ok();
        Text::new(&enabled_str, Point::new(250, 25), info_style).draw(display)?;

        // Draw skills list
        let mut y = 60;
        let card_height = 70;
        let card_spacing = 8;

        let learned_count = rustymon.skills.learned_skills.len();

        if learned_count == 0 {
            let no_skills_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(120, 120, 120));
            Text::new("No skills learned yet", Point::new(20, y + 20), no_skills_style).draw(display)?;
        } else {
            for (idx, &skill_id) in rustymon.skills.learned_skills.iter().enumerate() {
                if let Some(skill) = game_data.get_skill(skill_id) {
                    let card_y = y + (idx as i32 * (card_height + card_spacing));

                    // Skip if card would go off screen (leaving room for back button)
                    if card_y + card_height > 410 {
                        break;
                    }

                    let is_enabled = rustymon.skills.enabled_skills.iter().any(|&s| s == Some(skill_id));

                    // Card background
                    let card_bg_color = if is_enabled {
                        Rgb888::new(30, 50, 40) // Greenish for enabled
                    } else {
                        Rgb888::new(40, 40, 50) // Default
                    };

                    Rectangle::new(Point::new(10, card_y), Size::new(348, card_height as u32))
                        .into_styled(PrimitiveStyle::with_fill(card_bg_color))
                        .draw(display)?;

                    // Card border
                    let border_color = if is_enabled {
                        Rgb888::new(80, 160, 80)
                    } else {
                        Rgb888::new(80, 80, 100)
                    };
                    Rectangle::new(Point::new(10, card_y), Size::new(348, card_height as u32))
                        .into_styled(PrimitiveStyle::with_stroke(border_color, 1))
                        .draw(display)?;

                    // Skill type indicator (passive vs active)
                    let type_color = if skill.is_passive() {
                        Rgb888::new(200, 150, 255) // Purple for passive
                    } else {
                        Rgb888::new(150, 200, 255) // Blue for active
                    };

                    // Skill name
                    let name_style = MonoTextStyle::new(&FONT_10X20, type_color);
                    let mut name_str = heapless::String::<32>::new();
                    write!(name_str, "{}", &skill.name[..skill.name.len().min(24)]).ok();
                    Text::new(&name_str, Point::new(20, card_y + 20), name_style).draw(display)?;

                    // Skill type label
                    let type_label = if skill.is_passive() { "PASSIVE" } else { "ACTIVE" };
                    let type_label_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(120, 120, 120));
                    Text::new(type_label, Point::new(250, card_y + 20), type_label_style).draw(display)?;

                    // Skill description (truncated)
                    let desc_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(160, 160, 160));
                    let desc_len = skill.description.len().min(32);
                    let mut desc_str = heapless::String::<40>::new();
                    write!(desc_str, "{}", &skill.description[..desc_len]).ok();
                    if skill.description.len() > 32 {
                        desc_str.push_str("...").ok();
                    }
                    Text::new(&desc_str, Point::new(20, card_y + 42), desc_style).draw(display)?;

                    // Status indicator text
                    let status_text = if is_enabled { "ENABLED" } else { "TAP TO ENABLE" };
                    let status_color = if is_enabled {
                        Rgb888::new(100, 200, 100)
                    } else {
                        Rgb888::new(120, 120, 120)
                    };
                    let status_style = MonoTextStyle::new(&FONT_10X20, status_color);
                    Text::new(status_text, Point::new(20, card_y + 62), status_style).draw(display)?;

                    // Add touch area for entire card to toggle
                    self.touch_areas.push(TouchArea {
                        bounds: (10, card_y, 348, card_height as u32),
                        action: RustymonSkillsAction::ToggleSkill(idx),
                    });
                }
            }
        }

        // Back button at bottom
        Rectangle::new(Point::new(134, 420), Size::new(100, 30))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb888::new(60, 60, 80))
                    .stroke_color(Rgb888::new(120, 120, 160))
                    .stroke_width(2)
                    .build(),
            )
            .draw(display)?;

        let back_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("Back", Point::new(160, 440), back_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (134, 420, 100, 30),
            action: RustymonSkillsAction::Close,
        });

        display.flush()?;
        Ok(())
    }
}

impl Default for RustymonSkillsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for RustymonSkillsPage {
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
        log::info!("Entering Rustymon skills page");
        self.needs_full_redraw = true;
        self.scroll_offset = 0;
    }

    fn on_exit(&mut self) {
        log::info!("Exiting Rustymon skills page");
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
