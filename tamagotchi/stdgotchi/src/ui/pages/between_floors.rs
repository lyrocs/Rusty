//! Between Floors Page
//!
//! Shown between dungeon floors, displays rewards and team status.
//! Allows player to continue to next floor or abandon run.

use crate::display::St7789pDriver;
use crate::game::core::{Element, Monster, ActiveBonus, ActiveBonusType};
use crate::game::core::bonus::StatBoostType;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action from between floors page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BetweenFloorsAction {
    /// No action
    None,
    /// Continue to next floor
    Continue,
    /// Abandon run (keep rewards)
    Abandon,
}

/// Data for displaying team monster status
#[derive(Clone)]
pub struct MonsterStatusData {
    pub name: String,
    pub element: Element,
    pub level: u8,
    pub hp_current: u16,
    pub hp_max: u16,
    pub is_alive: bool,
    pub xp_current: u32,
    pub xp_to_next: u32,
    pub xp_gained: u32,
}

/// Between floors page
pub struct BetweenFloorsPage {
    dungeon_name: String,
    current_floor: u16,
    floors_cleared: u16,
    crystals_earned: u32,
    xp_earned: u32,
    pub team_status: Vec<MonsterStatusData>,
    active_bonuses: Vec<ActiveBonus>,

    // Touch areas
    continue_button: Option<Rectangle>,
    abandon_button: Option<Rectangle>,

    dirty: bool,
}

impl BetweenFloorsPage {
    pub fn new(
        dungeon_name: String,
        current_floor: u16,
        floors_cleared: u16,
        crystals_earned: u32,
        xp_earned: u32,
        team_status: Vec<MonsterStatusData>,
        active_bonuses: Vec<ActiveBonus>,
    ) -> Self {
        Self {
            dungeon_name,
            current_floor,
            floors_cleared,
            crystals_earned,
            xp_earned,
            team_status,
            active_bonuses,
            continue_button: None,
            abandon_button: None,
            dirty: true,
        }
    }

    /// Update with combat results
    pub fn update_from_combat(&mut self, monsters: &[Monster], crystals: u32, xp: u32, floor: u16) {
        self.current_floor = floor;
        self.floors_cleared += 1;
        self.crystals_earned += crystals;
        self.xp_earned += xp;

        let alive_count = monsters.iter().filter(|m| m.is_alive()).count();
        let xp_per_monster = if alive_count > 0 { xp / alive_count as u32 } else { 0 };

        self.team_status = monsters.iter().map(|m| MonsterStatusData {
            name: m.name.clone(),
            element: m.element,
            level: m.level,
            hp_current: m.hp_current,
            hp_max: m.hp_max,
            is_alive: m.is_alive(),
            xp_current: m.xp,
            xp_to_next: m.xp_to_next,
            xp_gained: if m.is_alive() { xp_per_monster } else { 0 },
        }).collect();

        self.dirty = true;
    }

    /// Update active bonuses
    pub fn set_active_bonuses(&mut self, bonuses: Vec<ActiveBonus>) {
        self.active_bonuses = bonuses;
        self.dirty = true;
    }

    /// Handle touch input
    pub fn handle_touch(&self, x: i32, y: i32) -> BetweenFloorsAction {
        if let Some(rect) = self.continue_button {
            if rect.contains(Point::new(x, y)) {
                // Only allow continue if at least one monster alive
                if self.team_status.iter().any(|m| m.is_alive) {
                    return BetweenFloorsAction::Continue;
                }
            }
        }

        if let Some(rect) = self.abandon_button {
            if rect.contains(Point::new(x, y)) {
                return BetweenFloorsAction::Abandon;
            }
        }

        BetweenFloorsAction::None
    }

    fn element_color(element: Element) -> Rgb888 {
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(150, 120, 50),
            Element::Wind => Rgb888::new(100, 220, 150),
            Element::Thunder => Rgb888::new(255, 255, 100),
            Element::Shadow => Rgb888::new(150, 50, 200),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(180, 180, 220),
            Element::Neutral => Rgb888::new(180, 180, 180),
        }
    }

    fn element_char(element: Element) -> char {
        match element {
            Element::Fire => 'F',
            Element::Water => 'W',
            Element::Earth => 'E',
            Element::Wind => 'N',
            Element::Thunder => 'T',
            Element::Shadow => 'S',
            Element::Holy => 'H',
            Element::Ghost => 'G',
            Element::Neutral => 'N',
        }
    }
}

