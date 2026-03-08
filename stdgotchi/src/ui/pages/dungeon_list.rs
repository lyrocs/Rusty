//! Dungeon List Page
//!
//! Shows all dungeons sorted by level with basic info (elements, monster levels).
//! Clicking a dungeon opens the DungeonInfoPage.

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
use std::error::Error;

/// Action from dungeon list page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DungeonListAction {
    /// No action
    None,
    /// Go back to home
    Back,
    /// Selected a dungeon (dungeon_id)
    SelectDungeon(String),
}

/// Dungeon display data for the list
#[derive(Clone)]
pub struct DungeonDisplayData {
    pub dungeon_id: String,
    pub name: String,
    pub elements: Vec<Element>,
    pub level_min: u8,
    pub level_max: u8,
    pub highest_floor: u16,
    pub is_unlocked: bool,
}

/// Touch area for dungeon selection
struct DungeonTouchArea {
    rect: Rectangle,
    dungeon_id: String,
}

/// Dungeon list page
pub struct DungeonListPage {
    dungeons: Vec<DungeonDisplayData>,
    scroll_offset: i32,

    // Touch areas
    back_area: Option<Rectangle>,
    dungeon_areas: Vec<DungeonTouchArea>,

    dirty: bool,
}

impl DungeonListPage {
    pub fn new(mut dungeons: Vec<DungeonDisplayData>) -> Self {
        // Sort dungeons by level_min
        dungeons.sort_by_key(|d| d.level_min);

        Self {
            dungeons,
            scroll_offset: 0,
            back_area: None,
            dungeon_areas: Vec::new(),
            dirty: true,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&self, x: i32, y: i32) -> DungeonListAction {
        let point = Point::new(x, y);

        // Check back button
        if let Some(ref rect) = self.back_area {
            if rect.contains(point) {
                return DungeonListAction::Back;
            }
        }

        // Check dungeon areas
        for area in &self.dungeon_areas {
            if area.rect.contains(point) {
                return DungeonListAction::SelectDungeon(area.dungeon_id.clone());
            }
        }

        DungeonListAction::None
    }

    /// Handle swipe for scrolling
    pub fn handle_swipe(&mut self, up: bool) {
        let max_scroll = (self.dungeons.len() as i32 - 4).max(0) * 52;
        if up {
            self.scroll_offset = (self.scroll_offset + 52).min(max_scroll);
        } else {
            self.scroll_offset = (self.scroll_offset - 52).max(0);
        }
        self.dirty = true;
    }

    fn element_color(element: &Element) -> Rgb888 {
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(150, 100, 50),
            Element::Wind => Rgb888::new(100, 200, 100),
            Element::Thunder => Rgb888::new(255, 255, 50),
            Element::Shadow => Rgb888::new(100, 50, 150),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(150, 150, 200),
            Element::Neutral => Rgb888::new(180, 180, 180),
        }
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
}

impl Page for DungeonListPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let _title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));

        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        // Header
        let header_rect = Rectangle::new(Point::new(0, 0), Size::new(240, 28));
        display.fill_solid(&header_rect, Rgb888::new(220, 100, 80))?;
        Text::new("DUNGEONS", Point::new(80, 18), MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE)).draw(display)?;

        // Dungeon list
        self.dungeon_areas.clear();
        let list_y = 32;
        let item_height = 48u32;
        let visible_items = 4;

        let scroll_item = self.scroll_offset / item_height as i32;

        for (i, dungeon) in self.dungeons.iter().enumerate().skip(scroll_item as usize).take(visible_items + 1) {
            let visual_index = i as i32 - scroll_item;
            let y = list_y + (visual_index * item_height as i32) - (self.scroll_offset % item_height as i32);

            // Skip if out of view
            if y < list_y - item_height as i32 || y > 250 {
                continue;
            }

            let rect = Rectangle::new(Point::new(8, y), Size::new(224, item_height - 4));
            let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(8, 8)));

            // Background color based on unlock status
            let (bg_color, border_color) = if dungeon.is_unlocked {
                (Rgb888::new(250, 250, 255), Rgb888::new(180, 140, 100))
            } else {
                (Rgb888::new(200, 200, 205), Rgb888::new(150, 150, 155))
            };

            // Fill
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(bg_color)
                .build())
                .draw(display)?;

            // Border
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(border_color)
                .stroke_width(2)
                .build())
                .draw(display)?;

            self.dungeon_areas.push(DungeonTouchArea {
                rect,
                dungeon_id: dungeon.dungeon_id.clone(),
            });

            // Dungeon name
            let name_style = if dungeon.is_unlocked { text_style } else { dim_style };
            let name = if dungeon.name.len() > 18 { &dungeon.name[..18] } else { &dungeon.name };
            Text::new(name, Point::new(16, y + 16), name_style).draw(display)?;

            // Level range
            let level_text = format!("Lv.{}-{}", dungeon.level_min, dungeon.level_max);
            Text::new(&level_text, Point::new(160, y + 16), dim_style).draw(display)?;

            // Elements
            let mut elem_x = 16;
            for elem in &dungeon.elements {
                let elem_style = MonoTextStyle::new(&FONT_6X10, Self::element_color(elem));
                Text::new(&Self::element_char(elem).to_string(), Point::new(elem_x, y + 32), elem_style).draw(display)?;
                elem_x += 12;
            }

            // Progress (highest floor)
            if dungeon.highest_floor > 0 {
                let progress_text = format!("F{}", dungeon.highest_floor);
                Text::new(&progress_text, Point::new(180, y + 32), dim_style).draw(display)?;
            }

            // Lock indicator
            if !dungeon.is_unlocked {
                Text::new("LOCKED", Point::new(100, y + 32), dim_style).draw(display)?;
            }
        }

        // Scroll indicator
        if self.dungeons.len() > visible_items {
            let total_height = 200;
            let scroll_ratio = self.scroll_offset as f32 / ((self.dungeons.len() as i32 - visible_items as i32).max(1) * item_height as i32) as f32;
            let indicator_y = list_y + (scroll_ratio * (total_height - 30) as f32) as i32;
            let indicator = Rectangle::new(Point::new(234, indicator_y), Size::new(4, 30));
            display.fill_solid(&indicator, Rgb888::new(180, 180, 185))?;
        }

        // Back hint at bottom
        Text::new("< BACK (swipe right)", Point::new(60, 275), dim_style).draw(display)?;
        self.back_area = Some(Rectangle::new(Point::new(0, 260), Size::new(240, 24)));

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
