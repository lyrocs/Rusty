//! Collection Page
//!
//! Shows collection progress - which species have been captured, organized by zone.

use crate::display::St7789pDriver;
use crate::game::core::Element;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
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
        const SCROLL_AMOUNT: i32 = 130; // ~2 items * 65 pixels each

        if is_up {
            // Swipe up = scroll down (show more content below)
            let max_scroll = (self.zones.len() as i32 * 65).saturating_sub(180);
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
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));

        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        // Header with rounded background
        let header_rect = Rectangle::new(Point::new(10, 4), Size::new(220, 24));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 150, 200))
            .build())
            .draw(display)?;

        Text::new("COLLECTION", Point::new(75, 20), title_style).draw(display)?;

        // Progress
        let progress_text = format!("{}/{}", self.total_captured, self.total_species);
        Text::new(&progress_text, Point::new(180, 20), text_style).draw(display)?;

        // Zones (scrollable area)
        let content_y_start = 32;
        let content_height = 240;

        // Clear content area for scrolling
        let content_bg = Rectangle::new(
            Point::new(0, content_y_start),
            Size::new(240, content_height as u32)
        );
        display.fill_solid(&content_bg, Rgb888::new(240, 240, 245))?;

        let mut y_pos = content_y_start - self.scroll_offset;
        let zone_height = 62;

        // Clear zone touch areas
        self.zone_areas.clear();

        for zone in &self.zones {
            // Skip if above visible area
            if y_pos + zone_height < content_y_start {
                y_pos += zone_height + 3;
                continue;
            }

            // Stop if below visible area
            if y_pos > content_y_start + content_height {
                break;
            }

            // Zone card
            let zone_y = y_pos;
            if zone_y >= content_y_start - 20 && zone_y < content_y_start + content_height {
                let zone_rect = Rectangle::new(Point::new(10, zone_y), Size::new(220, zone_height as u32));
                let zone_rounded = RoundedRectangle::new(zone_rect, CornerRadii::new(Size::new(8, 8)));

                let (bg_color, border_color) = if zone.is_unlocked {
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

                // Register touch area for this zone (only if unlocked)
                if zone.is_unlocked {
                    self.zone_areas.push(ZoneTouchArea {
                        rect: zone_rect,
                        zone_id: zone.zone_id.clone(),
                    });
                }

                // Zone name and progress
                let captured_in_zone = zone.species.iter().filter(|s| s.is_captured).count();
                let total_in_zone = zone.species.len();

                // Truncate zone name if needed
                let zone_name = if zone.zone_name.len() > 16 {
                    &zone.zone_name[..16]
                } else {
                    &zone.zone_name
                };

                let zone_header = if zone.is_unlocked {
                    format!("{} {}/{}", zone_name, captured_in_zone, total_in_zone)
                } else {
                    format!("{} [LOCKED]", zone_name)
                };

                let header_style = if zone.is_unlocked { text_style } else { dim_style };
                Text::new(&zone_header, Point::new(18, zone_y + 14), header_style).draw(display)?;

                // Species icons (only if unlocked)
                if zone.is_unlocked {
                    let icons_y = zone_y + 22;
                    let mut icon_x = 18;
                    let icon_spacing = 26;

                    for (_i, species) in zone.species.iter().take(7).enumerate() {
                        if icon_x > 200 {
                            break;
                        }

                        let icon_rect = Rectangle::new(
                            Point::new(icon_x, icons_y),
                            Size::new(22, 18)
                        );
                        let icon_rounded = RoundedRectangle::new(icon_rect, CornerRadii::new(Size::new(4, 4)));

                        if species.is_captured {
                            // Show element icon for captured species
                            let elem_char = Self::element_char(species.element);
                            let elem_color = Self::element_color(species.element);
                            let style = MonoTextStyle::new(&FONT_6X10, elem_color);

                            // Light background for icon
                            icon_rounded.into_styled(PrimitiveStyleBuilder::new()
                                .fill_color(Rgb888::new(230, 235, 245))
                                .build())
                                .draw(display)?;

                            Text::new(&format!("{}", elem_char), Point::new(icon_x + 7, icons_y + 13), style).draw(display)?;
                        } else {
                            // Show ? for uncaptured
                            icon_rounded.into_styled(PrimitiveStyleBuilder::new()
                                .fill_color(Rgb888::new(210, 215, 220))
                                .build())
                                .draw(display)?;
                            Text::new("?", Point::new(icon_x + 8, icons_y + 13), dim_style).draw(display)?;
                        }

                        icon_x += icon_spacing;
                    }

                    // Show +N if more than 7 species
                    if zone.species.len() > 7 {
                        let more = format!("+{}", zone.species.len() - 7);
                        Text::new(&more, Point::new(icon_x, icons_y + 13), dim_style).draw(display)?;
                    }
                }

                // Progress bar at bottom of card
                if zone.is_unlocked && total_in_zone > 0 {
                    let bar_y = zone_y + 46;
                    let bar_width = 200u32;
                    let filled_width = ((captured_in_zone as u32 * bar_width) / total_in_zone as u32) as u32;

                    // Background bar
                    let bar_bg = Rectangle::new(Point::new(18, bar_y), Size::new(bar_width, 6));
                    display.fill_solid(&bar_bg, Rgb888::new(200, 205, 215))?;

                    // Filled bar
                    if filled_width > 0 {
                        let bar_fill = Rectangle::new(Point::new(18, bar_y), Size::new(filled_width, 6));
                        display.fill_solid(&bar_fill, Rgb888::new(100, 180, 100))?;
                    }
                }
            }

            y_pos += zone_height + 3;
        }

        // Scroll hint at bottom (no back button needed)
        if self.zones.len() > 3 {
            Text::new("swipe to scroll", Point::new(75, 278), dim_style).draw(display)?;
        }

        self.back_area = None;

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
