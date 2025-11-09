//! Map Navigation Page
//!
//! Displays current location and allows navigation to connected locations.

use crate::assets::{AssetId, AssetLoader};
use crate::display::Sh8601Driver;
use crate::ecs::resources::SdCardWrapper;
use crate::game::{Direction, MapData, WorldMap};
use crate::ui::page::Page;
use crate::ui::sprite::Background;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
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
    location_id: Option<u32>,     // None for FIGHT button
    direction: Option<Direction>,  // None for FIGHT button
    is_fight_button: bool,
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
    background: Option<Background>,
    background_color: Rgb888,
    needs_full_redraw: bool,
    asset_loader: Option<AssetLoader<SdCardWrapper>>,
}

/// Touch action result
#[derive(Debug, Clone, Copy)]
pub enum TouchAction {
    Travel(u32),  // Travel to location ID
    Fight,        // Enter battle on current map
}

impl MapPage {
    /// Load map background with SD card fallback to embedded
    fn load_map_background(
        map_id: u32,
        asset_loader: &Option<AssetLoader<SdCardWrapper>>,
    ) -> Option<Background> {
        // Try using asset loader if available (handles SD card + embedded fallback)
        if let Some(mut loader) = asset_loader.clone() {
            if let Ok(asset_source) = loader.load(&AssetId::MapBackground(map_id)) {
                match Background::new(asset_source.bytes(), (0, 0)) {
                    Ok(bg) => {
                        log::info!("✅ Loaded map {} background", map_id);
                        return Some(bg);
                    }
                    Err(e) => {
                        log::warn!("⚠️  Failed to create background: {}", e);
                    }
                }
            }
        }

        // Fallback to embedded assets
        let embedded_data = match map_id {
            1 => Some(include_bytes!("../../../assets/images/map/1.gif").as_slice()),
            2 => Some(include_bytes!("../../../assets/images/map/2.gif").as_slice()),
            3 => Some(include_bytes!("../../../assets/images/map/3.gif").as_slice()),
            5 => Some(include_bytes!("../../../assets/images/map/5.gif").as_slice()),
            _ => {
                log::warn!("No embedded background for map {}", map_id);
                None
            }
        };

        if let Some(data) = embedded_data {
            match Background::new(data, (0, 0)) {
                Ok(bg) => {
                    log::info!("📦 Loaded map {} background from embedded assets", map_id);
                    Some(bg)
                }
                Err(e) => {
                    log::error!("Failed to load embedded background for map {}: {}", map_id, e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Create a new map page with world map data
    pub fn new(
        world_map: WorldMap,
        asset_loader: Option<AssetLoader<SdCardWrapper>>,
    ) -> Self {
        let map_id = world_map.current_location_id();
        let background = Self::load_map_background(map_id, &asset_loader);

        Self {
            world_map,
            touch_areas: Vec::new(),
            background,
            background_color: Rgb888::new(20, 30, 40),
            needs_full_redraw: true,
            asset_loader,
        }
    }

    /// Create map page from save data (with specific location)
    pub fn from_save(
        mut world_map: WorldMap,
        current_location_id: u32,
        asset_loader: Option<AssetLoader<SdCardWrapper>>,
    ) -> Self {
        // Set the current location from save data
        world_map.set_current_location(current_location_id);
        Self::new(world_map, asset_loader)
    }

    /// Get reference to world map
    pub fn world_map(&self) -> &WorldMap {
        &self.world_map
    }

    /// Handle touch input at coordinates
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<TouchAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                if area.is_fight_button {
                    log::info!("Fight button pressed!");
                    return Some(TouchAction::Fight);
                } else if let Some(location_id) = area.location_id {
                    if let Some(direction) = area.direction {
                        log::info!("Traveling to location {} via {}", location_id, direction.as_str());
                    }
                    return Some(TouchAction::Travel(location_id));
                }
            }
        }
        None
    }

    /// Navigate to a location by ID
    pub fn travel_to(&mut self, location_id: u32) -> Result<(), String> {
        self.world_map.travel_to(location_id)?;
        self.touch_areas.clear(); // Rebuild touch areas on next draw
        self.needs_full_redraw = true;

        // Reload background for new location
        self.background = Self::load_map_background(location_id, &self.asset_loader);

        Ok(())
    }

    /// Draw current location info at top
    fn draw_location_header(
        &self,
        display: &mut Sh8601Driver,
        location: &MapData,
    ) -> Result<(), Box<dyn Error>> {
        let header_height = 50;

        // Draw semi-transparent header background
        Rectangle::new(Point::new(0, 0), Size::new(368, header_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        let text_style_name = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 200));
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        // Location name
        Text::new(&location.name, Point::new(10, 20), text_style_name).draw(display)?;

        // Status: Safe Zone or enemy count
        use core::fmt::Write;
        if !location.enemies.is_empty() {
            let mut enemies_text = heapless::String::<64>::new();
            write!(enemies_text, "{} enemy types", location.enemies.len()).ok();
            Text::new(&enemies_text, Point::new(10, 40), text_style_info).draw(display)?;
        } else {
            Text::new("Safe Zone", Point::new(10, 40), text_style_info).draw(display)?;
        }

        Ok(())
    }

    /// Draw list of monsters on current map
    fn draw_monster_list(
        &self,
        display: &mut Sh8601Driver,
        location: &MapData,
    ) -> Result<(), Box<dyn Error>> {
        if location.enemies.is_empty() {
            return Ok(());
        }

        let list_start_y = 60;
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 200, 100));
        let text_style_monster = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 255));

        // Title
        Text::new("Monsters:", Point::new(10, list_start_y + 20), text_style_title).draw(display)?;

        // List monsters (get names from game data)
        use core::fmt::Write;
        for (i, enemy_id) in location.enemies.iter().enumerate() {
            if let Some(enemy_data) = self.world_map.game_data().get_enemy(*enemy_id) {
                let y = list_start_y + 45 + (i as i32 * 30);

                // Draw background for monster name
                let bg_rect = Rectangle::new(
                    Point::new(10, y - 18),
                    Size::new(348, 26)
                );
                bg_rect
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
                    .draw(display)?;

                // Draw monster text
                let mut monster_text = heapless::String::<64>::new();
                write!(monster_text, "- {} (Lv {})", enemy_data.name, enemy_data.level).ok();
                Text::new(&monster_text, Point::new(15, y), text_style_monster).draw(display)?;
            }
        }

        Ok(())
    }

    /// Draw navigation buttons at bottom
    fn draw_navigation_buttons(
        &mut self,
        display: &mut Sh8601Driver,
        connections: &[(Direction, &MapData)],
        has_enemies: bool,
    ) -> Result<(), Box<dyn Error>> {
        self.touch_areas.clear();

        let bottom_start_y = 310; // Start of bottom navigation area (moved up)
        let button_height = 42;   // Reduced from 50 to fit 3 buttons
        let button_spacing = 5;   // Reduced from 8

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        use core::fmt::Write;

        // Draw directional navigation buttons
        for (direction, location) in connections {
            let mut button_text = heapless::String::<32>::new();
            write!(button_text, "{} - {}", direction.as_str(), location.name).ok();

            let y = bottom_start_y + (self.touch_areas.len() as i32 * (button_height + button_spacing));

            // Draw button background
            let button_rect = Rectangle::new(Point::new(10, y), Size::new(348, button_height as u32));
            button_rect
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 60, 100)))
                .draw(display)?;

            // Draw button text (centered vertically)
            Text::new(&button_text, Point::new(20, y + 27), text_style).draw(display)?;

            // Store touch area
            self.touch_areas.push(TouchArea {
                bounds: (10, y, 348, button_height as u32),
                location_id: Some(location.id),
                direction: Some(*direction),
                is_fight_button: false,
            });
        }

        // Draw FIGHT button if enemies present
        if has_enemies {
            let y = bottom_start_y + (self.touch_areas.len() as i32 * (button_height + button_spacing));

            // Draw FIGHT button with red background
            let button_rect = Rectangle::new(Point::new(10, y), Size::new(348, button_height as u32));
            button_rect
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 30, 30)))
                .draw(display)?;

            // Draw FIGHT text (centered)
            Text::new("FIGHT!", Point::new(140, y + 27), text_style).draw(display)?;

            // Store touch area
            self.touch_areas.push(TouchArea {
                bounds: (10, y, 348, button_height as u32),
                location_id: None,
                direction: None,
                is_fight_button: true,
            });
        }

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
            // Draw background (map image or solid color)
            if let Some(background) = &self.background {
                // Draw map background image
                background.draw(display)?;
            } else {
                // Fallback to solid color background
                display.clear(self.background_color)?;
            }
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

        // Check if current location has enemies
        let has_enemies = !current_location.enemies.is_empty();

        // Draw header with current location info
        self.draw_location_header(display, &current_location)?;

        // Draw monster list if present
        self.draw_monster_list(display, &current_location)?;

        // Draw navigation buttons at bottom
        let connections_refs: Vec<(Direction, &MapData)> = connections
            .iter()
            .map(|(dir, map_data)| (*dir, map_data))
            .collect();
        self.draw_navigation_buttons(display, &connections_refs, has_enemies)?;

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
