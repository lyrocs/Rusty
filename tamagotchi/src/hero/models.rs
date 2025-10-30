/// Hero model and core functionality
///
/// Contains the main Hero struct and its core methods for progression,
/// combat, and persistence.
use core::fmt::Write;
use heapless::String;

use super::equipment::Equipment;
use super::inventory::{Inventory, InventoryExt};

/// Equipment preset for quick-swapping
#[derive(Debug, Clone, Copy, Default)]
pub struct EquipmentPreset {
    pub weapon_id: u16,
    pub weapon_refine: u8,
    pub armor_id: u16,
    pub armor_refine: u8,
    pub shoes_id: u16,
    pub shoes_refine: u8,
    pub garment_id: u16,
    pub garment_refine: u8,
    pub accessory1_id: u16,
    pub accessory1_refine: u8,
    pub accessory2_id: u16,
    pub accessory2_refine: u8,
    // TODO: Add card data when implementing card save/load
}

/// Main hero character
#[derive(Debug, Clone)]
pub struct Hero {
    pub name: &'static str,
    pub level: u16,
    pub exp: u32,
    pub exp_to_next_level: u32,
    pub job: &'static str,
    pub hp: u16,
    pub max_hp: u16,
    pub sp: u16,
    pub max_sp: u16,
    pub zeny: u32,            // Currency
    pub inventory: Inventory, // Item inventory

    // Base stats (allocatable)
    pub base_str: u16, // Strength (affects ATK)
    pub base_agi: u16, // Agility (affects evasion, double attack, ASPD)
    pub base_vit: u16, // Vitality (affects HP)
    pub base_int: u16, // Intelligence (affects SP, magic damage, healing)
    pub base_dex: u16, // Dexterity (affects accuracy, skill damage)
    pub base_luk: u16, // Luck (affects critical rate)

    // Stat points available for allocation
    pub stat_points: u16,

    // Equipped items (6 slots)
    pub equipped_weapon: Equipment,
    pub equipped_armor: Equipment,
    pub equipped_shoes: Equipment,
    pub equipped_garment: Equipment,
    pub equipped_accessory1: Equipment,
    pub equipped_accessory2: Equipment,

    // Equipment presets (3 slots for quick-swap)
    pub equipment_presets: [Option<EquipmentPreset>; 3],
    pub active_preset: Option<u8>, // Which preset is currently active (0-2)
}

impl Hero {
    pub fn new() -> Self {
        Self {
            name: "Novice",
            level: 1,
            exp: 0,
            exp_to_next_level: 100,
            job: "Novice",
            hp: 100,
            max_hp: 100,
            sp: 50,
            max_sp: 50,
            zeny: 0,
            inventory: Inventory::new(),

            // Starting base stats (1 in each)
            base_str: 1,
            base_agi: 1,
            base_vit: 1,
            base_int: 1,
            base_dex: 1,
            base_luk: 1,

            // Starting stat points (level 1 = 0 points, gain 3 per level)
            stat_points: 0,

            // Starting equipment (Novice gear - 6 slots)
            equipped_weapon: Equipment::starter_weapon_novice(),
            equipped_armor: Equipment::starter_armor_novice(),
            equipped_shoes: Equipment::starter_shoes_novice(),
            equipped_garment: Equipment::starter_garment_novice(),
            equipped_accessory1: Equipment::starter_accessory_novice(),
            equipped_accessory2: Equipment::starter_accessory_novice(),

            // Equipment presets (empty initially)
            equipment_presets: [None, None, None],
            active_preset: None,
        }
    }

    /// Add experience and handle level up
    pub fn add_exp(&mut self, amount: u32) {
        self.exp += amount;

        // Check for level up
        while self.exp >= self.exp_to_next_level {
            self.level_up();
        }
    }

    /// Level up the hero
    fn level_up(&mut self) {
        self.level += 1;
        self.exp -= self.exp_to_next_level;

        // Grant stat points (3 per level like Ragnarok Online)
        self.stat_points += 3;

        // Increase base stats
        self.max_hp += 10;
        self.max_sp += 5;
        self.hp = self.max_hp;
        self.sp = self.max_sp;

        // Increase exp requirement
        self.exp_to_next_level = (self.exp_to_next_level as f32 * 1.2) as u32;

        // Job progression (Novice → Swordman at 10, Swordman → Knight at 40)
        if self.level == 10 && self.job == "Novice" {
            self.job = "Swordman";
            self.name = "Swordman";
            esp_println::println!("[LEVEL UP] Hero evolved to Swordman!");
        } else if self.level == 40 && self.job == "Swordman" {
            self.job = "Knight";
            self.name = "Knight";
            esp_println::println!("[LEVEL UP] Hero evolved to Knight!");
        }
    }

