//! Collection Page
//!
//! Shows collection progress - which species have been captured, organized by zone.

use crate::display::Sh8601Driver;
use crate::game::core::Element;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    text::Text,
};
use std::collections::HashSet;
use std::error::Error;

/// Action from collection page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionAction {
    /// No action
    None,
    /// Go back
    Back,
    /// Selected a zone (zone_id)
    SelectZone(String),
}

/// Zone collection data for display
#[derive(Clone)]
pub struct ZoneCollectionData {
    pub zone_id: String,
    pub zone_name: String,
    pub is_unlocked: bool,
    pub species: Vec<SpeciesCollectionData>,
    /// Min level for sorting
    pub level_min: u8,
}

/// Species collection data for display
#[derive(Clone)]
pub struct SpeciesCollectionData {
    pub species_id: String,
    pub name: String,
    pub element: Element,
    pub is_captured: bool,
}

/// Zone touch area
struct ZoneTouchArea {
    rect: Rectangle,
    zone_id: String,
}

/// Collection page
pub struct CollectionPage {
    total_captured: usize,
    total_species: usize,
    zones: Vec<ZoneCollectionData>,
    scroll_offset: i32,

    // Touch areas
    back_area: Option<Rectangle>,
    zone_areas: Vec<ZoneTouchArea>,

    dirty: bool,
}

impl CollectionPage {
    pub fn new(
        zones: Vec<ZoneCollectionData>,
        captured_species: &HashSet<String>,
    ) -> Self {
        let mut total_captured = 0;
        let mut total_species = 0;

        let zones: Vec<ZoneCollectionData> = zones.into_iter().map(|mut zone| {
            for species in &mut zone.species {
                species.is_captured = captured_species.contains(&species.species_id);
                if species.is_captured {
                    total_captured += 1;
                }
                total_species += 1;
            }
            zone
        }).collect();

        Self {
            total_captured,
            total_species,
            zones,
            scroll_offset: 0,
            back_area: None,
            zone_areas: Vec::new(),
            dirty: true,
        }
    }

    /// Handle touch input (back button and zone selection)
    pub fn handle_touch(&self, x: i32, y: i32) -> CollectionAction {
        let point = Point::new(x, y);

        if let Some(rect) = self.back_area {
            if rect.contains(point) {
                return CollectionAction::Back;
            }
        }

        // Check zone touch areas
        for zone_area in &self.zone_areas {
            if zone_area.rect.contains(point) {
                return CollectionAction::SelectZone(zone_area.zone_id.clone());
            }
        }

        CollectionAction::None
    }

    /// Handle swipe for scrolling (2 items per swipe)
    pub fn handle_swipe(&mut self, is_up: bool) {
        const SCROLL_AMOUNT: i32 = 200; // 2 items * 100 pixels each

        if is_up {
            // Swipe up = scroll down (show more content below)
            let max_scroll = (self.zones.len() as i32 * 100).saturating_sub(300);
            if self.scroll_offset < max_scroll {
                self.scroll_offset = (self.scroll_offset + SCROLL_AMOUNT).min(max_scroll);
                self.dirty = true;
            }
        } else {
            // Swipe down = scroll up (show more content above)
            if self.scroll_offset > 0 {
                self.scroll_offset = (self.scroll_offset - SCROLL_AMOUNT).max(0);
                self.dirty = true;
            }
        }
    }

    fn element_char(element: Element) -> char {
        match element {
            Element::Fire => 'F',
            Element::Water => 'W',
            Element::Earth => 'E',
            Element::Wind => 'N',
            Element::Thunder => 'T',
            Element::Shadow => 'S',
            Element::Holy => 'H',
            Element::Ghost => 'G',
        }
    }

    fn element_color(element: Element) -> Rgb888 {
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(150, 120, 50),
            Element::Wind => Rgb888::new(100, 220, 150),
            Element::Thunder => Rgb888::new(255, 255, 100),
            Element::Shadow => Rgb888::new(150, 50, 200),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(180, 180, 220),
        }
    }
}

