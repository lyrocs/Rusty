//! Map Navigation Page
//!
//! Displays current location and allows navigation to connected locations.

use crate::display::Sh8601Driver;
use crate::game::{MapLocation, WorldMap};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;

/// Touch area for location selection
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32), // (x, y, width, height)
    location_id: String,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Map navigation page
pub struct MapPage {
    world_map: WorldMap,
    touch_areas: Vec<TouchArea>,
    selected_index: usize,
    background_color: Rgb888,
    needs_full_redraw: bool,
}

impl MapPage {
    /// Create a new map page with world map data
    pub fn new(world_map: WorldMap) -> Self {
        Self {
            world_map,
            touch_areas: Vec::new(),
            selected_index: 0,
            background_color: Rgb888::new(20, 30, 40),
            needs_full_redraw: true,
        }
    }

    /// Create map page from save data (with specific location)
    pub fn from_save(mut world_map: WorldMap, current_location_id: String) -> Self {
        // Set the current location from save data
        world_map.set_current_location(current_location_id);
        Self::new(world_map)
    }

    /// Get reference to world map
    pub fn world_map(&self) -> &WorldMap {
        &self.world_map
    }

    /// Handle touch input at coordinates
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<String> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Selected location: {}", area.location_id);
                return Some(area.location_id.clone());
            }
        }
        None
    }

    /// Navigate to a location
    pub fn travel_to(&mut self, location_id: &str) -> Result<(), String> {
        self.world_map.travel_to(location_id)?;
        self.touch_areas.clear(); // Rebuild touch areas on next draw
        Ok(())
    }

    /// Draw current location info at top
    fn draw_location_header(
        &self,
        display: &mut Sh8601Driver,
        location: &MapLocation,
    ) -> Result<(), Box<dyn Error>> {
        let header_height = 50;

        // Draw header background
        Rectangle::new(Point::new(0, 0), Size::new(368, header_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 50, 70)))
            .draw(display)?;

        let text_style_name = MonoTextStyle::new(&FONT_6X10, Rgb888::new(255, 255, 200));
        let text_style_info = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 180, 180));

        // Location name
        Text::new(&location.name, Point::new(10, 15), text_style_name).draw(display)?;

        // Location type indicator
        let location_type_text = if location.is_city() {
            "[CITY]"
        } else {
            "[FIELD]"
        };
        let type_color = if location.is_city() {
            Rgb888::new(100, 200, 100)
        } else {
            Rgb888::new(200, 100, 100)
        };
        let text_style_type = MonoTextStyle::new(&FONT_6X10, type_color);
        Text::new(location_type_text, Point::new(10, 30), text_style_type).draw(display)?;

        // Draw services or monsters
        if let Some(services) = location.services() {
            use core::fmt::Write;
            let mut services_text = heapless::String::<64>::new();
            for (i, service) in services.iter().enumerate() {
                if i > 0 {
                    write!(services_text, " ").ok();
                }
                write!(services_text, "[{}]", service.icon()).ok();
            }
            Text::new(&services_text, Point::new(10, 42), text_style_info).draw(display)?;
        } else if let Some(monsters) = location.monsters() {
            use core::fmt::Write;
            let mut monsters_text = heapless::String::<64>::new();
            write!(monsters_text, "Monsters: {}", monsters.len()).ok();
            Text::new(&monsters_text, Point::new(10, 42), text_style_info).draw(display)?;
        }

        Ok(())
    }

    /// Draw list of connected locations
    fn draw_location_list(
        &mut self,
        display: &mut Sh8601Driver,
        connected: &[MapLocation],
    ) -> Result<(), Box<dyn Error>> {
        let list_start_y = 60;
        let item_height = 40;

        self.touch_areas.clear();

        let text_style_name = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let text_style_type = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));

        for (i, location) in connected.iter().enumerate() {
            let y = list_start_y + (i as i32 * item_height);

            // Draw item background
            let is_selected = i == self.selected_index;
            let bg_color = if is_selected {
                Rgb888::new(60, 80, 120)
            } else {
                Rgb888::new(30, 40, 50)
            };

            let item_rect = Rectangle::new(Point::new(5, y), Size::new(358, item_height as u32 - 5));
            item_rect
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            // Store touch area
            self.touch_areas.push(TouchArea {
                bounds: (5, y, 358, item_height as u32 - 5),
                location_id: location.id.clone(),
            });

            // Draw location name
            Text::new(&location.name, Point::new(15, y + 15), text_style_name).draw(display)?;

            // Draw location type
            let type_text = if location.is_city() { "City" } else { "Field" };
            Text::new(type_text, Point::new(15, y + 28), text_style_type).draw(display)?;

            // Draw arrow indicator if selected
            if is_selected {
                Text::new(">", Point::new(340, y + 20), text_style_name).draw(display)?;
            }
        }

        Ok(())
    }

    /// Draw help text at bottom
    fn draw_help_text(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(120, 120, 120));

        Text::new(
            "Tap location to travel",
            Point::new(10, 430),
            text_style,
        )
        .draw(display)?;

        Ok(())
    }
}

impl Page for MapPage {
    fn update(&mut self) -> bool {
        // Map page doesn't need animation updates
        true
    }

    fn draw(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_full_redraw {
            // Clear screen with background color
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        // Get current location
        let current_location = self
            .world_map
            .current_location()
            .ok_or("No current location")?
            .clone();

        // Get connected locations - clone to avoid borrow issues
        let connected: Vec<MapLocation> = self
            .world_map
            .connected_locations()
            .into_iter()
            .cloned()
            .collect();

        // Draw header with current location info
        self.draw_location_header(display, &current_location)?;

        // Draw list of connected locations
        self.draw_location_list(display, &connected)?;

        // Draw help text
        self.draw_help_text(display)?;

        // Flush to display
        display.flush()?;

        Ok(())
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
