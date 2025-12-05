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

/// Navigation page state for 3-page navigation system
#[derive(Debug, Clone, Copy, PartialEq)]
enum NavigationPage {
    WorldMapGrid { scroll_offset: usize }, // Page 1: Grid view of all maps
    MapDetails { map_id: u32 },            // Page 2: Detailed view with actions
    MonsterList { map_id: u32 },           // Page 3: Monster list for a map
    OldCardView,                            // Legacy: Original card-based navigation
}

/// Touch area for location selection
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32), // (x, y, width, height)
    location_id: Option<u32>,     // None for FIGHT buttons
    direction: Option<Direction>,  // None for FIGHT buttons
    is_fight_button: bool,
    is_expedition_button: bool,
    is_back_button: bool,
    is_view_world_map_button: bool,
    is_view_monsters_button: bool,
    is_mvp_button: bool,
    mvp_enemy_id: Option<u32>,
    is_hunt_button: bool,
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
    details_background: Option<Background>, // Cache for map details page
    cached_details_map_id: Option<u32>,     // Track which map is cached
    background_color: Rgb888,
    needs_full_redraw: bool,
    asset_loader: Option<AssetLoader<SdCardWrapper>>,
    navigation_page: NavigationPage,
}

/// Touch action result
#[derive(Debug, Clone, Copy)]
pub enum TouchAction {
    Travel(u32),         // Travel to location ID
    Fight,               // Enter battle on current map
    Expedition,          // Enter expedition setup
    ViewMapDetails(u32), // View details for a specific map (Page 1 → Page 2)
    ViewMonsterList(u32), // View monster list for a map (Page 2 → Page 3)
    BackToWorldMap,      // Return to world map grid (Page 2 → Page 1)
    BackToMapDetails,    // Return to map details (Page 3 → Page 2)
    MvpFight(u32),       // Fight MVP on map (enemy_id)
    Hunt(u32),           // Open hunt monster list for map
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
            details_background: None,
            cached_details_map_id: None,
            background_color: Rgb888::new(20, 30, 40),
            needs_full_redraw: true,
            asset_loader,
            navigation_page: NavigationPage::WorldMapGrid { scroll_offset: 0 },
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

    /// Draw a rectangle outline with specified thickness
    fn draw_rect_outline(
        display: &mut Sh8601Driver,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Rgb888,
        thickness: u32,
    ) -> Result<(), Box<dyn Error>> {
        for i in 0..thickness {
            let offset = i as i32;
            Rectangle::new(
                Point::new(x + offset, y + offset),
                Size::new(
                    width.saturating_sub(i * 2),
                    height.saturating_sub(i * 2),
                ),
            )
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(display)?;
        }
        Ok(())
    }

