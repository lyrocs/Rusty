//! Monster Detail Page
//!
//! Displays detailed information about a single monster including stats, skill, and actions.

use crate::display::Sh8601Driver;
use crate::game::core::{Monster, Skill};
use crate::game::systems::progression::fusion::format_fusion;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    text::Text,
};
use std::error::Error;

/// Action from monster detail page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterDetailAction {
    /// No action
    None,
    /// Go back to list
    Back,
    /// Add to team
    AddToTeam,
    /// Remove from team
    RemoveFromTeam,
    /// Open upgrade screen
    Upgrade,
}

/// Monster Detail page
pub struct MonsterDetailPage {
    // Display data (snapshot)
    name: String,
    species_id: String,
    level: u8,
    element_name: String,
    fusion_count: u8,
    hp_current: u16,
    hp_max: u16,
    atk: u16,
    def: u16,
    spd: u16,
    xp: u32,
    xp_to_next: u32,
    power: u16,
    skill_name: String,
    skill_description: String,
    is_in_team: bool,

    // Touch areas
    back_area: Option<Rectangle>,
    team_button_area: Option<Rectangle>,
    upgrade_button_area: Option<Rectangle>,
    dirty: bool,
}

impl MonsterDetailPage {
    pub fn new(monster: &Monster, is_in_team: bool) -> Self {
        Self {
            name: monster.name.clone(),
            species_id: monster.species_id.clone(),
            level: monster.level,
            element_name: format!("{:?}", monster.element),
            fusion_count: monster.fusion_count,
            hp_current: monster.hp_current,
            hp_max: monster.hp_max,
            atk: monster.atk,
            def: monster.def,
            spd: monster.spd,
            xp: monster.xp,
            xp_to_next: monster.xp_to_next,
            power: monster.power(),
            skill_name: monster.skill.name.clone(),
            skill_description: monster.skill.description.clone(),
            is_in_team,
            back_area: None,
            team_button_area: None,
            upgrade_button_area: None,
            dirty: true,
        }
    }

    /// Handle touch and return action
    pub fn handle_touch(&mut self, x: i32, y: i32) -> MonsterDetailAction {
        let point = Point::new(x, y);

        // Check back button
        if let Some(ref rect) = self.back_area {
            if rect.contains(point) {
                return MonsterDetailAction::Back;
            }
        }

        // Check team button
        if let Some(ref rect) = self.team_button_area {
            if rect.contains(point) {
                if self.is_in_team {
                    return MonsterDetailAction::RemoveFromTeam;
                } else {
                    return MonsterDetailAction::AddToTeam;
                }
            }
        }

        // Check upgrade button
        if let Some(ref rect) = self.upgrade_button_area {
            if rect.contains(point) {
                return MonsterDetailAction::Upgrade;
            }
        }

        MonsterDetailAction::None
    }
}

