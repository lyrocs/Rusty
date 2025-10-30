/// Inventory management system
///
/// Handles item storage, stacking, and serialization.

use heapless::Vec as HeaplessVec;
use core::fmt::Write;
use heapless::String;

/// Item in inventory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub id: u32,
    pub name: &'static str,
    pub quantity: u16,
}

impl Item {
    pub fn new(id: u32, name: &'static str, quantity: u16) -> Self {
        Self { id, name, quantity }
    }
}

/// Inventory with max 50 unique items
pub type Inventory = HeaplessVec<Item, 50>;

/// Inventory management methods
pub trait InventoryExt {
    /// Add item to inventory (stacks if same item exists)
    fn add_item(&mut self, id: u32, name: &'static str, quantity: u16) -> bool;

    /// Remove item from inventory (returns true if successful)
    fn remove_item(&mut self, id: u32, quantity: u16) -> bool;

    /// Check if inventory has enough of an item
    fn has_item(&self, id: u32, quantity: u16) -> bool;

    /// Serialize inventory to a string for saving (item_id:quantity,item_id:quantity,...)
    fn to_save_string(&self) -> String<512>;

    /// Deserialize inventory from save string
    fn from_save_string(data: &str) -> Self;
}

impl InventoryExt for Inventory {
    fn add_item(&mut self, id: u32, name: &'static str, quantity: u16) -> bool {
        // Check if item already exists in inventory
        for item in self.iter_mut() {
            if item.id == id {
                // Stack the item
                item.quantity = item.quantity.saturating_add(quantity);
                esp_println::println!(
                    "[INVENTORY] Added {} x{} (total: {})",
                    name,
                    quantity,
                    item.quantity
                );
                return true;
            }
        }

        // Add as new item
        let new_item = Item::new(id, name, quantity);
        match self.push(new_item) {
            Ok(_) => {
                esp_println::println!("[INVENTORY] Added new item: {} x{}", name, quantity);
                true
            }
            Err(_) => {
                esp_println::println!("[INVENTORY] Inventory full! Cannot add {}", name);
                false
            }
        }
    }

    fn remove_item(&mut self, id: u32, quantity: u16) -> bool {
        // Find the item
        if let Some(pos) = self.iter().position(|item| item.id == id) {
            let item = &mut self[pos];

            if item.quantity >= quantity {
                item.quantity -= quantity;
                esp_println::println!("[INVENTORY] Removed {} x{} (remaining: {})", item.name, quantity, item.quantity);

                // Remove item completely if quantity reaches 0
                if item.quantity == 0 {
                    self.swap_remove(pos);
                    esp_println::println!("[INVENTORY] Item removed from inventory (quantity 0)");
                }

                return true;
            } else {
                esp_println::println!("[INVENTORY] Not enough {} (have {}, need {})", item.name, item.quantity, quantity);
                return false;
            }
        }

        esp_println::println!("[INVENTORY] Item {} not found in inventory", id);
        false
    }

    fn has_item(&self, id: u32, quantity: u16) -> bool {
        self.iter()
            .find(|item| item.id == id)
            .map(|item| item.quantity >= quantity)
            .unwrap_or(false)
    }

    fn to_save_string(&self) -> String<512> {
        let mut save_str = String::<512>::new();
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                write!(save_str, ",").ok();
            }
            write!(save_str, "{}:{}", item.id, item.quantity).ok();
        }
        save_str
    }

    fn from_save_string(data: &str) -> Self {
        use crate::tamagotchi::get_item_name;

        let mut inventory = Inventory::new();
        let data = data.trim();
        if data.is_empty() {
            return inventory;
        }

        for pair in data.split(',') {
            if let Some((id_str, qty_str)) = pair.split_once(':') {
                if let (Ok(id), Ok(quantity)) = (id_str.parse::<u32>(), qty_str.parse::<u16>()) {
                    // Find item name from game data
                    let item_name = get_item_name(id);
                    inventory.add_item(id, item_name, quantity);
                }
            }
        }

        inventory
    }
}
