/// Equipment system for the hero
///
/// Handles equipment slots, types, stats, refinement system, and card socketing.

/// Card effects that can be applied to equipment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardEffect {
    pub exp_bonus: u8,        // +X% EXP gain
    pub sp_regen: u8,          // +X SP regen per second
    pub aspd_bonus: u8,        // +X% ASPD
    pub hp_bonus: u16,         // +X HP
    pub vit_bonus: u8,         // +X VIT
}

impl CardEffect {
    pub fn none() -> Self {
        Self {
            exp_bonus: 0,
            sp_regen: 0,
            aspd_bonus: 0,
            hp_bonus: 0,
            vit_bonus: 0,
        }
    }
}

/// Represents a card that can be socketed into equipment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Card {
    pub id: u16,
    pub name: &'static str,
    pub allowed_slot: EquipmentSlot, // Which slot type this card can go in
    pub effects: CardEffect,
}

/// Equipment slot types (6 total)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Shoes,
    Garment,
    Accessory1,
    Accessory2,
}

/// Equipment type determines what slot it goes in and stat bonuses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentType {
    // Weapons (Weapon slot)
    Knife,
    Sword,
    Spear,
    Axe,
    Bow,
    Staff,

    // Armor (Armor slot)
    ClothArmor,
    LeatherArmor,
    ChainMail,
    PlateMail,

    // Shoes (Shoes slot)
    Shoes,
    Boots,
    Sandals,

    // Garment (Garment slot)
    Garment,
    Cape,
    Mantle,

    // Accessories (Accessory1/Accessory2 slots)
    Ring,
    Necklace,
    Gloves,
}

impl EquipmentType {
    /// Get the slot this equipment type goes in
    pub fn slot(&self) -> EquipmentSlot {
        match self {
            EquipmentType::Knife
            | EquipmentType::Sword
            | EquipmentType::Spear
            | EquipmentType::Axe
            | EquipmentType::Bow
            | EquipmentType::Staff => EquipmentSlot::Weapon,
            EquipmentType::ClothArmor
            | EquipmentType::LeatherArmor
            | EquipmentType::ChainMail
            | EquipmentType::PlateMail => EquipmentSlot::Armor,
            EquipmentType::Shoes
            | EquipmentType::Boots
            | EquipmentType::Sandals => EquipmentSlot::Shoes,
            EquipmentType::Garment
            | EquipmentType::Cape
            | EquipmentType::Mantle => EquipmentSlot::Garment,
            EquipmentType::Ring | EquipmentType::Necklace | EquipmentType::Gloves => {
                EquipmentSlot::Accessory1  // Default to first accessory slot
            }
        }
    }
}

/// Equipment item with stats, refinement, and card slots
#[derive(Debug, Clone, Copy)]
pub struct Equipment {
    pub id: u16,
    pub name: &'static str,
    pub equipment_type: EquipmentType,
    pub slot: EquipmentSlot,

    // Level requirement
    pub level_req: u16,
    pub job_req: Option<&'static str>, // None = all jobs

    // Base stats (before refinement)
    pub atk_bonus: u16,
    pub def_bonus: u16,
    pub hp_bonus: u16,
    pub sp_bonus: u16,

    // Stat bonuses
    pub str_bonus: i16, // Can be negative (heavy armor penalty)
    pub agi_bonus: i16,
    pub vit_bonus: i16,
    pub int_bonus: i16,
    pub dex_bonus: i16,
    pub luk_bonus: i16,

    // Special bonuses
    pub crit_rate_bonus: u16, // +X% crit rate
    pub aspd_bonus: u16,      // +X% attack speed
    pub flee_bonus: u16,      // +X flee (dodge)
    pub hit_bonus: u16,       // +X hit rate
    pub damage_reduction: u16, // +X% damage reduction

    // Refinement data
    pub refine_level: u8,  // 0 to 10 (+0 to +10)
    pub max_refine: u8,    // Usually 10

