//! Map Navigation Page
//!
//! Displays zones and allows navigation based on GDD structure:
//! - Zone List (CARTE DU MONDE)
//! - Zone Detail (DÉTAIL ZONE)
//! - Dungeon Selection

use crate::display::Sh8601Driver;
use crate::game::WorldMap;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::collections::HashMap;
use std::error::Error;

/// Navigation page state based on GDD
#[derive(Debug, Clone, PartialEq)]
enum NavigationPage {
    /// Zone list (CARTE DU MONDE) - shows all zones with dungeon records
    ZoneList { scroll_offset: usize },
    /// Zone detail (DÉTAIL ZONE) - shows expedition maps and dungeon for one zone
    ZoneDetail { zone_id: String, scroll_offset: usize },
    /// Dungeon selection - shows checkpoint selection and team
    DungeonSelect { zone_id: String },
}

/// Touch area for map navigation
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32), // (x, y, width, height)
    action: TouchAreaAction,
}

/// Actions for touch areas
#[derive(Debug, Clone)]
enum TouchAreaAction {
    /// Select a zone to view details
    SelectZone(String),
    /// Select a map within a zone for expedition
    SelectMap(String),
    /// Enter dungeon for a zone
    EnterDungeon(String),
    /// Back button
    Back,
    /// Start dungeon combat
    StartDungeonCombat,
}

impl TouchArea {
    fn new(x: i32, y: i32, width: u32, height: u32, action: TouchAreaAction) -> Self {
        Self {
            bounds: (x, y, width, height),
            action,
        }
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Map navigation page - follows GDD structure
pub struct MapPage {
    world_map: WorldMap,
    touch_areas: Vec<TouchArea>,
    background_color: Rgb888,
    needs_full_redraw: bool,
    navigation_page: NavigationPage,
    /// Dungeon progress for unlock checking
    dungeon_progress: HashMap<String, u16>,
    /// Cached zones data (to avoid loading from disk every frame)
    zones: Vec<crate::game::core::Zone>,
    /// Cached maps data
    tamer_maps: Vec<crate::game::core::TamerMap>,
}

/// Touch action result - actions that need to be handled by the navigation system
#[derive(Debug, Clone)]
pub enum TouchAction {
    /// Start expedition on a map (handled by expedition system)
    StartExpedition(String),
    /// Enter dungeon combat (dungeon_id, start_floor)
    StartDungeon { dungeon_id: String, start_floor: u16 },
    /// Return to home
    BackToHome,
    /// No action (internal navigation handled)
    None,
}

impl MapPage {
    /// Create a new map page with world map data
    pub fn new(world_map: WorldMap, _asset_loader: Option<crate::ecs::resources::SdCardWrapper>) -> Self {
        Self {
            world_map,
            touch_areas: Vec::new(),
            background_color: Rgb888::new(20, 30, 40),
            needs_full_redraw: true,
            navigation_page: NavigationPage::ZoneList { scroll_offset: 0 },
            dungeon_progress: HashMap::new(),
            zones: Vec::new(),
            tamer_maps: Vec::new(),
        }
    }

    /// Update cached game data (call once when entering Map mode, not every frame)
    pub fn set_game_data(&mut self, zones: Vec<crate::game::core::Zone>, tamer_maps: Vec<crate::game::core::TamerMap>) {
        self.zones = zones;
        self.tamer_maps = tamer_maps;
        self.needs_full_redraw = true;
    }

    /// Check if game data has been set
    pub fn has_game_data(&self) -> bool {
        !self.zones.is_empty()
    }

    /// Create map page from save data (with specific location)
    pub fn from_save(
        mut world_map: WorldMap,
        current_location_id: u32,
        asset_loader: Option<crate::ecs::resources::SdCardWrapper>,
    ) -> Self {
        world_map.set_current_location(current_location_id);
        Self::new(world_map, asset_loader)
    }

    /// Get reference to world map
    pub fn world_map(&self) -> &WorldMap {
        &self.world_map
    }

    /// Update dungeon progress from game state
    pub fn update_dungeon_progress(&mut self, progress: &HashMap<String, u16>) {
        self.dungeon_progress = progress.clone();
    }

    /// Get element icon character
    fn element_icon(element: &crate::game::core::Element) -> &'static str {
        use crate::game::core::Element;
        match element {
            Element::Fire => "F",
            Element::Water => "W",
            Element::Earth => "E",
            Element::Wind => "A",
            Element::Thunder => "T",
            Element::Shadow => "S",
            Element::Holy => "H",
            Element::Ghost => "G",
        }
    }

    /// Get element color
    fn element_color(element: &crate::game::core::Element) -> Rgb888 {
        use crate::game::core::Element;
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(100, 180, 80),
            Element::Wind => Rgb888::new(100, 200, 150),
            Element::Thunder => Rgb888::new(255, 255, 50),
            Element::Shadow => Rgb888::new(150, 100, 200),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(180, 180, 220),
        }
    }

