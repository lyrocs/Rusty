//! Home Page (Accueil)
//!
//! Main dashboard showing expedition status, active team, and navigation.
//! Based on GDD section 3.3.1

use crate::assets::{get_monster_sprite_path, SpriteAction, SpriteCache};
use crate::display::{St7789pDriver, GifPlayer, DynamicGifMeta, SharedCanvas};
use crate::ecs::resources::SdCardWrapper;
use crate::game::core::{Element, Monster, MonsterStatus};
use crate::game::systems::expedition::Expedition;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13, FONT_9X15}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Actions from the home page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeAction {
    /// No action
    None,
    /// Navigate to Dungeon List (Battle)
    GoToDungeonList,
    /// Navigate to Collection (main entry to monsters/regions)
    GoToCollection,
    /// View expedition details (slot index)
    ViewExpedition(usize),
    /// Start new expedition (go to expedition map)
    StartExpedition,
    /// View monster detail (index in team)
    ViewMonster(usize),
}

/// Data for displaying an expedition slot
#[derive(Clone)]
pub struct ExpeditionSlotData {
    pub is_active: bool,
    pub map_name: String,
    pub progress_percent: u8,
    pub time_remaining: String,
    pub is_complete: bool,
}

/// Data for displaying a team monster
#[derive(Clone)]
pub struct TeamMonsterData {
    pub species_id: String,
    pub element: Element,
    pub level: u8,
    pub name: String,
}

/// Home Page
pub struct HomePage {
    // Display data
    crystals: u32,
    expedition_slots: [Option<ExpeditionSlotData>; 2],
    team_monsters: Vec<TeamMonsterData>,

    // Touch areas
    expedition_areas: [Option<Rectangle>; 2],
    team_areas: Vec<Rectangle>,
    map_button: Option<Rectangle>,
    collection_button: Option<Rectangle>,

    // State
    dirty: bool,

    // Icon GIFs loaded from SD card
    monster_icons: Vec<Option<DynamicGifMeta>>,
    // Shared canvas for rendering icons
    shared_canvas: SharedCanvas,
    // Track if icons need reload
    icons_need_reload: bool,
    // Track team species for icon reload detection
    loaded_species: Vec<String>,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            crystals: 0,
            expedition_slots: [None, None],
            team_monsters: Vec::new(),
            expedition_areas: [None, None],
            team_areas: Vec::new(),
            map_button: None,
            collection_button: None,
            dirty: true,
            monster_icons: Vec::new(),
            shared_canvas: SharedCanvas::new(32, 32), // Icons are 32x32
            icons_need_reload: true,
            loaded_species: Vec::new(),
        }
    }

    /// Check if icons need to be reloaded (team changed)
    pub fn needs_icon_reload(&self) -> bool {
        if self.icons_need_reload {
            return true;
        }
        // Check if team species have changed
        let current_species: Vec<&str> = self.team_monsters.iter()
            .map(|m| m.species_id.as_str())
            .collect();
        let loaded: Vec<&str> = self.loaded_species.iter()
            .map(|s| s.as_str())
            .collect();
        current_species != loaded
    }

    /// Load monster icons from SD card
    pub fn load_icons(&mut self, sd_card: &mut SdCardWrapper) {
        self.monster_icons.clear();
        self.loaded_species.clear();
        for monster_data in &self.team_monsters {
            let path = get_monster_sprite_path(&monster_data.species_id, SpriteAction::Icon);
            let icon = match sd_card.load_binary_file(&path) {
                Ok(data) => {
                    log::info!("Loaded icon: {} ({} bytes)", path, data.len());
                    DynamicGifMeta::new(data).ok()
                }
                Err(e) => {
                    log::warn!("Failed to load icon {}: {:?}", path, e);
                    None
                }
            };
            self.monster_icons.push(icon);
            self.loaded_species.push(monster_data.species_id.clone());
        }
        self.icons_need_reload = false;
    }

    /// Update home page data from game state
    pub fn update_data(
        &mut self,
        crystals: u32,
        expeditions: &[Option<Expedition>; 2],
        team_monsters: &[&Monster],
        current_time: u64,
        get_map_name: impl Fn(&str) -> String,
    ) {
        self.crystals = crystals;

        // Update expedition slots
        for (i, exp_opt) in expeditions.iter().enumerate() {
            self.expedition_slots[i] = exp_opt.as_ref().map(|exp| {
                let total_seconds = exp.duration.seconds() as u64;
                let elapsed = current_time.saturating_sub(exp.started_at);
                let remaining = total_seconds.saturating_sub(elapsed);
                let progress = ((elapsed as f32 / total_seconds as f32) * 100.0).min(100.0) as u8;
                let is_complete = remaining == 0;

                let time_remaining = if is_complete {
                    "Complete!".to_string()
                } else {
                    let mins = remaining / 60;
                    let secs = remaining % 60;
                    if mins > 60 {
                        format!("{}h{}m", mins / 60, mins % 60)
                    } else {
                        format!("{}m{}s", mins, secs)
                    }
                };

                ExpeditionSlotData {
                    is_active: true,
                    map_name: get_map_name(&exp.map_id),
                    progress_percent: progress,
                    time_remaining,
                    is_complete,
                }
            });
        }

        // Update team monsters
        self.team_monsters = team_monsters.iter().map(|m| TeamMonsterData {
            species_id: m.species_id.clone(),
            element: m.element,
            level: m.level,
            name: m.name.clone(),
        }).collect();

        self.dirty = true;
    }

    /// Handle touch and return action
    pub fn handle_touch(&mut self, x: i32, y: i32) -> HomeAction {
        // Check expedition areas
        for (i, area_opt) in self.expedition_areas.iter().enumerate() {
            if let Some(ref rect) = area_opt {
                if Self::rect_contains(rect, x, y) {
                    if self.expedition_slots[i].is_some() {
                        return HomeAction::ViewExpedition(i);
                    } else {
                        return HomeAction::StartExpedition;
                    }
                }
            }
        }

        // Check team monster areas
        for (i, rect) in self.team_areas.iter().enumerate() {
            if Self::rect_contains(rect, x, y) {
                return HomeAction::ViewMonster(i);
            }
        }

        // Check navigation buttons
        if let Some(ref rect) = self.map_button {
            if Self::rect_contains(rect, x, y) {
                return HomeAction::GoToDungeonList;
            }
        }

        if let Some(ref rect) = self.collection_button {
            if Self::rect_contains(rect, x, y) {
                return HomeAction::GoToCollection;
            }
        }

        HomeAction::None
    }

    fn rect_contains(rect: &Rectangle, x: i32, y: i32) -> bool {
        x >= rect.top_left.x
            && x < rect.top_left.x + rect.size.width as i32
            && y >= rect.top_left.y
            && y < rect.top_left.y + rect.size.height as i32
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
            Element::Neutral => 'N',
        }
    }

    fn element_color(element: &Element) -> Rgb888 {
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

    /// Draw progress bar
    fn draw_progress_bar(
        &self,
        display: &mut St7789pDriver,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        progress: u8,
        is_complete: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Background - light gray for light theme
        let bg_rect = Rectangle::new(Point::new(x, y), Size::new(width, height));
        display.fill_solid(&bg_rect, Rgb888::new(180, 180, 190))?;

        // Fill
        let fill_width = ((width as f32 * progress as f32) / 100.0) as u32;
        if fill_width > 0 {
            let fill_color = if is_complete {
                Rgb888::new(80, 180, 80) // Green when complete
            } else {
                Rgb888::new(60, 120, 200) // Blue while in progress
            };
            let fill_rect = Rectangle::new(Point::new(x, y), Size::new(fill_width, height));
            display.fill_solid(&fill_rect, fill_color)?;
        }

        Ok(())
    }
}