    /// Add zeny (currency)
    pub fn add_zeny(&mut self, amount: u32) {
        self.zeny += amount;
    }

    /// Use SP for activities
    pub fn use_sp(&mut self, amount: u16) -> bool {
        if self.sp >= amount {
            self.sp -= amount;
            true
        } else {
            false
        }
    }

    /// Regenerate SP while resting
    pub fn regenerate_sp(&mut self, amount: u16) {
        self.sp = (self.sp + amount).min(self.max_sp);
    }

    /// Add a stat point to a specific stat
    pub fn increase_stat(&mut self, stat_name: &str) -> bool {
        if self.stat_points == 0 {
            return false;
        }

        match stat_name {
            "STR" => self.base_str += 1,
            "AGI" => self.base_agi += 1,
            "VIT" => {
                self.base_vit += 1;
                self.max_hp += 10; // VIT increases HP
                self.hp = self.max_hp;
            }
            "INT" => {
                self.base_int += 1;
                self.max_sp += 5; // INT increases SP
                self.sp = self.max_sp;
            }
            "DEX" => self.base_dex += 1,
            "LUK" => self.base_luk += 1,
            _ => return false,
        }

        self.stat_points -= 1;
        true
    }

    /// Reset all stats (refund all spent stat points)
    pub fn reset_stats(&mut self) {
        let total_stats = self.base_str
            + self.base_agi
            + self.base_vit
            + self.base_int
            + self.base_dex
            + self.base_luk;
        let starting_stats = 6; // 1 in each stat
        let spent_points = total_stats - starting_stats;

        // Reset to base values
        self.base_str = 1;
        self.base_agi = 1;
        self.base_vit = 1;
        self.base_int = 1;
        self.base_dex = 1;
        self.base_luk = 1;

        // Refund points
        self.stat_points += spent_points;

        // Reset HP/SP to base values (level-based only)
        self.max_hp = 100 + ((self.level - 1) * 10);
        self.max_sp = 50 + ((self.level - 1) * 5);
        self.hp = self.max_hp;
        self.sp = self.max_sp;
    }

    /// Add item to inventory (stacks if same item exists)
    pub fn add_item(&mut self, id: u32, name: &'static str, quantity: u16) -> bool {
        self.inventory.add_item(id, name, quantity)
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: u16) {
        self.hp = self.hp.saturating_sub(damage);
    }

    /// Heal HP
    pub fn heal(&mut self, amount: u16) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Check if hero is alive
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Get HP percentage
    pub fn hp_percent(&self) -> u8 {
        ((self.hp as u32 * 100) / self.max_hp as u32) as u8
    }

    /// Get SP percentage
    pub fn sp_percent(&self) -> u8 {
        ((self.sp as u32 * 100) / self.max_sp as u32) as u8
    }

    /// Get EXP percentage
    pub fn exp_percent(&self) -> u8 {
        ((self.exp as u64 * 100) / self.exp_to_next_level as u64) as u8
    }

    /// Calculate total card bonuses from all equipped items
    pub fn get_total_card_bonuses(&self) -> super::equipment::CardEffect {
        use super::equipment::CardEffect;

        let mut total = CardEffect::none();

        // Array of all equipment pieces
        let equipment_pieces = [
            &self.equipped_weapon,
            &self.equipped_armor,
            &self.equipped_shoes,
            &self.equipped_garment,
            &self.equipped_accessory1,
            &self.equipped_accessory2,
        ];

        // Sum bonuses from all cards in all equipment
        for equipment in equipment_pieces.iter() {
            for card_id in equipment.socketed_cards.iter().flatten() {
                if let Some(card) = crate::data::get_card_by_id(*card_id) {
                    total.exp_bonus += card.effects.exp_bonus;
                    total.sp_regen += card.effects.sp_regen;
                    total.aspd_bonus += card.effects.aspd_bonus;
                    total.hp_bonus += card.effects.hp_bonus;
                    total.vit_bonus += card.effects.vit_bonus;
                }
            }
        }

        total
    }

