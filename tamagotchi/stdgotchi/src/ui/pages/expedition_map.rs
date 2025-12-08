//! Expedition Map Selection Page
//!
//! Allows players to select a zone and map for expeditions.
//! Shows element requirements and capturable species.

use crate::display::Sh8601Driver;
use crate::game::core::{Zone, TamerMap, Element};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Rectangle,
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
        // Check back button
        if let Some(ref rect) = self.back_area {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                return ExpeditionMapAction::Back;
            }
        }

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
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            // Clear screen
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));

        // Title
        Text::new("EXPEDITION", Point::new(120, 30), title_style).draw(display)?;

        // Zone tabs
        self.zone_areas.clear();
        let tab_y = 50;
        let tab_width = 70u32;
        let tab_height = 25u32;

        for (i, zone) in self.zones.iter().take(4).enumerate() {
            let x = 15 + (i as i32 * (tab_width as i32 + 5));
            let rect = Rectangle::new(Point::new(x, tab_y), Size::new(tab_width, tab_height));

            let bg_color = if i == self.selected_zone_index {
                Rgb888::new(60, 80, 120)
            } else if zone.is_unlocked {
                Rgb888::new(40, 45, 55)
            } else {
                Rgb888::new(30, 30, 35)
            };

            display.fill_solid(&rect, bg_color)?;
            self.zone_areas.push(rect);

            let text_color = if zone.is_unlocked {
                Rgb888::WHITE
            } else {
                Rgb888::new(80, 80, 80)
            };
            let zone_style = MonoTextStyle::new(&FONT_9X15, text_color);

            // Truncate zone name if too long
            let name = if zone.name.len() > 10 {
                &zone.name[..10]
            } else {
                &zone.name
            };
            Text::new(name, Point::new(x + 5, tab_y + 17), zone_style).draw(display)?;
        }

        // Selected zone info
        if !self.zones.is_empty() {
            let zone = &self.zones[self.selected_zone_index];
            let info = format!("Lv.{}-{}", zone.level_range.0, zone.level_range.1);
            Text::new(&info, Point::new(15, 95), dim_style).draw(display)?;
        }

        // Maps list
        self.map_areas.clear();
        let map_indices = self.map_indices_for_selected_zone();
        let list_y = 110;
        let item_height = 60u32;

        for (i, &map_idx) in map_indices.iter().take(5).enumerate() {
            let map = &self.maps[map_idx];
            let y = list_y + (i as i32 * (item_height as i32 + 5));
            let rect = Rectangle::new(Point::new(15, y), Size::new(338, item_height));

            display.fill_solid(&rect, Rgb888::new(35, 40, 50))?;
            self.map_areas.push(rect);

            // Map name
            Text::new(&map.name, Point::new(25, y + 18), text_style).draw(display)?;

            // Level range
            let level_text = format!("Lv.{}-{}", map.level_range.0, map.level_range.1);
            Text::new(&level_text, Point::new(25, y + 32), dim_style).draw(display)?;

            // Required elements
            let mut elem_x = 150;
            Text::new("Need:", Point::new(elem_x, y + 35), dim_style).draw(display)?;
            elem_x += 50;

            for elem in &map.required_elements {
                let c = Self::element_char(elem);
                let elem_style = MonoTextStyle::new(&FONT_9X15, Self::element_color(elem));
                Text::new(&c.to_string(), Point::new(elem_x, y + 35), elem_style).draw(display)?;
                elem_x += 15;
            }

            // Capturable count
            let capture_text = format!("{} species", map.capturable_count);
            Text::new(&capture_text, Point::new(25, y + 48), dim_style).draw(display)?;
        }

        if map_indices.is_empty() {
            Text::new("No maps available", Point::new(100, 200), dim_style).draw(display)?;
        }

        // Back button
        let back_rect = Rectangle::new(Point::new(15, 410), Size::new(80, 30));
        display.fill_solid(&back_rect, Rgb888::new(80, 60, 60))?;
        Text::new("< BACK", Point::new(25, 430), text_style).draw(display)?;
        self.back_area = Some(back_rect);

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