    /// Draw Page 1: World Map Grid (3×2 grid showing 6 maps)
    fn draw_world_map_grid(
        &mut self,
        display: &mut Sh8601Driver,
        scroll_offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        self.touch_areas.clear();

        // Color constants
        let color_yellow = Rgb888::new(255, 215, 0);    // Header text
        let color_dark_gray = Rgb888::new(60, 60, 60);  // Locked maps (future use)
        let color_gray = Rgb888::new(120, 120, 120);    // Default border

        let text_style_header = MonoTextStyle::new(&FONT_10X20, color_yellow);
        let text_style_name = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));

        // Header: "WORLD MAP"
        let margin = 15;
        Text::new("WORLD MAP", Point::new(margin + 5, 30), text_style_header).draw(display)?;

        // Grid configuration - Display is 368×448
        // Grid: 3 columns × 2 rows = 6 maps per page
        let cols = 3;
        let rows = 2;
        let grid_start_y = 50;
        let cell_width = 110u32;
        let cell_height = 120u32; // Increased to fit 2-line map names
        let cell_spacing = 10;
        let thumbnail_width = 100u32;
        let thumbnail_height = 75u32;

        // Calculate grid position to center it
        let total_grid_width = cols * (cell_width + cell_spacing) - cell_spacing;
        let grid_start_x = (368 - total_grid_width as i32) / 2;

        // Get all maps from game data
        let all_map_ids: Vec<u32> = self.world_map.game_data().get_all_map_ids();

        // Calculate visible maps for this page
        let maps_per_page = (cols * rows) as usize;
        let visible_maps: Vec<u32> = all_map_ids
            .iter()
            .skip(scroll_offset)
            .take(maps_per_page)
            .copied()
            .collect();

        // Draw grid cells
        for (idx, &map_id) in visible_maps.iter().enumerate() {
            let row = idx / (cols as usize);
            let col = idx % (cols as usize);

            let cell_x = grid_start_x + (col as i32 * (cell_width + cell_spacing) as i32);
            let cell_y = grid_start_y + (row as i32 * (cell_height + cell_spacing) as i32);

            // Get map data
            if let Some(map_data) = self.world_map.get_location(map_id) {
                // All maps have same gray border (no current location indicator)
                let border_color = color_gray;
                let border_thickness = 1;

                // Draw cell background
                Rectangle::new(
                    Point::new(cell_x, cell_y),
                    Size::new(cell_width, cell_height),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 40, 50)))
                .draw(display)?;

                // Draw thumbnail placeholder
                // TODO: Add thumbnail GIF support when {id}_thumb.gif files are available
                let thumb_x = cell_x + ((cell_width - thumbnail_width) / 2) as i32;
                let thumb_y = cell_y + 5;

                Rectangle::new(
                    Point::new(thumb_x, thumb_y),
                    Size::new(thumbnail_width, thumbnail_height),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 60, 70)))
                .draw(display)?;

                // Draw a simple icon to represent the map type
                let icon_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));
                let icon = if map_data.npcs.is_some() && !map_data.npcs.as_ref().unwrap().is_empty() {
                    "TOWN"
                } else if !map_data.enemies.is_empty() {
                    "FIELD"
                } else {
                    "SAFE"
                };
                Text::new(icon, Point::new(thumb_x + 10, thumb_y + 40), icon_style).draw(display)?;

                // Draw level range if map has enemies
                if !map_data.enemies.is_empty() {
                    let mut min_level = u32::MAX;
                    let mut max_level = u32::MIN;

                    // Calculate level range from enemies
                    for enemy_id in &map_data.enemies {
                        if let Some(enemy_data) = self.world_map.game_data().get_enemy(*enemy_id) {
                            min_level = min_level.min(enemy_data.level);
                            max_level = max_level.max(enemy_data.level);
                        }
                    }

                    if min_level != u32::MAX && max_level != u32::MIN {
                        // Draw level range on top of thumbnail
                        let level_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0)); // Yellow
                        let mut level_str = heapless::String::<16>::new();

                        if min_level == max_level {
                            write!(level_str, "Lv {}", min_level).ok();
                        } else {
                            write!(level_str, "Lv {}-{}", min_level, max_level).ok();
                        }

                        Text::new(&level_str, Point::new(thumb_x + 5, thumb_y + 15), level_style).draw(display)?;
                    }
                }

                // Draw border
                Self::draw_rect_outline(
                    display,
                    cell_x,
                    cell_y,
                    cell_width,
                    cell_height,
                    border_color,
                    border_thickness,
                )?;

                // Draw map name (split into 2 lines, max 10 chars per line)
                let name_y = thumb_y + thumbnail_height as i32 + 15;
                let map_name = &map_data.name;

                // Use character count, not byte length
                let char_count = map_name.chars().count();

                if char_count <= 10 {
                    // Name fits in one line
                    Text::new(map_name, Point::new(cell_x + 5, name_y), text_style_name).draw(display)?;
                } else {
                    // Split name into 2 lines
                    // Find a good split point (prefer space, otherwise split at 10 chars)
                    let first_10_chars: String = map_name.chars().take(10).collect();
                    let split_pos = if let Some(space_pos) = first_10_chars.rfind(' ') {
                        space_pos
                    } else {
                        10.min(char_count)
                    };

                    // Extract line 1
                    let line1: String = map_name.chars().take(split_pos).collect();

                    // Extract line 2 (skip space if present, take next 10 chars)
                    let remaining_chars: Vec<char> = map_name.chars().skip(split_pos).collect();
                    let line2: String = if !remaining_chars.is_empty() {
                        // Skip leading space if present
                        let start_idx = if remaining_chars[0] == ' ' { 1 } else { 0 };
                        remaining_chars.iter().skip(start_idx).take(10).collect()
                    } else {
                        String::new()
                    };

                    Text::new(&line1, Point::new(cell_x + 5, name_y), text_style_name).draw(display)?;
                    if !line2.is_empty() {
                        Text::new(&line2, Point::new(cell_x + 5, name_y + 12), text_style_name).draw(display)?;
                    }
                }

                // Current location is indicated by yellow border only (no star)

                // Store touch area
                self.touch_areas.push(TouchArea {
                    bounds: (cell_x, cell_y, cell_width, cell_height),
                    location_id: Some(map_id),
                    direction: None,
                    is_fight_button: false,
                    is_expedition_button: false,
                    is_back_button: false,
                    is_view_world_map_button: false,
                    is_view_monsters_button: false,
                    is_mvp_button: false,
                    mvp_enemy_id: None,
                    is_hunt_button: false,
                });
            }
        }

        // Draw pagination info at bottom (if needed)
        let total_pages = (all_map_ids.len() + (maps_per_page as usize) - 1) / (maps_per_page as usize);
        let current_page = (scroll_offset / (maps_per_page as usize)) + 1;

        if total_pages > 1 {
            let mut page_text = heapless::String::<32>::new();
            write!(page_text, "Page {}/{}", current_page, total_pages).ok();
            let page_info_y = grid_start_y + (rows as i32 * (cell_height + cell_spacing) as i32) + 20;
            Text::new(&page_text, Point::new(margin + 5, page_info_y), text_style_info).draw(display)?;
        }

        Ok(())
    }

    /// Draw Page 2: Map Details with full image and action buttons
    fn draw_map_details(
        &mut self,
        display: &mut Sh8601Driver,
        map_id: u32,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 15;
        let color_yellow = Rgb888::new(255, 215, 0);
        let text_style_header = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        // Get map data
        let map_data = match self.world_map.get_location(map_id) {
            Some(data) => data.clone(),
            None => return Err("Map not found".into()),
        };

        // Draw map name centered at top
        let header_y = 30;
        let name_x = (368 - (map_data.name.len() as i32 * 10)) / 2; // Approximate centering
        Text::new(
            &map_data.name,
            Point::new(name_x.max(margin), header_y),
            MonoTextStyle::new(&FONT_10X20, color_yellow),
        )
        .draw(display)?;

        // Draw large map image
        let img_y = 55;
        let img_width = 338u32; // Leave margins
        let img_height = 200u32;
        let img_x = (368 - img_width as i32) / 2;

        // Check if we need to reload the background (if map changed)
        if self.cached_details_map_id != Some(map_id) {
            // Load and cache the map background
            self.details_background = Self::load_map_background(map_id, &self.asset_loader, (img_x, img_y));
            self.cached_details_map_id = Some(map_id);
        }

        // Draw the cached background
        if let Some(ref bg) = self.details_background {
            // Draw the actual map GIF
            bg.draw(display)?;
        } else {
            // Fallback to placeholder if no image available
            Rectangle::new(Point::new(img_x, img_y), Size::new(img_width, img_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 60, 70)))
                .draw(display)?;
        }

        // Draw info bar overlay (semi-transparent black background)
        let info_bar_y = img_y + img_height as i32 - 25;
        let info_bar_height = 25u32;

        Rectangle::new(
            Point::new(img_x, info_bar_y),
            Size::new(img_width, info_bar_height),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

        // Draw info text: "{icon} {type} │ {count} {Monsters/NPCs}"
        let mut info_text = heapless::String::<64>::new();
        let is_city = map_data.npcs.as_ref().map_or(false, |npcs| !npcs.is_empty());

        if is_city {
            if let Some(npcs) = &map_data.npcs {
                write!(info_text, "Town | {} NPCs", npcs.len()).ok();
            }
        } else if !map_data.enemies.is_empty() {
            let map_type = if map_data.name.contains("Dungeon") { "Dungeon" } else { "Field" };
            write!(info_text, "{} | {} Monsters", map_type, map_data.enemies.len()).ok();
        } else {
            write!(info_text, "Safe Zone").ok();
        }

        Text::new(
            &info_text,
            Point::new(img_x + 5, info_bar_y + 18),
            text_style_info,
        )
        .draw(display)?;

        // Display monster names below info bar (for fields/dungeons)
        let monster_list_y = img_y + img_height as i32 + 10;
        if !is_city && !map_data.enemies.is_empty() {
            let monster_label_style = MonoTextStyle::new(&FONT_10X20, color_yellow);
            Text::new("MONSTERS:", Point::new(margin, monster_list_y), monster_label_style).draw(display)?;

            // Show up to 3 monster names
            let mut y_offset = monster_list_y + 20;
            for (i, enemy_id) in map_data.enemies.iter().take(3).enumerate() {
                if let Some(enemy_data) = self.world_map.game_data().get_enemy(*enemy_id) {
                    let mut monster_text = heapless::String::<32>::new();
                    write!(monster_text, "- {} (Lv {})", enemy_data.name, enemy_data.level).ok();
                    Text::new(&monster_text, Point::new(margin + 5, y_offset), text_style_info).draw(display)?;
                    y_offset += 18;
                }
            }
            if map_data.enemies.len() > 3 {
                Text::new("...", Point::new(margin + 5, y_offset), text_style_info).draw(display)?;
            }
        }

        // Draw action buttons below monster list
        let button_y = if !is_city && !map_data.enemies.is_empty() {
            monster_list_y + 80 // Extra space for monster list
        } else {
            img_y + img_height as i32 + 15
        };
        let button_height = 50u32;
        let button_width = 110u32;
        let button_spacing = 6;
        let text_style_button = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        if is_city {
            // City buttons: "WORLD MAP" only (crafting removed)
            // World Map button
            Rectangle::new(
                Point::new(margin, button_y),
                Size::new(button_width, button_height),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 120, 80)))
            .draw(display)?;

            Text::new(
                "WORLD MAP",
                Point::new(margin + 10, button_y + 32),
                text_style_button,
            )
            .draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (margin, button_y, button_width, button_height),
                location_id: None,
                direction: None,
                is_fight_button: false,
                is_expedition_button: false,
                is_back_button: false,
                is_view_world_map_button: true,
                is_view_monsters_button: false,
                is_mvp_button: false,
                mvp_enemy_id: None,
                is_hunt_button: false,
            });
        } else {
            // Field/Dungeon buttons: "HUNT", "MONSTERS", "WORLD MAP"
            // Hunt button (opens monster selection)
            Rectangle::new(
                Point::new(margin, button_y),
                Size::new(button_width, button_height),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 80, 40)))
            .draw(display)?;

            Text::new(
                "HUNT",
                Point::new(margin + 35, button_y + 32),
                text_style_button,
            )
            .draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (margin, button_y, button_width, button_height),
                location_id: Some(map_id),
                direction: None,
                is_fight_button: false,
                is_expedition_button: false,
                is_back_button: false,
                is_view_world_map_button: false,
                is_view_monsters_button: false,
                is_mvp_button: false,
                mvp_enemy_id: None,
                is_hunt_button: true,
            });

            // Monsters button
            let monsters_x = margin + button_width as i32 + button_spacing;
            Rectangle::new(
                Point::new(monsters_x, button_y),
                Size::new(button_width, button_height),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 80, 140)))
            .draw(display)?;

            Text::new(
                "MONSTERS",
                Point::new(monsters_x + 15, button_y + 32),
                text_style_button,
            )
            .draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (monsters_x, button_y, button_width, button_height),
                location_id: Some(map_id),
                direction: None,
                is_fight_button: false,
                is_expedition_button: false,
                is_back_button: false,
                is_view_world_map_button: false,
                is_view_monsters_button: true,
                is_mvp_button: false,
                mvp_enemy_id: None,
                is_hunt_button: false,
            });

            // World Map button
            let world_map_x = monsters_x + button_width as i32 + button_spacing;
            Rectangle::new(
                Point::new(world_map_x, button_y),
                Size::new(button_width, button_height),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 120, 80)))
            .draw(display)?;

            Text::new(
                "WORLD MAP",
                Point::new(world_map_x + 10, button_y + 32),
                text_style_button,
            )
            .draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (world_map_x, button_y, button_width, button_height),
                location_id: None,
                direction: None,
                is_fight_button: false,
                is_expedition_button: false,
                is_back_button: false,
                is_view_world_map_button: true,
                is_view_monsters_button: false,
                is_mvp_button: false,
                mvp_enemy_id: None,
                is_hunt_button: false,
            });

            // Check for MVP monsters on this map first to determine layout
            let mvp_enemies = self.world_map.game_data().get_mvp_enemies();
            let mvp_for_map: Vec<_> = mvp_enemies
                .iter()
                .filter(|e| e.spawn_map_id == Some(map_id))
                .collect();

            // Second row: AFK FARM and MVP button (if exists) side by side
            let second_row_y = button_y + button_height as i32 + 8;

            if !mvp_for_map.is_empty() {
                // Two buttons side by side: AFK FARM | MVP
                let half_width = 170u32;
                let spacing = 8;
                let total_width = half_width * 2 + spacing as u32;
                let start_x = (368 - total_width as i32) / 2;

                // EXPEDITION button (left)
                Rectangle::new(
                    Point::new(start_x, second_row_y),
                    Size::new(half_width, button_height),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 150, 200)))
                .draw(display)?;

                Text::new(
                    "EXPEDITION",
                    Point::new(start_x + 30, second_row_y + 32),
                    text_style_button,
                )
                .draw(display)?;

                self.touch_areas.push(TouchArea {
                    bounds: (start_x, second_row_y, half_width, button_height),
                    location_id: None,
                    direction: None,
                    is_fight_button: false,
                    is_expedition_button: true,
                    is_back_button: false,
                    is_view_world_map_button: false,
                    is_view_monsters_button: false,
                    is_mvp_button: false,
                    mvp_enemy_id: None,
                    is_hunt_button: false,
                });

                // MVP button (right)
                let mvp = mvp_for_map[0];
                let mvp_button_x = start_x + half_width as i32 + spacing;

                Rectangle::new(
                    Point::new(mvp_button_x, second_row_y),
                    Size::new(half_width, button_height),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(180, 60, 180)))
                .draw(display)?;

                // MVP name (truncated if needed)
                let mut mvp_text = heapless::String::<16>::new();
                use core::fmt::Write as FmtWrite;
                let name_short: String = mvp.name.chars().take(10).collect();
                write!(mvp_text, "{}", name_short).ok();
                Text::new(
                    &mvp_text,
                    Point::new(mvp_button_x + 40, second_row_y + 32),
                    text_style_button,
                )
                .draw(display)?;

                self.touch_areas.push(TouchArea {
                    bounds: (mvp_button_x, second_row_y, half_width, button_height),
                    location_id: Some(map_id),
                    direction: None,
                    is_fight_button: false,
                    is_expedition_button: false,
                    is_back_button: false,
                    is_view_world_map_button: false,
                    is_view_monsters_button: false,
                    is_mvp_button: true,
                    mvp_enemy_id: Some(mvp.id),
                    is_hunt_button: false,
                });
            } else {
                // No MVP - center the EXPEDITION button
                let exp_button_width = 230u32;
                let exp_button_x = (368 - exp_button_width as i32) / 2;

                Rectangle::new(
                    Point::new(exp_button_x, second_row_y),
                    Size::new(exp_button_width, button_height),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 150, 200)))
                .draw(display)?;

                Text::new(
                    "EXPEDITION",
                    Point::new(exp_button_x + 55, second_row_y + 32),
                    text_style_button,
                )
                .draw(display)?;

                self.touch_areas.push(TouchArea {
                    bounds: (exp_button_x, second_row_y, exp_button_width, button_height),
                    location_id: None,
                    direction: None,
                    is_fight_button: false,
                    is_expedition_button: true,
                    is_back_button: false,
                    is_view_world_map_button: false,
                    is_view_monsters_button: false,
                    is_mvp_button: false,
                    mvp_enemy_id: None,
                    is_hunt_button: false,
                });
            }
        }

        Ok(())
    }

    /// Draw Page 3: Monster List with details
    fn draw_monster_list(
        &mut self,
        display: &mut Sh8601Driver,
        map_id: u32,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 15;
        let color_yellow = Rgb888::new(255, 215, 0);
        let text_style_header = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_name = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        // Get map data
        let map_data = match self.world_map.get_location(map_id) {
            Some(data) => data.clone(),
            None => return Err("Map not found".into()),
        };

        // Draw back button
        let back_btn_width = 80u32;
        let back_btn_height = 30u32;
        let back_btn_x = margin;
        let back_btn_y = 15;

        Rectangle::new(
            Point::new(back_btn_x, back_btn_y),
            Size::new(back_btn_width, back_btn_height),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 60, 80)))
        .draw(display)?;

        Text::new(
            "< BACK",
            Point::new(back_btn_x + 8, back_btn_y + 20),
            text_style_header,
        )
        .draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (back_btn_x, back_btn_y, back_btn_width, back_btn_height),
            location_id: Some(map_id), // Remember which map we came from
            direction: None,
            is_fight_button: false,
            is_expedition_button: false,
            is_back_button: true,
            is_view_world_map_button: false,
            is_view_monsters_button: false,
            is_mvp_button: false,
            mvp_enemy_id: None,
            is_hunt_button: false,
        });

        // Draw header
        let header_y = 30;
        let mut header_text = heapless::String::<32>::new();
        write!(header_text, "MONSTERS ({})", map_data.enemies.len()).ok();
        Text::new(
            &header_text,
            Point::new((368 - header_text.len() as i32 * 10) / 2, header_y),
            MonoTextStyle::new(&FONT_10X20, color_yellow),
        )
        .draw(display)?;

        // Draw monster list (scrollable in future, for now show all)
        let list_start_y = 60;
        let card_height = 90;
        let card_spacing = 8;

        for (idx, enemy_id) in map_data.enemies.iter().enumerate() {
            if let Some(enemy_data) = self.world_map.game_data().get_enemy(*enemy_id) {
                let card_y = list_start_y + (idx as i32 * (card_height + card_spacing));

                // Skip if card would go off screen
                if card_y + card_height > 400 {
                    break;
                }

                // Draw card background
                Rectangle::new(
                    Point::new(margin, card_y),
                    Size::new(338, card_height as u32),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 50, 60)))
                .draw(display)?;

                // Draw border
                Rectangle::new(
                    Point::new(margin, card_y),
                    Size::new(338, card_height as u32),
                )
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(80, 90, 100), 1))
                .draw(display)?;

                // Draw sprite placeholder (48×48)
                let sprite_x = margin + 8;
                let sprite_y = card_y + 8;
                Rectangle::new(
                    Point::new(sprite_x, sprite_y),
                    Size::new(48, 48),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 70, 80)))
                .draw(display)?;

                // Draw "MON" text as placeholder
                Text::new(
                    "MON",
                    Point::new(sprite_x + 8, sprite_y + 30),
                    text_style_info,
                )
                .draw(display)?;

                // Draw monster info (right side)
                let info_x = sprite_x + 56;
                let mut current_y = card_y + 18;

                // Line 1: Monster name
                Text::new(&enemy_data.name, Point::new(info_x, current_y), text_style_name).draw(display)?;
                current_y += 18;

                // Line 2: Level
                let mut level_text = heapless::String::<16>::new();
                write!(level_text, "Lvl {}", enemy_data.level).ok();
                Text::new(&level_text, Point::new(info_x, current_y), MonoTextStyle::new(&FONT_10X20, color_yellow)).draw(display)?;
                current_y += 18;

                // Line 3: HP estimate
                let mut hp_text = heapless::String::<24>::new();
                write!(hp_text, "HP: ~{}", enemy_data.hp).ok();
                Text::new(&hp_text, Point::new(info_x, current_y), text_style_info).draw(display)?;
                let _ = current_y; // Silence unused variable warning
            }
        }

        // Draw "START BATTLE" button at bottom if there are monsters
        if !map_data.enemies.is_empty() {
            let button_y = 395;
            let button_width = 160u32;
            let button_height = 45u32;
            let button_x = (368 - button_width as i32) / 2;

            Rectangle::new(
                Point::new(button_x, button_y),
                Size::new(button_width, button_height),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 40, 40)))
            .draw(display)?;

            Text::new(
                "START BATTLE",
                Point::new(button_x + 10, button_y + 30),
                text_style_header,
            )
            .draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (button_x, button_y, button_width, button_height),
                location_id: Some(map_id),
                direction: None,
                is_fight_button: true,
                is_expedition_button: false,
                is_back_button: false,
                is_view_world_map_button: false,
                is_view_monsters_button: false,
                is_mvp_button: false,
                mvp_enemy_id: None,
                is_hunt_button: false,
            });
        }

        Ok(())
    }

    /// Handle touch input at coordinates
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<TouchAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                // Handle back button
                if area.is_back_button {
                    // Check current page to determine where to go back to
                    match self.navigation_page {
                        NavigationPage::MonsterList { map_id } => {
                            // Page 3 → Page 2 (back to map details)
                            log::info!("Back button pressed from monster list, returning to map details");
                            self.navigation_page = NavigationPage::MapDetails { map_id };
                            self.needs_full_redraw = true;
                            return Some(TouchAction::BackToMapDetails);
                        }
                        _ => {
                            // Page 2 → Page 1 (back to world map)
                            log::info!("Back button pressed, returning to world map");
                            self.navigation_page = NavigationPage::WorldMapGrid { scroll_offset: 0 };
                            self.needs_full_redraw = true;
                            return Some(TouchAction::BackToWorldMap);
                        }
                    }
                }

                // Handle "VIEW MONSTERS" button (Page 2 → Page 3)
                if area.is_view_monsters_button {
                    if let Some(map_id) = area.location_id {
                        log::info!("View monsters button pressed for map {}", map_id);
                        self.navigation_page = NavigationPage::MonsterList { map_id };
                        self.needs_full_redraw = true;
                        self.touch_areas.clear();
                        return Some(TouchAction::ViewMonsterList(map_id));
                    }
                }

                // Handle "Travel to Another Map" button (Page 2 → Page 1)
                if area.is_view_world_map_button {
                    log::info!("Travel button pressed, showing world map");
                    self.navigation_page = NavigationPage::WorldMapGrid { scroll_offset: 0 };
                    self.needs_full_redraw = true;
                    self.touch_areas.clear(); // Clear touch areas to force rebuild
                    return Some(TouchAction::BackToWorldMap);
                }

                // Handle fight button
                if area.is_fight_button {
                    log::info!("Fight button pressed!");

                    // If fight button has a location_id (from Page 2), travel there first
                    if let Some(target_map_id) = area.location_id {
                        log::info!("Traveling to map {} to fight", target_map_id);
                        if let Err(e) = self.travel_to(target_map_id) {
                            log::error!("Failed to travel before fight: {}", e);
                            return None;
                        }
                    }

                    return Some(TouchAction::Fight);
                }

                // Handle expedition button
                if area.is_expedition_button {
                    log::info!("🗺️ Expedition button pressed!");
                    return Some(TouchAction::Expedition);
                }

                // Handle MVP fight button
                if area.is_mvp_button {
                    if let Some(enemy_id) = area.mvp_enemy_id {
                        log::info!("MVP Fight button pressed for enemy {}", enemy_id);
                        return Some(TouchAction::MvpFight(enemy_id));
                    }
                }

                // Handle Hunt button
                if area.is_hunt_button {
                    if let Some(map_id) = area.location_id {
                        log::info!("Hunt button pressed for map {}", map_id);
                        return Some(TouchAction::Hunt(map_id));
                    }
                }

                // Handle location selection
                if let Some(location_id) = area.location_id {
                    // Check which page we're on
                    match self.navigation_page {
                        NavigationPage::WorldMapGrid { .. } => {
                            // On Page 1: Tap map → Go to Page 2 (Map Details)
                            log::info!("Viewing map {} details", location_id);
                            self.navigation_page = NavigationPage::MapDetails { map_id: location_id };
                            self.needs_full_redraw = true;
                            self.touch_areas.clear(); // Clear to force rebuild
                            return Some(TouchAction::ViewMapDetails(location_id));
                        }
                        NavigationPage::OldCardView => {
                            // Legacy card view: Travel directly
                            if let Some(direction) = area.direction {
                                log::info!("Traveling to location {} via {}", location_id, direction.as_str());
                            }
                            return Some(TouchAction::Travel(location_id));
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Navigate to a location by ID (no restrictions - can go to any map)
    pub fn travel_to(&mut self, location_id: u32) -> Result<(), String> {
        // Check if map exists
        if self.world_map.game_data().get_map(location_id).is_none() {
            return Err(format!("Map {} does not exist", location_id));
        }

        // Travel directly without checking connections
        self.world_map.set_current_location(location_id);
        log::info!("Traveled to map: {}", location_id);

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

        // Card background with black border
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 60, 80)))
            .draw(display)?;

        // Black border
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::BLACK, 2))
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

        // Monster names if field (only show first monster name with level)
        if !location.enemies.is_empty() {
            let mut monsters_text = heapless::String::<32>::new();
            if let Some(enemy_data) = location.enemies.get(0)
                .and_then(|id| self.world_map.game_data().get_enemy(*id)) {
                write!(monsters_text, "{} (Lv {})", enemy_data.name, enemy_data.level).ok();
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
            is_expedition_button: false,
            is_back_button: false,
            is_view_world_map_button: false,
            is_view_monsters_button: false,
            is_mvp_button: false,
            mvp_enemy_id: None,
            is_hunt_button: false,
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

            // Show monster names with levels (up to 3)
            let mut monster_text = heapless::String::<64>::new();
            for (i, enemy_id) in location.enemies.iter().take(3).enumerate() {
                if let Some(enemy_data) = self.world_map.game_data().get_enemy(*enemy_id) {
                    if i > 0 {
                        write!(monster_text, ", {} (Lv {})", enemy_data.name, enemy_data.level).ok();
                    } else {
                        write!(monster_text, "{} (Lv {})", enemy_data.name, enemy_data.level).ok();
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
                is_expedition_button: false,
                is_back_button: false,
                is_view_world_map_button: false,
                is_view_monsters_button: false,
                is_mvp_button: false,
                mvp_enemy_id: None,
                is_hunt_button: false,
            });

            // AFK FARM button (next to FIGHT button)
            let afk_x = margin + button_width as i32 + button_spacing;
            Rectangle::new(Point::new(afk_x, button_y), Size::new(button_width, button_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 150, 200)))
                .draw(display)?;

            Text::new("AFK FARM", Point::new(afk_x + 25, button_y + 40), text_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (afk_x, button_y, button_width, button_height),
                location_id: None,
                direction: None,
                is_fight_button: false,
                is_expedition_button: true,
                is_back_button: false,
                is_view_world_map_button: false,
                is_view_monsters_button: false,
                is_mvp_button: false,
                mvp_enemy_id: None,
                is_hunt_button: false,
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

        // Dispatch based on navigation page
        match self.navigation_page {
            NavigationPage::WorldMapGrid { scroll_offset } => {
                // Page 1: World Map Grid
                self.draw_world_map_grid(display, scroll_offset)?;
            }
            NavigationPage::MapDetails { map_id } => {
                // Page 2: Map Details
                self.draw_map_details(display, map_id)?;
            }
            NavigationPage::MonsterList { map_id } => {
                // Page 3: Monster List
                self.draw_monster_list(display, map_id)?;
            }
            NavigationPage::OldCardView => {
                // Legacy: Original card-based navigation
                let current_location = self
                    .world_map
                    .current_location()
                    .ok_or("No current location")?
                    .clone();

                let connections: Vec<(Direction, MapData)> = self
                    .world_map
                    .connected_locations_with_directions()
                    .into_iter()
                    .map(|(dir, map_data)| (dir, map_data.clone()))
                    .collect();

                let has_enemies = !current_location.enemies.is_empty();
                let is_city = current_location.npcs.as_ref().map_or(false, |npcs| !npcs.is_empty());

                self.draw_header_with_minimap(display, &current_location)?;
                self.draw_card_navigation(display, &connections)?;
                self.draw_current_area_info(display, &current_location)?;
                self.draw_action_buttons(display, has_enemies, is_city)?;
            }
        }

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
