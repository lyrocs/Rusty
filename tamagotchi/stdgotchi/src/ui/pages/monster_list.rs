//! Monster List Page
//!
//! Displays monsters - either player's owned monsters or all species from a zone.
//! Unowned species are shown as disabled/grayed out.

use crate::assets::get_monster_icon;
use crate::display::{Sh8601Driver, StaticImage};
use crate::game::core::{Element, Monster, MonsterStatus};
use crate::game::systems::progression::fusion::format_fusion;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
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
        const SCROLL_AMOUNT: i32 = 150; // 2 items * 75 pixels each
        const ITEM_HEIGHT: i32 = 75;

        if is_up {
            // Swipe up = scroll down (show more content below)
            let max_scroll = ((self.total_items as i32) * ITEM_HEIGHT).saturating_sub(300);
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
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            // Clear screen
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let disabled_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(80, 80, 80));

        // Draw header
        let title_x = if self.title.len() > 10 { 80 } else { 130 };
        Text::new(&self.title, Point::new(title_x, 30), title_style).draw(display)?;

        // Draw monster count
        let owned_count = self.monsters.iter().filter(|m| m.is_owned).count();
        let count_text = format!("{}/{}", owned_count, self.total_items);
        Text::new(&count_text, Point::new(300, 30), dim_style).draw(display)?;

        // Clear and rebuild touch areas
        self.touch_areas.clear();

        // Content area
        let content_y_start = 50;
        let content_height = 350;

        // Clear content area
        let content_bg = Rectangle::new(
            Point::new(0, content_y_start),
            Size::new(368, content_height as u32)
        );
        display.fill_solid(&content_bg, Rgb888::new(20, 25, 35))?;

        // Draw monsters
        let card_height = 70i32;
        let card_spacing = 5i32;
        let card_x = 15;
        let card_width = 338u32;

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

            // Card background
            let card_color = if !monster.is_owned {
                Rgb888::new(25, 25, 30) // Disabled/unowned
            } else if monster.is_in_team {
                Rgb888::new(40, 50, 70) // Team member highlighted
            } else {
                Rgb888::new(30, 35, 45)
            };
            let card_rect = Rectangle::new(Point::new(card_x, y_pos), Size::new(card_width, card_height as u32));
            display.fill_solid(&card_rect, card_color)?;

            // Store touch area
            self.touch_areas.push(MonsterTouchArea {
                rect: card_rect,
                monster_index: monster.monster_index,
            });

            // Draw monster icon on left side
            let icon_x = card_x + 5;
            let icon_y = y_pos + 5;
            let icon_size = 40i32;
            let text_x = card_x + icon_size + 15; // Text starts after icon

            if let Some(icon_data) = get_monster_icon(&monster.species_id) {
                if let Ok(icon) = StaticImage::new(icon_data) {
                    let _ = icon.render(display, (icon_x, icon_y));
                }
            } else {
                // Fallback: element colored square
                let elem_color = if monster.is_owned {
                    Self::element_color(monster.element)
                } else {
                    Rgb888::new(50, 50, 50) // Gray for unowned
                };
                let icon_rect = Rectangle::new(Point::new(icon_x, icon_y), Size::new(icon_size as u32, icon_size as u32));
                display.fill_solid(&icon_rect, elem_color)?;
                let elem_char = Self::element_char(monster.element);
                let char_style = if monster.is_owned {
                    MonoTextStyle::new(&FONT_10X20, Rgb888::BLACK)
                } else {
                    MonoTextStyle::new(&FONT_10X20, Rgb888::new(80, 80, 80))
                };
                Text::new(&elem_char.to_string(), Point::new(icon_x + 12, icon_y + 28), char_style).draw(display)?;
            }

            if monster.is_owned {
                // Draw owned monster with full details
                let elem_color = Self::element_color(monster.element);
                let elem_style = MonoTextStyle::new(&FONT_9X15, elem_color);

                // Row 1: Name with element, fusion, level
                let name_with_fusion = if monster.fusion.is_empty() {
                    format!("{} {} Lv.{}", Self::element_char(monster.element), monster.name, monster.level)
                } else {
                    format!("{} {} {} Lv.{}", Self::element_char(monster.element), monster.name, monster.fusion, monster.level)
                };
                Text::new(&name_with_fusion, Point::new(text_x, y_pos + 18), elem_style).draw(display)?;

                // Row 2: PWR + [TEAM] + [EXP]/[DGN] status (same line)
                let power_text = format!("PWR:{}", monster.power);
                Text::new(&power_text, Point::new(text_x, y_pos + 38), dim_style).draw(display)?;

                let mut status_x = text_x + 70;
                if monster.is_in_team {
                    let team_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 255));
                    Text::new("[TEAM]", Point::new(status_x, y_pos + 38), team_style).draw(display)?;
                    status_x += 60;
                }

                let status_text = match monster.status {
                    MonsterStatus::Available => "",
                    MonsterStatus::InExpedition => "[EXP]",
                    MonsterStatus::InDungeon => "[DGN]",
                };
                if !status_text.is_empty() {
                    let status_color = MonoTextStyle::new(&FONT_9X15, Rgb888::new(200, 180, 100));
                    Text::new(status_text, Point::new(status_x, y_pos + 38), status_color).draw(display)?;
                }

                // Row 3: Small XP bar (reduced size)
                let bar_x = text_x;
                let bar_y = y_pos + 48;
                let bar_width = 80u32;
                let bar_height = 6u32;

                let xp_bg = Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width, bar_height));
                display.fill_solid(&xp_bg, Rgb888::new(40, 40, 60))?;

                let xp_fill_width = ((bar_width as f32) * monster.xp_percent) as u32;
                if xp_fill_width > 0 {
                    let xp_fill = Rectangle::new(Point::new(bar_x, bar_y), Size::new(xp_fill_width, bar_height));
                    display.fill_solid(&xp_fill, Rgb888::new(100, 150, 255))?;
                }
                Text::new("XP", Point::new(bar_x + bar_width as i32 + 5, bar_y + 5), dim_style).draw(display)?;
            } else {
                // Draw unowned species as disabled (no icon fallback already handled above)
                let elem_char = Self::element_char(monster.element);
                let disabled_name = format!("{} {}", elem_char, monster.name);
                Text::new(&disabled_name, Point::new(text_x, y_pos + 25), disabled_style).draw(display)?;

                // "Not captured" text
                Text::new("Not captured", Point::new(text_x, y_pos + 45), disabled_style).draw(display)?;
            }

            y_pos += card_height + card_spacing;
        }

        // Draw back button
        let back_rect = Rectangle::new(Point::new(15, 410), Size::new(80, 30));
        display.fill_solid(&back_rect, Rgb888::new(80, 60, 60))?;
        Text::new("< BACK", Point::new(25, 430), text_style).draw(display)?;
        self.back_area = Some(back_rect);

        // Hint text
        if self.total_items > 4 {
            Text::new("swipe to scroll", Point::new(230, 430), dim_style).draw(display)?;
        } else {
            Text::new("Tap for details", Point::new(230, 430), dim_style).draw(display)?;
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
