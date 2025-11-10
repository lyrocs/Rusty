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
    location_id: Option<u32>,     // None for FIGHT/CRAFT buttons
    direction: Option<Direction>,  // None for FIGHT/CRAFT buttons
    is_fight_button: bool,
    is_craft_button: bool,
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
    Craft,        // Open crafting menu
}

impl MapPage {
    /// Load map background with SD card fallback to embedded
    fn load_map_background(
        map_id: u32,
        asset_loader: &Option<AssetLoader<SdCardWrapper>>,
        position: (i32, i32),
    ) -> Option<Background> {
        // Try using asset loader if available (handles SD card + embedded fallback)
        if let Some(mut loader) = asset_loader.clone() {
            if let Ok(asset_source) = loader.load(&AssetId::MapBackground(map_id)) {
                match Background::new(asset_source.bytes(), position) {
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
            match Background::new(data, position) {
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
        let background = Self::load_map_background(map_id, &asset_loader, (50, 15));

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
                } else if area.is_craft_button {
                    log::info!("Craft button pressed!");
                    return Some(TouchAction::Craft);
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
        self.background = Self::load_map_background(location_id, &self.asset_loader, (50, 15));

        Ok(())
    }

    /// Draw header with mini-map and location info
    fn draw_header_with_minimap(
        &self,
        display: &mut Sh8601Driver,
        location: &MapData,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Safe margin from top and left (for rounded corners)
        let margin_top = 15;
        let margin_left = 50; // Move map away from left corner
        let map_size = 90;

        // Draw mini-map background (placeholder box)
        Rectangle::new(Point::new(margin_left, margin_top), Size::new(map_size, map_size))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 40, 50)))
            .draw(display)?;

        // Try to draw map image if available
        if let Some(background) = &self.background {
            // Scale and draw the map at 90×90
            background.draw(display)?;
        }

        // Location info next to map
        let info_x = margin_left + map_size as i32 + 10;
        let text_style_name = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 200));

        // Location name
        Text::new(&location.name, Point::new(info_x, margin_top + 20), text_style_name).draw(display)?;

        // Status
        let status = if location.npcs.as_ref().map_or(false, |npcs| !npcs.is_empty()) {
            "City - Safe"
        } else if location.enemies.is_empty() {
            "Safe Zone"
        } else {
            "Field"
        };
        Text::new(status, Point::new(info_x, margin_top + 42), text_style_info).draw(display)?;

        // Enemy/NPC count
        let mut count_text = heapless::String::<32>::new();
        if let Some(npcs) = &location.npcs {
            if !npcs.is_empty() {
                write!(count_text, "NPCs: {}", npcs.len()).ok();
            }
        } else if !location.enemies.is_empty() {
            write!(count_text, "Monsters: {}", location.enemies.len()).ok();
        }
        if !count_text.is_empty() {
            Text::new(&count_text, Point::new(info_x, margin_top + 64), text_style_info).draw(display)?;
        }