impl Page for BetweenFloorsPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(60, 60, 60));
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let green_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 150, 50));
        let red_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 80, 80));
        let buff_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 50, 180));

        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        // Header card
        let header_rect = Rectangle::new(Point::new(10, 2), Size::new(220, 20));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(150, 200, 150))
            .build())
            .draw(display)?;

        let dungeon_name = if self.dungeon_name.len() > 10 { &self.dungeon_name[..10] } else { &self.dungeon_name };
        let header_text = format!("{} Fl.{} Clear!", dungeon_name, self.current_floor);
        Text::new(&header_text, Point::new(20, 16), title_style).draw(display)?;

        // Active Buffs section (if any)
        let mut next_y = 24;
        if !self.active_bonuses.is_empty() {
            let buffs_rect = Rectangle::new(Point::new(10, next_y), Size::new(220, 22));
            let buffs_rounded = RoundedRectangle::new(buffs_rect, CornerRadii::new(Size::new(4, 4)));
            buffs_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(240, 235, 255))
                .build())
                .draw(display)?;

            // Build buff string
            let mut buff_str = String::new();
            for bonus in &self.active_bonuses {
                if !buff_str.is_empty() { buff_str.push_str(" "); }
                match &bonus.bonus_type {
                    ActiveBonusType::StatBoost { stat, percent } => {
                        let stat_name = match stat {
                            StatBoostType::Atk => "ATK",
                            StatBoostType::Def => "DEF",
                            StatBoostType::Spd => "SPD",
                            StatBoostType::AllStats => "ALL",
                        };
                        buff_str.push_str(&format!("{}+{}%[{}]", stat_name, (percent * 100.0) as u8, bonus.floors_remaining));
                    }
                    ActiveBonusType::CaptureBoost { multiplier } => {
                        buff_str.push_str(&format!("CAP{}x[{}]", multiplier, bonus.floors_remaining));
                    }
                }
            }
            // Truncate if too long
            if buff_str.len() > 35 { buff_str = buff_str[..35].to_string(); }
            Text::new(&buff_str, Point::new(14, next_y + 14), buff_style).draw(display)?;
            next_y += 24;
        }

        // Rewards section
        let rewards_rect = Rectangle::new(Point::new(10, next_y), Size::new(220, 24));
        let rewards_rounded = RoundedRectangle::new(rewards_rect, CornerRadii::new(Size::new(4, 4)));
        rewards_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(250, 250, 255))
            .build())
            .draw(display)?;

        let crystal_text = format!("+{} Crystals", self.crystals_earned);
        Text::new(&crystal_text, Point::new(14, next_y + 15), green_style).draw(display)?;
        let xp_text = format!("+{} XP (total)", self.xp_earned);
        Text::new(&xp_text, Point::new(110, next_y + 15), green_style).draw(display)?;
        next_y += 28;

        // Team status with HP and XP bars
        for (i, monster) in self.team_status.iter().take(3).enumerate() {
            let y = next_y + (i as i32 * 52);

            let (bg_color, border_color) = if monster.is_alive {
                (Rgb888::new(235, 255, 235), Rgb888::new(150, 200, 150))
            } else {
                (Rgb888::new(255, 235, 235), Rgb888::new(200, 150, 150))
            };

            let card_rect = Rectangle::new(Point::new(10, y), Size::new(220, 48));
            let card_rounded = RoundedRectangle::new(card_rect, CornerRadii::new(Size::new(6, 6)));
            card_rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(bg_color).build()).draw(display)?;
            card_rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(border_color).stroke_width(1).build()).draw(display)?;

            // Element and name with XP gained
            let elem_color = Self::element_color(monster.element);
            let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
            let monster_name = if monster.name.len() > 8 { &monster.name[..8] } else { &monster.name };
            Text::new(&format!("{} {} Lv.{}", Self::element_char(monster.element), monster_name, monster.level),
                Point::new(14, y + 12), elem_style).draw(display)?;

            // XP gained this floor
            let xp_gained_text = format!("+{}xp", monster.xp_gained);
            Text::new(&xp_gained_text, Point::new(160, y + 12), green_style).draw(display)?;

            // HP bar
            let bar_x = 14;
            let hp_bar_y = y + 18;
            let bar_width = 130u32;
            let bar_height = 10u32;

            // HP label
            Text::new("HP", Point::new(bar_x, hp_bar_y + 8), small_style).draw(display)?;
            let hp_bar_x = bar_x + 16;

            let hp_bg = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(bar_width, bar_height));
            display.fill_solid(&hp_bg, Rgb888::new(180, 180, 180))?;

            let hp_percent = if monster.hp_max > 0 { monster.hp_current as f32 / monster.hp_max as f32 } else { 0.0 };
            let hp_fill_width = ((bar_width as f32) * hp_percent) as u32;
            if hp_fill_width > 0 {
                let hp_color = if hp_percent > 0.5 { Rgb888::new(80, 180, 80) } else if hp_percent > 0.25 { Rgb888::new(200, 180, 60) } else { Rgb888::new(200, 80, 80) };
                let hp_fill = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(hp_fill_width, bar_height));
                display.fill_solid(&hp_fill, hp_color)?;
            }

            let hp_text = if monster.is_alive { format!("{}/{}", monster.hp_current, monster.hp_max) } else { "KO".to_string() };
            let hp_style = if monster.is_alive { small_style } else { red_style };
            Text::new(&hp_text, Point::new(hp_bar_x + bar_width as i32 + 4, hp_bar_y + 8), hp_style).draw(display)?;

            // XP bar
            let xp_bar_y = y + 32;
            Text::new("XP", Point::new(bar_x, xp_bar_y + 8), small_style).draw(display)?;
            let xp_bar_x = bar_x + 16;

            let xp_bg = Rectangle::new(Point::new(xp_bar_x, xp_bar_y), Size::new(bar_width, bar_height));
            display.fill_solid(&xp_bg, Rgb888::new(180, 180, 180))?;

            let xp_percent = if monster.xp_to_next > 0 { (monster.xp_current as f32 / monster.xp_to_next as f32).min(1.0) } else { 0.0 };
            let xp_fill_width = ((bar_width as f32) * xp_percent) as u32;
            if xp_fill_width > 0 {
                let xp_fill = Rectangle::new(Point::new(xp_bar_x, xp_bar_y), Size::new(xp_fill_width, bar_height));
                display.fill_solid(&xp_fill, Rgb888::new(100, 150, 220))?;
            }

            let xp_label = format!("{}/{}", monster.xp_current, monster.xp_to_next);
            Text::new(&xp_label, Point::new(xp_bar_x + bar_width as i32 + 4, xp_bar_y + 8), small_style).draw(display)?;
        }

        // Action buttons
        let button_y = 234;
        let button_width = 100u32;
        let button_height = 28u32;

        let can_continue = self.team_status.iter().any(|m| m.is_alive);

        // Continue button
        let (cont_bg, cont_border) = if can_continue {
            (Rgb888::new(180, 230, 180), Rgb888::new(100, 180, 100))
        } else {
            (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
        };

        let continue_rect = Rectangle::new(Point::new(15, button_y), Size::new(button_width, button_height));
        let continue_rounded = RoundedRectangle::new(continue_rect, CornerRadii::new(Size::new(8, 8)));
        continue_rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(cont_bg).build()).draw(display)?;
        continue_rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(cont_border).stroke_width(2).build()).draw(display)?;

        let next_floor_text = format!("FLOOR {}", self.current_floor + 1);
        let continue_style = if can_continue { text_style } else { dim_style };
        Text::new(&next_floor_text, Point::new(30, button_y + 18), continue_style).draw(display)?;
        self.continue_button = Some(continue_rect);

        // Abandon button
        let abandon_rect = Rectangle::new(Point::new(125, button_y), Size::new(button_width, button_height));
        let abandon_rounded = RoundedRectangle::new(abandon_rect, CornerRadii::new(Size::new(8, 8)));
        abandon_rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb888::new(240, 200, 200)).build()).draw(display)?;
        abandon_rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(Rgb888::new(200, 120, 120)).stroke_width(2).build()).draw(display)?;

        Text::new("ABANDON", Point::new(143, button_y + 18), text_style).draw(display)?;
        self.abandon_button = Some(abandon_rect);

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
