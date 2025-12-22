//! Monster Detail Page
//!
//! Displays detailed information about a single monster including stats, skill, and actions.

use crate::display::St7789pDriver;
use crate::game::core::Monster;
use crate::game::systems::progression::fusion::format_fusion;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
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
        // Get first equipped skill for display
        let (skill_name, skill_description) = monster.equipped_skills.first()
            .map(|s| (s.name.clone(), s.description.clone()))
            .unwrap_or_else(|| ("No Skill".to_string(), "No skill equipped".to_string()));

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
            skill_name,
            skill_description,
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
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let stat_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 80, 150));

        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        // Header card with name and level
        let header_rect = Rectangle::new(Point::new(10, 4), Size::new(220, 28));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 150, 200))
            .build())
            .draw(display)?;

        let fusion_str = format_fusion(self.fusion_count);
        let name = if self.name.len() > 12 { &self.name[..12] } else { &self.name };
        let header = if fusion_str.is_empty() {
            format!("{} Lv.{}", name, self.level)
        } else {
            format!("{} {} Lv.{}", name, fusion_str, self.level)
        };
        Text::new(&header, Point::new(20, 22), title_style).draw(display)?;

        // Element and power on header
        let info_text = format!("{} PWR:{}", self.element_name, self.power);
        Text::new(&info_text, Point::new(150, 22), text_style).draw(display)?;

        // Stats card
        let stats_rect = Rectangle::new(Point::new(10, 36), Size::new(220, 70));
        let stats_rounded = RoundedRectangle::new(stats_rect, CornerRadii::new(Size::new(8, 8)));
        stats_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(250, 250, 255))
            .build())
            .draw(display)?;
        stats_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(180, 185, 195))
            .stroke_width(1)
            .build())
            .draw(display)?;

        Text::new("STATS", Point::new(100, 48), dim_style).draw(display)?;

        // HP bar
        let bar_width = 90u32;
        let bar_height = 8u32;
        Text::new(&format!("HP {}/{}", self.hp_current, self.hp_max), Point::new(18, 62), stat_style).draw(display)?;
        let hp_bar_y = 66;
        let hp_bg = Rectangle::new(Point::new(18, hp_bar_y), Size::new(bar_width, bar_height));
        display.fill_solid(&hp_bg, Rgb888::new(200, 205, 215))?;

        let hp_percent = self.hp_current as f32 / self.hp_max as f32;
        let hp_fill_width = ((bar_width as f32) * hp_percent) as u32;
        if hp_fill_width > 0 {
            let hp_fill = Rectangle::new(Point::new(18, hp_bar_y), Size::new(hp_fill_width, bar_height));
            let hp_color = if hp_percent > 0.5 {
                Rgb888::new(80, 180, 80)
            } else if hp_percent > 0.25 {
                Rgb888::new(200, 180, 60)
            } else {
                Rgb888::new(200, 80, 80)
            };
            display.fill_solid(&hp_fill, hp_color)?;
        }

        // XP bar
        Text::new(&format!("XP {}/{}", self.xp, self.xp_to_next), Point::new(125, 62), stat_style).draw(display)?;
        let xp_bg = Rectangle::new(Point::new(125, hp_bar_y), Size::new(bar_width, bar_height));
        display.fill_solid(&xp_bg, Rgb888::new(200, 205, 215))?;

        let xp_percent = if self.xp_to_next > 0 { self.xp as f32 / self.xp_to_next as f32 } else { 0.0 };
        let xp_fill_width = ((bar_width as f32) * xp_percent) as u32;
        if xp_fill_width > 0 {
            let xp_fill = Rectangle::new(Point::new(125, hp_bar_y), Size::new(xp_fill_width, bar_height));
            display.fill_solid(&xp_fill, Rgb888::new(100, 150, 220))?;
        }

        // Combat stats row
        let combat_y = 88;
        Text::new(&format!("ATK:{}", self.atk), Point::new(18, combat_y), stat_style).draw(display)?;
        Text::new(&format!("DEF:{}", self.def), Point::new(75, combat_y), stat_style).draw(display)?;
        Text::new(&format!("SPD:{}", self.spd), Point::new(132, combat_y), stat_style).draw(display)?;

        // Fusion bonus
        if self.fusion_count > 0 {
            let bonus = self.fusion_count * 5;
            Text::new(&format!("+{}%", bonus), Point::new(190, combat_y), dim_style).draw(display)?;
        }

        // Skill card
        let skill_rect = Rectangle::new(Point::new(10, 110), Size::new(220, 55));
        let skill_rounded = RoundedRectangle::new(skill_rect, CornerRadii::new(Size::new(8, 8)));
        skill_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(250, 250, 255))
            .build())
            .draw(display)?;
        skill_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(180, 185, 195))
            .stroke_width(1)
            .build())
            .draw(display)?;

        Text::new("SKILL", Point::new(100, 122), dim_style).draw(display)?;

        // Skill name
        let skill_name = if self.skill_name.len() > 25 { &self.skill_name[..25] } else { &self.skill_name };
        Text::new(skill_name, Point::new(18, 138), title_style).draw(display)?;

        // Skill description (word wrap for small screen)
        let desc = &self.skill_description;
        let desc_y = 152;
        if desc.len() <= 35 {
            Text::new(desc, Point::new(18, desc_y), text_style).draw(display)?;
        } else {
            let mid = desc.char_indices()
                .take(35)
                .filter(|(_, c)| *c == ' ')
                .last()
                .map(|(i, _)| i)
                .unwrap_or(35);
            Text::new(&desc[..mid], Point::new(18, desc_y), text_style).draw(display)?;
        }

        // Buttons section
        let button_width = 100u32;
        let button_height = 28u32;

        // Team button
        let team_y = 172;
        let (button_text, bg_color, border_color) = if self.is_in_team {
            ("REMOVE", Rgb888::new(240, 180, 180), Rgb888::new(200, 100, 100))
        } else {
            ("ADD TEAM", Rgb888::new(180, 230, 180), Rgb888::new(100, 180, 100))
        };

        let team_rect = Rectangle::new(Point::new(15, team_y), Size::new(button_width, button_height));
        let team_rounded = RoundedRectangle::new(team_rect, CornerRadii::new(Size::new(8, 8)));
        team_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(bg_color)
            .build())
            .draw(display)?;
        team_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(border_color)
            .stroke_width(2)
            .build())
            .draw(display)?;
        self.team_button_area = Some(team_rect);

        let text_x = 15 + (button_width as i32 - (button_text.len() as i32 * 6)) / 2;
        Text::new(button_text, Point::new(text_x, team_y + 18), text_style).draw(display)?;

        // Upgrade button
        let upgrade_rect = Rectangle::new(Point::new(125, team_y), Size::new(button_width, button_height));
        let upgrade_rounded = RoundedRectangle::new(upgrade_rect, CornerRadii::new(Size::new(8, 8)));
        upgrade_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(230, 210, 170))
            .build())
            .draw(display)?;
        upgrade_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(180, 150, 100))
            .stroke_width(2)
            .build())
            .draw(display)?;
        self.upgrade_button_area = Some(upgrade_rect);
        Text::new("UPGRADE", Point::new(147, team_y + 18), text_style).draw(display)?;

        // No back button (use BOOT)
        self.back_area = None;

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