        Ok(())
    }

    /// Draw card-based navigation
    fn draw_card_navigation(
        &mut self,
        display: &mut Sh8601Driver,
        connections: &[(Direction, MapData)],
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 15;
        let card_start_y = 120; // Below header
        let card_height = 70;
        let card_spacing = 8;
        let card_width = 318u32; // 368 - (15*2) - rounded corners safe area

        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style_dest = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));

        let mut north_card: Option<(Direction, MapData)> = None;
        let mut south_card: Option<(Direction, MapData)> = None;
        let mut east_card: Option<(Direction, MapData)> = None;
        let mut west_card: Option<(Direction, MapData)> = None;

        // Separate connections by direction
        for (direction, location) in connections {
            match direction {
                Direction::North => north_card = Some((*direction, location.clone())),
                Direction::South => south_card = Some((*direction, location.clone())),
                Direction::East => east_card = Some((*direction, location.clone())),
                Direction::West => west_card = Some((*direction, location.clone())),
            }
        }

        // Full width card for North/South
        let full_width = 338u32;
        // Half width for East/West (2 columns)
        let half_width = 165u32;
        let column_spacing = 8;

        let mut current_y = card_start_y;

        // Draw NORTH card (full width at top)
        if let Some((dir, loc)) = north_card {
            self.draw_direction_card(display, &dir, &loc, margin, current_y, full_width, card_height as u32, &text_style_title, &text_style_dest, &text_style_info)?;
            current_y += card_height + card_spacing;
        }

        // Draw WEST (left) and EAST (right) cards side by side
        if west_card.is_some() || east_card.is_some() {
            if let Some((dir, loc)) = west_card {
                self.draw_direction_card(display, &dir, &loc, margin, current_y, half_width, card_height as u32, &text_style_title, &text_style_dest, &text_style_info)?;
            }

            if let Some((dir, loc)) = east_card {
                let east_x = margin + half_width as i32 + column_spacing;
                self.draw_direction_card(display, &dir, &loc, east_x, current_y, half_width, card_height as u32, &text_style_title, &text_style_dest, &text_style_info)?;
            }

            current_y += card_height + card_spacing;
        }

        // Draw SOUTH card (full width at bottom)
        if let Some((dir, loc)) = south_card {
            self.draw_direction_card(display, &dir, &loc, margin, current_y, full_width, card_height as u32, &text_style_title, &text_style_dest, &text_style_info)?;
        }

        Ok(())
    }

    /// Draw a single direction card
    fn draw_direction_card(
        &mut self,
        display: &mut Sh8601Driver,
        direction: &Direction,
        location: &MapData,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        text_style_dir: &MonoTextStyle<Rgb888>,
        text_style_dest: &MonoTextStyle<Rgb888>,
        text_style_info: &MonoTextStyle<Rgb888>,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Card background
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 60, 80)))
            .draw(display)?;

        // Direction label
        let dir_text = match direction {
            Direction::North => "NORTH",
            Direction::South => "SOUTH",
            Direction::East => "EAST",
            Direction::West => "WEST",
        };
        Text::new(dir_text, Point::new(x + 8, y + 18), *text_style_dir).draw(display)?;

        // Destination name
        Text::new(&location.name, Point::new(x + 8, y + 36), *text_style_dest).draw(display)?;

        // Monster names if field (only show first monster name)
        if !location.enemies.is_empty() {
            let mut monsters_text = heapless::String::<32>::new();
            if let Some(enemy_data) = location.enemies.get(0)
                .and_then(|id| self.world_map.game_data().get_enemy(*id)) {
                write!(monsters_text, "{}", enemy_data.name).ok();
                if location.enemies.len() > 1 {
                    write!(monsters_text, "...").ok();
                }
            }
            Text::new(&monsters_text, Point::new(x + 8, y + 52), *text_style_info).draw(display)?;
        }

        // Store touch area
        self.touch_areas.push(TouchArea {
            bounds: (x, y, width, height),
            location_id: Some(location.id),
            direction: Some(*direction),
            is_fight_button: false,
            is_craft_button: false,
        });

        Ok(())
    }

    /// Draw current area info
    fn draw_current_area_info(
        &self,
        display: &mut Sh8601Driver,
        location: &MapData,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        let margin = 15;
        let y_start = 320; // Start higher to fit multiple lines

        let text_style_label = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        // Draw info based on location type
        if let Some(npcs) = &location.npcs {
            if !npcs.is_empty() {
                // City info
                let mut info_text = heapless::String::<64>::new();
                write!(info_text, "NPCs: {} | Services: Craft", npcs.len()).ok();
                Text::new(&info_text, Point::new(margin + 5, y_start), text_style_info).draw(display)?;
            }
        } else if !location.enemies.is_empty() {
            // Field info - show "MONSTERS:" label
            Text::new("MONSTERS:", Point::new(margin + 5, y_start), text_style_label).draw(display)?;

            // Show monster names (up to 3)
            let mut monster_text = heapless::String::<64>::new();
            for (i, enemy_id) in location.enemies.iter().take(3).enumerate() {
                if let Some(enemy_data) = self.world_map.game_data().get_enemy(*enemy_id) {
                    if i > 0 {
                        write!(monster_text, ", {}", enemy_data.name).ok();
                    } else {
                        write!(monster_text, "{}", enemy_data.name).ok();
                    }
                }
            }
            if location.enemies.len() > 3 {
                write!(monster_text, "...").ok();
            }
            Text::new(&monster_text, Point::new(margin + 5, y_start + 20), text_style_info).draw(display)?;
        } else {
            // Empty area
            Text::new("Safe Zone - No Monsters", Point::new(margin + 5, y_start), text_style_info).draw(display)?;
        }

        Ok(())
    }

    /// Draw action buttons at bottom
    fn draw_action_buttons(
        &mut self,
        display: &mut Sh8601Driver,
        has_enemies: bool,
        is_city: bool,
    ) -> Result<(), Box<dyn Error>> {
        let margin = 15;
        let button_y = 368; // Safe from bottom edge (448 - 70 - 10)
        let button_height = 65;
        let button_width = 160;
        let button_spacing = 18;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        // FIGHT button (only on fields with enemies)
        if has_enemies && !is_city {
            Rectangle::new(Point::new(margin, button_y), Size::new(button_width, button_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 40, 40)))
                .draw(display)?;

            Text::new("FIGHT", Point::new(margin + 45, button_y + 40), text_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (margin, button_y, button_width, button_height),
                location_id: None,
                direction: None,
                is_fight_button: true,
                is_craft_button: false,
            });
        }

        // CRAFT button (only in cities)
        if is_city {
            let craft_x = margin + button_width as i32 + button_spacing;

            Rectangle::new(Point::new(craft_x, button_y), Size::new(button_width, button_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 60, 120)))
                .draw(display)?;

            Text::new("CRAFT", Point::new(craft_x + 45, button_y + 40), text_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (craft_x, button_y, button_width, button_height),
                location_id: None,
                direction: None,
                is_fight_button: false,
                is_craft_button: true,
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
            // Clear with background color
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

        // Check if current location has enemies
        let has_enemies = !current_location.enemies.is_empty();

        // Check if current location is a city (has NPCs)
        let is_city = current_location.npcs.as_ref().map_or(false, |npcs| !npcs.is_empty());

        // Draw new card-based layout
        self.draw_header_with_minimap(display, &current_location)?;
        self.draw_card_navigation(display, &connections)?;
        self.draw_current_area_info(display, &current_location)?;
        self.draw_action_buttons(display, has_enemies, is_city)?;

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
