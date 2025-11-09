//! Map Navigation Page
//!
//! Displays current location and allows navigation to connected locations.

use crate::display::Sh8601Driver;
use crate::game::{Direction, MapData, WorldMap};
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
    location_id: u32,
    direction: Direction,
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
    pub fn from_save(mut world_map: WorldMap, current_location_id: u32) -> Self {
        // Set the current location from save data
        world_map.set_current_location(current_location_id);
        Self::new(world_map)
    }

    /// Get reference to world map
    pub fn world_map(&self) -> &WorldMap {
        &self.world_map
    }

    /// Handle touch input at coordinates
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<u32> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Selected location: {} via {}", area.location_id, area.direction.as_str());
                return Some(area.location_id);
            }
        }
        None
    }

    /// Navigate to a location by ID
    pub fn travel_to(&mut self, location_id: u32) -> Result<(), String> {
        self.world_map.travel_to(location_id)?;
        self.touch_areas.clear(); // Rebuild touch areas on next draw
        self.needs_full_redraw = true;
        Ok(())
    }

    /// Draw current location info at top
    fn draw_location_header(
        &self,
        display: &mut Sh8601Driver,
        location: &MapData,
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

        // Location ID
        use core::fmt::Write;
        let mut id_text = heapless::String::<32>::new();
        write!(id_text, "Map ID: {}", location.id).ok();
        Text::new(&id_text, Point::new(10, 30), text_style_info).draw(display)?;

        // Draw enemies count
        if !location.enemies.is_empty() {
            let mut enemies_text = heapless::String::<64>::new();
            write!(enemies_text, "Enemies: {}", location.enemies.len()).ok();
            Text::new(&enemies_text, Point::new(10, 42), text_style_info).draw(display)?;
        } else {
            Text::new("Safe Zone", Point::new(10, 42), text_style_info).draw(display)?;
        }

        Ok(())
    }

    /// Draw list of connected locations with directional navigation
    fn draw_location_list(
        &mut self,
        display: &mut Sh8601Driver,
        connections: &[(Direction, &MapData)],
    ) -> Result<(), Box<dyn Error>> {
        let list_start_y = 60;
        let item_height = 50;

        self.touch_areas.clear();

        let text_style_name = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let text_style_dir = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 200, 255));
        let text_style_info = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));

        for (i, (direction, location)) in connections.iter().enumerate() {
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
                location_id: location.id,
                direction: *direction,
            });

            // Draw direction indicator
            use core::fmt::Write;
            let mut dir_text = heapless::String::<16>::new();
            write!(dir_text, "[{}]", direction.as_str()).ok();
            Text::new(&dir_text, Point::new(15, y + 15), text_style_dir).draw(display)?;

            // Draw location name
            Text::new(&location.name, Point::new(80, y + 15), text_style_name).draw(display)?;

            // Draw enemy count or safe zone
            let mut info_text = heapless::String::<32>::new();
            if location.enemies.is_empty() {
                write!(info_text, "Safe").ok();
            } else {
                write!(info_text, "{} enemies", location.enemies.len()).ok();
            }
            Text::new(&info_text, Point::new(80, y + 30), text_style_info).draw(display)?;

            // Draw arrow indicator if selected
            if is_selected {
                Text::new(">", Point::new(340, y + 22), text_style_name).draw(display)?;
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

        // Get current location (clone to avoid borrow issues)
        let current_location = self
            .world_map
            .current_location()
            .ok_or("No current location")?
            .clone();

        // Get connected locations with directions (collect into Vec to own the data)
        let connections: Vec<(Direction, MapData)> = self
            .world_map
            .connected_locations_with_directions()
            .into_iter()
            .map(|(dir, map_data)| (dir, map_data.clone()))
            .collect();

        // Draw header with current location info
        self.draw_location_header(display, &current_location)?;

        // Draw list of connected locations
        let connections_refs: Vec<(Direction, &MapData)> = connections
            .iter()
            .map(|(dir, map_data)| (*dir, map_data))
            .collect();
        self.draw_location_list(display, &connections_refs)?;

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