impl Page for MonsterDetailPage {
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            // Clear screen
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let stat_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(180, 200, 255));

        // Draw header with name and level
        let fusion_str = format_fusion(self.fusion_count);
        let header = if fusion_str.is_empty() {
            format!("{} Lv.{}", self.name, self.level)
        } else {
            format!("{} {} Lv.{}", self.name, fusion_str, self.level)
        };
        Text::new(&header, Point::new(20, 35), title_style).draw(display)?;

        // Element and power
        Text::new(&format!("{} | PWR: {}", self.element_name, self.power), Point::new(20, 55), dim_style).draw(display)?;

        // Stats section
        let stats_y = 85;
        Text::new("--- STATS ---", Point::new(20, stats_y), dim_style).draw(display)?;

        let stat_col1_x = 30;
        let stat_col2_x = 180;

        // HP
        Text::new(&format!("HP: {}/{}", self.hp_current, self.hp_max), Point::new(stat_col1_x, stats_y + 20), stat_style).draw(display)?;

        // Draw HP bar
        let hp_bar_y = stats_y + 30;
        let bar_width = 120u32;
        let bar_height = 10u32;
        let hp_bg = Rectangle::new(Point::new(stat_col1_x, hp_bar_y), Size::new(bar_width, bar_height));
        display.fill_solid(&hp_bg, Rgb888::new(60, 60, 60))?;

        let hp_percent = self.hp_current as f32 / self.hp_max as f32;
        let hp_fill_width = ((bar_width as f32) * hp_percent) as u32;
        if hp_fill_width > 0 {
            let hp_fill = Rectangle::new(Point::new(stat_col1_x, hp_bar_y), Size::new(hp_fill_width, bar_height));
            let hp_color = if hp_percent > 0.5 {
                Rgb888::new(80, 200, 80)
            } else if hp_percent > 0.25 {
                Rgb888::new(200, 180, 60)
            } else {
                Rgb888::new(200, 60, 60)
            };
            display.fill_solid(&hp_fill, hp_color)?;
        }

        // XP
        Text::new(&format!("XP: {}/{}", self.xp, self.xp_to_next), Point::new(stat_col2_x, stats_y + 20), stat_style).draw(display)?;

        // Draw XP bar
        let xp_bg = Rectangle::new(Point::new(stat_col2_x, hp_bar_y), Size::new(bar_width, bar_height));
        display.fill_solid(&xp_bg, Rgb888::new(40, 40, 60))?;

        let xp_percent = if self.xp_to_next > 0 { self.xp as f32 / self.xp_to_next as f32 } else { 0.0 };
        let xp_fill_width = ((bar_width as f32) * xp_percent) as u32;
        if xp_fill_width > 0 {
            let xp_fill = Rectangle::new(Point::new(stat_col2_x, hp_bar_y), Size::new(xp_fill_width, bar_height));
            display.fill_solid(&xp_fill, Rgb888::new(100, 150, 255))?;
        }

        // Combat stats
        let combat_y = stats_y + 60;
        Text::new(&format!("ATK: {}", self.atk), Point::new(stat_col1_x, combat_y), stat_style).draw(display)?;
        Text::new(&format!("DEF: {}", self.def), Point::new(stat_col1_x + 80, combat_y), stat_style).draw(display)?;
        Text::new(&format!("SPD: {}", self.spd), Point::new(stat_col2_x, combat_y), stat_style).draw(display)?;

        // Fusion bonus
        if self.fusion_count > 0 {
            let bonus = self.fusion_count * 5;
            Text::new(&format!("Fusion Bonus: +{}%", bonus), Point::new(stat_col1_x, combat_y + 20), dim_style).draw(display)?;
        }

        // Skill section
        let skill_y = 210;
        Text::new("--- SKILL ---", Point::new(20, skill_y), dim_style).draw(display)?;

        Text::new(&self.skill_name, Point::new(30, skill_y + 20), title_style).draw(display)?;

        // Skill description (may need word wrap)
        let desc_y = skill_y + 45;
        // Simple word wrap - split at ~40 chars
        let desc = &self.skill_description;
        if desc.len() <= 45 {
            Text::new(desc, Point::new(30, desc_y), text_style).draw(display)?;
        } else {
            // Split description
            let mid = desc.char_indices()
                .take(45)
                .filter(|(_, c)| *c == ' ')
                .last()
                .map(|(i, _)| i)
                .unwrap_or(45);
            Text::new(&desc[..mid], Point::new(30, desc_y), text_style).draw(display)?;
            Text::new(&desc[mid..].trim(), Point::new(30, desc_y + 15), text_style).draw(display)?;
        }

        // Team status section
        let team_y = 320;
        let team_button_width = 150u32;
        let team_button_height = 35u32;

        let (button_text, button_color) = if self.is_in_team {
            ("REMOVE FROM TEAM", Rgb888::new(120, 60, 60))
        } else {
            ("ADD TO TEAM", Rgb888::new(60, 100, 60))
        };

        let team_btn = Rectangle::new(
            Point::new(184 - (team_button_width as i32 / 2), team_y),
            Size::new(team_button_width, team_button_height)
        );
        display.fill_solid(&team_btn, button_color)?;
        self.team_button_area = Some(team_btn);

        let text_x = 184 - ((button_text.len() as i32 * 6) / 2);
        Text::new(button_text, Point::new(text_x, team_y + 22), text_style).draw(display)?;

        // Upgrade button
        let upgrade_y = team_y + team_button_height as i32 + 10;
        let upgrade_btn = Rectangle::new(
            Point::new(184 - (team_button_width as i32 / 2), upgrade_y),
            Size::new(team_button_width, team_button_height)
        );
        display.fill_solid(&upgrade_btn, Rgb888::new(100, 80, 50))?;
        self.upgrade_button_area = Some(upgrade_btn);
        Text::new("UPGRADE", Point::new(143, upgrade_y + 22), text_style).draw(display)?;

        // Back button
        let back_rect = Rectangle::new(Point::new(15, 410), Size::new(80, 30));
        display.fill_solid(&back_rect, Rgb888::new(80, 60, 60))?;
        Text::new("< BACK", Point::new(25, 430), text_style).draw(display)?;
        self.back_area = Some(back_rect);

        display.flush()?;
        self.dirty = false;
        Ok(())
    }

    fn update(&mut self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.dirty
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
