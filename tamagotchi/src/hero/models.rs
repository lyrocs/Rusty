/// Hero model and core functionality
///
/// Contains the main Hero struct and its core methods for progression,
/// combat, and persistence.

use core::fmt::Write;
use heapless::String;

use super::equipment::Equipment;
use super::inventory::{Inventory, InventoryExt};

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
    pub base_str: u16,  // Strength (affects ATK)
    pub base_agi: u16,  // Agility (affects evasion, double attack, ASPD)
    pub base_vit: u16,  // Vitality (affects HP)
    pub base_int: u16,  // Intelligence (affects SP, magic damage, healing)
    pub base_dex: u16,  // Dexterity (affects accuracy, skill damage)
    pub base_luk: u16,  // Luck (affects critical rate)

    // Stat points available for allocation
    pub stat_points: u16,

    // Equipped items (3 slots)
    pub equipped_weapon: Equipment,
    pub equipped_armor: Equipment,
    pub equipped_accessory: Equipment,
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

            // Starting equipment (Novice gear)
            equipped_weapon: Equipment::starter_weapon_novice(),
            equipped_armor: Equipment::starter_armor_novice(),
            equipped_accessory: Equipment::starter_accessory_novice(),
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

        // Job progression
        if self.level == 10 && self.job == "Novice" {
            self.job = "Swordsman";
            self.name = "Swordsman";
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
        let total_stats = self.base_str + self.base_agi + self.base_vit +
                          self.base_int + self.base_dex + self.base_luk;
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

    /// Serialize hero data to a CSV-like string format
    /// Format: level,exp,exp_to_next,job,hp,max_hp,sp,max_sp,zeny,str,agi,vit,int,dex,luk,stat_points,weapon_id,weapon_refine,armor_id,armor_refine,accessory_id,accessory_refine
    pub fn to_save_string(&self) -> String<256> {
        let mut save_str = String::<256>::new();
        write!(
            save_str,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
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
            self.equipped_accessory.id,
            self.equipped_accessory.refine_level
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

    /// Get equipment from a specific slot
    pub fn get_equipment(&self, slot: super::equipment::EquipmentSlot) -> Option<&Equipment> {
        use super::equipment::EquipmentSlot;
        match slot {
            EquipmentSlot::Weapon => Some(&self.equipped_weapon),
            EquipmentSlot::Armor => Some(&self.equipped_armor),
            EquipmentSlot::Accessory => Some(&self.equipped_accessory),
        }
    }

    /// Refine equipment in a specific slot
    /// Returns Ok((success, new_level)) or Err(insufficient funds)
    /// On failure for risky refines (+5 and above), equipment downgrades 1 level
    pub fn refine_equipment(&mut self, slot: super::equipment::EquipmentSlot, rng_value: u8) -> Result<(bool, u8), &'static str> {
        use super::equipment::EquipmentSlot;

        let equipment = match slot {
            EquipmentSlot::Weapon => &mut self.equipped_weapon,
            EquipmentSlot::Armor => &mut self.equipped_armor,
            EquipmentSlot::Accessory => &mut self.equipped_accessory,
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

        // Try to parse extended format with stats and equipment (13 more fields)
        let (base_str, base_agi, base_vit, base_int, base_dex, base_luk, stat_points, weapon_id, weapon_refine, armor_id, armor_refine, accessory_id, accessory_refine) =
            if let (Some(str_val), Some(agi_val), Some(vit_val), Some(int_val), Some(dex_val), Some(luk_val), Some(pts_val),
                    Some(w_id), Some(w_ref), Some(a_id), Some(a_ref), Some(acc_id), Some(acc_ref)) =
                (parts.next(), parts.next(), parts.next(), parts.next(), parts.next(), parts.next(), parts.next(),
                 parts.next(), parts.next(), parts.next(), parts.next(), parts.next(), parts.next()) {
                // New format with all stats and equipment
                (str_val.parse().ok()?, agi_val.parse().ok()?, vit_val.parse().ok()?, int_val.parse().ok()?,
                 dex_val.parse().ok()?, luk_val.parse().ok()?, pts_val.parse().ok()?,
                 w_id.parse().ok()?, w_ref.parse().ok()?, a_id.parse().ok()?, a_ref.parse().ok()?,
                 acc_id.parse().ok()?, acc_ref.parse().ok()?)
            } else {
                // Old format - initialize with defaults
                (1, 1, 1, 1, 1, 1, if level > 1 { (level - 1) * 3 } else { 0 },
                 1000, 0, 2000, 0, 3000, 0)
            };

        // Parse job to a static string
        let job: &'static str = if job_str == "Novice" {
            "Novice"
        } else {
            "Swordsman"
        };
        let name: &'static str = if job_str == "Novice" {
            "Novice"
        } else {
            "Swordsman"
        };

        // Load equipment by ID
        let mut equipped_weapon = Self::get_equipment_by_id(weapon_id);
        equipped_weapon.refine_level = weapon_refine;

        let mut equipped_armor = Self::get_equipment_by_id(armor_id);
        equipped_armor.refine_level = armor_refine;

        let mut equipped_accessory = Self::get_equipment_by_id(accessory_id);
        equipped_accessory.refine_level = accessory_refine;

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
            equipped_accessory,
        })
    }

    /// Get equipment by ID (simple lookup for now)
    fn get_equipment_by_id(id: u16) -> Equipment {
        match id {
            1000 => Equipment::starter_weapon_novice(),
            2000 => Equipment::starter_armor_novice(),
            3000 => Equipment::starter_accessory_novice(),
            _ => {
                esp_println::println!("[HERO] Unknown equipment ID {}, using starter", id);
                // Return appropriate starter based on ID range
                if id < 2000 {
                    Equipment::starter_weapon_novice()
                } else if id < 3000 {
                    Equipment::starter_armor_novice()
                } else {
                    Equipment::starter_accessory_novice()
                }
            }
        }
    }
}