    /// Serialize hero data to a CSV-like string format
    /// Format: level,exp,exp_to_next,job,hp,max_hp,sp,max_sp,zeny,str,agi,vit,int,dex,luk,stat_points,
    ///         weapon_id,weapon_refine,armor_id,armor_refine,shoes_id,shoes_refine,
    ///         garment_id,garment_refine,accessory1_id,accessory1_refine,accessory2_id,accessory2_refine
    pub fn to_save_string(&self) -> String<384> {
        let mut save_str = String::<384>::new();
        write!(
            save_str,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.level,
            self.exp,
            self.exp_to_next_level,
            self.job,
            self.hp,
            self.max_hp,
            self.sp,
            self.max_sp,
            self.zeny,
            self.base_str,
            self.base_agi,
            self.base_vit,
            self.base_int,
            self.base_dex,
            self.base_luk,
            self.stat_points,
            self.equipped_weapon.id,
            self.equipped_weapon.refine_level,
            self.equipped_armor.id,
            self.equipped_armor.refine_level,
            self.equipped_shoes.id,
            self.equipped_shoes.refine_level,
            self.equipped_garment.id,
            self.equipped_garment.refine_level,
            self.equipped_accessory1.id,
            self.equipped_accessory1.refine_level,
            self.equipped_accessory2.id,
            self.equipped_accessory2.refine_level
        )
        .ok();
        save_str
    }

    /// Serialize inventory to a string for saving (item_id:quantity,item_id:quantity,...)
    pub fn inventory_to_save_string(&self) -> String<512> {
        self.inventory.to_save_string()
    }

    /// Deserialize inventory from save string
    pub fn inventory_from_save_string(&mut self, data: &str) {
        self.inventory = Inventory::from_save_string(data);
    }

    /// Serialize equipment details to string (id,refine,card_slots,card0,card1,card2,card3;...)
    /// Format: weapon;armor;shoes;garment;accessory1;accessory2
    pub fn equipment_to_save_string(&self) -> String<256> {
        let mut save_str = String::<256>::new();

        let equipment_pieces = [
            &self.equipped_weapon,
            &self.equipped_armor,
            &self.equipped_shoes,
            &self.equipped_garment,
            &self.equipped_accessory1,
            &self.equipped_accessory2,
        ];

        for (i, eq) in equipment_pieces.iter().enumerate() {
            if i > 0 {
                write!(save_str, ";").ok();
            }
            write!(
                save_str,
                "{},{},{},{},{},{},{}",
                eq.id,
                eq.refine_level,
                eq.card_slots,
                eq.socketed_cards[0].unwrap_or(0),
                eq.socketed_cards[1].unwrap_or(0),
                eq.socketed_cards[2].unwrap_or(0),
                eq.socketed_cards[3].unwrap_or(0),
            ).ok();
        }

        save_str
    }

    /// Deserialize equipment details from save string
    pub fn equipment_from_save_string(&mut self, data: &str) {
        let equipment_parts: heapless::Vec<&str, 6> = data.split(';').collect();

        if equipment_parts.len() < 6 {
            esp_println::println!("[LOAD] Invalid equipment save format (expected 6 pieces)");
            return;
        }

        // Parse each equipment piece
        for (i, eq_str) in equipment_parts.iter().enumerate() {
            let parts: heapless::Vec<&str, 7> = eq_str.split(',').collect();

            if parts.len() != 7 {
                esp_println::println!("[LOAD] Invalid equipment data at index {}", i);
                continue;
            }

            if let (Ok(id), Ok(refine), Ok(slots), Ok(c0), Ok(c1), Ok(c2), Ok(c3)) = (
                parts[0].parse::<u16>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
                parts[3].parse::<u16>(),
                parts[4].parse::<u16>(),
                parts[5].parse::<u16>(),
                parts[6].parse::<u16>(),
            ) {
                // Load equipment by ID
                if let Some(mut equipment) = crate::data::get_equipment_by_id(id) {
                    equipment.refine_level = refine;
                    equipment.card_slots = slots;
                    equipment.socketed_cards = [
                        if c0 > 0 { Some(c0) } else { None },
                        if c1 > 0 { Some(c1) } else { None },
                        if c2 > 0 { Some(c2) } else { None },
                        if c3 > 0 { Some(c3) } else { None },
                    ];

                    // Assign to correct slot
                    match i {
                        0 => self.equipped_weapon = equipment,
                        1 => self.equipped_armor = equipment,
                        2 => self.equipped_shoes = equipment,
                        3 => self.equipped_garment = equipment,
                        4 => self.equipped_accessory1 = equipment,
                        5 => self.equipped_accessory2 = equipment,
                        _ => {}
                    }
                }
            }
        }

        esp_println::println!("[LOAD] Loaded 6 equipment pieces with card data");
    }

