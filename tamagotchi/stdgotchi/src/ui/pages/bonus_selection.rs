//! Bonus Selection Page
//!
//! Displayed after clearing a dungeon floor (non-boss).
//! Player chooses one of 3 random bonus options.

use crate::display::St7789pDriver;
use crate::game::core::{DungeonBonus, StatBoostType};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action from bonus selection
#[derive(Debug, Clone, PartialEq)]
pub enum BonusSelectionAction {
    /// No action yet
    None,
    /// Player selected a bonus (index 0, 1, or 2)
    Selected(usize),
}

/// Bonus selection page
pub struct BonusSelectionPage {
    /// The three bonus options
    bonuses: [DungeonBonus; 3],
    /// Current floor (for display)
    current_floor: u16,
    /// Dungeon name (for display)
    dungeon_name: String,
    /// Touch areas for each bonus card
    bonus_areas: [Option<Rectangle>; 3],
    /// Selected bonus index (for visual feedback)
    selected_index: Option<usize>,
    /// Dirty flag for redraw
    dirty: bool,
    /// XP earned this floor (for BetweenFloors page)
    floor_xp: u32,
}

impl BonusSelectionPage {
    /// Create new bonus selection page
    pub fn new(bonuses: Vec<DungeonBonus>, current_floor: u16, dungeon_name: String, floor_xp: u32) -> Self {
        let bonuses_arr = [
            bonuses.get(0).cloned().unwrap_or(DungeonBonus::HealTeam { percent: 0.20 }),
            bonuses.get(1).cloned().unwrap_or(DungeonBonus::ExtraCrystals { amount: 10 }),
            bonuses.get(2).cloned().unwrap_or(DungeonBonus::HealTeam { percent: 0.15 }),
        ];

        Self {
            bonuses: bonuses_arr,
            current_floor,
            dungeon_name,
            bonus_areas: [None, None, None],
            selected_index: None,
            dirty: true,
            floor_xp,
        }
    }

    /// Get the XP earned this floor
    pub fn floor_xp(&self) -> u32 {
        self.floor_xp
    }

    /// Handle touch and return action
    pub fn handle_touch(&mut self, x: i32, y: i32) -> BonusSelectionAction {
        let point = Point::new(x, y);

        for (i, area) in self.bonus_areas.iter().enumerate() {
            if let Some(rect) = area {
                if rect.contains(point) {
                    self.selected_index = Some(i);
                    self.dirty = true;
                    return BonusSelectionAction::Selected(i);
                }
            }
        }

        BonusSelectionAction::None
    }

    /// Get the selected bonus
    pub fn get_bonus(&self, index: usize) -> Option<&DungeonBonus> {
        self.bonuses.get(index)
    }

    /// Get icon color for bonus type
    fn bonus_color(bonus: &DungeonBonus) -> Rgb888 {
        match bonus {
            DungeonBonus::HealTeam { .. } => Rgb888::new(100, 200, 100),    // Green
            DungeonBonus::StatBoost { stat, .. } => match stat {
                StatBoostType::Atk => Rgb888::new(220, 100, 100),           // Red
                StatBoostType::Def => Rgb888::new(100, 150, 220),           // Blue
                StatBoostType::Spd => Rgb888::new(220, 200, 100),           // Yellow
                StatBoostType::AllStats => Rgb888::new(200, 150, 220),      // Purple
            },
            DungeonBonus::CaptureBoost { .. } => Rgb888::new(220, 180, 100), // Orange
            DungeonBonus::ExtraCrystals { .. } => Rgb888::new(150, 220, 220), // Cyan
            DungeonBonus::SkipFloor => Rgb888::new(180, 180, 180),          // Gray
            DungeonBonus::ReviveMonster { .. } => Rgb888::new(255, 200, 200), // Light red
        }
    }
}

impl Page for BonusSelectionPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(80, 80, 80));

        if full_redraw {
            // Dark background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(40, 45, 60))?;
        }

        // Header
        let header_rect = Rectangle::new(Point::new(5, 5), Size::new(230, 30));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(8, 8)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(80, 120, 80))
            .build())
            .draw(display)?;

        let title = format!("Floor {} Clear!", self.current_floor);
        Text::new(&title, Point::new(65, 24), title_style).draw(display)?;

        // Subtitle
        Text::new("Choose a bonus:", Point::new(75, 50), small_style).draw(display)?;

        // Three bonus cards
        let card_height = 65u32;
        let card_spacing = 8i32;
        let start_y = 60i32;

        for (i, bonus) in self.bonuses.iter().enumerate() {
            let card_y = start_y + (i as i32) * (card_height as i32 + card_spacing);

            // Card background
            let is_selected = self.selected_index == Some(i);
            let card_bg = if is_selected {
                Rgb888::new(100, 150, 100)
            } else {
                Rgb888::new(250, 250, 255)
            };

            let card_rect = Rectangle::new(Point::new(10, card_y), Size::new(220, card_height));
            let card_rounded = RoundedRectangle::new(card_rect, CornerRadii::new(Size::new(10, 10)));
            card_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(card_bg)
                .build())
                .draw(display)?;

            // Border
            let border_color = if is_selected {
                Rgb888::new(50, 180, 50)
            } else {
                Self::bonus_color(bonus)
            };
            card_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(border_color)
                .stroke_width(2)
                .build())
                .draw(display)?;

            // Icon/badge area
            let badge_rect = Rectangle::new(Point::new(15, card_y + 8), Size::new(50, 48));
            let badge_rounded = RoundedRectangle::new(badge_rect, CornerRadii::new(Size::new(8, 8)));
            badge_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Self::bonus_color(bonus))
                .build())
                .draw(display)?;

            // Bonus name
            Text::new(bonus.name(), Point::new(75, card_y + 22), title_style).draw(display)?;

            // Bonus description
            let desc = bonus.description();
            let desc_text = if desc.len() > 28 { &desc[..28] } else { &desc };
            Text::new(desc_text, Point::new(75, card_y + 42), text_style).draw(display)?;

            // If there are more floors, show floor count
            if let DungeonBonus::StatBoost { floors, .. } = bonus {
                let floors_text = format!("{} floors", floors);
                Text::new(&floors_text, Point::new(180, card_y + 55), small_style).draw(display)?;
            }
            if let DungeonBonus::CaptureBoost { floors, .. } = bonus {
                let floors_text = format!("{} floors", floors);
                Text::new(&floors_text, Point::new(180, card_y + 55), small_style).draw(display)?;
            }

            // Store touch area
            self.bonus_areas[i] = Some(card_rect);
        }

        // Footer hint
        Text::new("Tap to select", Point::new(85, 270), small_style).draw(display)?;

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