impl Page for HomePage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.dirty {
            // Clear screen - Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let gold_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 150, 0));

        // ═══════════════════════════════════════
        // HEADER - Time and Crystals
        // ═══════════════════════════════════════
        // Add margin: start at y=2 instead of 0, reduced height to fit content better
        let header_rect = Rectangle::new(Point::new(2, 2), Size::new(236, 24));
        display.fill_solid(&header_rect, Rgb888::new(200, 210, 220))?;

        // Current time (simplified - just show static for now)
        // Move text away from left edge: x=10 instead of 8, y=20 for better vertical centering
        Text::new("HOME", Point::new(10, 20), title_style).draw(display)?;

        // Crystals - move away from right edge to prevent cutoff
        let crystal_text = format!("{} Cr", self.crystals);
        Text::new(&crystal_text, Point::new(170, 20), gold_style).draw(display)?;

        // ═══════════════════════════════════════
        // EXPEDITIONS SECTION
        // ═══════════════════════════════════════
        Text::new("Expeditions:", Point::new(10, 42), text_style).draw(display)?;

        let slot_y_start = 50;
        let slot_height = 35u32;
        let slot_spacing = 40;

        for i in 0..2 {
            let y = slot_y_start + (i as i32 * slot_spacing);
            let slot_rect = Rectangle::new(Point::new(10, y), Size::new(220, slot_height));

            if let Some(ref exp_data) = self.expedition_slots[i] {
                // Active expedition
                let bg_color = if exp_data.is_complete {
                    Rgb888::new(200, 240, 200) // Light green tint when complete
                } else {
                    Rgb888::new(220, 225, 235)
                };
                display.fill_solid(&slot_rect, bg_color)?;

                // Slot number and map name (truncate if needed)
                let map_name = if exp_data.map_name.len() > 15 {
                    &exp_data.map_name[..15]
                } else {
                    &exp_data.map_name
                };
                let slot_text = format!("{}. {}", i + 1, map_name);
                Text::new(&slot_text, Point::new(14, y + 12), text_style).draw(display)?;

                // Progress bar
                self.draw_progress_bar(display, 14, y + 20, 140, 6, exp_data.progress_percent, exp_data.is_complete)?;

                // Time remaining
                let time_style = if exp_data.is_complete {
                    MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 150, 50))
                } else {
                    dim_style
                };
                Text::new(&exp_data.time_remaining, Point::new(160, y + 25), time_style).draw(display)?;
            } else {
                // Empty slot
                display.fill_solid(&slot_rect, Rgb888::new(210, 210, 215))?;
                let slot_text = format!("{}. Available", i + 1);
                Text::new(&slot_text, Point::new(14, y + 20), dim_style).draw(display)?;
            }

            self.expedition_areas[i] = Some(slot_rect);
        }

        // ═══════════════════════════════════════
        // ACTIVE TEAM SECTION
        // ═══════════════════════════════════════
        Text::new("Active Team:", Point::new(10, 144), text_style).draw(display)?;

        self.team_areas.clear();
        let team_y = 154;
        let monster_width = 68u32;
        let monster_height = 60u32;
        let monster_spacing = 76;

        // Pre-calculate icon positions and check which ones to render
        let team_count = self.team_monsters.len().min(3);
        for i in 0..team_count {
            let monster_data = &self.team_monsters[i];
            let x = 14 + (i as i32 * monster_spacing);
            let monster_rect = Rectangle::new(Point::new(x, team_y), Size::new(monster_width, monster_height));
            display.fill_solid(&monster_rect, Rgb888::new(220, 225, 235))?;

            // Display monster icon from SD card
            let has_icon = i < self.monster_icons.len() && self.monster_icons[i].is_some();
            let icon_rendered = if has_icon {
                let gif = self.monster_icons[i].as_ref().unwrap();
                // Center the icon in the card area
                let icon_x = x + 34; // Center X for 32x32 icon in 68 width card
                let icon_y = team_y + 22; // Center Y accounting for icon height
                gif.render_frame(display, 0, &mut self.shared_canvas, Some((icon_x, icon_y)), false, true).is_ok()
            } else {
                false
            };

            if !icon_rendered {
                // Fallback to element square if icon not found
                let elem_color = Self::element_color(&monster_data.element);
                let elem_rect = Rectangle::new(Point::new(x + 19, team_y + 8), Size::new(30, 24));
                display.fill_solid(&elem_rect, elem_color)?;
                let elem_char = Self::element_char(&monster_data.element);
                let elem_text_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
                Text::new(&elem_char.to_string(), Point::new(x + 30, team_y + 24), elem_text_style).draw(display)?;
            }

            // Level
            let level_text = format!("Lv.{}", monster_data.level);
            Text::new(&level_text, Point::new(x + 22, team_y + 42), text_style).draw(display)?;

            // Name (truncated)
            let name = if monster_data.name.len() > 10 {
                &monster_data.name[..10]
            } else {
                &monster_data.name
            };
            Text::new(name, Point::new(x + 4, team_y + 54), dim_style).draw(display)?;

            self.team_areas.push(monster_rect);
        }

        // Empty slots
        for i in self.team_monsters.len()..3 {
            let x = 14 + (i as i32 * monster_spacing);
            let monster_rect = Rectangle::new(Point::new(x, team_y), Size::new(monster_width, monster_height));
            display.fill_solid(&monster_rect, Rgb888::new(210, 210, 215))?;
            Text::new("Empty", Point::new(x + 20, team_y + 33), dim_style).draw(display)?;
            self.team_areas.push(monster_rect);
        }

        // ═══════════════════════════════════════
        // NAVIGATION BUTTONS (2 buttons side by side with rounded corners)
        // ═══════════════════════════════════════
        let nav_y = 218;
        let button_width = 112u32;
        let button_height = 62u32;  // Large touch zone
        let button_spacing = 8;      // Gap between buttons
        let corner_radius = 8u32;

        // BATTLE button - Red/Orange themed with rounded corners
        let map_rect = Rectangle::new(Point::new(5, nav_y), Size::new(button_width, button_height));
        let map_rounded = RoundedRectangle::new(
            map_rect,
            CornerRadii::new(Size::new(corner_radius, corner_radius))
        );

        // Draw filled rounded button
        map_rounded
            .into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(220, 100, 80))
                .build())
            .draw(display)?;

        // Draw rounded border
        map_rounded
            .into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(180, 60, 50))
                .stroke_width(2)
                .build())
            .draw(display)?;

        Text::new("BATTLE", Point::new(28, nav_y + 36), text_style).draw(display)?;
        self.map_button = Some(map_rect);

        // COLLECTION button - Green themed with rounded corners
        let coll_x = 5 + button_width as i32 + button_spacing;
        let coll_rect = Rectangle::new(Point::new(coll_x, nav_y), Size::new(button_width, button_height));
        let coll_rounded = RoundedRectangle::new(
            coll_rect,
            CornerRadii::new(Size::new(corner_radius, corner_radius))
        );

        // Draw filled rounded button
        coll_rounded
            .into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(100, 200, 150))
                .build())
            .draw(display)?;

        // Draw rounded border
        coll_rounded
            .into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(60, 150, 100))
                .stroke_width(2)
                .build())
            .draw(display)?;

        Text::new("COLLECT", Point::new(coll_x + 22, nav_y + 36), text_style).draw(display)?;
        self.collection_button = Some(coll_rect);

        display.flush()?;
        self.dirty = false;
        Ok(())
    }

    fn update(&mut self) -> bool {
        true // Always active
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