    /// Get equipment from a specific slot
    pub fn get_equipment(&self, slot: super::equipment::EquipmentSlot) -> Option<&Equipment> {
        use super::equipment::EquipmentSlot;
        match slot {
            EquipmentSlot::Weapon => Some(&self.equipped_weapon),
            EquipmentSlot::Armor => Some(&self.equipped_armor),
            EquipmentSlot::Shoes => Some(&self.equipped_shoes),
            EquipmentSlot::Garment => Some(&self.equipped_garment),
            EquipmentSlot::Accessory1 => Some(&self.equipped_accessory1),
            EquipmentSlot::Accessory2 => Some(&self.equipped_accessory2),
        }
    }

    /// Refine equipment in a specific slot
    /// Returns Ok((success, new_level)) or Err(insufficient funds)
    /// On failure for risky refines (+5 and above), equipment downgrades 1 level
    pub fn refine_equipment(
        &mut self,
        slot: super::equipment::EquipmentSlot,
        rng_value: u8,
    ) -> Result<(bool, u8), &'static str> {
        use super::equipment::EquipmentSlot;

        let equipment = match slot {
            EquipmentSlot::Weapon => &mut self.equipped_weapon,
            EquipmentSlot::Armor => &mut self.equipped_armor,
            EquipmentSlot::Shoes => &mut self.equipped_shoes,
            EquipmentSlot::Garment => &mut self.equipped_garment,
            EquipmentSlot::Accessory1 => &mut self.equipped_accessory1,
            EquipmentSlot::Accessory2 => &mut self.equipped_accessory2,
        };

        // Check if can refine
        if !equipment.can_refine() {
            return Err("Cannot refine further");
        }

        // Check zeny
        let cost = equipment.refine_cost();
        if self.zeny < cost {
            return Err("Insufficient zeny");
        }

        // Deduct cost
        self.zeny -= cost;

        // Calculate success
        let success_rate = equipment.refine_success_rate() as u16;
        let roll = (rng_value as u16 * 100) / 255;
        let success = roll < success_rate;

