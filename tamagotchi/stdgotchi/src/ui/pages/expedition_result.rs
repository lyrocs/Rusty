//! Expedition Result Page
//!
//! Displays results from a completed expedition including
//! XP gained, crystals, essences, and any captured monsters.

use crate::display::Sh8601Driver;
use crate::game::core::Element;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Rectangle,
    text::Text,
};
use std::error::Error;

/// Action from result page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpeditionResultAction {
    /// No action
    None,
    /// Continue (dismiss results)
    Continue,
    /// Rerun same expedition with same monsters
    Rerun,
}

/// Expedition result data
#[derive(Clone)]
pub struct ExpeditionResultData {
    pub map_name: String,
    pub map_id: String,
    pub duration_minutes: u32,
    pub duration_seconds: u64,
    pub xp_per_monster: u32,
    pub monster_names: Vec<String>,
    pub monster_ids: Vec<String>,
    pub crystals: u16,
    pub essences: Vec<(Element, u8)>,
    pub captured_species: Option<String>,
    pub was_fusion: bool,
}

/// Expedition Result Page
pub struct ExpeditionResultPage {
    result: ExpeditionResultData,
    dirty: bool,
    continue_area: Option<Rectangle>,
    rerun_area: Option<Rectangle>,
}

impl ExpeditionResultPage {
    pub fn new(result: ExpeditionResultData) -> Self {
        Self {
            result,
            dirty: true,
            continue_area: None,
            rerun_area: None,
        }
    }

    /// Get rerun data (map_id, monster_ids, duration_seconds)
    pub fn rerun_data(&self) -> (&str, &[String], u64) {
        (&self.result.map_id, &self.result.monster_ids, self.result.duration_seconds)
    }

    /// Handle touch and return action
    pub fn handle_touch(&mut self, x: i32, y: i32) -> ExpeditionResultAction {
        if let Some(ref rect) = self.continue_area {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                return ExpeditionResultAction::Continue;
            }
        }

        if let Some(ref rect) = self.rerun_area {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                return ExpeditionResultAction::Rerun;
            }
        }

        ExpeditionResultAction::None
    }

    fn element_color(element: &Element) -> Rgb888 {
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(150, 100, 50),
            Element::Wind => Rgb888::new(100, 200, 100),
            Element::Thunder => Rgb888::new(255, 255, 50),
            Element::Shadow => Rgb888::new(100, 50, 150),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(150, 150, 200),
        }
    }

    fn element_name(element: &Element) -> &'static str {
        match element {
            Element::Fire => "Fire",
            Element::Water => "Water",
            Element::Earth => "Earth",
            Element::Wind => "Wind",
            Element::Thunder => "Thunder",
            Element::Shadow => "Shadow",
            Element::Holy => "Holy",
            Element::Ghost => "Ghost",
        }
    }
}

impl Page for ExpeditionResultPage {
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let gold_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 215, 0));
        let green_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 100));

        // Title
        Text::new("EXPEDITION COMPLETE", Point::new(60, 30), title_style).draw(display)?;

        // Map and duration
        Text::new(&self.result.map_name, Point::new(15, 60), text_style).draw(display)?;
        let dur_text = format!("{}min expedition", self.result.duration_minutes);
        Text::new(&dur_text, Point::new(200, 60), dim_style).draw(display)?;

        // Separator
        let sep = Rectangle::new(Point::new(15, 75), Size::new(338, 2));
        display.fill_solid(&sep, Rgb888::new(60, 65, 75))?;

        // XP rewards section
        let mut y = 95;
        Text::new("--- EXPERIENCE ---", Point::new(80, y), dim_style).draw(display)?;
        y += 22;

        for name in &self.result.monster_names {
            let xp_text = format!("{}: +{} XP", name, self.result.xp_per_monster);
            Text::new(&xp_text, Point::new(30, y), green_style).draw(display)?;
            y += 20;
        }

        // Resources section
        y += 15;
        Text::new("--- RESOURCES ---", Point::new(80, y), dim_style).draw(display)?;
        y += 22;

        // Crystals
        let crystal_text = format!("Crystals: +{}", self.result.crystals);
        Text::new(&crystal_text, Point::new(30, y), gold_style).draw(display)?;
        y += 22;

        // Essences
        for (element, amount) in &self.result.essences {
            let ess_style = MonoTextStyle::new(&FONT_9X15, Self::element_color(element));
            let ess_text = format!("{} Essence: +{}", Self::element_name(element), amount);
            Text::new(&ess_text, Point::new(30, y), ess_style).draw(display)?;
            y += 20;
        }

        // Capture section
        y += 15;
        Text::new("--- CAPTURE ---", Point::new(95, y), dim_style).draw(display)?;
        y += 22;

        if let Some(ref species_name) = self.result.captured_species {
            if self.result.was_fusion {
                let capture_text = format!("Fused: {} (+5% stats)", species_name);
                let fusion_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(200, 150, 255));
                Text::new(&capture_text, Point::new(30, y), fusion_style).draw(display)?;
            } else {
                let capture_text = format!("Captured: {}", species_name);
                let capture_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 255));
                Text::new(&capture_text, Point::new(30, y), capture_style).draw(display)?;
            }
        } else {
            Text::new("No capture this time", Point::new(30, y), dim_style).draw(display)?;
        }

        // Buttons row - positioned like home page buttons for consistent touch
        let button_y = 370i32;
        let button_height = 55u32;
        let button_width = 160u32;
        let button_spacing = 175i32;

        // RERUN button (left)
        let rerun_rect = Rectangle::new(Point::new(15, button_y), Size::new(button_width, button_height));
        display.fill_solid(&rerun_rect, Rgb888::new(80, 80, 120))?;
        Text::new("RERUN", Point::new(60, button_y + 35), text_style).draw(display)?;
        self.rerun_area = Some(rerun_rect);

        // CONTINUE button (right)
        let continue_rect = Rectangle::new(Point::new(15 + button_spacing, button_y), Size::new(button_width, button_height));
        display.fill_solid(&continue_rect, Rgb888::new(60, 100, 60))?;
        Text::new("CONTINUE", Point::new(15 + button_spacing + 30, button_y + 35), text_style).draw(display)?;
        self.continue_area = Some(continue_rect);

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
