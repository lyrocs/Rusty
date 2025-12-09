//! Expedition Map Selection Page
//!
//! Allows players to select a zone and map for expeditions.
//! Shows element requirements and capturable species.

use crate::display::St7789pDriver;
use crate::game::core::{Zone, TamerMap, Element};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13, FONT_9X15}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action from expedition map page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpeditionMapAction {
    /// No action
    None,
    /// Go back
    Back,
    /// Selected a map for expedition
    SelectMap(String),
}

/// Display data for a zone
#[derive(Clone)]
pub struct ZoneDisplayData {
    pub id: String,
    pub name: String,
    pub level_range: (u8, u8),
    pub is_unlocked: bool,
}

/// Display data for a map
#[derive(Clone)]
pub struct MapDisplayData {
    pub id: String,
    pub name: String,
    pub zone_id: String,
    pub level_range: (u8, u8),
    pub required_elements: Vec<Element>,
    pub capturable_count: usize,
}

/// Expedition Map Selection Page
pub struct ExpeditionMapPage {
    zones: Vec<ZoneDisplayData>,
    maps: Vec<MapDisplayData>,
    selected_zone_index: usize,
    selected_map_index: Option<usize>,
    scroll_offset: usize,
    dirty: bool,

    // Touch areas
    back_area: Option<Rectangle>,
    zone_areas: Vec<Rectangle>,
    map_areas: Vec<Rectangle>,
}

impl ExpeditionMapPage {
    pub fn new(zones: Vec<ZoneDisplayData>, maps: Vec<MapDisplayData>) -> Self {
        Self {
            zones,
            maps,
            selected_zone_index: 0,
            selected_map_index: None,
            scroll_offset: 0,
            dirty: true,
            back_area: None,
            zone_areas: Vec::new(),
            map_areas: Vec::new(),
        }
    }

    /// Get map indices for current zone
    fn map_indices_for_selected_zone(&self) -> Vec<usize> {
        if self.zones.is_empty() {
            return Vec::new();
        }
        let zone_id = &self.zones[self.selected_zone_index].id;
        self.maps.iter().enumerate().filter_map(|(i, m)| {
            // Match maps to zone using zone_id field
            if m.zone_id == *zone_id {
                Some(i)
            } else {
                None
            }
        }).collect()
    }

    /// Handle touch and return action
    pub fn handle_touch(&mut self, x: i32, y: i32) -> ExpeditionMapAction {
        // Check zone tabs
        for (i, rect) in self.zone_areas.iter().enumerate() {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                if i < self.zones.len() && self.zones[i].is_unlocked {
                    self.selected_zone_index = i;
                    self.selected_map_index = None;
                    self.dirty = true;
                }
                return ExpeditionMapAction::None;
            }
        }

        // Check map selection
        let map_indices = self.map_indices_for_selected_zone();
        for (i, rect) in self.map_areas.iter().enumerate() {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                if i < map_indices.len() {
                    let map_idx = map_indices[i];
                    return ExpeditionMapAction::SelectMap(self.maps[map_idx].id.clone());
                }
            }
        }

        ExpeditionMapAction::None
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

    fn element_char(element: &Element) -> char {
        match element {
            Element::Fire => 'F',
            Element::Water => 'W',
            Element::Earth => 'E',
            Element::Wind => 'A',
            Element::Thunder => 'T',
            Element::Shadow => 'S',
            Element::Holy => 'H',
            Element::Ghost => 'G',
        }
    }
}

impl Page for ExpeditionMapPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            // Clear screen - Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));

        // Title with margin
        Text::new("EXPEDITION", Point::new(75, 18), title_style).draw(display)?;

        // Zone tabs - rounded tabs for light theme
        self.zone_areas.clear();
        let tab_y = 28;
        let tab_width = 52u32;
        let tab_height = 24u32;

        for (i, zone) in self.zones.iter().take(4).enumerate() {
            let x = 10 + (i as i32 * (tab_width as i32 + 4));
            let rect = Rectangle::new(Point::new(x, tab_y), Size::new(tab_width, tab_height));
            let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(6, 6)));

            let (bg_color, border_color) = if i == self.selected_zone_index {
                (Rgb888::new(100, 150, 220), Rgb888::new(60, 100, 180))
            } else if zone.is_unlocked {
                (Rgb888::new(200, 210, 220), Rgb888::new(150, 160, 170))
            } else {
                (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
            };

            // Fill
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(bg_color)
                .build())
                .draw(display)?;

            // Border
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(border_color)
                .stroke_width(1)
                .build())
                .draw(display)?;

            self.zone_areas.push(rect);

            let text_color = if zone.is_unlocked {
                Rgb888::BLACK
            } else {
                Rgb888::new(140, 140, 140)
            };
            let zone_style = MonoTextStyle::new(&FONT_6X10, text_color);

            // Truncate zone name if too long
            let name = if zone.name.len() > 7 {
                &zone.name[..7]
            } else {
                &zone.name
            };
            Text::new(name, Point::new(x + 4, tab_y + 16), zone_style).draw(display)?;
        }

        // Selected zone info
        if !self.zones.is_empty() {
            let zone = &self.zones[self.selected_zone_index];
            let info = format!("Lv.{}-{}", zone.level_range.0, zone.level_range.1);
            Text::new(&info, Point::new(10, 64), dim_style).draw(display)?;
        }

        // Maps list - rounded cards with light theme
        self.map_areas.clear();
        let map_indices = self.map_indices_for_selected_zone();
        let list_y = 72;
        let item_height = 48u32;

        for (i, &map_idx) in map_indices.iter().take(4).enumerate() {
            let map = &self.maps[map_idx];
            let y = list_y + (i as i32 * (item_height as i32 + 4));
            let rect = Rectangle::new(Point::new(10, y), Size::new(220, item_height));
            let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(8, 8)));

            // Fill
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(220, 225, 235))
                .build())
                .draw(display)?;

            // Border
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(150, 160, 180))
                .stroke_width(2)
                .build())
                .draw(display)?;

            self.map_areas.push(rect);

            // Map name (truncate if too long)
            let name = if map.name.len() > 20 {
                &map.name[..20]
            } else {
                &map.name
            };
            Text::new(name, Point::new(16, y + 12), text_style).draw(display)?;

            // Level range
            let level_text = format!("Lv.{}-{}", map.level_range.0, map.level_range.1);
            Text::new(&level_text, Point::new(16, y + 24), dim_style).draw(display)?;

            // Required elements
            let mut elem_x = 16;
            Text::new("Need:", Point::new(elem_x, y + 36), dim_style).draw(display)?;
            elem_x += 36;

            for elem in &map.required_elements {
                let c = Self::element_char(elem);
                let elem_style = MonoTextStyle::new(&FONT_6X10, Self::element_color(elem));
                Text::new(&c.to_string(), Point::new(elem_x, y + 36), elem_style).draw(display)?;
                elem_x += 10;
            }

            // Capturable count
            let capture_text = format!("{} sp", map.capturable_count);
            Text::new(&capture_text, Point::new(120, y + 36), dim_style).draw(display)?;
        }

        if map_indices.is_empty() {
            Text::new("No maps available", Point::new(60, 140), dim_style).draw(display)?;
        }

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
