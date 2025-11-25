//! Rustymon Summon Page
//!
//! Preview a new Rustymon before confirming summoning

use crate::display::Sh8601Driver;
use crate::game::Rustymon;
use crate::game::element_system::get_element_color;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, PrimitiveStyleBuilder},
    text::Text,
};
use std::error::Error;

/// Actions from summon page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustymonSummonAction {
    Confirm, // Confirm summoning
    Cancel,  // Cancel and return
}

/// Touch area
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: RustymonSummonAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Rustymon Summon page
pub struct RustymonSummonPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
}

impl RustymonSummonPage {
    /// Create new summon page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<RustymonSummonAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Summon action: {:?}", area.action);
                return Some(area.action);
            }
        }
        None
    }

    /// Draw summon preview screen
    pub fn draw_summon_preview(
        &mut self,
        display: &mut Sh8601Driver,
        rustymon: &Rustymon,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.touch_areas.clear();

        let element_color = get_element_color(rustymon.element);

        // Determine if this is summon or evolution (evolution_level > 0 means existing monster)
        let is_evolution = rustymon.evolution_level > 0;

        // Draw animated header with different color for evolution
        let header_color = if is_evolution {
            Rgb888::new(40, 80, 120) // Blue for evolution
        } else {
            Rgb888::new(80, 40, 120) // Purple for summon
        };
        Rectangle::new(Point::new(0, 0), Size::new(368, 70))
            .into_styled(PrimitiveStyle::with_fill(header_color))
            .draw(display)?;

        // Draw title based on action type
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 100));
        if is_evolution {
            let mut title = heapless::String::<32>::new();
            write!(title, "Evolution +{}", rustymon.evolution_level + 1).ok();
            Text::new(&title, Point::new(80, 25), title_style).draw(display)?;

            let subtitle_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 200));
            Text::new("All Stats +5%", Point::new(100, 50), subtitle_style).draw(display)?;
        } else {
            Text::new("✨ New Rustymon! ✨", Point::new(60, 25), title_style).draw(display)?;

            let subtitle_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 200));
            Text::new("Summoning Preview", Point::new(70, 50), subtitle_style).draw(display)?;
        }

        // Draw element-colored name panel
        Rectangle::new(Point::new(10, 80), Size::new(348, 50))
            .into_styled(PrimitiveStyle::with_fill(element_color))
            .draw(display)?;

        // Draw name
        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::BLACK);
        let mut name_str = heapless::String::<32>::new();
        write!(name_str, "{}", rustymon.name).ok();
        Text::new(&name_str, Point::new(20, 105), name_style).draw(display)?;

        // Draw element
        let elem_str = rustymon.element.as_str();
        Text::new(elem_str, Point::new(250, 105), name_style).draw(display)?;

        // Stats section
        let label_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 200));
        let value_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 255));
        let good_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 255, 100));

        let mut y = 150;
        let label_x = 30;
        let value_x = 180;
        let line_height = 25;

        // Base Stats
        let section_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 100));
        Text::new("Base Stats", Point::new(label_x, y), section_style).draw(display)?;
        y += line_height + 5;

        // Helper to determine if stat is good (above average)
        let avg_stat = 5 + rustymon.level + 2; // Roughly middle of range

        // STR
        let mut label_str = heapless::String::<16>::new();
        write!(label_str, "STR:").ok();
        Text::new(&label_str, Point::new(label_x, y), label_style).draw(display)?;

        let mut value_str = heapless::String::<16>::new();
        write!(value_str, "{}", rustymon.str).ok();
        let stat_style = if rustymon.str >= avg_stat {
            good_style
        } else {
            value_style
        };
        Text::new(&value_str, Point::new(value_x, y), stat_style).draw(display)?;
        y += line_height;

        // DEX
        label_str.clear();
        write!(label_str, "DEX:").ok();
        Text::new(&label_str, Point::new(label_x, y), label_style).draw(display)?;

        value_str.clear();
        write!(value_str, "{}", rustymon.dex).ok();
        let stat_style = if rustymon.dex >= avg_stat {
            good_style
        } else {
            value_style
        };
        Text::new(&value_str, Point::new(value_x, y), stat_style).draw(display)?;
        y += line_height;

        // VIT
        label_str.clear();
        write!(label_str, "VIT:").ok();
        Text::new(&label_str, Point::new(label_x, y), label_style).draw(display)?;

        value_str.clear();
        write!(value_str, "{}", rustymon.vit).ok();
        let stat_style = if rustymon.vit >= avg_stat {
            good_style
        } else {
            value_style
        };
        Text::new(&value_str, Point::new(value_x, y), stat_style).draw(display)?;
        y += line_height;

        // INT
        label_str.clear();
        write!(label_str, "INT:").ok();
        Text::new(&label_str, Point::new(label_x, y), label_style).draw(display)?;

        value_str.clear();
        write!(value_str, "{}", rustymon.int).ok();
        let stat_style = if rustymon.int >= avg_stat {
            good_style
        } else {
            value_style
        };
        Text::new(&value_str, Point::new(value_x, y), stat_style).draw(display)?;
        y += line_height;

        // LUK
        label_str.clear();
        write!(label_str, "LUK:").ok();
        Text::new(&label_str, Point::new(label_x, y), label_style).draw(display)?;

        value_str.clear();
        write!(value_str, "{}", rustymon.luk).ok();
        let stat_style = if rustymon.luk >= avg_stat {
            good_style
        } else {
            value_style
        };
        Text::new(&value_str, Point::new(value_x, y), stat_style).draw(display)?;
        y += line_height + 10;

        // Combat Stats Summary
        Text::new("Combat Stats", Point::new(label_x, y), section_style).draw(display)?;
        y += line_height + 5;

        // HP
        label_str.clear();
        write!(label_str, "HP:").ok();
        Text::new(&label_str, Point::new(label_x, y), label_style).draw(display)?;

        value_str.clear();
        write!(value_str, "{}", rustymon.max_hp).ok();
        Text::new(&value_str, Point::new(value_x, y), value_style).draw(display)?;
        y += line_height;

        // ATK
        label_str.clear();
        write!(label_str, "ATK:").ok();
        Text::new(&label_str, Point::new(label_x, y), label_style).draw(display)?;

        value_str.clear();
        write!(value_str, "{}", rustymon.atk).ok();
        Text::new(&value_str, Point::new(value_x, y), value_style).draw(display)?;
        y += line_height;

        // DEF
        label_str.clear();
        write!(label_str, "DEF:").ok();
        Text::new(&label_str, Point::new(label_x, y), label_style).draw(display)?;

        value_str.clear();
        write!(value_str, "{}", rustymon.def).ok();
        Text::new(&value_str, Point::new(value_x, y), value_style).draw(display)?;

        // Draw buttons at bottom
        // Confirm button (green)
        Rectangle::new(Point::new(200, 410), Size::new(150, 35))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb888::new(40, 100, 40))
                    .stroke_color(Rgb888::new(80, 200, 80))
                    .stroke_width(3)
                    .build(),
            )
            .draw(display)?;

        let confirm_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("✓ Summon!", Point::new(210, 435), confirm_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (200, 410, 150, 35),
            action: RustymonSummonAction::Confirm,
        });

        // Cancel button (red)
        Rectangle::new(Point::new(20, 410), Size::new(150, 35))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb888::new(80, 40, 40))
                    .stroke_color(Rgb888::new(160, 80, 80))
                    .stroke_width(2)
                    .build(),
            )
            .draw(display)?;

        let cancel_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("✗ Cancel", Point::new(40, 435), cancel_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (20, 410, 150, 35),
            action: RustymonSummonAction::Cancel,
        });

        display.flush()?;
        Ok(())
    }
}

impl Default for RustymonSummonPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for RustymonSummonPage {
    fn update(&mut self) -> bool {
        true // Stay active until explicitly closed
    }

    fn draw(
        &mut self,
        _display: &mut Sh8601Driver,
        _full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // This page requires external data
        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering Rustymon summon page");
        self.needs_full_redraw = true;
    }

    fn on_exit(&mut self) {
        log::info!("Exiting Rustymon summon page");
    }

    fn mark_dirty(&mut self) {
        self.needs_full_redraw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.needs_full_redraw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
