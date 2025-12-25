//! Map Navigation Page
//!
//! Displays zones and allows navigation based on GDD structure:
//! - Zone List (CARTE DU MONDE)
//! - Zone Detail (DÉTAIL ZONE)
//! - Dungeon Selection

use crate::display::St7789pDriver;
use crate::game::WorldMap;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::{FONT_6X10, FONT_7X13}, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
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
            background_color: Rgb888::new(240, 240, 245), // Light theme
            needs_full_redraw: true,
            navigation_page: NavigationPage::ZoneList { scroll_offset: 0 },
            dungeon_progress: HashMap::new(),
            zones: Vec::new(),
            tamer_maps: Vec::new(),
        }
    }

    /// Update cached game data (call once when entering Map mode, not every frame)
    /// Sorts zones and maps by level for better navigation
    pub fn set_game_data(&mut self, zones: Vec<crate::game::core::Zone>, tamer_maps: Vec<crate::game::core::TamerMap>) {
        // Sort zones by minimum level
        let mut sorted_zones = zones;
        sorted_zones.sort_by_key(|z| z.level_range.0);
        self.zones = sorted_zones;

        // Sort maps by minimum level
        let mut sorted_maps = tamer_maps;
        sorted_maps.sort_by_key(|m| m.level_range.0);
        self.tamer_maps = sorted_maps;

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
            Element::Neutral => "N",
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
            Element::Neutral => Rgb888::new(180, 180, 180),
        }
    }

    /// Draw Zone List page (CARTE DU MONDE - GDD 3.3.2)
    fn draw_zone_list(
        &mut self,
        display: &mut St7789pDriver,
        scroll_offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        let zones = &self.zones;
        let tamer_maps = &self.tamer_maps;
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 10;
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let locked_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));

        // Header with rounded background
        let header_rect = Rectangle::new(Point::new(margin, 4), Size::new(220, 24));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 150, 200))
            .build())
            .draw(display)?;

        Text::new("CARTE", Point::new(95, 20), title_style).draw(display)?;

        // Zone list
        let zone_start_y = 32;
        let zone_height = 58i32;
        let zone_spacing = 4;

        for (idx, zone) in zones.iter().skip(scroll_offset).take(4).enumerate() {
            let y = zone_start_y + (idx as i32 * (zone_height + zone_spacing));

            // Check if zone is unlocked
            let is_unlocked = zone.is_unlocked(&self.dungeon_progress);

            // Zone card with rounded corners
            let zone_rect = Rectangle::new(Point::new(margin, y), Size::new(220, zone_height as u32));
            let zone_rounded = RoundedRectangle::new(zone_rect, CornerRadii::new(Size::new(8, 8)));

            let (bg_color, border_color) = if is_unlocked {
                (Rgb888::new(250, 250, 255), Rgb888::new(180, 185, 195))
            } else {
                (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
            };

            // Fill
            zone_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(bg_color)
                .build())
                .draw(display)?;

            // Border
            zone_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(border_color)
                .stroke_width(1)
                .build())
                .draw(display)?;

            if is_unlocked {
                // Zone name
                let zone_name = if zone.name.len() > 18 { &zone.name[..18] } else { &zone.name };
                let mut zone_text = heapless::String::<32>::new();
                write!(zone_text, "> {}", zone_name).ok();
                Text::new(&zone_text, Point::new(margin + 8, y + 14), text_style).draw(display)?;

                // Count maps for this zone
                let map_count = tamer_maps.iter().filter(|m| m.zone_id == zone.id).count();

                // Maps info
                let mut maps_text = heapless::String::<32>::new();
                write!(maps_text, "{} maps Lv.{}-{}", map_count, zone.level_range.0, zone.level_range.1).ok();
                Text::new(&maps_text, Point::new(margin + 16, y + 28), dim_style).draw(display)?;

                // Dungeon info with record
                let record = self.dungeon_progress.get(&zone.dungeon_id).copied().unwrap_or(0);
                let mut dungeon_text = heapless::String::<32>::new();
                if record > 0 {
                    write!(dungeon_text, "Donjon Rec:Et.{}", record).ok();
                } else {
                    write!(dungeon_text, "Donjon: {}", zone.dungeon_id).ok();
                }
                Text::new(&dungeon_text, Point::new(margin + 16, y + 42), dim_style).draw(display)?;

                // Add touch area
                self.touch_areas.push(TouchArea::new(
                    margin, y, 220, zone_height as u32,
                    TouchAreaAction::SelectZone(zone.id.clone()),
                ));
            } else {
                // Locked zone
                let zone_name = if zone.name.len() > 12 { &zone.name[..12] } else { &zone.name };
                let mut zone_text = heapless::String::<32>::new();
                write!(zone_text, "{} [LOCKED]", zone_name).ok();
                Text::new(&zone_text, Point::new(margin + 8, y + 18), locked_style).draw(display)?;

                // Unlock condition
                if let Some(crate::game::core::UnlockCondition::DungeonFloor { dungeon_id, floor }) = &zone.unlock_condition {
                    let mut unlock_text = heapless::String::<32>::new();
                    write!(unlock_text, "Need {} Et.{}", dungeon_id, floor).ok();
                    Text::new(&unlock_text, Point::new(margin + 16, y + 36), locked_style).draw(display)?;
                }
            }
        }

        // Scroll indicator
        if zones.len() > 4 {
            Text::new("swipe to scroll", Point::new(75, 278), dim_style).draw(display)?;
        }

        Ok(())
    }

    /// Draw Zone Detail page (DÉTAIL ZONE - GDD 3.3.3)
    fn draw_zone_detail(
        &mut self,
        display: &mut St7789pDriver,
        zone_id: &str,
        scroll_offset: usize,
    ) -> Result<(), Box<dyn Error>> {
        let zones = &self.zones;
        let tamer_maps = &self.tamer_maps;
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 10;
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));

        // Find the zone
        let zone = zones.iter().find(|z| z.id == zone_id);
        let zone_name = zone.map(|z| z.name.as_str()).unwrap_or("Unknown");
        let zone_name = if zone_name.len() > 16 { &zone_name[..16] } else { zone_name };
        let dungeon_id = zone.map(|z| z.dungeon_id.as_str()).unwrap_or("");

        // Header with zone name
        let header_rect = Rectangle::new(Point::new(margin, 4), Size::new(220, 24));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 150, 200))
            .build())
            .draw(display)?;

        Text::new(zone_name, Point::new(margin + 8, 20), title_style).draw(display)?;

        // EXPEDITIONS section
        Text::new("EXPEDITIONS", Point::new(margin, 40), dim_style).draw(display)?;

        // Get maps for this zone
        let zone_maps: Vec<&crate::game::core::TamerMap> = tamer_maps
            .iter()
            .filter(|m| m.zone_id == zone_id)
            .collect();

        let map_start_y = 48;
        let map_height = 42i32;
        let map_spacing = 4;

        for (idx, map) in zone_maps.iter().skip(scroll_offset).take(3).enumerate() {
            let y = map_start_y + (idx as i32 * (map_height + map_spacing));

            // Map card with rounded corners
            let map_rect = Rectangle::new(Point::new(margin, y), Size::new(220, map_height as u32));
            let map_rounded = RoundedRectangle::new(map_rect, CornerRadii::new(Size::new(6, 6)));

            map_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(250, 250, 255))
                .build())
                .draw(display)?;
            map_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(180, 185, 195))
                .stroke_width(1)
                .build())
                .draw(display)?;

            // Map name
            let map_name = if map.name.len() > 20 { &map.name[..20] } else { &map.name };
            let mut name_text = heapless::String::<32>::new();
            write!(name_text, "> {}", map_name).ok();
            Text::new(&name_text, Point::new(margin + 8, y + 14), text_style).draw(display)?;

            // Level range and required elements
            let mut info_text = heapless::String::<32>::new();
            write!(info_text, "Lv.{}-{}", map.level_range.0, map.level_range.1).ok();
            Text::new(&info_text, Point::new(margin + 16, y + 28), dim_style).draw(display)?;

            // Required elements
            let mut elem_x = margin + 70;
            for elem in &map.required_elements {
                let elem_icon = Self::element_icon(elem);
                let elem_color = Self::element_color(elem);
                let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
                Text::new(elem_icon, Point::new(elem_x, y + 28), elem_style).draw(display)?;
                elem_x += 10;
            }

            // Add touch area
            self.touch_areas.push(TouchArea::new(
                margin, y, 220, map_height as u32,
                TouchAreaAction::SelectMap(map.id.clone()),
            ));
        }

        // DONJON section
        let dungeon_section_y = map_start_y + (zone_maps.len().min(3) as i32 * (map_height + map_spacing)) + 8;
        Text::new("DONJON", Point::new(margin, dungeon_section_y), dim_style).draw(display)?;

        // Dungeon entry button
        let dungeon_y = dungeon_section_y + 8;
        let dungeon_height = 40u32;

        let dungeon_rect = Rectangle::new(Point::new(margin, dungeon_y), Size::new(220, dungeon_height));
        let dungeon_rounded = RoundedRectangle::new(dungeon_rect, CornerRadii::new(Size::new(8, 8)));

        dungeon_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(240, 200, 200))
            .build())
            .draw(display)?;
        dungeon_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(200, 120, 120))
            .stroke_width(2)
            .build())
            .draw(display)?;

        // Dungeon name
        let dungeon_name = if dungeon_id.len() > 12 { &dungeon_id[..12] } else { dungeon_id };
        let mut dungeon_text = heapless::String::<32>::new();
        write!(dungeon_text, "Donjon: {}", dungeon_name).ok();
        Text::new(&dungeon_text, Point::new(margin + 12, dungeon_y + 16), text_style).draw(display)?;

        // Record
        let record = self.dungeon_progress.get(dungeon_id).copied().unwrap_or(0);
        let mut record_text = heapless::String::<32>::new();
        if record > 0 {
            write!(record_text, "Record: Et.{}", record).ok();
        } else {
            write!(record_text, "Non explore").ok();
        }
        Text::new(&record_text, Point::new(margin + 12, dungeon_y + 30), dim_style).draw(display)?;

        self.touch_areas.push(TouchArea::new(
            margin, dungeon_y, 220, dungeon_height,
            TouchAreaAction::EnterDungeon(dungeon_id.to_string()),
        ));

        Ok(())
    }

    /// Draw Dungeon Selection page (GDD 3.3.8)
    fn draw_dungeon_select(
        &mut self,
        display: &mut St7789pDriver,
        zone_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let zones = &self.zones;
        use core::fmt::Write;

        self.touch_areas.clear();

        let margin = 10;
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let locked_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));

        // Find zone
        let zone = zones.iter().find(|z| z.id == zone_id);
        let dungeon_id = zone.map(|z| z.dungeon_id.as_str()).unwrap_or("Unknown");
        let dungeon_name = if dungeon_id.len() > 12 { &dungeon_id[..12] } else { dungeon_id };
        let record = self.dungeon_progress.get(dungeon_id).copied().unwrap_or(0);

        // Header
        let header_rect = Rectangle::new(Point::new(margin, 4), Size::new(220, 24));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(200, 120, 120))
            .build())
            .draw(display)?;

        let mut header = heapless::String::<32>::new();
        write!(header, "DONJON: {}", dungeon_name).ok();
        Text::new(&header, Point::new(margin + 8, 20), title_style).draw(display)?;

        // Record display
        let mut record_text = heapless::String::<32>::new();
        write!(record_text, "Record: Etage {}", record).ok();
        Text::new(&record_text, Point::new(margin, 38), dim_style).draw(display)?;

        // Checkpoint selection
        Text::new("Commencer depuis:", Point::new(margin, 54), text_style).draw(display)?;

        let checkpoints = [1, 10, 20, 30, 40, 50];
        let checkpoint_y_start = 62;
        let checkpoint_height = 26i32;
        let checkpoint_spacing = 4;

        for (idx, &checkpoint) in checkpoints.iter().enumerate() {
            let y = checkpoint_y_start + (idx as i32 * (checkpoint_height + checkpoint_spacing));
            let is_unlocked = checkpoint <= record || checkpoint == 1;

            let cp_rect = Rectangle::new(Point::new(margin, y), Size::new(220, checkpoint_height as u32));
            let cp_rounded = RoundedRectangle::new(cp_rect, CornerRadii::new(Size::new(6, 6)));

            let (bg_color, border_color) = if is_unlocked {
                if checkpoint == 1 {
                    (Rgb888::new(200, 230, 200), Rgb888::new(100, 180, 100)) // Selected/default
                } else {
                    (Rgb888::new(250, 250, 255), Rgb888::new(180, 185, 195))
                }
            } else {
                (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
            };

            cp_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(bg_color)
                .build())
                .draw(display)?;
            cp_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(border_color)
                .stroke_width(1)
                .build())
                .draw(display)?;

            let mut cp_text = heapless::String::<32>::new();
            if is_unlocked {
                write!(cp_text, "> Etage {}", checkpoint).ok();
                Text::new(&cp_text, Point::new(margin + 12, y + 17), text_style).draw(display)?;
            } else {
                write!(cp_text, "Etage {} [LOCKED]", checkpoint).ok();
                Text::new(&cp_text, Point::new(margin + 12, y + 17), locked_style).draw(display)?;
            }
        }

        // ENTER button at bottom
        let enter_y = 248;
        let enter_height = 30u32;

        let enter_rect = Rectangle::new(Point::new(margin, enter_y), Size::new(220, enter_height));
        let enter_rounded = RoundedRectangle::new(enter_rect, CornerRadii::new(Size::new(8, 8)));

        enter_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(240, 150, 150))
            .build())
            .draw(display)?;
        enter_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(200, 100, 100))
            .stroke_width(2)
            .build())
            .draw(display)?;

        Text::new("ENTRER", Point::new(95, enter_y + 20), title_style).draw(display)?;

        self.touch_areas.push(TouchArea::new(
            margin, enter_y, 220, enter_height,
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
        display: &mut St7789pDriver,
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
