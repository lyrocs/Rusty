//! Dungeon Defeat Page
//!
//! Shown when player loses in a dungeon. Displays rewards earned and offers retry/quit options.

use crate::display::St7789pDriver;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
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
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::new(180, 60, 60));
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let green_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 150, 50));
        let gold_style = MonoTextStyle::new(&FONT_7X13, Rgb888::new(180, 140, 50));

        if full_redraw {
            // Light theme background with slight red tint
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(250, 240, 240))?;
        }

        // Header card
        let header_rect = Rectangle::new(Point::new(10, 4), Size::new(220, 28));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(220, 150, 150))
            .build())
            .draw(display)?;

        Text::new("DEFEAT", Point::new(95, 22), title_style).draw(display)?;

        // Floor reached
        let dungeon_name = if self.dungeon_name.len() > 16 { &self.dungeon_name[..16] } else { &self.dungeon_name };
        let floor_text = format!("Floor {} - {}", self.floor_reached, dungeon_name);
        Text::new(&floor_text, Point::new(40, 46), dim_style).draw(display)?;

        // Rewards card
        let rewards_y = 54;
        let rewards_rect = Rectangle::new(Point::new(10, rewards_y), Size::new(220, 50));
        let rewards_rounded = RoundedRectangle::new(rewards_rect, CornerRadii::new(Size::new(8, 8)));
        rewards_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(250, 250, 255))
            .build())
            .draw(display)?;
        rewards_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(180, 185, 195))
            .stroke_width(1)
            .build())
            .draw(display)?;

        Text::new("Rewards:", Point::new(18, rewards_y + 14), dim_style).draw(display)?;

        let crystal_text = format!("+{} Crystals", self.crystals_earned);
        Text::new(&crystal_text, Point::new(18, rewards_y + 28), green_style).draw(display)?;

        let xp_text = format!("+{} XP/monster", self.xp_earned);
        Text::new(&xp_text, Point::new(18, rewards_y + 42), green_style).draw(display)?;

        // Record section
        let record_y = 110;
        if self.is_new_record {
            let record_rect = Rectangle::new(Point::new(10, record_y), Size::new(220, 32));
            let record_rounded = RoundedRectangle::new(record_rect, CornerRadii::new(Size::new(8, 8)));
            record_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(255, 245, 220))
                .build())
                .draw(display)?;
            record_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(200, 170, 100))
                .stroke_width(2)
                .build())
                .draw(display)?;

            let record_text = format!("NEW RECORD: Floor {}!", self.floor_reached);
            Text::new(&record_text, Point::new(45, record_y + 20), gold_style).draw(display)?;
        } else {
            let record_text = format!("Best: Floor {}", self.previous_record);
            Text::new(&record_text, Point::new(85, record_y + 18), dim_style).draw(display)?;
        }

        // Action buttons
        let button_y = 155;
        let button_width = 100u32;
        let button_height = 36u32;

        // Retry button
        let retry_rect = Rectangle::new(Point::new(15, button_y), Size::new(button_width, button_height));
        let retry_rounded = RoundedRectangle::new(retry_rect, CornerRadii::new(Size::new(8, 8)));
        retry_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(180, 230, 180))
            .build())
            .draw(display)?;
        retry_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(100, 180, 100))
            .stroke_width(2)
            .build())
            .draw(display)?;

        Text::new("RETRY", Point::new(42, button_y + 16), text_style).draw(display)?;
        let checkpoint_text = format!("Floor {}", self.last_checkpoint);
        Text::new(&checkpoint_text, Point::new(38, button_y + 30), dim_style).draw(display)?;

        self.retry_button = Some(retry_rect);

        // Quit button
        let quit_rect = Rectangle::new(Point::new(125, button_y), Size::new(button_width, button_height));
        let quit_rounded = RoundedRectangle::new(quit_rect, CornerRadii::new(Size::new(8, 8)));
        quit_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(240, 200, 200))
            .build())
            .draw(display)?;
        quit_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(200, 120, 120))
            .stroke_width(2)
            .build())
            .draw(display)?;

        Text::new("QUIT", Point::new(158, button_y + 22), text_style).draw(display)?;

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
