//! Expedition Result Page
//!
//! Displays results from a completed expedition including
//! XP gained, crystals, essences, and any captured monsters.

use crate::display::St7789pDriver;
use crate::game::core::Element;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
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
            Element::Neutral => Rgb888::new(180, 180, 180),
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
            Element::Neutral => "Neutral",
        }
    }
}

impl Page for ExpeditionResultPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let gold_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 150, 0));
        let green_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 150, 50));

        // Title with rounded header
        let header_rect = Rectangle::new(Point::new(10, 4), Size::new(220, 22));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 180, 100))
            .build())
            .draw(display)?;
        Text::new("EXPEDITION COMPLETE", Point::new(45, 20), title_style).draw(display)?;

        // Map and duration card
        let info_rect = Rectangle::new(Point::new(10, 30), Size::new(220, 28));
        let info_rounded = RoundedRectangle::new(info_rect, CornerRadii::new(Size::new(6, 6)));
        info_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(220, 225, 235))
            .build())
            .draw(display)?;

        // Map name (truncate if needed)
        let map_name = if self.result.map_name.len() > 18 {
            &self.result.map_name[..18]
        } else {
            &self.result.map_name
        };
        Text::new(map_name, Point::new(16, 42), text_style).draw(display)?;
        let dur_text = format!("{}min", self.result.duration_minutes);
        Text::new(&dur_text, Point::new(180, 42), dim_style).draw(display)?;

        // Results card
        let results_rect = Rectangle::new(Point::new(10, 62), Size::new(220, 130));
        let results_rounded = RoundedRectangle::new(results_rect, CornerRadii::new(Size::new(8, 8)));
        results_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(250, 250, 255))
            .build())
            .draw(display)?;
        results_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(180, 185, 195))
            .stroke_width(1)
            .build())
            .draw(display)?;

        // XP rewards section
        let mut y = 76;
        Text::new("EXPERIENCE", Point::new(85, y), dim_style).draw(display)?;
        y += 12;

        for (i, name) in self.result.monster_names.iter().take(3).enumerate() {
            let truncated_name = if name.len() > 12 { &name[..12] } else { name };
            let xp_text = format!("{}: +{} XP", truncated_name, self.result.xp_per_monster);
            Text::new(&xp_text, Point::new(20, y), green_style).draw(display)?;
            y += 12;
        }

        // Resources section
        y += 6;
        Text::new("RESOURCES", Point::new(90, y), dim_style).draw(display)?;
        y += 12;

        // Crystals
        let crystal_text = format!("Crystals: +{}", self.result.crystals);
        Text::new(&crystal_text, Point::new(20, y), gold_style).draw(display)?;
        y += 12;

        // Essences (show up to 2)
        for (element, amount) in self.result.essences.iter().take(2) {
            let ess_style = MonoTextStyle::new(&FONT_6X10, Self::element_color(element));
            let ess_text = format!("{}: +{}", Self::element_name(element), amount);
            Text::new(&ess_text, Point::new(20, y), ess_style).draw(display)?;
            y += 12;
        }

        // Capture section
        y += 6;
        Text::new("CAPTURE", Point::new(95, y), dim_style).draw(display)?;
        y += 12;

        if let Some(ref species_name) = self.result.captured_species {
            if self.result.was_fusion {
                let truncated = if species_name.len() > 15 { &species_name[..15] } else { species_name };
                let capture_text = format!("Fused: {} (+5%)", truncated);
                let fusion_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 100, 200));
                Text::new(&capture_text, Point::new(20, y), fusion_style).draw(display)?;
            } else {
                let truncated = if species_name.len() > 18 { &species_name[..18] } else { species_name };
                let capture_text = format!("Got: {}", truncated);
                let capture_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 150, 200));
                Text::new(&capture_text, Point::new(20, y), capture_style).draw(display)?;
            }
        } else {
            Text::new("No capture", Point::new(20, y), dim_style).draw(display)?;
        }

        // Buttons row - rounded buttons
        let button_y = 200i32;
        let button_height = 32u32;
        let button_width = 100u32;

        // RERUN button (left) - Blue themed
        let rerun_rect = Rectangle::new(Point::new(15, button_y), Size::new(button_width, button_height));
        let rerun_rounded = RoundedRectangle::new(rerun_rect, CornerRadii::new(Size::new(8, 8)));
        rerun_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 150, 220))
            .build())
            .draw(display)?;
        rerun_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(60, 100, 180))
            .stroke_width(2)
            .build())
            .draw(display)?;
        Text::new("RERUN", Point::new(45, button_y + 20), text_style).draw(display)?;
        self.rerun_area = Some(rerun_rect);

        // CONTINUE button (right) - Green themed
        let continue_rect = Rectangle::new(Point::new(125, button_y), Size::new(button_width, button_height));
        let continue_rounded = RoundedRectangle::new(continue_rect, CornerRadii::new(Size::new(8, 8)));
        continue_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 200, 100))
            .build())
            .draw(display)?;
        continue_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(60, 160, 60))
            .stroke_width(2)
            .build())
            .draw(display)?;
        Text::new("CONTINUE", Point::new(145, button_y + 20), text_style).draw(display)?;
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
