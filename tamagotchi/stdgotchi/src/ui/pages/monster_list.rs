//! Monster List Page
//!
//! Displays monsters - either player's owned monsters or all species from a zone.
//! Unowned species are shown as disabled/grayed out.

use crate::assets::get_monster_icon;
use crate::display::{St7789pDriver, StaticImage};
use crate::game::core::{Element, Monster, MonsterStatus};
use crate::game::systems::progression::fusion::format_fusion;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action from monster list
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterListAction {
    /// No action
    None,
    /// Selected an owned monster at index in player's monster list
    Select(usize),
    /// Go back
    Back,
}

/// Touch area for monster selection
struct MonsterTouchArea {
    rect: Rectangle,
    /// Index in player's monsters Vec (None if unowned)
    monster_index: Option<usize>,
}

/// Display data for a monster/species entry
struct MonsterDisplay {
    name: String,
    species_id: String,
    level: u8,
    element: Element,
    fusion: String,
    hp_percent: f32,
    xp_percent: f32,
    power: u16,
    status: MonsterStatus,
    is_in_team: bool,
    /// True if player owns this monster
    is_owned: bool,
    /// Index in player's monsters Vec (if owned)
    monster_index: Option<usize>,
}

/// Monster List page
pub struct MonsterListPage {
    /// Header title (zone name or "MONSTERS")
    title: String,
    monsters: Vec<MonsterDisplay>,
    touch_areas: Vec<MonsterTouchArea>,
    back_area: Option<Rectangle>,
    dirty: bool,
    scroll_offset: i32,
    total_items: usize,
}

impl MonsterListPage {
    /// Create monster list showing player's owned monsters (legacy behavior)
    pub fn new(monsters: &[Monster], team_ids: &[String]) -> Self {
        let displays: Vec<MonsterDisplay> = monsters.iter().enumerate().map(|(i, m)| {
            let is_in_team = team_ids.iter().any(|id| id == &m.id);
            MonsterDisplay {
                name: m.name.clone(),
                species_id: m.species_id.clone(),
                level: m.level,
                element: m.element,
                fusion: format_fusion(m.fusion_count),
                hp_percent: m.hp_percentage(),
                xp_percent: m.xp_percentage(),
                power: m.power(),
                status: m.status,
                is_in_team,
                is_owned: true,
                monster_index: Some(i),
            }
        }).collect();

        let total = displays.len();
        Self {
            title: "MONSTERS".to_string(),
            monsters: displays,
            touch_areas: Vec::new(),
            back_area: None,
            dirty: true,
            scroll_offset: 0,
            total_items: total,
        }
    }

    /// Create monster list filtered by zone, showing owned and unowned species
    pub fn from_zone(
        zone_name: &str,
        zone_species: &[(String, String, Element)], // (species_id, name, element)
        player_monsters: &[Monster],
        team_ids: &[String],
    ) -> Self {
        let mut displays: Vec<MonsterDisplay> = Vec::new();

        for (species_id, species_name, element) in zone_species {
            // Check if player owns this species
            let owned_monster = player_monsters.iter().enumerate()
                .find(|(_, m)| &m.species_id == species_id);

            if let Some((idx, monster)) = owned_monster {
                // Player owns this species - show full details
                let is_in_team = team_ids.iter().any(|id| id == &monster.id);
                displays.push(MonsterDisplay {
                    name: monster.name.clone(),
                    species_id: species_id.clone(),
                    level: monster.level,
                    element: *element,
                    fusion: format_fusion(monster.fusion_count),
                    hp_percent: monster.hp_percentage(),
                    xp_percent: monster.xp_percentage(),
                    power: monster.power(),
                    status: monster.status,
                    is_in_team,
                    is_owned: true,
                    monster_index: Some(idx),
                });
            } else {
                // Player doesn't own this species - show as disabled
                displays.push(MonsterDisplay {
                    name: species_name.clone(),
                    species_id: species_id.clone(),
                    level: 0,
                    element: *element,
                    fusion: String::new(),
                    hp_percent: 0.0,
                    xp_percent: 0.0,
                    power: 0,
                    status: MonsterStatus::Available,
                    is_in_team: false,
                    is_owned: false,
                    monster_index: None,
                });
            }
        }

        // Sort: owned+team first, then owned, then unowned
        displays.sort_by(|a, b| {
            let priority_a = if a.is_owned && a.is_in_team { 0 } else if a.is_owned { 1 } else { 2 };
            let priority_b = if b.is_owned && b.is_in_team { 0 } else if b.is_owned { 1 } else { 2 };
            priority_a.cmp(&priority_b)
        });

        let total = displays.len();
        Self {
            title: zone_name.to_string(),
            monsters: displays,
            touch_areas: Vec::new(),
            back_area: None,
            dirty: true,
            scroll_offset: 0,
            total_items: total,
        }
    }

