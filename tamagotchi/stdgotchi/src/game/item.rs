//! Item system
//!
//! Defines items, materials, and equipment

use serde::{Deserialize, Serialize};

/// Item categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemCategory {
    Material,
    Equipment,
    Consumable,
}

/// Equipment slot types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Shoes,
    Garment,
    Accessory,
    Headgear,
}

impl EquipmentSlot {
    pub fn all_slots() -> [EquipmentSlot; 6] {
        [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Shoes,
            EquipmentSlot::Garment,
            EquipmentSlot::Accessory,
            EquipmentSlot::Headgear,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            EquipmentSlot::Weapon => "Weapon",
            EquipmentSlot::Armor => "Armor",
            EquipmentSlot::Shoes => "Shoes",
            EquipmentSlot::Garment => "Garment",
            EquipmentSlot::Accessory => "Accessory",
            EquipmentSlot::Headgear => "Headgear",
        }
    }
}

/// Base item definition (from JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub category: ItemCategory,
    pub stack_size: u32,
    pub sell_price: u32,

    // Equipment-specific fields (None for materials)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<EquipmentSlot>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_level: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_atk: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_def: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_hit: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_flee: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_bonus_atk: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_bonus_def: Option<u32>,
}

/// Item instance in inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub item_id: u32,
    pub quantity: u32,

    // Equipment-specific (None for stackable materials)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_level: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_id: Option<u64>, // For tracking individual equipment pieces
}

impl Item {
    /// Create a new material item
    pub fn new_material(item_id: u32, quantity: u32) -> Self {
        Self {
            item_id,
            quantity,
            upgrade_level: None,
            unique_id: None,
        }
    }

    /// Create a new equipment item
    pub fn new_equipment(item_id: u32, unique_id: u64) -> Self {
        Self {
            item_id,
            quantity: 1,
            upgrade_level: Some(0),
            unique_id: Some(unique_id),
        }
    }

    /// Check if item is equipment
    pub fn is_equipment(&self) -> bool {
        self.unique_id.is_some()
    }

    /// Get upgrade level (0 if not equipment)
    pub fn get_upgrade_level(&self) -> u32 {
        self.upgrade_level.unwrap_or(0)
    }

    /// Upgrade equipment by one level
    pub fn upgrade(&mut self) -> Result<(), String> {
        if !self.is_equipment() {
            return Err("Cannot upgrade non-equipment items".to_string());
        }

        let current_level = self.get_upgrade_level();
        if current_level >= 10 {
            return Err("Maximum upgrade level reached".to_string());
        }

        self.upgrade_level = Some(current_level + 1);
        Ok(())
    }

    /// Downgrade equipment by one level (on failed upgrade)
    pub fn downgrade(&mut self) {
        if let Some(level) = self.upgrade_level {
            if level > 0 {
                self.upgrade_level = Some(level - 1);
            }
        }
    }
}

/// Material requirement for crafting/upgrading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRequirement {
    pub item_id: u32,
    pub name: String,
    pub quantity: u32,
}

/// Crafting recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: u32,
    pub result_item_id: u32,
    pub result_item_name: String,
    pub npc: String,
    pub required_level: u32,
    pub gold_cost: u32,
    pub materials: Vec<MaterialRequirement>,
}

/// Upgrade recipe for equipment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeRecipe {
    pub from_level: u32,
    pub to_level: u32,
    pub success_rate: u32, // Percentage (0-100)
    pub gold_cost: u32,
    pub materials: Vec<MaterialRequirement>,
}

/// Item drop definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDrop {
    pub item_id: u32,
    pub name: String,
    pub drop_rate: u32, // Per-mille (0-1000, where 1000 = 100%, 150 = 15%, 5 = 0.5%)
    pub min_quantity: u32,
    pub max_quantity: u32,
}

impl ItemDrop {
    /// Calculate if item should drop (random check)
    pub fn should_drop(&self) -> bool {
        let roll = rand::random::<u32>() % 1000; // Roll 0-999
        roll < self.drop_rate
    }

    /// Get random quantity within range
    pub fn random_quantity(&self) -> u32 {
        if self.min_quantity == self.max_quantity {
            self.min_quantity
        } else {
            let range = self.max_quantity - self.min_quantity + 1;
            self.min_quantity + (rand::random::<u32>() % range)
        }
    }
}