    /// Draw Zone List page (CARTE DU MONDE - GDD 3.3.2)
    fn draw_zone_list(
        &mut self,
        display: &mut Sh8601Driver,
        scroll_offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        let zones = &self.zones;
        let tamer_maps = &self.tamer_maps;
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 15;
        let color_yellow = Rgb888::new(255, 215, 0);
        let color_gray = Rgb888::new(150, 150, 150);
        let color_locked = Rgb888::new(80, 80, 80);

        let text_style_header = MonoTextStyle::new(&FONT_10X20, color_yellow);
        let text_style_name = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_info = MonoTextStyle::new(&FONT_10X20, color_gray);
        let text_style_locked = MonoTextStyle::new(&FONT_10X20, color_locked);

        // Header: "CARTE" with swipe hint
        Text::new("CARTE", Point::new(margin, 30), text_style_header).draw(display)?;
        Text::new("swipe >", Point::new(280, 30), text_style_info).draw(display)?;

        // Zone list
        let zone_start_y = 55;
        let zone_height = 75;
        let zone_spacing = 8;

        for (idx, zone) in zones.iter().skip(scroll_offset).take(5).enumerate() {
            let y = zone_start_y + (idx as i32 * (zone_height + zone_spacing));
            let zone_rect_height = zone_height as u32;

            // Check if zone is unlocked
            let is_unlocked = zone.is_unlocked(&self.dungeon_progress);

            // Zone background
            let bg_color = if is_unlocked {
                Rgb888::new(35, 45, 55)
            } else {
                Rgb888::new(25, 30, 35)
            };
            Rectangle::new(Point::new(margin, y), Size::new(338, zone_rect_height))
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            // Border
            let border_color = if is_unlocked { color_gray } else { color_locked };
            Rectangle::new(Point::new(margin, y), Size::new(338, zone_rect_height))
                .into_styled(PrimitiveStyle::with_stroke(border_color, 1))
                .draw(display)?;

            if is_unlocked {
                // Zone name with arrow
                let mut zone_name = heapless::String::<32>::new();
                write!(zone_name, "> {}", zone.name).ok();
                Text::new(&zone_name, Point::new(margin + 10, y + 22), text_style_name).draw(display)?;

                // Count maps for this zone
                let map_count = tamer_maps.iter().filter(|m| m.zone_id == zone.id).count();

                // Maps info
                let mut maps_text = heapless::String::<32>::new();
                write!(maps_text, "{} maps (Niv.{}-{})", map_count, zone.level_range.0, zone.level_range.1).ok();
                Text::new(&maps_text, Point::new(margin + 20, y + 42), text_style_info).draw(display)?;

                // Dungeon info with record
                let record = self.dungeon_progress.get(&zone.dungeon_id).copied().unwrap_or(0);
                let mut dungeon_text = heapless::String::<48>::new();
                if record > 0 {
                    write!(dungeon_text, "Donjon: {} - Record: Et.{}", zone.dungeon_id, record).ok();
                } else {
                    write!(dungeon_text, "Donjon: {}", zone.dungeon_id).ok();
                }
                Text::new(&dungeon_text, Point::new(margin + 20, y + 62), text_style_info).draw(display)?;

                // Add touch area
                self.touch_areas.push(TouchArea::new(
                    margin, y, 338, zone_rect_height,
                    TouchAreaAction::SelectZone(zone.id.clone()),
                ));
            } else {
                // Locked zone
                let mut zone_name = heapless::String::<32>::new();
                write!(zone_name, "> {} [LOCKED]", zone.name).ok();
                Text::new(&zone_name, Point::new(margin + 10, y + 22), text_style_locked).draw(display)?;

                // Unlock condition
                if let Some(crate::game::core::UnlockCondition::DungeonFloor { dungeon_id, floor }) = &zone.unlock_condition {
                    let mut unlock_text = heapless::String::<48>::new();
                    write!(unlock_text, "Debloquer: {} Et.{}", dungeon_id, floor).ok();
                    Text::new(&unlock_text, Point::new(margin + 20, y + 45), text_style_locked).draw(display)?;
                }
            }
        }

        // Scroll indicator
        if zones.len() > 5 {
            Text::new("scroll", Point::new(160, 430), text_style_info).draw(display)?;
        }

        Ok(())
    }

