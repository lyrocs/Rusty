//! Inventory system
//!
//! Manages player inventory and item stacking

use super::item::{Item, ItemCategory, ItemData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum inventory slots
pub const MAX_INVENTORY_SLOTS: usize = 30;

/// Inventory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<Item>,
    next_unique_id: u64,
}

impl Inventory {
    /// Create a new empty inventory
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_unique_id: 1,
        }
    }

    /// Get all items
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// Get mutable reference to items
    pub fn items_mut(&mut self) -> &mut Vec<Item> {
        &mut self.items
    }

    /// Get number of items (counting stacks as one)
    pub fn slot_count(&self) -> usize {
        self.items.len()
    }

    /// Check if inventory is full
    pub fn is_full(&self) -> bool {
        self.items.len() >= MAX_INVENTORY_SLOTS
    }

    /// Add a material item (stackable)
    pub fn add_material(&mut self, item_id: u32, quantity: u32, item_data: &HashMap<u32, ItemData>) -> Result<(), String> {
        // Find existing stack
        if let Some(existing) = self.items.iter_mut().find(|item| {
            item.item_id == item_id && !item.is_equipment()
        }) {
            // Get max stack size
            let max_stack = item_data
                .get(&item_id)
                .map(|data| data.stack_size)
                .unwrap_or(999);

            let new_quantity = existing.quantity + quantity;
            if new_quantity > max_stack {
                return Err(format!("Stack size exceeded (max: {})", max_stack));
            }

            existing.quantity = new_quantity;
            log::info!("Added {} x{} to existing stack (total: {})", item_id, quantity, new_quantity);
            Ok(())
        } else {
            // Create new stack
            if self.is_full() {
                return Err("Inventory is full".to_string());
            }

            self.items.push(Item::new_material(item_id, quantity));
            log::info!("Added new item {} x{}", item_id, quantity);
            Ok(())
        }
    }

    /// Add an equipment item (non-stackable)
    pub fn add_equipment(&mut self, item_id: u32) -> Result<u64, String> {
        if self.is_full() {
            return Err("Inventory is full".to_string());
        }

        let unique_id = self.next_unique_id;
        self.next_unique_id += 1;

        self.items.push(Item::new_equipment(item_id, unique_id));
        log::info!("Added equipment {} (unique_id: {})", item_id, unique_id);
        Ok(unique_id)
    }

    /// Remove material by quantity
    pub fn remove_material(&mut self, item_id: u32, quantity: u32) -> Result<(), String> {
        if let Some(index) = self.items.iter().position(|item| {
            item.item_id == item_id && !item.is_equipment()
        }) {
            let item = &mut self.items[index];

            if item.quantity < quantity {
                return Err(format!("Not enough {} (have: {}, need: {})", item_id, item.quantity, quantity));
            }

            item.quantity -= quantity;
            log::info!("Removed {} x{}", item_id, quantity);

            // Remove empty stacks
            if item.quantity == 0 {
                self.items.remove(index);
                log::info!("Removed empty stack of {}", item_id);
            }

            Ok(())
        } else {
            Err(format!("Item {} not found in inventory", item_id))
        }
    }

    /// Remove equipment by unique ID
    pub fn remove_equipment(&mut self, unique_id: u64) -> Result<Item, String> {
        if let Some(index) = self.items.iter().position(|item| {
            item.unique_id == Some(unique_id)
        }) {
            let item = self.items.remove(index);
            log::info!("Removed equipment with unique_id: {}", unique_id);
            Ok(item)
        } else {
            Err(format!("Equipment with unique_id {} not found", unique_id))
        }
    }

    /// Get equipment by unique ID
    pub fn get_equipment(&self, unique_id: u64) -> Option<&Item> {
        self.items.iter().find(|item| item.unique_id == Some(unique_id))
    }

    /// Get mutable equipment by unique ID
    pub fn get_equipment_mut(&mut self, unique_id: u64) -> Option<&mut Item> {
        self.items.iter_mut().find(|item| item.unique_id == Some(unique_id))
    }

    /// Get quantity of a material
    pub fn get_material_quantity(&self, item_id: u32) -> u32 {
        self.items
            .iter()
            .find(|item| item.item_id == item_id && !item.is_equipment())
            .map(|item| item.quantity)
            .unwrap_or(0)
    }

    /// Check if player has required materials
    pub fn has_materials(&self, requirements: &[(u32, u32)]) -> bool {
        requirements.iter().all(|(item_id, quantity)| {
            self.get_material_quantity(*item_id) >= *quantity
        })
    }

    /// Consume materials for crafting/upgrading
    pub fn consume_materials(&mut self, requirements: &[(u32, u32)]) -> Result<(), String> {
        // First check if we have all materials
        for (item_id, quantity) in requirements {
            if self.get_material_quantity(*item_id) < *quantity {
                return Err(format!("Not enough item {}", item_id));
            }
        }

        // Consume all materials
        for (item_id, quantity) in requirements {
            self.remove_material(*item_id, *quantity)?;
        }

        Ok(())
    }

    /// Get all equipment items
    pub fn get_all_equipment(&self) -> Vec<&Item> {
        self.items.iter().filter(|item| item.is_equipment()).collect()
    }

    /// Get all material items
    pub fn get_all_materials(&self) -> Vec<&Item> {
        self.items.iter().filter(|item| !item.is_equipment()).collect()
    }

    /// Sort inventory (materials first, then equipment)
    pub fn sort(&mut self) {
        self.items.sort_by(|a, b| {
            match (a.is_equipment(), b.is_equipment()) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.item_id.cmp(&b.item_id),
            }
        });
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
