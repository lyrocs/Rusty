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
    Switch,       // Open item selection dialog
    Back,         // Go back to previous view
    Close,
}

/// View mode for equipment page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    SlotList,      // Main equipment list
    ItemDetail,    // Detail view of equipped item
    ItemSelection, // Item selection dialog
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
    selected_slot: Option<EquipmentSlot>, // Currently selected slot for viewing/equipping
    pub dialog_scroll_offset: usize,      // Scroll position in item selection dialog
    view_mode: ViewMode,                  // Current view mode
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
            view_mode: ViewMode::SlotList,
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

        match self.view_mode {
            ViewMode::SlotList => {
                // Main equipment list view
                self.draw_header(display)?;
                self.draw_equipment_slots(display, hero, game_data)?;
                self.draw_stats_summary(display, hero, game_data)?;
                self.draw_action_buttons(display)?;
            }
            ViewMode::ItemDetail => {
                // Item detail view
                if let Some(slot) = self.selected_slot {
                    self.draw_item_detail(display, hero, game_data, slot)?;
                }
            }
            ViewMode::ItemSelection => {
                // Item selection dialog
                if self.is_dialog_open() {
                    self.draw_dialog(display, hero, game_data)?;
                }
            }
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

    /// Draw equipment slots in 2-column layout
    fn draw_equipment_slots(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        let margin = 15;
        let start_y = 45;
        let slot_height = 65u32;
        let slot_spacing = 8;
        let column_spacing = 8;
        let slot_width = 165u32;  // (368 - 15*2 - 8) / 2

        let text_style_label = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 255));
        let text_style_item = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_empty = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 100, 100));

        let all_slots = EquipmentSlot::all_slots();

        for (index, slot) in all_slots.iter().enumerate() {
            // Calculate position (2 columns)
            let column = index % 2;
            let row = index / 2;

            let x = if column == 0 {
                margin
            } else {
                margin + slot_width as i32 + column_spacing
            };
            let y = start_y + (row as i32 * (slot_height as i32 + slot_spacing));

            // Slot background
            let bg_color = if hero.equipped_items.get_slot(*slot).is_some() {
                Rgb888::new(40, 60, 80) // Blue tint if equipped
            } else {
                Rgb888::new(40, 40, 40) // Gray if empty
            };

            Rectangle::new(Point::new(x, y), Size::new(slot_width, slot_height))
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            // Slot label
            let mut label_text = heapless::String::<16>::new();
            write!(label_text, "{}", slot.name()).ok();
            Text::new(&label_text, Point::new(x + 8, y + 20), text_style_label).draw(display)?;

            // Equipped item name
            if let Some(unique_id) = hero.equipped_items.get_slot(*slot) {
                if let Some(item) = hero.inventory.get_equipment(unique_id) {
                    if let Some(item_data) = game_data.get_item(item.item_id) {
                        let mut item_text = heapless::String::<20>::new();
                        let upgrade = item.get_upgrade_level();
                        if upgrade > 0 {
                            write!(item_text, "{} +{}", item_data.name, upgrade).ok();
                        } else {
                            write!(item_text, "{}", item_data.name).ok();
                        }
                        Text::new(&item_text, Point::new(x + 8, y + 42), text_style_item)
                            .draw(display)?;
                    }
                }
            } else {
                Text::new("(Empty)", Point::new(x + 8, y + 42), text_style_empty).draw(display)?;
            }

            // Make slot touchable
            self.touch_areas.push(TouchArea {
                bounds: (x, y, slot_width, slot_height),
                action: EquipmentAction::SelectSlot(*slot),
            });
        }

        Ok(())
    }

    /// Draw stats summary
    fn draw_stats_summary(
        &self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
    ) -> Result<(), Box<dyn Error>> {
        let start_y = 285;
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 200, 100));
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        // Title
        Text::new("Equipment Bonus:", Point::new(15, start_y), text_style_title).draw(display)?;

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
        Text::new(&atk_text, Point::new(15, start_y + 25), text_style).draw(display)?;

        let mut def_text = heapless::String::<32>::new();
        write!(def_text, "DEF: +{}", equipment_stats.def).ok();
        Text::new(&def_text, Point::new(15, start_y + 50), text_style).draw(display)?;

        let mut hit_text = heapless::String::<32>::new();
        write!(hit_text, "HIT: +{}", equipment_stats.hit).ok();
        Text::new(&hit_text, Point::new(195, start_y + 25), text_style).draw(display)?;

        let mut flee_text = heapless::String::<32>::new();
        write!(flee_text, "FLEE: +{}", equipment_stats.flee).ok();
        Text::new(&flee_text, Point::new(195, start_y + 50), text_style).draw(display)?;

        Ok(())
    }

    /// Draw action buttons
    fn draw_action_buttons(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let margin = 15;
        let button_y = 368;
        let button_height = 60u32;
        let button_width = 165u32;
        let button_spacing = 8;
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        // Inventory button
        let inv_button = Rectangle::new(Point::new(margin, button_y), Size::new(button_width, button_height));
        inv_button
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 100, 60)))
            .draw(display)?;
        Text::new("INVENTORY", Point::new(margin + 30, button_y + 38), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (margin, button_y, button_width, button_height),
            action: EquipmentAction::SwitchToInventory,
        });

        // Close button
        let close_x = margin + button_width as i32 + button_spacing;
        let close_button = Rectangle::new(Point::new(close_x, button_y), Size::new(button_width, button_height));
        close_button
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 60, 60)))
            .draw(display)?;
        Text::new("CLOSE", Point::new(close_x + 50, button_y + 38), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (close_x, button_y, button_width, button_height),
            action: EquipmentAction::Close,
        });

        Ok(())
    }

    /// Open item detail view for a slot
    pub fn open_dialog(&mut self, slot: EquipmentSlot) {
        self.selected_slot = Some(slot);
        self.view_mode = ViewMode::ItemDetail;
        self.dialog_scroll_offset = 0;
        self.needs_full_redraw = true;
        log::info!("Opened item detail for {:?}", slot);
    }

    /// Open item selection dialog
    pub fn open_selection(&mut self) {
        self.view_mode = ViewMode::ItemSelection;
        self.dialog_scroll_offset = 0;
        self.needs_full_redraw = true;
        log::info!("Opened item selection dialog");
    }

    /// Go back from detail/selection to slot list
    pub fn back_to_list(&mut self) {
        self.view_mode = ViewMode::SlotList;
        self.selected_slot = None;
        self.dialog_scroll_offset = 0;
        self.needs_full_redraw = true;
        log::info!("Returned to slot list");
    }

    /// Go back from selection to detail
    pub fn back_to_detail(&mut self) {
        self.view_mode = ViewMode::ItemDetail;
        self.dialog_scroll_offset = 0;
        self.needs_full_redraw = true;
        log::info!("Returned to item detail");
    }

    /// Close the selection dialog (legacy - redirects to back_to_list)
    pub fn close_dialog(&mut self) {
        self.back_to_list();
    }

    /// Check if dialog is open
    pub fn is_dialog_open(&self) -> bool {
        self.view_mode == ViewMode::ItemSelection
    }

    /// Get the currently selected slot
    pub fn selected_slot(&self) -> Option<EquipmentSlot> {
        self.selected_slot
    }

    /// Draw item detail view
    fn draw_item_detail(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
        slot: EquipmentSlot,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        let margin = 15;
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style_label = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 255));
        let text_style_value = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_empty = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));

        let mut y = margin + 15;

        // Slot name
        let mut title = heapless::String::<32>::new();
        write!(title, "{}", slot.name()).ok();
        Text::new(&title, Point::new(margin + 5, y), text_style_title).draw(display)?;
        y += 30;

        // Get equipped item
        let equipped_id = hero.equipped_items.get_slot(slot);

        if let Some(unique_id) = equipped_id {
            if let Some(item) = hero.inventory.get_equipment(unique_id) {
                if let Some(item_data) = game_data.get_item(item.item_id) {
                    // Item name with upgrade level
                    let mut item_text = heapless::String::<32>::new();
                    let upgrade = item.get_upgrade_level();
                    if upgrade > 0 {
                        write!(item_text, "{} +{}", item_data.name, upgrade).ok();
                    } else {
                        write!(item_text, "{}", item_data.name).ok();
                    }
                    Text::new(&item_text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                    y += 25;

                    // Description
                    if !item_data.description.is_empty() {
                        Text::new(&item_data.description, Point::new(margin + 5, y), text_style_label).draw(display)?;
                        y += 20;
                    }

                    y += 10;

                    // Stats
                    Text::new("STATS:", Point::new(margin + 5, y), text_style_title).draw(display)?;
                    y += 22;

                    if let Some(atk) = item_data.base_atk {
                        let total_atk = atk + upgrade * item_data.upgrade_bonus_atk.unwrap_or(0);
                        let mut text = heapless::String::<32>::new();
                        write!(text, "  ATK: +{}", total_atk).ok();
                        Text::new(&text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                        y += 20;
                    }

                    if let Some(def) = item_data.base_def {
                        let total_def = def + upgrade * item_data.upgrade_bonus_def.unwrap_or(0);
                        let mut text = heapless::String::<32>::new();
                        write!(text, "  DEF: +{}", total_def).ok();
                        Text::new(&text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                        y += 20;
                    }

                    if let Some(flee) = item_data.base_flee {
                        let mut text = heapless::String::<32>::new();
                        write!(text, "  FLEE: +{}", flee).ok();
                        Text::new(&text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                        y += 20;
                    }

                    if let Some(hit) = item_data.base_hit {
                        let mut text = heapless::String::<32>::new();
                        write!(text, "  HIT: +{}", hit).ok();
                        Text::new(&text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                        y += 20;
                    }

                    // Buttons at bottom (screen is 480px tall, safe margin at bottom is 15px)
                    let button_height = 55u32;
                    let button_width = 338u32;
                    let button_spacing = 5;

                    let can_upgrade = upgrade < 10;

                    // Calculate button positions from bottom up
                    // 3 buttons: 55 + 5 + 55 + 5 + 55 = 175px total
                    // Start at: 480 - 15 (margin) - 175 = 290
                    let button_start_y = 290;

                    // Upgrade button (only if upgradeable)
                    let mut current_y = button_start_y;
                    if can_upgrade {
                        let upgrade_color = Rgb888::new(40, 100, 120);
                        Rectangle::new(Point::new(margin, current_y), Size::new(button_width, button_height))
                            .into_styled(PrimitiveStyle::with_fill(upgrade_color))
                            .draw(display)?;

                        let mut upgrade_text = heapless::String::<32>::new();
                        write!(upgrade_text, "UPGRADE +{}", upgrade + 1).ok();
                        Text::new(&upgrade_text, Point::new(margin + 100, current_y + 35), text_style_value).draw(display)?;

                        self.touch_areas.push(TouchArea {
                            bounds: (margin, current_y, button_width, button_height),
                            action: EquipmentAction::Upgrade(unique_id),
                        });
                        current_y += button_height as i32 + button_spacing;
                    }

                    // Switch button
                    Rectangle::new(Point::new(margin, current_y), Size::new(button_width, button_height))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 100, 60)))
                        .draw(display)?;
                    Text::new("SWITCH", Point::new(margin + 130, current_y + 35), text_style_value).draw(display)?;

                    self.touch_areas.push(TouchArea {
                        bounds: (margin, current_y, button_width, button_height),
                        action: EquipmentAction::Switch,
                    });
                    current_y += button_height as i32 + button_spacing;

                    // Back button
                    Rectangle::new(Point::new(margin, current_y), Size::new(button_width, button_height))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 80, 80)))
                        .draw(display)?;
                    Text::new("BACK", Point::new(margin + 145, current_y + 35), text_style_value).draw(display)?;

                    self.touch_areas.push(TouchArea {
                        bounds: (margin, current_y, button_width, button_height),
                        action: EquipmentAction::Back,
                    });
                }
            }
        } else {
            // Empty slot
            Text::new("(Empty)", Point::new(margin + 5, y), text_style_empty).draw(display)?;
            y += 30;

            Text::new("No item equipped in this slot.", Point::new(margin + 5, y), text_style_label).draw(display)?;

            // Buttons at bottom
            let button_height = 55u32;
            let button_width = 338u32;
            let button_spacing = 5;

            // 2 buttons: 55 + 5 + 55 = 115px total
            // Start at: 480 - 15 (margin) - 115 = 350
            let button_start_y = 350;

            // Equip button
            Rectangle::new(Point::new(margin, button_start_y), Size::new(button_width, button_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 100, 60)))
                .draw(display)?;
            Text::new("EQUIP ITEM", Point::new(margin + 105, button_start_y + 35), text_style_value).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (margin, button_start_y, button_width, button_height),
                action: EquipmentAction::Switch,
            });

            // Back button
            let back_y = button_start_y + button_height as i32 + button_spacing;
            Rectangle::new(Point::new(margin, back_y), Size::new(button_width, button_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 80, 80)))
                .draw(display)?;
            Text::new("BACK", Point::new(margin + 145, back_y + 35), text_style_value).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (margin, back_y, button_width, button_height),
                action: EquipmentAction::Back,
            });
        }

        Ok(())
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
        let item_height = 45;
        let visible_items = 5; // Show 5 items at once (increased height)

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
                    Text::new(&item_text, Point::new(dialog_x + 10, y + 28), text_style).draw(display)?;

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
                        Text::new(&level_text, Point::new(dialog_x + 250, y + 28), level_style).draw(display)?;
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
            let button_y = dialog_y + dialog_height as i32 - 110;
            let button_height = 50u32;
            let button_width = 154u32;

            // Unequip button (left)
            Rectangle::new(Point::new(dialog_x + 10, button_y), Size::new(button_width, button_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(120, 40, 40)))
                .draw(display)?;
            Text::new("Unequip", Point::new(dialog_x + 40, button_y + 32), text_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (dialog_x + 10, button_y, button_width, button_height),
                action: EquipmentAction::SelectSlot(slot), // Special handling for unequip
            });

            // Upgrade button (right) - only if not max level
            if let Some(item) = hero.inventory.get_equipment(equipped_id) {
                let current_level = item.get_upgrade_level();
                if current_level < 10 {
                    Rectangle::new(Point::new(dialog_x + 10 + button_width as i32 + 10, button_y), Size::new(button_width, button_height))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 100, 120)))
                        .draw(display)?;

                    let mut upgrade_text = heapless::String::<16>::new();
                    write!(upgrade_text, "Upgrade +{}", current_level + 1).ok();
                    Text::new(&upgrade_text, Point::new(dialog_x + 180, button_y + 32), text_style).draw(display)?;

                    self.touch_areas.push(TouchArea {
                        bounds: (dialog_x + 10 + button_width as i32 + 10, button_y, button_width, button_height),
                        action: EquipmentAction::Upgrade(equipped_id),
                    });
                }
            }
        }

        // Close button
        let close_y = dialog_y + dialog_height as i32 - 55;
        let close_height = 50u32;
        Rectangle::new(Point::new(dialog_x + 10, close_y), Size::new(308, close_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 80, 80)))
            .draw(display)?;
        Text::new("Close", Point::new(dialog_x + 135, close_y + 32), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (dialog_x + 10, close_y, 308, close_height),
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
