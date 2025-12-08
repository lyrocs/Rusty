//! Between Floors Page
//!
//! Shown between dungeon floors, displays rewards and team status.
//! Allows player to continue to next floor or abandon run.

use crate::display::Sh8601Driver;
use crate::game::core::{Element, Monster};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
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
}

/// Between floors page
pub struct BetweenFloorsPage {
    dungeon_name: String,
    current_floor: u16,
    floors_cleared: u16,
    crystals_earned: u32,
    xp_earned: u32,
    pub team_status: Vec<MonsterStatusData>,

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
    ) -> Self {
        Self {
            dungeon_name,
            current_floor,
            floors_cleared,
            crystals_earned,
            xp_earned,
            team_status,
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

        self.team_status = monsters.iter().map(|m| MonsterStatusData {
            name: m.name.clone(),
            element: m.element,
            level: m.level,
            hp_current: m.hp_current,
            hp_max: m.hp_max,
            is_alive: m.is_alive(),
        }).collect();

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
        }
    }
}

impl Page for BetweenFloorsPage {
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let green_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 100));
        let red_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(200, 100, 100));

        if full_redraw {
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        // ═══════════════════════════════════════
        // HEADER
        // ═══════════════════════════════════════
        let header_text = format!("{} - Floor {} Cleared!", self.dungeon_name, self.current_floor);
        Text::new(&header_text, Point::new(15, 35), title_style).draw(display)?;

        // Floors cleared
        let progress_text = format!("Floors cleared: {}", self.floors_cleared);
        Text::new(&progress_text, Point::new(15, 60), dim_style).draw(display)?;

        // ═══════════════════════════════════════
        // REWARDS SECTION
        // ═══════════════════════════════════════
        let rewards_y = 90;
        Text::new("Rewards so far:", Point::new(15, rewards_y), text_style).draw(display)?;

        let crystal_text = format!("  Crystals: +{}", self.crystals_earned);
        Text::new(&crystal_text, Point::new(15, rewards_y + 25), green_style).draw(display)?;

        let xp_text = format!("  XP: +{}", self.xp_earned);
        Text::new(&xp_text, Point::new(15, rewards_y + 45), green_style).draw(display)?;

        // ═══════════════════════════════════════
        // TEAM STATUS SECTION
        // ═══════════════════════════════════════
        let team_y = 165;
        Text::new("Team Status:", Point::new(15, team_y), text_style).draw(display)?;

        for (i, monster) in self.team_status.iter().take(3).enumerate() {
            let y = team_y + 25 + (i as i32 * 55);

            // Monster card background
            let bg_color = if monster.is_alive {
                Rgb888::new(35, 45, 35)
            } else {
                Rgb888::new(45, 35, 35)
            };
            let card_rect = Rectangle::new(Point::new(15, y), Size::new(338, 50));
            display.fill_solid(&card_rect, bg_color)?;

            // Element and name
            let elem_color = Self::element_color(monster.element);
            let elem_style = MonoTextStyle::new(&FONT_9X15, elem_color);
            let elem_char = Self::element_char(monster.element);
            Text::new(&format!("[{}] {} Lv.{}", elem_char, monster.name, monster.level),
                Point::new(25, y + 20), elem_style).draw(display)?;

            // HP bar
            let hp_bar_x = 25;
            let hp_bar_y = y + 30;
            let bar_width = 200u32;
            let bar_height = 12u32;

            // HP bar background
            let hp_bg = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(bar_width, bar_height));
            display.fill_solid(&hp_bg, Rgb888::new(40, 30, 30))?;

            // HP bar fill
            let hp_percent = if monster.hp_max > 0 {
                monster.hp_current as f32 / monster.hp_max as f32
            } else {
                0.0
            };
            let hp_fill_width = ((bar_width as f32) * hp_percent) as u32;
            if hp_fill_width > 0 {
                let hp_color = if hp_percent > 0.5 {
                    Rgb888::new(60, 180, 60)
                } else if hp_percent > 0.25 {
                    Rgb888::new(200, 180, 60)
                } else {
                    Rgb888::new(200, 60, 60)
                };
                let hp_fill = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(hp_fill_width, bar_height));
                display.fill_solid(&hp_fill, hp_color)?;
            }

            // HP text
            let hp_text = if monster.is_alive {
                format!("{}/{}", monster.hp_current, monster.hp_max)
            } else {
                "FAINTED".to_string()
            };
            let hp_style = if monster.is_alive { dim_style } else { red_style };
            Text::new(&hp_text, Point::new(hp_bar_x + bar_width as i32 + 10, hp_bar_y + 10), hp_style).draw(display)?;
        }

        // ═══════════════════════════════════════
        // ACTION BUTTONS
        // ═══════════════════════════════════════
        let button_y = 360;
        let button_height = 50u32;
        let button_spacing = 15;

        // Check if team can continue
        let can_continue = self.team_status.iter().any(|m| m.is_alive);

        // Continue button
        let continue_color = if can_continue {
            Rgb888::new(60, 100, 60)
        } else {
            Rgb888::new(40, 50, 40)
        };
        let continue_rect = Rectangle::new(Point::new(15, button_y), Size::new(165, button_height));
        display.fill_solid(&continue_rect, continue_color)?;
        Rectangle::new(Point::new(15, button_y), Size::new(165, button_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 200, 100), 2))
            .draw(display)?;

        let next_floor_text = format!("FLOOR {}", self.current_floor + 1);
        let continue_style = if can_continue { text_style } else { dim_style };
        Text::new(&next_floor_text, Point::new(45, button_y + 32), continue_style).draw(display)?;
        self.continue_button = Some(continue_rect);

        // Abandon button
        let abandon_rect = Rectangle::new(Point::new(188, button_y), Size::new(165, button_height));
        display.fill_solid(&abandon_rect, Rgb888::new(100, 60, 60))?;
        Rectangle::new(Point::new(188, button_y), Size::new(165, button_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(200, 100, 100), 2))
            .draw(display)?;
        Text::new("ABANDON", Point::new(228, button_y + 32), text_style).draw(display)?;
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
