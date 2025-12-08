//! Dungeon Defeat Page
//!
//! Shown when player loses in a dungeon. Displays rewards earned and offers retry/quit options.

use crate::display::Sh8601Driver;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    text::Text,
};
use std::error::Error;

/// Action from defeat page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonDefeatAction {
    /// No action
    None,
    /// Retry from last checkpoint
    Retry,
    /// Quit to home
    Quit,
}

/// Dungeon defeat page data
pub struct DungeonDefeatPage {
    dungeon_id: String,
    dungeon_name: String,
    floor_reached: u16,
    crystals_earned: u32,
    xp_earned: u32,
    is_new_record: bool,
    previous_record: u16,
    last_checkpoint: u16,

    // Touch areas
    retry_button: Option<Rectangle>,
    quit_button: Option<Rectangle>,

    dirty: bool,
}

impl DungeonDefeatPage {
    pub fn new(
        dungeon_id: String,
        dungeon_name: String,
        floor_reached: u16,
        crystals_earned: u32,
        xp_earned: u32,
        previous_record: u16,
        last_checkpoint: u16,
    ) -> Self {
        let is_new_record = floor_reached > previous_record;
        Self {
            dungeon_id,
            dungeon_name,
            floor_reached,
            crystals_earned,
            xp_earned,
            is_new_record,
            previous_record,
            last_checkpoint,
            retry_button: None,
            quit_button: None,
            dirty: true,
        }
    }

    /// Get dungeon ID for retry
    pub fn dungeon_id(&self) -> &str {
        &self.dungeon_id
    }

    /// Get checkpoint to retry from
    pub fn retry_checkpoint(&self) -> u16 {
        self.last_checkpoint
    }

    /// Get crystals earned
    pub fn crystals_earned(&self) -> u32 {
        self.crystals_earned
    }

    /// Get XP earned
    pub fn xp_earned(&self) -> u32 {
        self.xp_earned
    }

    /// Get floor reached
    pub fn floor_reached(&self) -> u16 {
        self.floor_reached
    }

    /// Check if new record
    pub fn is_new_record(&self) -> bool {
        self.is_new_record
    }

    /// Handle touch input
    pub fn handle_touch(&self, x: i32, y: i32) -> DungeonDefeatAction {
        if let Some(rect) = self.retry_button {
            if rect.contains(Point::new(x, y)) {
                return DungeonDefeatAction::Retry;
            }
        }

        if let Some(rect) = self.quit_button {
            if rect.contains(Point::new(x, y)) {
                return DungeonDefeatAction::Quit;
            }
        }

        DungeonDefeatAction::None
    }
}

impl Page for DungeonDefeatPage {
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 60, 60));
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let green_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 100));
        let gold_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));

        if full_redraw {
            // Dark red-tinted background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(30, 20, 25))?;
        }

        // ═══════════════════════════════════════
        // HEADER - Skull icon and defeat message
        // ═══════════════════════════════════════
        let header_y = 50;

        // Skull decoration (simple text-based)
        Text::new("DEFEAT", Point::new(130, header_y), title_style).draw(display)?;

        // Floor reached
        let floor_text = format!("Floor {} - {}", self.floor_reached, self.dungeon_name);
        Text::new(&floor_text, Point::new(60, header_y + 35), dim_style).draw(display)?;

        // ═══════════════════════════════════════
        // REWARDS SECTION
        // ═══════════════════════════════════════
        let rewards_y = 130;

        // Section header
        Text::new("Rewards Obtained:", Point::new(30, rewards_y), text_style).draw(display)?;

        // Rewards box background
        let rewards_box = Rectangle::new(Point::new(30, rewards_y + 15), Size::new(308, 80));
        display.fill_solid(&rewards_box, Rgb888::new(40, 35, 40))?;
        Rectangle::new(Point::new(30, rewards_y + 15), Size::new(308, 80))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(80, 70, 80), 2))
            .draw(display)?;

        // Crystals
        let crystal_text = format!("+{} Crystals", self.crystals_earned);
        Text::new(&crystal_text, Point::new(50, rewards_y + 45), green_style).draw(display)?;

        // XP
        let xp_text = format!("+{} XP per monster", self.xp_earned);
        Text::new(&xp_text, Point::new(50, rewards_y + 70), green_style).draw(display)?;

        // ═══════════════════════════════════════
        // NEW RECORD SECTION (if applicable)
        // ═══════════════════════════════════════
        if self.is_new_record {
            let record_y = 250;

            // Gold banner background
            let banner = Rectangle::new(Point::new(30, record_y), Size::new(308, 50));
            display.fill_solid(&banner, Rgb888::new(60, 50, 20))?;
            Rectangle::new(Point::new(30, record_y), Size::new(308, 50))
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(255, 215, 0), 2))
                .draw(display)?;

            let record_text = format!("NEW RECORD: Floor {}!", self.floor_reached);
            Text::new(&record_text, Point::new(70, record_y + 32), gold_style).draw(display)?;
        } else {
            // Show previous record
            let record_y = 250;
            let record_text = format!("Best: Floor {}", self.previous_record);
            Text::new(&record_text, Point::new(130, record_y + 25), dim_style).draw(display)?;
        }

        // ═══════════════════════════════════════
        // ACTION BUTTONS
        // ═══════════════════════════════════════
        let button_y = 340;
        let button_height = 60u32;

        // Retry button (left)
        let retry_rect = Rectangle::new(Point::new(30, button_y), Size::new(145, button_height));
        display.fill_solid(&retry_rect, Rgb888::new(50, 80, 50))?;
        Rectangle::new(Point::new(30, button_y), Size::new(145, button_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 180, 100), 2))
            .draw(display)?;

        Text::new("RETRY", Point::new(68, button_y + 25), text_style).draw(display)?;

        // Show checkpoint info
        let checkpoint_text = format!("Floor {}", self.last_checkpoint);
        Text::new(&checkpoint_text, Point::new(58, button_y + 45), dim_style).draw(display)?;

        self.retry_button = Some(retry_rect);

        // Quit button (right)
        let quit_rect = Rectangle::new(Point::new(193, button_y), Size::new(145, button_height));
        display.fill_solid(&quit_rect, Rgb888::new(80, 50, 50))?;
        Rectangle::new(Point::new(193, button_y), Size::new(145, button_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(180, 100, 100), 2))
            .draw(display)?;

        Text::new("QUIT", Point::new(238, button_y + 35), text_style).draw(display)?;

        self.quit_button = Some(quit_rect);

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