    /// Draw Zone Detail page (DÉTAIL ZONE - GDD 3.3.3)
    fn draw_zone_detail(
        &mut self,
        display: &mut Sh8601Driver,
        zone_id: &str,
        scroll_offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        let zones = &self.zones;
        let tamer_maps = &self.tamer_maps;
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 15;
        let color_yellow = Rgb888::new(255, 215, 0);
        let color_gray = Rgb888::new(150, 150, 150);

        let text_style_header = MonoTextStyle::new(&FONT_10X20, color_yellow);
        let text_style_section = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_info = MonoTextStyle::new(&FONT_10X20, color_gray);

        // Find the zone
        let zone = zones.iter().find(|z| z.id == zone_id);
        let zone_name = zone.map(|z| z.name.as_str()).unwrap_or("Unknown");
        let dungeon_id = zone.map(|z| z.dungeon_id.as_str()).unwrap_or("");

        // Header with zone name and swipe hint
        Text::new(zone_name, Point::new(margin, 30), text_style_header).draw(display)?;
        Text::new("swipe >", Point::new(280, 30), text_style_info).draw(display)?;

        // EXPEDITIONS section
        Text::new("EXPEDITIONS", Point::new(margin, 60), text_style_section).draw(display)?;

        // Get maps for this zone
        let zone_maps: Vec<&crate::game::core::TamerMap> = tamer_maps
            .iter()
            .filter(|m| m.zone_id == zone_id)
            .collect();

        let map_start_y = 80;
        let map_height = 50;
        let map_spacing = 5;

        for (idx, map) in zone_maps.iter().skip(scroll_offset).take(4).enumerate() {
            let y = map_start_y + (idx as i32 * (map_height + map_spacing));

            // Map background
            Rectangle::new(Point::new(margin, y), Size::new(338, map_height as u32))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(35, 45, 55)))
                .draw(display)?;

            // Map name with arrow
            let mut map_name = heapless::String::<32>::new();
            write!(map_name, "> {}", map.name).ok();
            Text::new(&map_name, Point::new(margin + 10, y + 20), text_style_section).draw(display)?;

            // Level range and required elements
            let mut info_text = heapless::String::<48>::new();
            write!(info_text, "Niv.{}-{} | ", map.level_range.0, map.level_range.1).ok();
            Text::new(&info_text, Point::new(margin + 20, y + 40), text_style_info).draw(display)?;

            // Draw required elements
            let mut elem_x = margin + 100;
            for elem in &map.required_elements {
                let elem_icon = Self::element_icon(elem);
                let elem_color = Self::element_color(elem);
                let elem_style = MonoTextStyle::new(&FONT_10X20, elem_color);
                Text::new(elem_icon, Point::new(elem_x, y + 40), elem_style).draw(display)?;
                elem_x += 15;
            }
            Text::new("requis", Point::new(elem_x + 5, y + 40), text_style_info).draw(display)?;