    // Card system
    pub card_slots: u8,      // Current number of card slots (1-4)
    pub max_card_slots: u8,  // Maximum card slots for this equipment
    pub socketed_cards: [Option<u16>; 4], // Card IDs socketed in each slot

    // Upgrade path (evolution)
    pub can_upgrade: bool,
    pub upgrade_level_req: u16,   // Level needed to upgrade
    pub upgrade_cost: u32,        // Zeny cost
    pub upgrades_to: Option<u16>, // Equipment ID it upgrades to
}

impl Equipment {
    /// Create starter weapon for Novice (loads from JSON)
    pub fn starter_weapon_novice() -> Self {
        crate::data::get_equipment_by_id(1000)
            .expect("Failed to load starter weapon (ID: 1000)")
    }

    /// Create starter armor for Novice (loads from JSON)
    pub fn starter_armor_novice() -> Self {
        crate::data::get_equipment_by_id(2000)
            .expect("Failed to load starter armor (ID: 2000)")
    }

    /// Create starter shoes for Novice (loads from JSON)
    pub fn starter_shoes_novice() -> Self {
        crate::data::get_equipment_by_id(3000)
            .expect("Failed to load starter shoes (ID: 3000)")
    }

    /// Create starter garment for Novice (loads from JSON)
    pub fn starter_garment_novice() -> Self {
        crate::data::get_equipment_by_id(4000)
            .expect("Failed to load starter garment (ID: 4000)")
    }

    /// Create starter accessory for Novice (loads from JSON)
    pub fn starter_accessory_novice() -> Self {
        crate::data::get_equipment_by_id(5000)
            .expect("Failed to load starter accessory (ID: 5000)")
    }

    /// Get refine bonus based on slot and refine level
    pub fn get_refine_bonus(&self) -> u16 {
        match self.slot {
            EquipmentSlot::Weapon => self.refine_level as u16 * 2,  // +2 ATK per level
            EquipmentSlot::Armor => self.refine_level as u16 * 1,   // +1 DEF per level
            EquipmentSlot::Shoes => self.refine_level as u16 * 1,   // +1 AGI per level
            EquipmentSlot::Garment => self.refine_level as u16 * 1, // +1 DEF per level
            EquipmentSlot::Accessory1 => self.refine_level as u16 * 1, // +1 to primary stat
            EquipmentSlot::Accessory2 => self.refine_level as u16 * 1, // +1 to primary stat
        }
    }

    /// Get total ATK including refine bonus
    pub fn total_atk(&self) -> u16 {
        if self.slot == EquipmentSlot::Weapon {
            self.atk_bonus + self.get_refine_bonus()
        } else {
            self.atk_bonus
        }
    }

    /// Get total DEF including refine bonus
    pub fn total_def(&self) -> u16 {
        if self.slot == EquipmentSlot::Armor || self.slot == EquipmentSlot::Garment {
            self.def_bonus + self.get_refine_bonus()
        } else {
            self.def_bonus
        }
    }

    /// Calculate refine cost based on current level
    pub fn refine_cost(&self) -> u32 {
        100 * (self.refine_level as u32 + 1)
    }

    /// Calculate refine success rate based on current level
    pub fn refine_success_rate(&self) -> u8 {
        match self.refine_level {
            0..=3 => 100,  // +0 to +3: 100% safe
            4 => 100,      // +4: still safe
            5..=6 => 80,   // +5 to +6: 80%
            7 => 80,       // +7: 80%
            8 => 60,       // +8: 60%
            9 => 40,       // +9: 40%
            _ => 0,        // +10: cannot refine further
        }
    }

    /// Check if equipment can be refined further
    pub fn can_refine(&self) -> bool {
        self.refine_level < self.max_refine
    }

    /// Check if current refine level is risky (can drop on failure)
    pub fn is_risky_refine(&self) -> bool {
        self.refine_level >= 5
    }
}