        if success {
            equipment.refine_level += 1;
            Ok((true, equipment.refine_level))
        } else {
            // Failure: if risky, downgrade
            if equipment.is_risky_refine() && equipment.refine_level > 0 {
                equipment.refine_level -= 1;
            }
            Ok((false, equipment.refine_level))
        }
    }

    /// Add a card slot to equipment in a specific slot
    /// Costs essences + zeny based on current card slot count
    /// Returns Ok(new_card_slots) or Err(reason)
    pub fn add_card_slot(
        &mut self,
        slot: super::equipment::EquipmentSlot,
        essence_count: u16,
    ) -> Result<u8, &'static str> {
        use super::equipment::EquipmentSlot;

        let equipment = match slot {
            EquipmentSlot::Weapon => &mut self.equipped_weapon,
            EquipmentSlot::Armor => &mut self.equipped_armor,
            EquipmentSlot::Shoes => &mut self.equipped_shoes,
            EquipmentSlot::Garment => &mut self.equipped_garment,
            EquipmentSlot::Accessory1 => &mut self.equipped_accessory1,
            EquipmentSlot::Accessory2 => &mut self.equipped_accessory2,
        };

        // Check if equipment can have more card slots
        if equipment.card_slots >= equipment.max_card_slots {
            return Err("Maximum card slots reached");
        }

        // Calculate cost based on current + next slot
        // 2nd slot: 3 essence + 2000z
        // 3rd slot: 5 essence + 5000z
        // 4th slot: 10 essence + 10000z
        let (required_essence, required_zeny) = match equipment.card_slots {
            1 => (3, 2000),  // Adding 2nd slot
            2 => (5, 5000),  // Adding 3rd slot
            3 => (10, 10000), // Adding 4th slot
            _ => return Err("Invalid card slot count"),
        };

        // Check resources
        if essence_count < required_essence {
            return Err("Not enough essences");
        }
        if self.zeny < required_zeny {
            return Err("Not enough zeny");
        }

        // Deduct costs (essence removal handled by caller from inventory)
        self.zeny -= required_zeny;

        // Add card slot
        equipment.card_slots += 1;

        Ok(equipment.card_slots)
    }

    /// Socket a card into equipment
    /// Returns Ok(slot_index) or Err(reason)
    pub fn socket_card(
        &mut self,
        equipment_slot: super::equipment::EquipmentSlot,
        card_id: u16,
    ) -> Result<usize, &'static str> {
        use super::equipment::EquipmentSlot;

        // Check if card can be socketed in this equipment slot type
        if !crate::data::can_socket_card(card_id, equipment_slot) {
            return Err("Card cannot be socketed in this slot type");
        }

        let equipment = match equipment_slot {
            EquipmentSlot::Weapon => &mut self.equipped_weapon,
            EquipmentSlot::Armor => &mut self.equipped_armor,
            EquipmentSlot::Shoes => &mut self.equipped_shoes,
            EquipmentSlot::Garment => &mut self.equipped_garment,
            EquipmentSlot::Accessory1 => &mut self.equipped_accessory1,
            EquipmentSlot::Accessory2 => &mut self.equipped_accessory2,
        };

        // Find first empty card slot
        for i in 0..(equipment.card_slots as usize) {
            if equipment.socketed_cards[i].is_none() {
                equipment.socketed_cards[i] = Some(card_id);
                return Ok(i);
            }
        }

        Err("No empty card slots")
    }

    /// Remove a card from equipment
    /// Returns Ok(card_id) or Err(reason)
    /// Costs 1000z to remove
    pub fn remove_card(
        &mut self,
        equipment_slot: super::equipment::EquipmentSlot,
        card_slot_index: usize,
    ) -> Result<u16, &'static str> {
        use super::equipment::EquipmentSlot;

        // Check zeny for removal cost
        if self.zeny < 1000 {
            return Err("Not enough zeny (1000z required)");
        }

        let equipment = match equipment_slot {
            EquipmentSlot::Weapon => &mut self.equipped_weapon,
            EquipmentSlot::Armor => &mut self.equipped_armor,
            EquipmentSlot::Shoes => &mut self.equipped_shoes,
            EquipmentSlot::Garment => &mut self.equipped_garment,
            EquipmentSlot::Accessory1 => &mut self.equipped_accessory1,
            EquipmentSlot::Accessory2 => &mut self.equipped_accessory2,
        };

        // Check if card slot index is valid
        if card_slot_index >= equipment.card_slots as usize {
            return Err("Invalid card slot index");
        }

        // Get the card ID
        let card_id = equipment.socketed_cards[card_slot_index]
            .ok_or("No card in this slot")?;

        // Deduct removal cost
        self.zeny -= 1000;

        // Remove the card
        equipment.socketed_cards[card_slot_index] = None;

        Ok(card_id)
    }

    /// Save current equipment setup to a preset slot (0-2)
    pub fn save_equipment_preset(&mut self, preset_index: u8) -> Result<(), &'static str> {
        if preset_index >= 3 {
            return Err("Invalid preset index (must be 0-2)");
        }

        let preset = EquipmentPreset {
            weapon_id: self.equipped_weapon.id,
            weapon_refine: self.equipped_weapon.refine_level,
            armor_id: self.equipped_armor.id,
            armor_refine: self.equipped_armor.refine_level,
            shoes_id: self.equipped_shoes.id,
            shoes_refine: self.equipped_shoes.refine_level,
            garment_id: self.equipped_garment.id,
            garment_refine: self.equipped_garment.refine_level,
            accessory1_id: self.equipped_accessory1.id,
            accessory1_refine: self.equipped_accessory1.refine_level,
            accessory2_id: self.equipped_accessory2.id,
            accessory2_refine: self.equipped_accessory2.refine_level,
        };

        self.equipment_presets[preset_index as usize] = Some(preset);
        self.active_preset = Some(preset_index);

        esp_println::println!("[EQUIPMENT] Saved preset {} successfully", preset_index + 1);
        Ok(())
    }

    /// Load equipment from a preset slot (0-2)
    pub fn load_equipment_preset(&mut self, preset_index: u8) -> Result<(), &'static str> {
        if preset_index >= 3 {
            return Err("Invalid preset index (must be 0-2)");
        }

        let preset = self.equipment_presets[preset_index as usize]
            .ok_or("Preset slot is empty")?;

        // Load all equipment from preset
        self.equipped_weapon = Self::get_equipment_by_id(preset.weapon_id);
        self.equipped_weapon.refine_level = preset.weapon_refine;

        self.equipped_armor = Self::get_equipment_by_id(preset.armor_id);
        self.equipped_armor.refine_level = preset.armor_refine;

        self.equipped_shoes = Self::get_equipment_by_id(preset.shoes_id);
        self.equipped_shoes.refine_level = preset.shoes_refine;

        self.equipped_garment = Self::get_equipment_by_id(preset.garment_id);
        self.equipped_garment.refine_level = preset.garment_refine;

        self.equipped_accessory1 = Self::get_equipment_by_id(preset.accessory1_id);
        self.equipped_accessory1.refine_level = preset.accessory1_refine;

        self.equipped_accessory2 = Self::get_equipment_by_id(preset.accessory2_id);
        self.equipped_accessory2.refine_level = preset.accessory2_refine;

        self.active_preset = Some(preset_index);

        esp_println::println!("[EQUIPMENT] Loaded preset {} successfully", preset_index + 1);
        Ok(())
    }

    /// Clear a preset slot
    pub fn clear_equipment_preset(&mut self, preset_index: u8) -> Result<(), &'static str> {
        if preset_index >= 3 {
            return Err("Invalid preset index (must be 0-2)");
        }

        self.equipment_presets[preset_index as usize] = None;

        if self.active_preset == Some(preset_index) {
            self.active_preset = None;
        }

        esp_println::println!("[EQUIPMENT] Cleared preset {} successfully", preset_index + 1);
        Ok(())
    }

    /// Craft equipment using materials from inventory
    pub fn craft_equipment(&mut self, equipment_id: u16) -> Result<Equipment, &'static str> {
        // Get equipment data with crafting recipe
        let equip_data = crate::data::get_equipment_data_by_id(equipment_id)
            .ok_or("Equipment not found")?;

        // Check if this equipment is craftable
        let craft_materials = equip_data.craft_materials.as_ref()
            .ok_or("This equipment cannot be crafted")?;

        // Check level requirement
        if self.level < equip_data.level_req {
            return Err("Level too low for this equipment");
        }

        // Check zeny cost
        if self.zeny < equip_data.craft_cost {
            return Err("Not enough Zeny");
        }

        // Check all material requirements
        for (material_id, required_qty) in craft_materials.iter() {
            if !self.inventory.has_item(*material_id, *required_qty) {
                return Err("Not enough materials");
            }
        }

        // All checks passed - consume materials
        for (material_id, required_qty) in craft_materials.iter() {
            if !self.inventory.remove_item(*material_id, *required_qty) {
                // This shouldn't happen since we checked above, but handle it safely
                return Err("Failed to consume materials");
            }
        }

        // Deduct zeny cost
        self.zeny = self.zeny.saturating_sub(equip_data.craft_cost);

        // Create the equipment
        let equipment = crate::data::get_equipment_by_id(equipment_id)
            .ok_or("Failed to create equipment")?;

        esp_println::println!("[CRAFT] Successfully crafted {} for {}z", equipment.name, equip_data.craft_cost);
        Ok(equipment)
    }

    /// Swap equipped item with an item from inventory
    pub fn swap_equipment(&mut self, slot: crate::hero::equipment::EquipmentSlot, new_equipment_id: u16) -> Result<(), &'static str> {
        use crate::hero::equipment::EquipmentSlot;
        use crate::hero::inventory::InventoryExt;

        // Get the currently equipped item in this slot
        let current_equipment = match slot {
            EquipmentSlot::Weapon => &self.equipped_weapon,
            EquipmentSlot::Armor => &self.equipped_armor,
            EquipmentSlot::Shoes => &self.equipped_shoes,
            EquipmentSlot::Garment => &self.equipped_garment,
            EquipmentSlot::Accessory1 => &self.equipped_accessory1,
            EquipmentSlot::Accessory2 => &self.equipped_accessory2,
        };

        let current_id = current_equipment.id;
        let current_name = current_equipment.name;

        // Check if the new equipment exists in inventory
        if !self.inventory.has_item(new_equipment_id as u32, 1) {
            return Err("Equipment not in inventory");
        }

        // Remove new equipment from inventory
        if !self.inventory.remove_item(new_equipment_id as u32, 1) {
            return Err("Failed to remove equipment from inventory");
        }

        // Add current equipment back to inventory (if it's not starter equipment)
        if current_id >= 1000 {
            self.inventory.add_item(current_id as u32, current_name, 1);
        }

        // Load and equip the new equipment
        let new_equipment = crate::data::get_equipment_by_id(new_equipment_id)
            .ok_or("Failed to load new equipment")?;

        match slot {
            EquipmentSlot::Weapon => self.equipped_weapon = new_equipment,
            EquipmentSlot::Armor => self.equipped_armor = new_equipment,
            EquipmentSlot::Shoes => self.equipped_shoes = new_equipment,
            EquipmentSlot::Garment => self.equipped_garment = new_equipment,
            EquipmentSlot::Accessory1 => self.equipped_accessory1 = new_equipment,
            EquipmentSlot::Accessory2 => self.equipped_accessory2 = new_equipment,
        }

        esp_println::println!("[EQUIPMENT] Swapped {} for {}", current_name, new_equipment.name);
        Ok(())
    }

    /// Deserialize hero data from a CSV-like string
    pub fn from_save_string(data: &str) -> Option<Self> {
        // Trim whitespace and newlines first
        let data = data.trim();

        // Use splitn to limit splits and avoid overflow
        let mut parts = data.split(',');

        // Parse basic fields (9 fields)
        let level: u16 = parts.next()?.parse().ok()?;
        let exp: u32 = parts.next()?.parse().ok()?;
        let exp_to_next_level: u32 = parts.next()?.parse().ok()?;
        let job_str = parts.next()?;
        let hp: u16 = parts.next()?.parse().ok()?;
        let max_hp: u16 = parts.next()?.parse().ok()?;
        let sp: u16 = parts.next()?.parse().ok()?;
        let max_sp: u16 = parts.next()?.parse().ok()?;
        let zeny: u32 = parts.next()?.parse().ok()?;

        // Try to parse extended format with stats and equipment
        // New format has 19 fields for equipment (6 items * 2 fields + 7 stats)
        let (
            base_str,
            base_agi,
            base_vit,
            base_int,
            base_dex,
            base_luk,
            stat_points,
            weapon_id,
            weapon_refine,
            armor_id,
            armor_refine,
            shoes_id,
            shoes_refine,
            garment_id,
            garment_refine,
            accessory1_id,
            accessory1_refine,
            accessory2_id,
            accessory2_refine,
        ) = if let (
            Some(str_val),
            Some(agi_val),
            Some(vit_val),
            Some(int_val),
            Some(dex_val),
            Some(luk_val),
            Some(pts_val),
            Some(w_id),
            Some(w_ref),
            Some(a_id),
            Some(a_ref),
            Some(sh_id),
            Some(sh_ref),
            Some(g_id),
            Some(g_ref),
            Some(acc1_id),
            Some(acc1_ref),
            Some(acc2_id),
            Some(acc2_ref),
        ) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            // New format with all 6 equipment slots
            (
                str_val.parse().ok()?,
                agi_val.parse().ok()?,
                vit_val.parse().ok()?,
                int_val.parse().ok()?,
                dex_val.parse().ok()?,
                luk_val.parse().ok()?,
                pts_val.parse().ok()?,
                w_id.parse().ok()?,
                w_ref.parse().ok()?,
                a_id.parse().ok()?,
                a_ref.parse().ok()?,
                sh_id.parse().ok()?,
                sh_ref.parse().ok()?,
                g_id.parse().ok()?,
                g_ref.parse().ok()?,
                acc1_id.parse().ok()?,
                acc1_ref.parse().ok()?,
                acc2_id.parse().ok()?,
                acc2_ref.parse().ok()?,
            )
        } else {
            // Old format (3 slots) - try to parse that
            let mut parts_copy = data.split(',').skip(9); // Skip first 9 fields

            if let (
                Some(str_val),
                Some(agi_val),
                Some(vit_val),
                Some(int_val),
                Some(dex_val),
                Some(luk_val),
                Some(pts_val),
                Some(w_id),
                Some(w_ref),
                Some(a_id),
                Some(a_ref),
                Some(acc_id),
                Some(acc_ref),
            ) = (
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
                parts_copy.next(),
            ) {
                // Old format - add default shoes and garment
                (
                    str_val.parse().ok()?,
                    agi_val.parse().ok()?,
                    vit_val.parse().ok()?,
                    int_val.parse().ok()?,
                    dex_val.parse().ok()?,
                    luk_val.parse().ok()?,
                    pts_val.parse().ok()?,
                    w_id.parse().ok()?,
                    w_ref.parse().ok()?,
                    a_id.parse().ok()?,
                    a_ref.parse().ok()?,
                    3000,  // Default shoes
                    0,
                    4000,  // Default garment
                    0,
                    acc_id.parse().ok()?,
                    acc_ref.parse().ok()?,
                    5000,  // Default second accessory
                    0,
                )
            } else {
                // Very old format - initialize with all defaults
                (
                    1,
                    1,
                    1,
                    1,
                    1,
                    1,
                    if level > 1 { (level - 1) * 3 } else { 0 },
                    1000,
                    0,
                    2000,
                    0,
                    3000,
                    0,
                    4000,
                    0,
                    5000,
                    0,
                    5000,
                    0,
                )
            }
        };

        // Parse job to a static string
        let job: &'static str = if job_str == "Novice" {
            "Novice"
        } else {
            "Swordman"
        };
        let name: &'static str = if job_str == "Novice" {
            "Novice"
        } else {
            "Swordman"
        };

        // Load equipment by ID (all 6 slots)
        let mut equipped_weapon = Self::get_equipment_by_id(weapon_id);
        equipped_weapon.refine_level = weapon_refine;

        let mut equipped_armor = Self::get_equipment_by_id(armor_id);
        equipped_armor.refine_level = armor_refine;

        let mut equipped_shoes = Self::get_equipment_by_id(shoes_id);
        equipped_shoes.refine_level = shoes_refine;

        let mut equipped_garment = Self::get_equipment_by_id(garment_id);
        equipped_garment.refine_level = garment_refine;

        let mut equipped_accessory1 = Self::get_equipment_by_id(accessory1_id);
        equipped_accessory1.refine_level = accessory1_refine;

        let mut equipped_accessory2 = Self::get_equipment_by_id(accessory2_id);
        equipped_accessory2.refine_level = accessory2_refine;

        Some(Hero {
            name,
            level,
            exp,
            exp_to_next_level,
            job,
            hp,
            max_hp,
            sp,
            max_sp,
            zeny,
            inventory: Inventory::new(),
            base_str,
            base_agi,
            base_vit,
            base_int,
            base_dex,
            base_luk,
            stat_points,
            equipped_weapon,
            equipped_armor,
            equipped_shoes,
            equipped_garment,
            equipped_accessory1,
            equipped_accessory2,
            equipment_presets: [None, None, None], // TODO: Save/load presets
            active_preset: None,
        })
    }

    /// Get equipment by ID (loads from JSON data)
    fn get_equipment_by_id(id: u16) -> Equipment {
        crate::data::get_equipment_by_id(id).unwrap_or_else(|| {
            esp_println::println!("[HERO] Unknown equipment ID {}, using default starter", id);
            // Return appropriate starter based on ID range
            if id < 2000 {
                Equipment::starter_weapon_novice()
            } else if id < 3000 {
                Equipment::starter_armor_novice()
            } else if id < 4000 {
                Equipment::starter_shoes_novice()
            } else if id < 5000 {
                Equipment::starter_garment_novice()
            } else {
                Equipment::starter_accessory_novice()
            }
        })
    }
}
