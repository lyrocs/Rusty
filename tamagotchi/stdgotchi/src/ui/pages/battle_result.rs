//! Battle Result Page (Stub)
//!
//! NOTE: This is a placeholder for Phase 1 migration.
//! Will be replaced with proper battle result screen in Phase 2.

use crate::display::Sh8601Driver;
use crate::game::GameData;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;

/// Action returned from battle result page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleResultAction {
    None,
    Continue,
}

/// Battle result page - placeholder
pub struct BattleResultPage {
    exp_gained: u64,
    victory: bool,
    dirty: bool,
}

impl BattleResultPage {
    pub fn new(exp_gained: u64, victory: bool, _game_data: GameData) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            exp_gained,
            victory,
            dirty: true,
        })
    }

    pub fn handle_touch(&self, _x: i32, _y: i32) -> BattleResultAction {
        BattleResultAction::Continue
    }

    pub fn get_exp_gained(&self) -> u64 {
        self.exp_gained
    }

    pub fn is_victory(&self) -> bool {
        self.victory
    }
}

impl Page for BattleResultPage {
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            // Clear screen
            let bg_color = if self.victory {
                Rgb888::new(20, 40, 20)
            } else {
                Rgb888::new(40, 20, 20)
            };
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, bg_color)?;
        }

        let style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        let title = if self.victory { "VICTORY!" } else { "DEFEAT" };
        Text::new(title, Point::new(130, 150), style)
            .draw(display)?;

        let exp_text = format!("EXP: +{}", self.exp_gained);
        Text::new(&exp_text, Point::new(130, 220), style)
            .draw(display)?;

        Text::new("Tap to continue", Point::new(100, 350), style)
            .draw(display)?;

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