impl Page for CollectionPage {
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let green_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 100));

        if full_redraw {
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        // Header
        Text::new("COLLECTION", Point::new(120, 35), title_style).draw(display)?;

        // Progress
        let progress_text = format!("{}/{}", self.total_captured, self.total_species);
        Text::new(&progress_text, Point::new(280, 35), text_style).draw(display)?;

        // Zones (scrollable area)
        let content_y_start = 60;
        let content_height = 340;

        // Clear content area for scrolling
        let content_bg = Rectangle::new(
            Point::new(0, content_y_start),
            Size::new(368, content_height as u32)
        );
        display.fill_solid(&content_bg, Rgb888::new(20, 25, 35))?;

        let mut y_pos = content_y_start - self.scroll_offset;

        // Clear zone touch areas
        self.zone_areas.clear();

        for zone in &self.zones {
            // Skip if above visible area
            if y_pos + 100 < content_y_start {
                y_pos += 100;
                continue;
            }

            // Stop if below visible area
            if y_pos > content_y_start + content_height {
                break;
            }

            // Zone header
            let zone_y = y_pos;
            if zone_y >= content_y_start && zone_y < content_y_start + content_height {
                // Zone background
                let zone_bg = Rectangle::new(Point::new(15, zone_y), Size::new(338, 90));
                let bg_color = if zone.is_unlocked {
                    Rgb888::new(30, 35, 45)
                } else {
                    Rgb888::new(25, 25, 30)
                };
                display.fill_solid(&zone_bg, bg_color)?;

                // Register touch area for this zone (only if unlocked)
                if zone.is_unlocked {
                    self.zone_areas.push(ZoneTouchArea {
                        rect: zone_bg,
                        zone_id: zone.zone_id.clone(),
                    });
                }

                // Zone name and progress
                let captured_in_zone = zone.species.iter().filter(|s| s.is_captured).count();
                let total_in_zone = zone.species.len();

                let zone_header = if zone.is_unlocked {
                    format!("{} {}/{}", zone.zone_name, captured_in_zone, total_in_zone)
                } else {
                    format!("{} [LOCKED]", zone.zone_name)
                };

                let header_style = if zone.is_unlocked { text_style } else { dim_style };
                Text::new(&zone_header, Point::new(25, zone_y + 20), header_style).draw(display)?;

                // Species icons (only if unlocked)
                if zone.is_unlocked {
                    let icons_y = zone_y + 40;
                    let mut icon_x = 25;
                    let icon_spacing = 38;

                    for (i, species) in zone.species.iter().take(8).enumerate() {
                        if icon_x > 330 {
                            break;
                        }

                        if species.is_captured {
                            // Show element icon for captured species
                            let elem_char = Self::element_char(species.element);
                            let elem_color = Self::element_color(species.element);
                            let style = MonoTextStyle::new(&FONT_9X15, elem_color);

                            // Background for icon
                            let icon_bg = Rectangle::new(
                                Point::new(icon_x, icons_y),
                                Size::new(32, 25)
                            );
                            display.fill_solid(&icon_bg, Rgb888::new(40, 45, 55))?;

                            Text::new(&format!("{}", elem_char), Point::new(icon_x + 10, icons_y + 18), style).draw(display)?;
                        } else {
                            // Show ? for uncaptured
                            let icon_bg = Rectangle::new(
                                Point::new(icon_x, icons_y),
                                Size::new(32, 25)
                            );
                            display.fill_solid(&icon_bg, Rgb888::new(30, 30, 35))?;
                            Text::new("?", Point::new(icon_x + 12, icons_y + 18), dim_style).draw(display)?;
                        }

                        icon_x += icon_spacing;
                    }

                    // Show +N if more than 8 species
                    if zone.species.len() > 8 {
                        let more = format!("+{}", zone.species.len() - 8);
                        Text::new(&more, Point::new(icon_x, icons_y + 18), dim_style).draw(display)?;
                    }
                }
            }

            y_pos += 100;
        }

        // Back button
        let back_rect = Rectangle::new(Point::new(15, 410), Size::new(80, 30));
        display.fill_solid(&back_rect, Rgb888::new(80, 60, 60))?;
        Text::new("< BACK", Point::new(25, 430), text_style).draw(display)?;
        self.back_area = Some(back_rect);

        // Scroll hint
        if self.zones.len() > 3 {
            Text::new("swipe to scroll", Point::new(230, 430), dim_style).draw(display)?;
        }

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
