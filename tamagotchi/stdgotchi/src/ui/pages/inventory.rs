//! Inventory Page
//!
//! Displays player inventory with materials and equipment

use crate::display::Sh8601Driver;
use crate::game::{GameData, Hero, Item};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;

/// Inventory page actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAction {
    SwitchToEquipment,
    Close,
}

/// Touch area for inventory interactions
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: Option<InventoryAction>,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Inventory page displaying items
pub struct InventoryPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
    scroll_offset: usize, // For scrolling through items
}

impl InventoryPage {
    /// Create new inventory page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(10, 15, 25),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
            scroll_offset: 0,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<InventoryAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                if let Some(action) = area.action {
                    log::info!("Inventory action: {:?}", action);
                    return Some(action);
                }
            }
        }
        None
    }

    /// Draw inventory with hero data
    pub fn draw_inventory(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.touch_areas.clear();

        // Header section
        self.draw_header(display, hero)?;

        // Inventory grid
        self.draw_inventory_grid(display, hero, game_data)?;

        // Action buttons at bottom
        self.draw_action_buttons(display)?;

        display.flush()?;
        Ok(())
    }

    /// Draw header with gold and inventory count
    fn draw_header(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0)); // Gold color

        // Title
        Text::new("INVENTORY", Point::new(120, 20), text_style).draw(display)?;

        // Gold display
        use core::fmt::Write;
        let mut gold_text = heapless::String::<32>::new();
        write!(gold_text, "Gold: {}", hero.gold).ok();
        let info_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 200));
        Text::new(&gold_text, Point::new(10, 45), info_style).draw(display)?;

        // Inventory count
        let mut count_text = heapless::String::<32>::new();
        write!(count_text, "Items: {}/30", hero.inventory.slot_count()).ok();
        Text::new(&count_text, Point::new(230, 45), info_style).draw(display)?;

        Ok(())
    }

    /// Draw inventory items in a grid
    fn draw_inventory_grid(
        &self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
    ) -> Result<(), Box<dyn Error>> {
        let start_y = 60;
        let item_height = 35;
        let max_visible = 7; // Number of items visible at once

        let text_style_name = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_qty = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 255, 150));

        let items: Vec<&Item> = hero.inventory.items().iter().collect();

        // Calculate visible range
        let end_index = (self.scroll_offset + max_visible).min(items.len());

        for (display_index, &item) in items[self.scroll_offset..end_index].iter().enumerate() {
            let y = start_y + (display_index as i32 * item_height);

            // Item background
            let bg_color = if item.is_equipment() {
                Rgb888::new(60, 40, 80) // Purple tint for equipment
            } else {
                Rgb888::new(40, 40, 40) // Gray for materials
            };

            Rectangle::new(Point::new(10, y - 2), Size::new(348, item_height as u32 - 3))
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            // Get item name from game data
            if let Some(item_data) = game_data.get_item(item.item_id) {
                use core::fmt::Write;

                // Item name
                let mut name_text = heapless::String::<32>::new();
                if item.is_equipment() {
                    let upgrade = item.get_upgrade_level();
                    if upgrade > 0 {
                        write!(name_text, "{} +{}", item_data.name, upgrade).ok();
                    } else {
                        write!(name_text, "{}", item_data.name).ok();
                    }
                } else {
                    write!(name_text, "{}", item_data.name).ok();
                }
                Text::new(&name_text, Point::new(15, y + 20), text_style_name).draw(display)?;

                // Quantity (for stackable items)
                if !item.is_equipment() {
                    let mut qty_text = heapless::String::<16>::new();
                    write!(qty_text, "x{}", item.quantity).ok();
                    Text::new(&qty_text, Point::new(290, y + 20), text_style_qty).draw(display)?;
                }
            } else {
                // Unknown item
                let mut unknown_text = heapless::String::<32>::new();
                use core::fmt::Write;
                write!(unknown_text, "Unknown Item #{}", item.item_id).ok();
                Text::new(&unknown_text, Point::new(15, y + 20), text_style_name).draw(display)?;
            }
        }

        // Scroll indicators if needed
        let total_items = items.len();
        if total_items > max_visible {
            let indicator_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));

            if self.scroll_offset > 0 {
                Text::new("▲ More above", Point::new(135, start_y - 5), indicator_style)
                    .draw(display)?;
            }

            if end_index < total_items {
                let last_y = start_y + (max_visible as i32 * item_height);
                Text::new("▼ More below", Point::new(135, last_y + 5), indicator_style)
                    .draw(display)?;
            }
        }

        Ok(())
    }

    /// Draw action buttons at bottom
    fn draw_action_buttons(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let button_y = 390;
        let button_height = 50;
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        // Equipment button
        let equip_button = Rectangle::new(Point::new(10, button_y), Size::new(165, button_height));
        equip_button
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 100, 60)))
            .draw(display)?;
        Text::new("EQUIPMENT", Point::new(25, button_y + 30), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (10, button_y, 165, button_height),
            action: Some(InventoryAction::SwitchToEquipment),
        });

        // Close button
        let close_button = Rectangle::new(Point::new(193, button_y), Size::new(165, button_height));
        close_button
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 60, 60)))
            .draw(display)?;
        Text::new("CLOSE", Point::new(240, button_y + 30), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (193, button_y, 165, button_height),
            action: Some(InventoryAction::Close),
        });

        Ok(())
    }

    /// Scroll inventory up
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.needs_full_redraw = true;
        }
    }

    /// Scroll inventory down
    pub fn scroll_down(&mut self, total_items: usize) {
        let max_visible = 7;
        if self.scroll_offset + max_visible < total_items {
            self.scroll_offset += 1;
            self.needs_full_redraw = true;
        }
    }
}

impl Page for InventoryPage {
    fn update(&mut self) -> bool {
        true // No animation updates needed
    }

    fn draw(
        &mut self,
        _display: &mut Sh8601Driver,
        _full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Use draw_inventory instead
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