            // Add touch area
            self.touch_areas.push(TouchArea::new(
                margin, y, 338, map_height as u32,
                TouchAreaAction::SelectMap(map.id.clone()),
            ));
        }

        // Separator
        let separator_y = map_start_y + (zone_maps.len().min(4) as i32 * (map_height + map_spacing)) + 10;
        Rectangle::new(Point::new(margin, separator_y), Size::new(338, 2))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 70, 80)))
            .draw(display)?;

        // DONJON section
        let dungeon_section_y = separator_y + 20;
        Text::new("DONJON", Point::new(margin, dungeon_section_y), text_style_section).draw(display)?;

        // Dungeon entry button
        let dungeon_y = dungeon_section_y + 25;
        let dungeon_height = 55u32;

        Rectangle::new(Point::new(margin, dungeon_y), Size::new(338, dungeon_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
            .draw(display)?;
        Rectangle::new(Point::new(margin, dungeon_y), Size::new(338, dungeon_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(200, 100, 100), 2))
            .draw(display)?;

        // Dungeon name
        let mut dungeon_text = heapless::String::<32>::new();
        write!(dungeon_text, "Donjon: {}", dungeon_id).ok();
        Text::new(&dungeon_text, Point::new(margin + 20, dungeon_y + 25), text_style_section).draw(display)?;

        // Record
        let record = self.dungeon_progress.get(dungeon_id).copied().unwrap_or(0);
        let mut record_text = heapless::String::<32>::new();
        if record > 0 {
            write!(record_text, "Record: Etage {}", record).ok();
        } else {
            write!(record_text, "Non explore").ok();
        }
        Text::new(&record_text, Point::new(margin + 20, dungeon_y + 45), text_style_info).draw(display)?;

        self.touch_areas.push(TouchArea::new(
            margin, dungeon_y, 338, dungeon_height,
            TouchAreaAction::EnterDungeon(dungeon_id.to_string()),
        ));

        Ok(())
    }

    /// Draw Dungeon Selection page (GDD 3.3.8)
    fn draw_dungeon_select(
        &mut self,
        display: &mut Sh8601Driver,
        zone_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let zones = &self.zones;
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 15;
        let color_yellow = Rgb888::new(255, 215, 0);
        let color_gray = Rgb888::new(150, 150, 150);

        let text_style_header = MonoTextStyle::new(&FONT_10X20, color_yellow);
        let text_style_section = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_info = MonoTextStyle::new(&FONT_10X20, color_gray);

        // Find zone
        let zone = zones.iter().find(|z| z.id == zone_id);
        let dungeon_id = zone.map(|z| z.dungeon_id.as_str()).unwrap_or("Unknown");
        let record = self.dungeon_progress.get(dungeon_id).copied().unwrap_or(0);

        // Header
        let mut header = heapless::String::<32>::new();
        write!(header, "DONJON: {}", dungeon_id).ok();
        Text::new(&header, Point::new(margin, 30), text_style_header).draw(display)?;

        // Record display
        let mut record_text = heapless::String::<32>::new();
        write!(record_text, "Record: Etage {}", record).ok();
        Text::new(&record_text, Point::new(margin, 55), text_style_info).draw(display)?;

        // Checkpoint selection
        Text::new("Commencer depuis:", Point::new(margin, 90), text_style_section).draw(display)?;

        let checkpoints = [1, 10, 20, 30, 40, 50];
        let checkpoint_y_start = 115;
        let checkpoint_height = 40;
        let checkpoint_spacing = 8;

        for (idx, &checkpoint) in checkpoints.iter().enumerate() {
            let y = checkpoint_y_start + (idx as i32 * (checkpoint_height + checkpoint_spacing));
            let is_unlocked = checkpoint <= record || checkpoint == 1;

            let bg_color = if is_unlocked {
                if checkpoint == 1 {
                    Rgb888::new(60, 80, 60) // Selected/default
                } else {
                    Rgb888::new(40, 50, 60)
                }
            } else {
                Rgb888::new(30, 35, 40)
            };

            Rectangle::new(Point::new(margin, y), Size::new(338, checkpoint_height as u32))
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            let mut cp_text = heapless::String::<32>::new();
            if is_unlocked {
                let stars = match checkpoint {
                    1 => "***",
                    10 => "** ",
                    20 => "***",
                    30 => "***",
                    _ => "***",
                };
                write!(cp_text, "> Etage {}   {}", checkpoint, stars).ok();
                Text::new(&cp_text, Point::new(margin + 15, y + 27), text_style_section).draw(display)?;
            } else {
                write!(cp_text, "  Etage {}   [LOCKED]", checkpoint).ok();
                Text::new(&cp_text, Point::new(margin + 15, y + 27), text_style_info).draw(display)?;
            }
        }

        // ENTER button at bottom
        let enter_y = 395;
        let enter_height = 45u32;

        Rectangle::new(Point::new(margin, enter_y), Size::new(338, enter_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 60, 60)))
            .draw(display)?;
        Rectangle::new(Point::new(margin, enter_y), Size::new(338, enter_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(255, 100, 100), 2))
            .draw(display)?;

        Text::new("ENTRER", Point::new(150, enter_y + 30), text_style_header).draw(display)?;

        self.touch_areas.push(TouchArea::new(
            margin, enter_y, 338, enter_height,
            TouchAreaAction::StartDungeonCombat,
        ));

        Ok(())
    }

    /// Handle touch input and return action
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<TouchAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                match &area.action {
                    TouchAreaAction::SelectZone(zone_id) => {
                        log::info!("Selected zone: {}", zone_id);
                        self.navigation_page = NavigationPage::ZoneDetail {
                            zone_id: zone_id.clone(),
                            scroll_offset: 0,
                        };
                        self.needs_full_redraw = true;
                        return Some(TouchAction::None);
                    }
                    TouchAreaAction::SelectMap(map_id) => {
                        log::info!("Selected map for expedition: {}", map_id);
                        return Some(TouchAction::StartExpedition(map_id.clone()));
                    }
                    TouchAreaAction::EnterDungeon(dungeon_id) => {
                        log::info!("Entering dungeon selection: {}", dungeon_id);
                        // Find zone for this dungeon
                        if let NavigationPage::ZoneDetail { zone_id, .. } = &self.navigation_page {
                            self.navigation_page = NavigationPage::DungeonSelect {
                                zone_id: zone_id.clone(),
                            };
                            self.needs_full_redraw = true;
                        }
                        return Some(TouchAction::None);
                    }
                    TouchAreaAction::Back => {
                        self.go_back();
                        return Some(TouchAction::None);
                    }
                    TouchAreaAction::StartDungeonCombat => {
                        // Get dungeon_id from current navigation state
                        if let NavigationPage::DungeonSelect { zone_id } = &self.navigation_page {
                            // Find zone to get dungeon_id
                            if let Some(zone) = self.zones.iter().find(|z| z.id == *zone_id) {
                                let dungeon_id = zone.dungeon_id.clone();
                                log::info!("Starting dungeon: {} from floor 1", dungeon_id);
                                return Some(TouchAction::StartDungeon { dungeon_id, start_floor: 1 });
                            }
                        }
                        log::warn!("Could not determine dungeon_id for combat");
                        return Some(TouchAction::None);
                    }
                }
            }
        }
        None
    }

    /// Go back to previous page
    pub fn go_back(&mut self) {
        match &self.navigation_page {
            NavigationPage::ZoneList { .. } => {
                // Already at top level, handled by caller
            }
            NavigationPage::ZoneDetail { .. } => {
                self.navigation_page = NavigationPage::ZoneList { scroll_offset: 0 };
                self.needs_full_redraw = true;
            }
            NavigationPage::DungeonSelect { zone_id } => {
                self.navigation_page = NavigationPage::ZoneDetail {
                    zone_id: zone_id.clone(),
                    scroll_offset: 0,
                };
                self.needs_full_redraw = true;
            }
        }
    }

    /// Check if we're at the top level (can swipe back to home)
    pub fn is_at_top_level(&self) -> bool {
        matches!(self.navigation_page, NavigationPage::ZoneList { .. })
    }

    /// Navigate to a location by ID (legacy support)
    pub fn travel_to(&mut self, _location_id: u32) -> Result<(), String> {
        // This is now handled differently with the zone-based navigation
        Ok(())
    }
}

impl Page for MapPage {
    fn update(&mut self) -> bool {
        true
    }

    fn draw(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
        }

        // Use cached zones and maps data (set via set_game_data)
        match self.navigation_page.clone() {
            NavigationPage::ZoneList { scroll_offset } => {
                self.draw_zone_list(display, scroll_offset)?;
            }
            NavigationPage::ZoneDetail { zone_id, scroll_offset } => {
                self.draw_zone_detail(display, &zone_id, scroll_offset)?;
            }
            NavigationPage::DungeonSelect { zone_id } => {
                self.draw_dungeon_select(display, &zone_id)?;
            }
        }

        display.flush()?;
        self.needs_full_redraw = false;
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
