//! Equipment Page
//!
//! Displays equipped items and allows equipping/unequipping

use crate::display::Sh8601Driver;
use crate::game::{EquipmentSlot, EquipmentStats, GameData, Hero};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;

/// Equipment page actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentAction {
    SelectSlot(EquipmentSlot),
    SwitchToInventory,
    Upgrade(u64), // Upgrade item with unique_id
    Close,
}

/// Touch area
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: EquipmentAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Equipment page
pub struct EquipmentPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
    selected_slot: Option<EquipmentSlot>, // Currently selected slot for equipping
    pub dialog_scroll_offset: usize,      // Scroll position in item selection dialog
}

impl EquipmentPage {
    /// Create new equipment page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(10, 15, 25),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
            selected_slot: None,
            dialog_scroll_offset: 0,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<EquipmentAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Equipment action: {:?}", area.action);
                return Some(area.action);
            }
        }
        None
    }

    /// Draw equipment screen
    pub fn draw_equipment(
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

        // Header
        self.draw_header(display)?;

        // Equipment slots
        self.draw_equipment_slots(display, hero, game_data)?;

        // Stats summary
        self.draw_stats_summary(display, hero, game_data)?;

        // Action buttons
        self.draw_action_buttons(display)?;

        // Draw dialog overlay if a slot is selected
        if self.is_dialog_open() {
            self.draw_dialog(display, hero, game_data)?;
        }

        display.flush()?;
        Ok(())
    }

    /// Draw header
    fn draw_header(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        Text::new("EQUIPMENT", Point::new(120, 20), text_style).draw(display)?;
        Ok(())
    }

    /// Draw equipment slots
    fn draw_equipment_slots(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
    ) -> Result<(), Box<dyn Error>> {
        let start_y = 45;
        let slot_height = 40;
        let text_style_label = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 255));
        let text_style_item = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_empty = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 100, 100));

        for (index, slot) in EquipmentSlot::all_slots().iter().enumerate() {
            let y = start_y + (index as i32 * slot_height);

            // Slot background
            let bg_color = if hero.equipped_items.get_slot(*slot).is_some() {
                Rgb888::new(40, 60, 80) // Blue tint if equipped
            } else {
                Rgb888::new(40, 40, 40) // Gray if empty
            };

            Rectangle::new(Point::new(10, y), Size::new(348, slot_height as u32 - 3))
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            use core::fmt::Write;

            // Slot label
            let mut label_text = heapless::String::<16>::new();
            write!(label_text, "{}:", slot.name()).ok();
            Text::new(&label_text, Point::new(15, y + 25), text_style_label).draw(display)?;

            // Equipped item name
            if let Some(unique_id) = hero.equipped_items.get_slot(*slot) {
                if let Some(item) = hero.inventory.get_equipment(unique_id) {
                    if let Some(item_data) = game_data.get_item(item.item_id) {
                        let mut item_text = heapless::String::<32>::new();
                        let upgrade = item.get_upgrade_level();
                        if upgrade > 0 {
                            write!(item_text, "{} +{}", item_data.name, upgrade).ok();
                        } else {
                            write!(item_text, "{}", item_data.name).ok();
                        }
                        Text::new(&item_text, Point::new(130, y + 25), text_style_item)
                            .draw(display)?;
                    }
                }
            } else {
                Text::new("(Empty)", Point::new(130, y + 25), text_style_empty).draw(display)?;
            }

            // Make slot touchable
            self.touch_areas.push(TouchArea {
                bounds: (10, y, 348, slot_height as u32 - 3),
                action: EquipmentAction::SelectSlot(*slot),
            });
        }

        Ok(())
    }

    /// Draw stats summary on the right
    fn draw_stats_summary(
        &self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
    ) -> Result<(), Box<dyn Error>> {
        let start_y = 290;
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 200, 100));
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        // Title
        Text::new("Equipment Bonus:", Point::new(10, start_y), text_style_title).draw(display)?;

        // Calculate equipment stats
        let equipment_stats = EquipmentStats::calculate(
            &hero.equipped_items,
            hero.inventory.items(),
            game_data.get_all_items(),
        );

        use core::fmt::Write;

        // Display stats bonuses
        let mut atk_text = heapless::String::<32>::new();
        write!(atk_text, "ATK: +{}", equipment_stats.atk).ok();
        Text::new(&atk_text, Point::new(10, start_y + 25), text_style).draw(display)?;

        let mut def_text = heapless::String::<32>::new();
        write!(def_text, "DEF: +{}", equipment_stats.def).ok();
        Text::new(&def_text, Point::new(10, start_y + 50), text_style).draw(display)?;

        let mut hit_text = heapless::String::<32>::new();
        write!(hit_text, "HIT: +{}", equipment_stats.hit).ok();
        Text::new(&hit_text, Point::new(190, start_y + 25), text_style).draw(display)?;

        let mut flee_text = heapless::String::<32>::new();
        write!(flee_text, "FLEE: +{}", equipment_stats.flee).ok();
        Text::new(&flee_text, Point::new(190, start_y + 50), text_style).draw(display)?;

        Ok(())
    }

    /// Draw action buttons
    fn draw_action_buttons(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let button_y = 390;
        let button_height = 50;
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        // Inventory button
        let inv_button = Rectangle::new(Point::new(10, button_y), Size::new(165, button_height));
        inv_button
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 100, 60)))
            .draw(display)?;
        Text::new("INVENTORY", Point::new(30, button_y + 30), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (10, button_y, 165, button_height),
            action: EquipmentAction::SwitchToInventory,
        });

        // Close button
        let close_button = Rectangle::new(Point::new(193, button_y), Size::new(165, button_height));
        close_button
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 60, 60)))
            .draw(display)?;
        Text::new("CLOSE", Point::new(240, button_y + 30), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (193, button_y, 165, button_height),
            action: EquipmentAction::Close,
        });

        Ok(())
    }

    /// Open selection dialog for a slot
    pub fn open_dialog(&mut self, slot: EquipmentSlot) {
        self.selected_slot = Some(slot);
        self.dialog_scroll_offset = 0;
        self.needs_full_redraw = true;
        log::info!("Opened equipment dialog for {:?}", slot);
    }

    /// Close the selection dialog
    pub fn close_dialog(&mut self) {
        self.selected_slot = None;
        self.dialog_scroll_offset = 0;
        self.needs_full_redraw = true;
        log::info!("Closed equipment dialog");
    }

    /// Check if dialog is open
    pub fn is_dialog_open(&self) -> bool {
        self.selected_slot.is_some()
    }

    /// Get the currently selected slot
    pub fn selected_slot(&self) -> Option<EquipmentSlot> {
        self.selected_slot
    }

    /// Draw selection dialog overlay
    pub fn draw_dialog(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
    ) -> Result<(), Box<dyn Error>> {
        let Some(slot) = self.selected_slot else {
            return Ok(());
        };

        // Semi-transparent background overlay
        Rectangle::new(Point::new(0, 0), Size::new(368, 448))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        // Dialog box
        let dialog_x = 20;
        let dialog_y = 50;
        let dialog_width = 328u32;
        let dialog_height = 348u32;

        Rectangle::new(Point::new(dialog_x, dialog_y), Size::new(dialog_width, dialog_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 30, 40)))
            .draw(display)?;

        Rectangle::new(Point::new(dialog_x, dialog_y), Size::new(dialog_width, dialog_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 150, 200), 2))
            .draw(display)?;

        use core::fmt::Write;
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_empty = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));

        // Title
        let mut title = heapless::String::<32>::new();
        write!(title, "Select {}", slot.name()).ok();
        Text::new(&title, Point::new(dialog_x + 10, dialog_y + 25), text_style_title).draw(display)?;

        // Get eligible items from inventory
        let eligible_items: Vec<_> = hero
            .inventory
            .items()
            .iter()
            .filter(|item| {
                if let Some(item_data) = game_data.get_item(item.item_id) {
                    item_data.slot == Some(slot)
                } else {
                    false
                }
            })
            .collect();

        // Debug: Log eligible items
        log::info!("Equipment dialog for {:?}: found {} eligible items", slot, eligible_items.len());
        for (i, item) in eligible_items.iter().enumerate() {
            if let Some(item_data) = game_data.get_item(item.item_id) {
                log::info!("  [{}] {} (ID:{}, unique_id:{:?})", i, item_data.name, item.item_id, item.unique_id);
            }
        }

        // Draw items list
        let list_start_y = dialog_y + 50;
        let item_height = 35;
        let visible_items = 7; // Show 7 items at once

        if eligible_items.is_empty() {
            Text::new(
                "No items available",
                Point::new(dialog_x + 10, list_start_y + 20),
                text_style_empty,
            )
            .draw(display)?;
        } else {
            for (index, item) in eligible_items
                .iter()
                .skip(self.dialog_scroll_offset)
                .take(visible_items)
                .enumerate()
            {
                let y = list_start_y + (index as i32 * item_height);

                // Item background
                let currently_equipped = if let Some(unique_id) = item.unique_id {
                    hero.equipped_items
                        .get_slot(slot)
                        .map(|id| id == unique_id)
                        .unwrap_or(false)
                } else {
                    false
                };

                let bg_color = if currently_equipped {
                    Rgb888::new(60, 100, 60) // Green if currently equipped
                } else {
                    Rgb888::new(40, 50, 60)
                };

                Rectangle::new(Point::new(dialog_x + 5, y), Size::new(dialog_width - 10, item_height as u32 - 3))
                    .into_styled(PrimitiveStyle::with_fill(bg_color))
                    .draw(display)?;

                // Item name
                if let Some(item_data) = game_data.get_item(item.item_id) {
                    let mut item_text = heapless::String::<32>::new();
                    let upgrade = item.get_upgrade_level();
                    if upgrade > 0 {
                        write!(item_text, "{} +{}", item_data.name, upgrade).ok();
                    } else {
                        write!(item_text, "{}", item_data.name).ok();
                    }
                    Text::new(&item_text, Point::new(dialog_x + 10, y + 22), text_style).draw(display)?;

                    // Show required level if any
                    if let Some(req_level) = item_data.required_level {
                        let can_equip = hero.level >= req_level;
                        let level_style = if can_equip {
                            text_style
                        } else {
                            MonoTextStyle::new(&FONT_10X20, Rgb888::RED)
                        };
                        let mut level_text = heapless::String::<16>::new();
                        write!(level_text, "Lv{}", req_level).ok();
                        Text::new(&level_text, Point::new(dialog_x + 250, y + 22), level_style).draw(display)?;
                    }
                }

                // Register touch area for this item
                self.touch_areas.push(TouchArea {
                    bounds: (dialog_x + 5, y, dialog_width - 10, item_height as u32 - 3),
                    action: EquipmentAction::SelectSlot(slot), // Will be handled specially
                });
            }
        }

        // Unequip and Upgrade buttons (if something is equipped)
        if let Some(equipped_id) = hero.equipped_items.get_slot(slot) {
            let button_y = dialog_y + dialog_height as i32 - 80;

            // Unequip button (left)
            Rectangle::new(Point::new(dialog_x + 10, button_y), Size::new(150, 35))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(120, 40, 40)))
                .draw(display)?;
            Text::new("Unequip", Point::new(dialog_x + 40, button_y + 22), text_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (dialog_x + 10, button_y, 150, 35),
                action: EquipmentAction::SelectSlot(slot), // Special handling for unequip
            });

            // Upgrade button (right) - only if not max level
            if let Some(item) = hero.inventory.get_equipment(equipped_id) {
                let current_level = item.get_upgrade_level();
                if current_level < 10 {
                    Rectangle::new(Point::new(dialog_x + 168, button_y), Size::new(150, 35))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 100, 120)))
                        .draw(display)?;

                    let mut upgrade_text = heapless::String::<16>::new();
                    write!(upgrade_text, "Upgrade +{}", current_level + 1).ok();
                    Text::new(&upgrade_text, Point::new(dialog_x + 180, button_y + 22), text_style).draw(display)?;

                    self.touch_areas.push(TouchArea {
                        bounds: (dialog_x + 168, button_y, 150, 35),
                        action: EquipmentAction::Upgrade(equipped_id),
                    });
                }
            }
        }

        // Close button
        let close_y = dialog_y + dialog_height as i32 - 40;
        Rectangle::new(Point::new(dialog_x + 10, close_y), Size::new(308, 35))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 80, 80)))
            .draw(display)?;
        Text::new("Close", Point::new(dialog_x + 135, close_y + 22), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (dialog_x + 10, close_y, 308, 35),
            action: EquipmentAction::Close,
        });

        Ok(())
    }
}

impl Page for EquipmentPage {
    fn update(&mut self) -> bool {
        true
    }

    fn draw(
        &mut self,
        _display: &mut Sh8601Driver,
        _full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Use draw_equipment instead
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
