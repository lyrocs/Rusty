//! Home Page (Accueil)
//!
//! Main dashboard showing expedition status, active team, and navigation.
//! Based on GDD section 3.3.1

use crate::assets::get_monster_icon;
use crate::display::{Sh8601Driver, StaticImage};
use crate::game::core::{Element, Monster, MonsterStatus};
use crate::game::systems::expedition::Expedition;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    text::Text,
};
use std::error::Error;

/// Actions from the home page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeAction {
    /// No action
    None,
    /// Navigate to Map/Expedition
    GoToMap,
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
        }
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
                return HomeAction::GoToMap;
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
        }
    }

    /// Draw progress bar
    fn draw_progress_bar(
        &self,
        display: &mut Sh8601Driver,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        progress: u8,
        is_complete: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Background
        let bg_rect = Rectangle::new(Point::new(x, y), Size::new(width, height));
        display.fill_solid(&bg_rect, Rgb888::new(40, 40, 50))?;

        // Fill
        let fill_width = ((width as f32 * progress as f32) / 100.0) as u32;
        if fill_width > 0 {
            let fill_color = if is_complete {
                Rgb888::new(100, 200, 100) // Green when complete
            } else {
                Rgb888::new(80, 150, 200) // Blue while in progress
            };
            let fill_rect = Rectangle::new(Point::new(x, y), Size::new(fill_width, height));
            display.fill_solid(&fill_rect, fill_color)?;
        }

        Ok(())
    }
}

impl Page for HomePage {
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.dirty {
            // Clear screen
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let gold_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 215, 0));

        // ═══════════════════════════════════════
        // HEADER - Time and Crystals
        // ═══════════════════════════════════════
        let header_rect = Rectangle::new(Point::new(0, 0), Size::new(368, 40));
        display.fill_solid(&header_rect, Rgb888::new(30, 35, 45))?;

        // Current time (simplified - just show static for now)
        Text::new("HOME", Point::new(15, 28), title_style).draw(display)?;

        // Crystals
        let crystal_text = format!("{} Crystals", self.crystals);
        Text::new(&crystal_text, Point::new(250, 28), gold_style).draw(display)?;

        // ═══════════════════════════════════════
        // EXPEDITIONS SECTION
        // ═══════════════════════════════════════
        Text::new("Expeditions:", Point::new(15, 70), text_style).draw(display)?;

        let slot_y_start = 85;
        let slot_height = 50u32;
        let slot_spacing = 60;

        for i in 0..2 {
            let y = slot_y_start + (i as i32 * slot_spacing);
            let slot_rect = Rectangle::new(Point::new(15, y), Size::new(338, slot_height));

            if let Some(ref exp_data) = self.expedition_slots[i] {
                // Active expedition
                let bg_color = if exp_data.is_complete {
                    Rgb888::new(40, 60, 40) // Green tint when complete
                } else {
                    Rgb888::new(35, 40, 50)
                };
                display.fill_solid(&slot_rect, bg_color)?;

                // Slot number and map name
                let slot_text = format!("{}. {}", i + 1, exp_data.map_name);
                Text::new(&slot_text, Point::new(25, y + 18), text_style).draw(display)?;

                // Progress bar
                self.draw_progress_bar(display, 25, y + 25, 200, 8, exp_data.progress_percent, exp_data.is_complete)?;

                // Time remaining
                let time_style = if exp_data.is_complete {
                    MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 100))
                } else {
                    dim_style
                };
                Text::new(&exp_data.time_remaining, Point::new(240, y + 33), time_style).draw(display)?;
            } else {
                // Empty slot
                display.fill_solid(&slot_rect, Rgb888::new(30, 32, 38))?;
                let slot_text = format!("{}. Available", i + 1);
                Text::new(&slot_text, Point::new(25, y + 30), dim_style).draw(display)?;
            }

            self.expedition_areas[i] = Some(slot_rect);
        }

        // ═══════════════════════════════════════
        // ACTIVE TEAM SECTION
        // ═══════════════════════════════════════
        Text::new("Active Team:", Point::new(15, 220), text_style).draw(display)?;

        self.team_areas.clear();
        let team_y = 240;
        let monster_width = 100u32;
        let monster_height = 80u32;
        let monster_spacing = 110;

        for (i, monster_data) in self.team_monsters.iter().take(3).enumerate() {
            let x = 20 + (i as i32 * monster_spacing);
            let monster_rect = Rectangle::new(Point::new(x, team_y), Size::new(monster_width, monster_height));
            display.fill_solid(&monster_rect, Rgb888::new(35, 40, 50))?;

            // Load and display monster icon from embedded assets
            if let Some(icon_data) = get_monster_icon(&monster_data.species_id) {
                if let Ok(icon) = StaticImage::new(icon_data) {
                    // Center the icon in the card area (icon area: 40x40 centered)
                    let icon_x = x + 30;
                    let icon_y = team_y + 5;
                    let _ = icon.render(display, (icon_x, icon_y));
                }
            } else {
                // Fallback to element square if icon not found
                let elem_color = Self::element_color(&monster_data.element);
                let elem_rect = Rectangle::new(Point::new(x + 30, team_y + 10), Size::new(40, 30));
                display.fill_solid(&elem_rect, elem_color)?;
                let elem_char = Self::element_char(&monster_data.element);
                let elem_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::BLACK);
                Text::new(&elem_char.to_string(), Point::new(x + 43, team_y + 32), elem_text_style).draw(display)?;
            }

            // Level
            let level_text = format!("Lv.{}", monster_data.level);
            Text::new(&level_text, Point::new(x + 30, team_y + 55), text_style).draw(display)?;

            // Name (truncated)
            let name = if monster_data.name.len() > 12 {
                &monster_data.name[..12]
            } else {
                &monster_data.name
            };
            Text::new(name, Point::new(x + 5, team_y + 72), dim_style).draw(display)?;

            self.team_areas.push(monster_rect);
        }

        // Empty slots
        for i in self.team_monsters.len()..3 {
            let x = 20 + (i as i32 * monster_spacing);
            let monster_rect = Rectangle::new(Point::new(x, team_y), Size::new(monster_width, monster_height));
            display.fill_solid(&monster_rect, Rgb888::new(30, 32, 38))?;
            Text::new("Empty", Point::new(x + 30, team_y + 45), dim_style).draw(display)?;
            self.team_areas.push(monster_rect);
        }

        // ═══════════════════════════════════════
        // NAVIGATION BUTTONS (2 buttons side by side)
        // ═══════════════════════════════════════
        let nav_y = 370;
        let button_width = 160u32;
        let button_height = 55u32;
        let col_spacing = 175;

        // MAP button
        let map_rect = Rectangle::new(Point::new(15, nav_y), Size::new(button_width, button_height));
        display.fill_solid(&map_rect, Rgb888::new(60, 80, 100))?;
        Rectangle::new(Point::new(15, nav_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 2))
            .draw(display)?;
        Text::new("MAP", Point::new(75, nav_y + 35), text_style).draw(display)?;
        self.map_button = Some(map_rect);

        // COLLECTION button
        let coll_rect = Rectangle::new(Point::new(15 + col_spacing, nav_y), Size::new(button_width, button_height));
        display.fill_solid(&coll_rect, Rgb888::new(60, 100, 80))?;
        Rectangle::new(Point::new(15 + col_spacing, nav_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 2))
            .draw(display)?;
        Text::new("COLLECTION", Point::new(15 + col_spacing + 25, nav_y + 35), text_style).draw(display)?;
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