    /// Handle touch and return action
    pub fn handle_touch(&self, x: i32, y: i32) -> MonsterListAction {
        // Check back button
        if let Some(ref back_rect) = self.back_area {
            if x >= back_rect.top_left.x && x < back_rect.top_left.x + back_rect.size.width as i32
                && y >= back_rect.top_left.y && y < back_rect.top_left.y + back_rect.size.height as i32
            {
                return MonsterListAction::Back;
            }
        }

        // Check monster touch areas (only for owned monsters)
        for area in &self.touch_areas {
            if let Some(idx) = area.monster_index {
                if x >= area.rect.top_left.x && x < area.rect.top_left.x + area.rect.size.width as i32
                    && y >= area.rect.top_left.y && y < area.rect.top_left.y + area.rect.size.height as i32
                {
                    return MonsterListAction::Select(idx);
                }
            }
        }

        MonsterListAction::None
    }

    /// Handle swipe for scrolling (2 items per swipe)
    pub fn handle_swipe(&mut self, is_up: bool) {
        const SCROLL_AMOUNT: i32 = 110; // ~2 items * 55 pixels each
        const ITEM_HEIGHT: i32 = 52;

        if is_up {
            // Swipe up = scroll down (show more content below)
            let max_scroll = ((self.total_items as i32) * ITEM_HEIGHT).saturating_sub(200);
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
}

impl Page for MonsterListPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let disabled_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));

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

        // Truncate title if needed
        let title = if self.title.len() > 18 {
            &self.title[..18]
        } else {
            &self.title
        };
        let title_x = 120 - ((title.len() as i32 * 7) / 2);
        Text::new(title, Point::new(title_x, 20), title_style).draw(display)?;

        // Draw monster count
        let owned_count = self.monsters.iter().filter(|m| m.is_owned).count();
        let count_text = format!("{}/{}", owned_count, self.total_items);
        Text::new(&count_text, Point::new(185, 20), text_style).draw(display)?;

        // Clear and rebuild touch areas
        self.touch_areas.clear();

        // Content area
        let content_y_start = 32;
        let content_height = 240;

        // Clear content area
        let content_bg = Rectangle::new(
            Point::new(0, content_y_start),
            Size::new(240, content_height as u32)
        );
        display.fill_solid(&content_bg, Rgb888::new(240, 240, 245))?;

        // Draw monsters
        let card_height = 48i32;
        let card_spacing = 4i32;
        let card_x = 10;
        let card_width = 220u32;

        let mut y_pos = content_y_start - self.scroll_offset;

        for monster in &self.monsters {
            // Skip if above visible area
            if y_pos + card_height < content_y_start {
                y_pos += card_height + card_spacing;
                continue;
            }

            // Stop if below visible area
            if y_pos > content_y_start + content_height {
                break;
            }

            // Card with rounded corners
            let card_rect = Rectangle::new(Point::new(card_x, y_pos), Size::new(card_width, card_height as u32));
            let card_rounded = RoundedRectangle::new(card_rect, CornerRadii::new(Size::new(8, 8)));

            let (bg_color, border_color) = if !monster.is_owned {
                (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185)) // Disabled/unowned
            } else if monster.is_in_team {
                (Rgb888::new(200, 230, 255), Rgb888::new(100, 150, 200)) // Team member highlighted
            } else {
                (Rgb888::new(250, 250, 255), Rgb888::new(180, 185, 195))
            };

            // Fill
            card_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(bg_color)
                .build())
                .draw(display)?;

            // Border
            card_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(border_color)
                .stroke_width(1)
                .build())
                .draw(display)?;

            // Store touch area
            self.touch_areas.push(MonsterTouchArea {
                rect: card_rect,
                monster_index: monster.monster_index,
            });

            // Draw monster icon on left side
            let icon_x = card_x + 4;
            let icon_y = y_pos + 4;
            let icon_size = 28i32;
            let text_x = card_x + icon_size + 10; // Text starts after icon

            if let Some(icon_data) = get_monster_icon(&monster.species_id) {
                if let Ok(icon) = StaticImage::new(icon_data) {
                    let _ = icon.render(display, (icon_x, icon_y));
                }
            } else {
                // Fallback: element colored rounded square
                let elem_color = if monster.is_owned {
                    Self::element_color(monster.element)
                } else {
                    Rgb888::new(180, 180, 185) // Gray for unowned
                };
                let icon_rect = Rectangle::new(Point::new(icon_x, icon_y), Size::new(icon_size as u32, icon_size as u32));
                let icon_rounded = RoundedRectangle::new(icon_rect, CornerRadii::new(Size::new(6, 6)));
                icon_rounded.into_styled(PrimitiveStyleBuilder::new()
                    .fill_color(elem_color)
                    .build())
                    .draw(display)?;

                let elem_char = Self::element_char(monster.element);
                let char_style = if monster.is_owned {
                    MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE)
                } else {
                    MonoTextStyle::new(&FONT_7X13, Rgb888::new(120, 120, 120))
                };
                Text::new(&elem_char.to_string(), Point::new(icon_x + 9, icon_y + 20), char_style).draw(display)?;
            }

            if monster.is_owned {
                // Draw owned monster with full details
                let elem_color = Self::element_color(monster.element);
                let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);

                // Row 1: Name with element, fusion, level (truncate name)
                let name = if monster.name.len() > 10 {
                    &monster.name[..10]
                } else {
                    &monster.name
                };
                let name_with_fusion = if monster.fusion.is_empty() {
                    format!("{} {} Lv.{}", Self::element_char(monster.element), name, monster.level)
                } else {
                    format!("{} {} {} Lv.{}", Self::element_char(monster.element), name, monster.fusion, monster.level)
                };
                Text::new(&name_with_fusion, Point::new(text_x, y_pos + 14), elem_style).draw(display)?;

                // Row 2: PWR + status tags
                let power_text = format!("PWR:{}", monster.power);
                Text::new(&power_text, Point::new(text_x, y_pos + 26), dim_style).draw(display)?;

                let mut status_x = text_x + 48;
                if monster.is_in_team {
                    let team_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 120, 200));
                    Text::new("[T]", Point::new(status_x, y_pos + 26), team_style).draw(display)?;
                    status_x += 20;
                }

                let status_text = match monster.status {
                    MonsterStatus::Available => "",
                    MonsterStatus::InExpedition => "[E]",
                    MonsterStatus::InDungeon => "[D]",
                };
                if !status_text.is_empty() {
                    let status_color = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 140, 50));
                    Text::new(status_text, Point::new(status_x, y_pos + 26), status_color).draw(display)?;
                }

                // Row 3: Small XP bar
                let bar_x = text_x;
                let bar_y = y_pos + 34;
                let bar_width = 100u32;
                let bar_height = 5u32;

                let xp_bg = Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width, bar_height));
                display.fill_solid(&xp_bg, Rgb888::new(200, 205, 215))?;

                let xp_fill_width = ((bar_width as f32) * monster.xp_percent) as u32;
                if xp_fill_width > 0 {
                    let xp_fill = Rectangle::new(Point::new(bar_x, bar_y), Size::new(xp_fill_width, bar_height));
                    display.fill_solid(&xp_fill, Rgb888::new(100, 150, 220))?;
                }
                Text::new("XP", Point::new(bar_x + bar_width as i32 + 4, bar_y + 4), dim_style).draw(display)?;
            } else {
                // Draw unowned species as disabled
                let elem_char = Self::element_char(monster.element);
                let name = if monster.name.len() > 14 {
                    &monster.name[..14]
                } else {
                    &monster.name
                };
                let disabled_name = format!("{} {}", elem_char, name);
                Text::new(&disabled_name, Point::new(text_x, y_pos + 18), disabled_style).draw(display)?;

                // "Not captured" text
                Text::new("Not captured", Point::new(text_x, y_pos + 32), disabled_style).draw(display)?;
            }

            y_pos += card_height + card_spacing;
        }

        // Hint text at bottom (no back button)
        if self.total_items > 4 {
            Text::new("swipe to scroll", Point::new(75, 278), dim_style).draw(display)?;
        } else {
            Text::new("Tap for details", Point::new(75, 278), dim_style).draw(display)?;
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
