//! Equipment system
//!
//! Manages equipped items and their stat bonuses

use super::item::{EquipmentSlot, Item, ItemData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Equipment manager - tracks what's currently equipped
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquippedItems {
    weapon: Option<u64>,     // Unique ID of equipped weapon
    armor: Option<u64>,      // Unique ID of equipped armor
    shoes: Option<u64>,      // Unique ID of equipped shoes
    garment: Option<u64>,    // Unique ID of equipped garment
    accessory: Option<u64>,  // Unique ID of equipped accessory
    headgear: Option<u64>,   // Unique ID of equipped headgear
}

impl EquippedItems {
    /// Create new empty equipment slots
    pub fn new() -> Self {
        Self {
            weapon: None,
            armor: None,
            shoes: None,
            garment: None,
            accessory: None,
            headgear: None,
        }
    }

    /// Get equipped item ID for a slot
    pub fn get_slot(&self, slot: EquipmentSlot) -> Option<u64> {
        match slot {
            EquipmentSlot::Weapon => self.weapon,
            EquipmentSlot::Armor => self.armor,
            EquipmentSlot::Shoes => self.shoes,
            EquipmentSlot::Garment => self.garment,
            EquipmentSlot::Accessory => self.accessory,
            EquipmentSlot::Headgear => self.headgear,
        }
    }

    /// Set equipped item for a slot
    pub fn set_slot(&mut self, slot: EquipmentSlot, unique_id: Option<u64>) {
        match slot {
            EquipmentSlot::Weapon => self.weapon = unique_id,
            EquipmentSlot::Armor => self.armor = unique_id,
            EquipmentSlot::Shoes => self.shoes = unique_id,
            EquipmentSlot::Garment => self.garment = unique_id,
            EquipmentSlot::Accessory => self.accessory = unique_id,
            EquipmentSlot::Headgear => self.headgear = unique_id,
        }
    }

    /// Equip an item (returns previously equipped item's unique_id if any)
    pub fn equip(&mut self, slot: EquipmentSlot, unique_id: u64) -> Option<u64> {
        let previous = self.get_slot(slot);
        self.set_slot(slot, Some(unique_id));
        previous
    }

    /// Unequip an item (returns the unique_id if something was equipped)
    pub fn unequip(&mut self, slot: EquipmentSlot) -> Option<u64> {
        let previous = self.get_slot(slot);
        self.set_slot(slot, None);
        previous
    }

    /// Check if a slot is empty
    pub fn is_slot_empty(&self, slot: EquipmentSlot) -> bool {
        self.get_slot(slot).is_none()
    }

    /// Get all equipped unique IDs
    pub fn all_equipped_ids(&self) -> Vec<u64> {
        let mut ids = Vec::new();
        if let Some(id) = self.weapon {
            ids.push(id);
        }
        if let Some(id) = self.armor {
            ids.push(id);
        }
        if let Some(id) = self.shoes {
            ids.push(id);
        }
        if let Some(id) = self.garment {
            ids.push(id);
        }
        if let Some(id) = self.accessory {
            ids.push(id);
        }
        if let Some(id) = self.headgear {
            ids.push(id);
        }
        ids
    }
}

impl Default for EquippedItems {
    fn default() -> Self {
        Self::new()
    }
}

/// Equipment stats calculated from all equipped items
#[derive(Debug, Clone, Default)]
pub struct EquipmentStats {
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
}

impl EquipmentStats {
    /// Calculate total equipment stats from equipped items
    pub fn calculate(
        equipped: &EquippedItems,
        inventory_items: &[Item],
        item_database: &HashMap<u32, ItemData>,
    ) -> Self {
        let mut stats = EquipmentStats::default();

        // Get all equipped unique IDs
        for unique_id in equipped.all_equipped_ids() {
            // Find the item in inventory
            if let Some(item) = inventory_items.iter().find(|i| i.unique_id == Some(unique_id)) {
                // Get item data from database
                if let Some(item_data) = item_database.get(&item.item_id) {
                    let upgrade_level = item.get_upgrade_level();

                    // Add base stats
                    stats.atk += item_data.base_atk.unwrap_or(0);
                    stats.def += item_data.base_def.unwrap_or(0);
                    stats.hit += item_data.base_hit.unwrap_or(0);
                    stats.flee += item_data.base_flee.unwrap_or(0);

                    // Add upgrade bonuses
                    if upgrade_level > 0 {
                        stats.atk += item_data.upgrade_bonus_atk.unwrap_or(0) * upgrade_level;
                        stats.def += item_data.upgrade_bonus_def.unwrap_or(0) * upgrade_level;
                    }
                }
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equip_unequip() {
        let mut equipped = EquippedItems::new();

        // Initially empty
        assert!(equipped.is_slot_empty(EquipmentSlot::Weapon));

        // Equip weapon
        let weapon_id = 1001;
        let previous = equipped.equip(EquipmentSlot::Weapon, weapon_id);
        assert!(previous.is_none());
        assert_eq!(equipped.get_slot(EquipmentSlot::Weapon), Some(weapon_id));

        // Equip new weapon (should return old one)
        let new_weapon_id = 1002;
        let previous = equipped.equip(EquipmentSlot::Weapon, new_weapon_id);
        assert_eq!(previous, Some(weapon_id));
        assert_eq!(equipped.get_slot(EquipmentSlot::Weapon), Some(new_weapon_id));

        // Unequip
        let unequipped = equipped.unequip(EquipmentSlot::Weapon);
        assert_eq!(unequipped, Some(new_weapon_id));
        assert!(equipped.is_slot_empty(EquipmentSlot::Weapon));
    }

    #[test]
    fn test_all_equipped_ids() {
        let mut equipped = EquippedItems::new();

        equipped.equip(EquipmentSlot::Weapon, 1);
        equipped.equip(EquipmentSlot::Armor, 2);
        equipped.equip(EquipmentSlot::Headgear, 3);

        let ids = equipped.all_equipped_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
    }
}
