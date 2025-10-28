use bevy_ecs::prelude::*;
use core::fmt::Write;
use heapless::String;
use heapless::Vec as HeaplessVec;
use serde::Deserialize;

// Game data functions are re-exported from tamagotchi::game_data
use crate::tamagotchi::{
    MAP_PRONTERA_ID, get_city_npcs, get_enemy_data, get_item_name, get_map_connections,
    get_map_enemies, get_map_name, is_city,
};

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

/// Game pages/screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePage {
    Overview,
    Farm,
    Rest,
    Battle,     // Whac-A-Mole mini-game
    JrpgBattle, // Turn-based JRPG battle
    Map,        // Navigation and world map
    Menu,
    Inventory,  // Item inventory
    Quests,     // Quest list and management
    Settings,   // Settings page (brightness, etc.)
    Stats,      // Character stats allocation page
    Equipment,  // Equipment management page
}

/// Equipment slot types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Accessory,
}

/// Equipment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentType {
    // Weapons
    Sword,
    Staff,
    Bow,
    Dagger,
    Axe,
    Mace,
    Knife,

    // Armor
    ClothArmor,
    LeatherArmor,
    PlateArmor,
    Robe,
    Suit,
    Vest,

    // Accessories
    Ring,
    Necklace,
    Earring,
    Gloves,
    Coin,
    Bag,
}

/// Equipment item
#[derive(Debug, Clone)]
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
    pub aspd_bonus: u16,      // +X% double attack chance

    // Refinement data
    pub refine_level: u8,  // 0 to 10 (+0 to +10)
    pub max_refine: u8,    // Usually 10

    // Upgrade path (evolution)
    pub can_upgrade: bool,
    pub upgrade_level_req: u16,   // Level needed to upgrade
    pub upgrade_cost: u32,        // Zeny cost
    pub upgrades_to: Option<u16>, // Equipment ID it upgrades to
}

impl Equipment {
    /// Create starter weapon for Novice
    pub const fn starter_weapon_novice() -> Self {
        Equipment {
            id: 1000,
            name: "Rusty Knife",
            equipment_type: EquipmentType::Knife,
            slot: EquipmentSlot::Weapon,
            level_req: 1,
            job_req: None,
            atk_bonus: 8,
            def_bonus: 0,
            hp_bonus: 0,
            sp_bonus: 0,
            str_bonus: 0,
            agi_bonus: 0,
            vit_bonus: 0,
            int_bonus: 0,
            dex_bonus: 0,
            luk_bonus: 0,
            crit_rate_bonus: 0,
            aspd_bonus: 0,
            refine_level: 0,
            max_refine: 10,
            can_upgrade: true,
            upgrade_level_req: 10,
            upgrade_cost: 500,
            upgrades_to: Some(1001),
        }
    }

    /// Create starter armor for Novice
    pub const fn starter_armor_novice() -> Self {
        Equipment {
            id: 2000,
            name: "Cotton Shirt",
            equipment_type: EquipmentType::ClothArmor,
            slot: EquipmentSlot::Armor,
            level_req: 1,
            job_req: None,
            atk_bonus: 0,
            def_bonus: 5,
            hp_bonus: 10,
            sp_bonus: 0,
            str_bonus: 0,
            agi_bonus: 0,
            vit_bonus: 1,
            int_bonus: 0,
            dex_bonus: 0,
            luk_bonus: 0,
            crit_rate_bonus: 0,
            aspd_bonus: 0,
            refine_level: 0,
            max_refine: 10,
            can_upgrade: true,
            upgrade_level_req: 10,
            upgrade_cost: 500,
            upgrades_to: Some(2001),
        }
    }

    /// Create starter accessory for Novice
    pub const fn starter_accessory_novice() -> Self {
        Equipment {
            id: 3000,
            name: "Wooden Ring",
            equipment_type: EquipmentType::Ring,
            slot: EquipmentSlot::Accessory,
            level_req: 1,
            job_req: None,
            atk_bonus: 0,
            def_bonus: 0,
            hp_bonus: 5,
            sp_bonus: 5,
            str_bonus: 1,
            agi_bonus: 0,
            vit_bonus: 0,
            int_bonus: 0,
            dex_bonus: 0,
            luk_bonus: 0,
            crit_rate_bonus: 0,
            aspd_bonus: 0,
            refine_level: 0,
            max_refine: 10,
            can_upgrade: true,
            upgrade_level_req: 10,
            upgrade_cost: 500,
            upgrades_to: Some(3001),
        }
    }

    /// Get refine bonus based on slot and refine level
    pub fn get_refine_bonus(&self) -> u16 {
        match self.slot {
            EquipmentSlot::Weapon => self.refine_level as u16 * 2,  // +2 ATK per level
            EquipmentSlot::Armor => self.refine_level as u16 * 1,   // +1 DEF per level
            EquipmentSlot::Accessory => self.refine_level as u16 * 1, // +1 to primary stat
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
        if self.slot == EquipmentSlot::Armor {
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

// ============================================================================
// Quest System Structures
// ============================================================================

/// Quest types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum QuestType {
    Story,
    Daily,
    Achievement,
}

/// Quest objective (flat structure for no-std JSON parsing)
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct QuestObjective {
    #[serde(rename = "type")]
    pub objective_type: &'static str,
    #[serde(default)]
    pub enemy_id: u32, // For KillMonster (0 = any)
    #[serde(default)]
    pub item_id: u32, // For CollectItem
    #[serde(default)]
    pub count: u16, // For KillMonster, CollectItem, RefineEquipment, CompleteBattles
    #[serde(default)]
    pub level: u16, // For ReachLevel
    #[serde(default)]
    pub amount: u32, // For EarnZeny
}

/// Quest rewards
#[derive(Debug, Clone, Deserialize)]
pub struct QuestReward {
    pub base_exp: u32,
    pub job_exp: u32,
    pub zeny: u32,
    #[serde(default)]
    pub items: HeaplessVec<(u32, u16), 4>, // (item_id, quantity)
}

/// Quest definition (from JSON)
#[derive(Debug, Clone, Deserialize)]
pub struct QuestData {
    pub id: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub quest_type: QuestType,
    pub min_level: u16,
    pub max_level: u16, // 0 = no max
    #[serde(default)]
    pub objectives: HeaplessVec<QuestObjective, 4>,
    pub rewards: QuestReward,
}

/// Active quest progress (runtime state in GameState)
#[derive(Debug, Clone)]
pub struct ActiveQuest {
    pub quest_id: u32,
    pub started_at: u32, // timestamp (ms)
    pub progress: HeaplessVec<u16, 4>, // progress per objective
    pub completed: bool,
    pub claimed: bool,
}

impl ActiveQuest {
    pub fn new(quest_id: u32, objective_count: usize, started_at: u32) -> Self {
        let mut progress = HeaplessVec::new();
        for _ in 0..objective_count {
            progress.push(0).ok();
        }
        Self {
            quest_id,
            started_at,
            progress,
            completed: false,
            claimed: false,
        }
    }
}

/// Quest action events for updating quest progress
#[derive(Debug, Clone, Copy)]
pub enum QuestAction {
    MonsterKilled { enemy_id: u32 },
    ItemCollected { item_id: u32, count: u16 },
    LevelReached { level: u16 },
    ZenyEarned { amount: u32 },
    EquipmentRefined,
    BattleCompleted,
}

impl Hero {
    /// Get equipment reference for a specific slot
    pub fn get_equipment(&self, slot: EquipmentSlot) -> Option<&Equipment> {
        match slot {
            EquipmentSlot::Weapon => Some(&self.equipped_weapon),
            EquipmentSlot::Armor => Some(&self.equipped_armor),
            EquipmentSlot::Accessory => Some(&self.equipped_accessory),
        }
    }

    /// Attempt to refine equipment in a specific slot
    /// Returns (success, new_refine_level)
    pub fn refine_equipment(&mut self, slot: EquipmentSlot, rng_value: u8) -> Result<(bool, u8), &'static str> {
        let equipment = match slot {
            EquipmentSlot::Weapon => &mut self.equipped_weapon,
            EquipmentSlot::Armor => &mut self.equipped_armor,
            EquipmentSlot::Accessory => &mut self.equipped_accessory,
        };

        // Check if can refine
        if !equipment.can_refine() {
            return Err("Max refine level reached");
        }

        // Check cost
        let cost = equipment.refine_cost();
        if self.zeny < cost {
            return Err("Not enough Zeny");
        }

        // Deduct cost
        self.zeny -= cost;

        // Calculate success
        let success_rate = equipment.refine_success_rate();
        let roll = (rng_value as u16 * 100) / 255;
        let success = roll < success_rate as u16;

        let old_level = equipment.refine_level;

        if success {
            // Success: increase refine level
            equipment.refine_level += 1;
            Ok((true, equipment.refine_level))
        } else {
            // Failure
            if equipment.is_risky_refine() {
                // Risky refine: drop 1 level on failure
                equipment.refine_level = equipment.refine_level.saturating_sub(1);
            }
            // Safe refine: no penalty
            Ok((false, equipment.refine_level))
        }
    }
}

/// Hero character data
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
            inventory: HeaplessVec::new(),

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
            },
            "INT" => {
                self.base_int += 1;
                self.max_sp += 5; // INT increases SP
                self.sp = self.max_sp;
            },
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
        // Check if item already exists in inventory
        for item in self.inventory.iter_mut() {
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
        match self.inventory.push(new_item) {
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
    /// Format: level,exp,exp_to_next,job,hp,max_hp,sp,max_sp,zeny
    pub fn to_save_string(&self) -> String<128> {
        let mut save_str = String::<128>::new();
        write!(
            save_str,
            "{},{},{},{},{},{},{},{},{}",
            self.level,
            self.exp,
            self.exp_to_next_level,
            self.job,
            self.hp,
            self.max_hp,
            self.sp,
            self.max_sp,
            self.zeny
        )
        .ok();
        save_str
    }

    /// Serialize inventory to a string for saving (item_id:quantity,item_id:quantity,...)
    pub fn inventory_to_save_string(&self) -> String<512> {
        let mut save_str = String::<512>::new();
        for (i, item) in self.inventory.iter().enumerate() {
            if i > 0 {
                write!(save_str, ",").ok();
            }
            write!(save_str, "{}:{}", item.id, item.quantity).ok();
        }
        save_str
    }

    /// Deserialize inventory from save string
    pub fn inventory_from_save_string(&mut self, data: &str) {
        let data = data.trim();
        if data.is_empty() {
            return;
        }

        for pair in data.split(',') {
            if let Some((id_str, qty_str)) = pair.split_once(':') {
                if let (Ok(id), Ok(quantity)) = (id_str.parse::<u32>(), qty_str.parse::<u16>()) {
                    // Find item name from game data
                    let item_name = get_item_name(id);
                    self.add_item(id, item_name, quantity);
                }
            }
        }
    }

    /// Deserialize hero data from a CSV-like string
    pub fn from_save_string(data: &str) -> Option<Self> {
        // Trim whitespace and newlines first
        let data = data.trim();

        // Use splitn to limit splits and avoid overflow
        let mut parts = data.split(',');

        // Parse each field manually to avoid Vec overflow
        let level: u16 = parts.next()?.parse().ok()?;
        let exp: u32 = parts.next()?.parse().ok()?;
        let exp_to_next_level: u32 = parts.next()?.parse().ok()?;
        let job_str = parts.next()?;
        let hp: u16 = parts.next()?.parse().ok()?;
        let max_hp: u16 = parts.next()?.parse().ok()?;
        let sp: u16 = parts.next()?.parse().ok()?;
        let max_sp: u16 = parts.next()?.parse().ok()?;
        let zeny: u32 = parts.next()?.parse().ok()?;

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

            // Initialize base stats (1 in each + 3 per level from level 2+)
            base_str: 1,
            base_agi: 1,
            base_vit: 1,
            base_int: 1,
            base_dex: 1,
            base_luk: 1,

            // Calculate available stat points based on level
            // Level 1 = 0 points, Level 2+ = (level - 1) * 3 points
            stat_points: if level > 1 { (level - 1) * 3 } else { 0 },

            // Starter equipment (loaded saves get default Novice gear)
            equipped_weapon: Equipment::starter_weapon_novice(),
            equipped_armor: Equipment::starter_armor_novice(),
            equipped_accessory: Equipment::starter_accessory_novice(),
        })
    }
}

/// Enemy data (based on data/enemies.json)
/// Note: JSON files in data/ folder serve as source of truth
/// This struct contains runtime enemy data used in battles
#[derive(Debug, Clone)]
pub struct Enemy {
    pub id: u32, // Enemy ID from JSON
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub attack: u16,      // Added from JSON
    pub defense: u16,     // Added from JSON
    pub base_exp: u32,    // Renamed from exp_reward
    pub job_exp: u32,     // Added from JSON
    pub zeny_reward: u32, // Calculated zeny (base_exp / 10)
}

impl Enemy {
    /// Get a random enemy based on hero level (from enemies.json)
    /// Uses generated data from build.rs
    pub fn random_for_level(hero_level: u16, rng_value: u8) -> Self {
        // Enemy IDs from enemies.json: Poring=1002, Fabre=1007, Hornet=1004, Thief Bug=1051
        let enemy_id = match rng_value % 4 {
            0 => 1002, // Poring
            1 => 1007, // Fabre
            2 => 1004, // Hornet
            _ => 1051, // Thief Bug
        };

        // Use generated function to get enemy data from JSON
        get_enemy_data(enemy_id).expect("Enemy ID should exist in enemies.json")
    }

    /// Get enemy by ID from JSON data (convenience function)
    pub fn from_id(id: u32) -> Option<Self> {
        get_enemy_data(id)
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn hp_percent(&self) -> u8 {
        ((self.hp as u32 * 100) / self.max_hp as u32) as u8
    }
}

/// Farming state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarmState {
    Idle,
    Fighting,
    Victory,
    Defeat,
}

/// Monster GIF animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterAnimation {
    Idle,      // 0.gif - loops
    Attacking, // 16.gif - plays once
    Dying,     // 32.gif - plays once
}

impl MonsterAnimation {
    /// Get GIF data for a specific monster and animation state
    pub fn gif_data(&self, monster_name: &str) -> &'static [u8] {
        // Convert monster name to lowercase for folder matching
        let monster_lower = monster_name.to_lowercase();

        match (monster_lower.as_str(), self) {
            // Poring animations
            ("poring", MonsterAnimation::Idle) => include_bytes!("images/poring/0.gif"),
            ("poring", MonsterAnimation::Attacking) => include_bytes!("images/poring/16.gif"),
            ("poring", MonsterAnimation::Dying) => include_bytes!("images/poring/32.gif"),

            // Fabre animations
            ("fabre", MonsterAnimation::Idle) => include_bytes!("images/fabre/0.gif"),
            ("fabre", MonsterAnimation::Attacking) => include_bytes!("images/fabre/16.gif"),
            ("fabre", MonsterAnimation::Dying) => include_bytes!("images/fabre/32.gif"),

            // Default fallback to Poring if monster not found
            _ => {
                esp_println::println!(
                    "[WARNING] No GIF found for monster '{}', using Poring",
                    monster_name
                );
                match self {
                    MonsterAnimation::Idle => include_bytes!("images/poring/0.gif"),
                    MonsterAnimation::Attacking => include_bytes!("images/poring/16.gif"),
                    MonsterAnimation::Dying => include_bytes!("images/poring/32.gif"),
                }
            }
        }
    }

    pub fn should_loop(&self) -> bool {
        matches!(self, MonsterAnimation::Idle)
    }
}

/// Get monster attacked GIF (24.gif) for a specific monster
pub fn get_monster_attacked_gif(monster_name: &str) -> &'static [u8] {
    let monster_lower = monster_name.to_lowercase();

    match monster_lower.as_str() {
        "poring" => include_bytes!("images/poring/24.gif"),
        "fabre" => include_bytes!("images/fabre/24.gif"),
        _ => {
            esp_println::println!(
                "[WARNING] No attacked GIF found for monster '{}', using Poring",
                monster_name
            );
            include_bytes!("images/poring/24.gif")
        }
    }
}

/// Monster attacked animation (when hero attacks monster)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterAttackedAnimation {
    Normal,   // Not being attacked
    Attacked, // 24.gif - plays once when hero attacks
}

/// Hero GIF animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroAnimation {
    Resting,   // 16.gif - loops (shown on rest page)
    Idle,      // 36.gif - loops (main loop on battle/farm)
    Attacking, // 84.gif - plays once (hero attacks)
    Attacked,  // 52.gif - plays once (hero takes damage)
}

impl HeroAnimation {
    pub fn gif_data(&self) -> &'static [u8] {
        match self {
            HeroAnimation::Resting => include_bytes!("images/swordman/16.gif"),
            HeroAnimation::Idle => include_bytes!("images/swordman/36.gif"),
            HeroAnimation::Attacking => include_bytes!("images/swordman/84.gif"),
            HeroAnimation::Attacked => include_bytes!("images/swordman/52.gif"),
        }
    }

    pub fn should_loop(&self) -> bool {
        matches!(self, HeroAnimation::Resting | HeroAnimation::Idle)
    }
}

/// Get map background GIF by map ID
/// Map backgrounds are single-frame GIFs stored in images/map/
///
/// To add a new map:
/// 1. Add map data to maps.json with a unique ID
/// 2. Create a GIF file named with the map ID: images/map/{id}.gif
/// 3. Add a match arm below: id => include_bytes!("images/map/{id}.gif")
pub fn get_map_background(map_id: u32) -> Option<&'static [u8]> {
    match map_id {
        1 => Some(include_bytes!("images/map/1.gif")), // Prontera
        2 => Some(include_bytes!("images/map/2.gif")), // Prontera South
        3 => Some(include_bytes!("images/map/3.gif")), // Prontera West
        5 => Some(include_bytes!("images/map/5.gif")), // Prontera East
        _ => {
            esp_println::println!("[WARNING] No background image found for map ID {}", map_id);
            None
        }
    }
}

/// Rest state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestState {
    Resting,
    FullSP,
}

/// Battle state for Whac-A-Mole mini-game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleState {
    Idle,    // Waiting to start
    Playing, // Active gameplay
    Victory, // Won the game
    Defeat,  // Lost the game
}

/// Battle animation phase for manual fighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleAnimationPhase {
    BothIdle,         // Both hero and monster idle
    MonsterAttacking, // Monster attacks (16.gif), hero gets hit (52.gif)
    HeroAttacking,    // Hero attacks (84.gif), monster gets hit (24.gif)
}

/// JRPG Battle State - Turn-based combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JrpgBattleState {
    Start,           // Battle start - show enemy encounter
    PlayerTurn,      // Player choosing action
    PlayerAction,    // Player action animation
    EnemyTurn,       // Enemy choosing action (auto)
    EnemyAction,     // Enemy action animation
    Victory,         // Battle won
    Defeat,          // Battle lost
    Fleeing,         // Running away animation
    Escaped,         // Successfully escaped
}

/// JRPG Battle Action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JrpgBattleAction {
    Attack,    // Basic attack
    Skill,     // Use skill (costs SP)
    Item,      // Use item
    Defend,    // Reduce damage taken
    Run,       // Try to flee
}

/// JRPG Battle Menu Selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JrpgBattleMenu {
    Main,      // Main menu: Attack, Skill, Run
    Skills,    // Skill selection submenu
}

/// JRPG Battle Combatant (for both hero and enemy)
#[derive(Debug, Clone)]
pub struct JrpgCombatant {
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub sp: u16,
    pub max_sp: u16,
    pub attack: u16,
    pub defense: u16,

    // New stats for improved combat
    pub agility: u16,      // For double attack chance
    pub luck: u16,         // For critical/lucky hits
    pub intelligence: u16, // For magic damage
    pub dexterity: u16,    // For accuracy (future)

    // Active status effects (max 8 active effects)
    pub active_effects: heapless::Vec<ActiveStatusEffect, 8>,

    // Available skills (max 3 skills)
    pub available_skills: heapless::Vec<JrpgSkill, 3>,
}

/// Skill type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillType {
    Physical,
    Magic,
    Buff,
    Debuff,
    Healing,
    Utility,
}

/// Skill effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillEffect {
    Damage(u16),           // Base damage
    Heal(u16),             // Heal amount
    Stun(u8),              // Stun for X turns
    Poison(u16, u8),       // Damage per turn, duration
    BuffAtk(u16, u8),      // Increase ATK by X%, duration
    BuffDef(u16, u8),      // Increase DEF by X%, duration
    BuffAgi(u16, u8),      // Increase AGI by X%, duration
    DebuffAtk(u16, u8),    // Decrease ATK by X%, duration
    DebuffDef(u16, u8),    // Decrease DEF by X%, duration
    Steal(u16, u16),       // Min/max zeny to steal
    MultiHit(u8),          // Number of hits
    DodgeNext,             // Dodge next attack
}

/// JRPG Skill
#[derive(Debug, Clone, Copy)]
pub struct JrpgSkill {
    pub id: u16,
    pub name: &'static str,
    pub sp_cost: u16,
    pub skill_type: SkillType,
    pub power: u16,              // Damage multiplier (150 = 150%)
    pub effect: Option<SkillEffect>,
    pub duration: u8,            // For buffs/debuffs
}

/// Status effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffectType {
    Poison,
    Stun,
    Slow,
    Burn,
    Freeze,
    Blind,
    AtkBuff,
    DefBuff,
    AgiBuff,
    AtkDebuff,
    DefDebuff,
    AgiDebuff,
    Blessing,
    DodgeNext,
}

/// Active status effect on a combatant
#[derive(Debug, Clone, Copy)]
pub struct ActiveStatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: u8,     // Turns remaining
    pub power: u16,       // Effect strength (%)
}

/// Combat result for displaying effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResult {
    Normal,
    Critical,
    Lucky,
    Miss,
}

/// Skill database - skills available per job
impl JrpgSkill {
    /// Get skills for Swordsman job
    pub const fn get_swordsman_skills() -> [JrpgSkill; 3] {
        [
            // Bash - High damage single target
            JrpgSkill {
                id: 1,
                name: "Bash",
                sp_cost: 8,
                skill_type: SkillType::Physical,
                power: 150, // 150% ATK damage
                effect: Some(SkillEffect::Stun(1)), // 10% stun chance handled in code
                duration: 1,
            },
            // Provoke - Debuff enemy DEF, buff own ATK
            JrpgSkill {
                id: 2,
                name: "Provoke",
                sp_cost: 5,
                skill_type: SkillType::Debuff,
                power: 0, // No damage
                effect: Some(SkillEffect::DebuffDef(30, 3)), // -30% DEF for 3 turns
                duration: 3,
            },
            // Magnum Break - Medium damage
            JrpgSkill {
                id: 3,
                name: "Magnum Break",
                sp_cost: 15,
                skill_type: SkillType::Physical,
                power: 120, // 120% ATK damage
                effect: None, // Just damage
                duration: 0,
            },
        ]
    }

    /// Get skills for Mage job
    pub const fn get_mage_skills() -> [JrpgSkill; 3] {
        [
            // Fire Bolt - High INT-based magic damage
            JrpgSkill {
                id: 10,
                name: "Fire Bolt",
                sp_cost: 12,
                skill_type: SkillType::Magic,
                power: 200, // INT × 2
                effect: None,
                duration: 0,
            },
            // Cold Bolt - INT-based magic damage with slow
            JrpgSkill {
                id: 11,
                name: "Cold Bolt",
                sp_cost: 12,
                skill_type: SkillType::Magic,
                power: 180, // INT × 1.8
                effect: Some(SkillEffect::BuffAgi(50, 2)), // Implemented as slow (reduce AGI)
                duration: 2,
            },
            // Lightning Bolt - Highest INT-based magic damage with stun
            JrpgSkill {
                id: 12,
                name: "Lightning Bolt",
                sp_cost: 12,
                skill_type: SkillType::Magic,
                power: 220, // INT × 2.2
                effect: Some(SkillEffect::Stun(1)), // 10% stun chance
                duration: 1,
            },
        ]
    }

    /// Get skills for Archer job
    pub const fn get_archer_skills() -> [JrpgSkill; 3] {
        [
            // Double Strafe - Attack twice
            JrpgSkill {
                id: 20,
                name: "Double Strafe",
                sp_cost: 10,
                skill_type: SkillType::Physical,
                power: 100, // 100% ATK × 2 hits
                effect: Some(SkillEffect::MultiHit(2)),
                duration: 0,
            },
            // Arrow Shower - Area damage
            JrpgSkill {
                id: 21,
                name: "Arrow Shower",
                sp_cost: 15,
                skill_type: SkillType::Physical,
                power: 80, // 80% ATK
                effect: None,
                duration: 0,
            },
            // Improve Concentration - Buff AGI and DEX
            JrpgSkill {
                id: 22,
                name: "Concentration",
                sp_cost: 8,
                skill_type: SkillType::Buff,
                power: 0,
                effect: Some(SkillEffect::BuffAgi(30, 3)), // +30% AGI for 3 turns
                duration: 3,
            },
        ]
    }

    /// Get skills for Thief job
    pub const fn get_thief_skills() -> [JrpgSkill; 3] {
        [
            // Steal - Steal Zeny from enemy
            JrpgSkill {
                id: 30,
                name: "Steal",
                sp_cost: 10,
                skill_type: SkillType::Utility,
                power: 0,
                effect: Some(SkillEffect::Steal(10, 50)), // 10-50z
                duration: 0,
            },
            // Hiding - Dodge next attack and counter
            JrpgSkill {
                id: 31,
                name: "Hiding",
                sp_cost: 12,
                skill_type: SkillType::Utility,
                power: 80, // Counter for 80% ATK
                effect: Some(SkillEffect::DodgeNext),
                duration: 1,
            },
            // Envenom - Poison damage over time
            JrpgSkill {
                id: 32,
                name: "Envenom",
                sp_cost: 15,
                skill_type: SkillType::Physical,
                power: 120, // 120% ATK
                effect: Some(SkillEffect::Poison(5, 3)), // 5 dmg/turn for 3 turns
                duration: 3,
            },
        ]
    }

    /// Get skills for Acolyte job
    pub const fn get_acolyte_skills() -> [JrpgSkill; 3] {
        [
            // Heal - Restore HP
            JrpgSkill {
                id: 40,
                name: "Heal",
                sp_cost: 13,
                skill_type: SkillType::Healing,
                power: 300, // INT × 3
                effect: Some(SkillEffect::Heal(0)), // Amount calculated in code
                duration: 0,
            },
            // Blessing - Buff all stats
            JrpgSkill {
                id: 41,
                name: "Blessing",
                sp_cost: 10,
                skill_type: SkillType::Buff,
                power: 0,
                effect: Some(SkillEffect::BuffAtk(20, 4)), // +20% ATK/DEF for 4 turns
                duration: 4,
            },
            // Divine Protection - Reduce damage taken
            JrpgSkill {
                id: 42,
                name: "Divine Protect",
                sp_cost: 12,
                skill_type: SkillType::Buff,
                power: 0,
                effect: Some(SkillEffect::BuffDef(40, 2)), // +40% DEF for 2 turns
                duration: 2,
            },
        ]
    }

    /// Get skills for Merchant job
    pub const fn get_merchant_skills() -> [JrpgSkill; 3] {
        [
            // Mammonite - Spend Zeny for high damage
            JrpgSkill {
                id: 50,
                name: "Mammonite",
                sp_cost: 8,
                skill_type: SkillType::Physical,
                power: 180, // 180% ATK (costs 50z)
                effect: None,
                duration: 0,
            },
            // Discount - Steal item (implemented as Zeny)
            JrpgSkill {
                id: 51,
                name: "Discount",
                sp_cost: 5,
                skill_type: SkillType::Utility,
                power: 0,
                effect: Some(SkillEffect::Steal(20, 100)), // 20-100z
                duration: 0,
            },
            // Enlarge Weight - Increase max HP temporarily
            JrpgSkill {
                id: 52,
                name: "Enlarge Weight",
                sp_cost: 10,
                skill_type: SkillType::Buff,
                power: 0,
                effect: Some(SkillEffect::BuffDef(20, 3)), // +20% DEF (HP increase) for 3 turns
                duration: 3,
            },
        ]
    }

    /// Get skills for a specific job
    pub fn get_skills_for_job(job: &str) -> heapless::Vec<JrpgSkill, 3> {
        let mut skills = heapless::Vec::new();

        let skill_array = match job {
            "Swordsman" => Self::get_swordsman_skills(),
            "Mage" => Self::get_mage_skills(),
            "Archer" => Self::get_archer_skills(),
            "Thief" => Self::get_thief_skills(),
            "Acolyte" => Self::get_acolyte_skills(),
            "Merchant" => Self::get_merchant_skills(),
            _ => Self::get_swordsman_skills(), // Default to Swordsman
        };

        for skill in skill_array.iter() {
            let _ = skills.push(*skill);
        }

        skills
    }
}

/// Circle type for Whac-A-Mole game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleType {
    GoodTarget, // Click to hit enemy (green) - gain score
    BadTarget,  // Enemy attack (red) - must click to block, else take damage
}

/// Active circle in the Whac-A-Mole game
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub x: i32,
    pub y: i32,
    pub radius: u32,
    pub circle_type: CircleType,
    pub spawn_time: u32,  // When it spawned
    pub lifetime_ms: u32, // How long it lasts (2000ms)
}

impl Circle {
    pub fn new(x: i32, y: i32, circle_type: CircleType, spawn_time: u32) -> Self {
        Self {
            x,
            y,
            radius: 25, // Fixed radius
            circle_type,
            spawn_time,
            lifetime_ms: 2000, // 2 seconds to click (increased for better playability)
        }
    }

    pub fn is_expired(&self, current_time: u32) -> bool {
        current_time >= self.spawn_time + self.lifetime_ms
    }

    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        let dx = self.x - px;
        let dy = self.y - py;
        (dx * dx + dy * dy) <= (self.radius as i32 * self.radius as i32)
    }
}

/// Location type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    City,  // Cities with NPCs (Prontera, etc)
    Field, // Monster fields for hunting
}

/// Map location ID (loaded from maps.json)
pub type MapId = u32;

/// Map helper functions (uses generated data from maps.json)
pub struct MapHelper;

impl MapHelper {
    pub fn name(map_id: MapId) -> &'static str {
        get_map_name(map_id)
    }

    pub fn location_type(map_id: MapId) -> LocationType {
        if is_city(map_id) {
            LocationType::City
        } else {
            LocationType::Field
        }
    }

    /// Get available exits from this location (from maps.json)
    pub fn exits(map_id: MapId) -> heapless::Vec<MapExit, 4> {
        let (north, south, east, west) = get_map_connections(map_id);
        let mut exits = heapless::Vec::new();

        if let Some(dest) = north {
            exits
                .push(MapExit {
                    direction: "North",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = south {
            exits
                .push(MapExit {
                    direction: "South",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = east {
            exits
                .push(MapExit {
                    direction: "East",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = west {
            exits
                .push(MapExit {
                    direction: "West",
                    destination: dest,
                })
                .ok();
        }

        exits
    }

    /// Get enemy IDs for a map (from maps.json)
    pub fn enemies(map_id: MapId) -> heapless::Vec<u32, 8> {
        get_map_enemies(map_id)
    }

    /// Get NPCs for city locations (from maps.json)
    pub fn npcs(map_id: MapId) -> heapless::Vec<&'static str, 8> {
        get_city_npcs(map_id)
    }
}

/// Exit from a location
#[derive(Debug, Clone, Copy)]
pub struct MapExit {
    pub direction: &'static str,
    pub destination: MapId,
}

/// Main game state resource
#[derive(Resource)]
pub struct GameState {
    pub current_page: GamePage,
    pub hero: Hero,
    pub current_location: MapId, // Current map location
    pub current_enemy: Option<Enemy>,
    pub farm_state: FarmState,
    pub farm_progress: u32,       // 0-60000 (60 seconds in milliseconds)
    pub farm_duration_ms: u32,    // 60000 ms = 1 minute
    pub farm_touch_cooldown: u32, // Cooldown in ms to prevent immediate re-touch
    pub rest_state: RestState,
    pub rest_progress: u32,                  // Progress in milliseconds
    pub sp_regen_rate: u16,                  // SP per second while resting
    pub menu_selection: u8, // 0 = Overview, 1 = Farm, 2 = Rest, 3 = Battle, 4 = Save
    pub battle_state: BattleState, // Current battle state
    pub battle_enemy: Option<Enemy>, // Enemy being fought
    pub battle_circles: [Option<Circle>; 4], // Up to 4 active circles
    pub battle_score: u16,  // Hits made in current battle
    pub battle_missed: u16, // Circles missed or bad targets hit
    pub battle_combo: u16,  // Current combo count (consecutive green hits)
    pub battle_next_spawn: u32, // When next circle spawns
    pub battle_spawn_interval: u32, // Time between spawns (800ms)
    pub battle_duration: u32, // Total battle time (30 seconds)
    pub battle_elapsed: u32, // Time elapsed in battle
    pub battle_last_touch_x: i32, // Last touch X position for debug display
    pub battle_last_touch_y: i32, // Last touch Y position for debug display
    pub battle_last_touch_time: u32, // When last touch occurred (for fade out)
    pub battle_end_time: u32, // When battle ended (for preventing accidental clicks)
    pub battle_animation_phase: BattleAnimationPhase, // Current animation phase
    pub battle_animation_phase_started_ms: u32, // When current phase started
    // JRPG Battle state
    pub jrpg_battle_state: JrpgBattleState,    // Current JRPG battle state
    pub jrpg_battle_menu: JrpgBattleMenu,      // Current menu
    pub jrpg_menu_selection: u8,               // Current menu item selected (0-4)
    pub jrpg_hero_combatant: Option<JrpgCombatant>, // Hero battle stats
    pub jrpg_enemy_combatant: Option<JrpgCombatant>, // Enemy battle stats
    pub jrpg_battle_message: Option<&'static str>, // Battle message (e.g., "Hero attacks!")
    pub jrpg_battle_message_timer: u32,        // How long to show message
    pub jrpg_damage_dealt: u16,                // Last damage dealt (for display)
    pub jrpg_damage_animation_timer: u32,      // Timer for damage text animation (0-1000ms)
    pub jrpg_damage_x: i32,                    // X position for damage text
    pub jrpg_damage_y: i32,                    // Y position for damage text
    pub jrpg_action_animation_timer: u32,      // Timer for action animations
    pub jrpg_combo_count: u8,                  // Current combo count (hits in a row)
    pub jrpg_combo_ready: bool,                // Combo attack available (3 hits)
    pub jrpg_last_combat_result: CombatResult, // Last attack result (normal/crit/lucky)
    pub jrpg_skill_menu_selection: u8,         // Selected skill in skill menu (0-2)
    pub jrpg_selected_skill_index: Option<usize>, // Index of skill being used
    // Equipment refinement UI state
    pub equipment_selection_open: bool,         // Whether equipment selection menu is shown
    pub refine_popup_open: bool,                // Whether refine popup is shown
    pub refine_slot: Option<EquipmentSlot>,     // Which slot is being refined
    pub refine_result_message: Option<&'static str>, // Result message (success/failure)
    pub refine_result_timer: u32,               // How long to show result (0-2000ms)
    // Quest system state
    pub active_quests: HeaplessVec<ActiveQuest, 16>, // Currently active quests
    pub completed_quest_ids: HeaplessVec<u32, 64>, // IDs of all completed quests
    pub daily_quest_refresh_time: u32,          // When daily quests last refreshed (ms)
    pub quest_page_scroll: u8,                  // Scroll position in quest list (0-255)
    pub last_update_ms: u32, // Last update time for progress tracking
    pub save_requested: bool, // Flag to trigger save
    pub save_status_msg: Option<&'static str>, // Status message after save
    pub save_status_timeout: u32, // Time when save message should clear (0 = no message)
    pub fps: u32,           // Current FPS
    pub frame_count: u32,   // Total frames rendered
    pub last_fps_update_ms: u32, // Last time FPS was calculated
    pub needs_redraw: bool, // Flag to indicate screen needs redrawing
    pub screen_on: bool,    // Screen power state (controlled by PWR button)
    pub last_drops: HeaplessVec<(u32, &'static str, u16), 4>, // Last items that dropped
    pub brightness: u8,     // Screen brightness (0-255)
    pub monster_animation: MonsterAnimation, // Current monster animation
    pub monster_animation_frame: usize, // Current frame in animation
    pub monster_animation_started_ms: u32, // When current animation started
    pub monster_attacked_animation: MonsterAttackedAnimation, // Monster attacked state
    pub monster_attacked_frame: usize, // Current frame in attacked animation
    pub monster_attacked_started_ms: u32, // When monster attacked animation started
    pub hero_animation: HeroAnimation, // Current hero animation
    pub hero_animation_frame: usize, // Current frame in hero animation
    pub hero_animation_started_ms: u32, // When current hero animation started
    pub last_attack_animation_ms: u32, // When last attack animation was triggered (for timing)
    pub last_hero_attack_ms: u32, // When hero last attacked (for triggering hero attack anim)
    pub map_monster_animation_frame: usize, // Current frame for monster idle animations on map page
    pub map_monster_animation_last_update: u32, // Last time map monster animation was updated
    pub gif_animation_clock_ms: u32, // Global clock for synchronized GIF animations (increments every 100ms)
    pub gif_animation_last_update_ms: u32, // Last time GIF animation clock was updated
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            current_page: GamePage::Overview,
            hero: Hero::new(),
            current_location: MAP_PRONTERA_ID, // Start in Prontera (ID from maps.json)
            current_enemy: None,
            farm_state: FarmState::Idle,
            farm_progress: 0,
            farm_duration_ms: 60000, // 1 minute
            farm_touch_cooldown: 0,
            rest_state: RestState::Resting,
            rest_progress: 0,
            sp_regen_rate: 5, // 5 SP per second
            menu_selection: 0,
            battle_state: BattleState::Idle,
            battle_enemy: None,
            battle_circles: [None, None, None, None],
            battle_score: 0,
            battle_missed: 0,
            battle_combo: 0,
            battle_next_spawn: 0,
            battle_spawn_interval: 800, // 800ms between spawns
            battle_duration: 30000,     // 30 seconds
            battle_elapsed: 0,
            battle_last_touch_x: 0,
            battle_last_touch_y: 0,
            battle_last_touch_time: 0,
            battle_end_time: 0,
            battle_animation_phase: BattleAnimationPhase::BothIdle,
            battle_animation_phase_started_ms: 0,
            jrpg_battle_state: JrpgBattleState::Start,
            jrpg_battle_menu: JrpgBattleMenu::Main,
            jrpg_menu_selection: 0,
            jrpg_hero_combatant: None,
            jrpg_enemy_combatant: None,
            jrpg_battle_message: None,
            jrpg_battle_message_timer: 0,
            jrpg_damage_dealt: 0,
            jrpg_damage_animation_timer: 0,
            jrpg_damage_x: 0,
            jrpg_damage_y: 0,
            jrpg_action_animation_timer: 0,
            jrpg_combo_count: 0,
            jrpg_combo_ready: false,
            jrpg_last_combat_result: CombatResult::Normal,
            jrpg_skill_menu_selection: 0,
            jrpg_selected_skill_index: None,
            // Equipment refinement UI state
            equipment_selection_open: false,
            refine_popup_open: false,
            refine_slot: None,
            refine_result_message: None,
            refine_result_timer: 0,
            // Quest system state
            active_quests: HeaplessVec::new(),
            completed_quest_ids: HeaplessVec::new(),
            daily_quest_refresh_time: 0,
            quest_page_scroll: 0,
            last_update_ms: 0,
            save_requested: false,
            save_status_msg: None,
            save_status_timeout: 0,
            fps: 0,
            frame_count: 0,
            last_fps_update_ms: 0,
            needs_redraw: true, // Start with needing a redraw
            screen_on: true,    // Screen starts on
            last_drops: HeaplessVec::new(),
            brightness: 204, // 80% brightness by default (204/255 = 0.8)
            monster_animation: MonsterAnimation::Idle,
            monster_animation_frame: 0,
            monster_animation_started_ms: 0,
            monster_attacked_animation: MonsterAttackedAnimation::Normal,
            monster_attacked_frame: 0,
            monster_attacked_started_ms: 0,
            hero_animation: HeroAnimation::Idle,
            hero_animation_frame: 0,
            hero_animation_started_ms: 0,
            last_attack_animation_ms: 0,
            last_hero_attack_ms: 0,
            map_monster_animation_frame: 0,
            map_monster_animation_last_update: 0,
            gif_animation_clock_ms: 0,
            gif_animation_last_update_ms: 0,
        }
    }
}

/// Calculate damage for JRPG battles with variance, crits, lucky strikes, and miss chance
fn calculate_jrpg_damage(
    attacker_atk: u16,
    attacker_luck: u16,
    attacker_dex: u16,
    defender_def: u16,
    defender_agi: u16,
    rng_value: u8, // 0-255 random value
) -> (u16, CombatResult) {
    // Calculate hit chance based on DEX vs AGI
    // Base hit rate: 80%
    // +1% hit per 5 DEX difference
    // -1% hit per 5 AGI difference
    let dex_bonus = (attacker_dex as i32) / 5;
    let agi_penalty = (defender_agi as i32) / 5;
    let hit_rate = 80 + dex_bonus - agi_penalty;
    let hit_rate = hit_rate.clamp(20, 95) as u16; // Min 20%, Max 95%

    // Check if attack hits
    let hit_roll = (rng_value as u16 * 100) / 255;
    if hit_roll >= hit_rate {
        // Miss!
        return (0, CombatResult::Miss);
    }

    // Base damage calculation
    let base_damage = if attacker_atk > defender_def {
        attacker_atk - (defender_def / 2)
    } else {
        1 // Minimum damage
    };

    // Apply damage variance (±20%)
    // Use rng_value to get variance between 80% and 120%
    let variance_percent = 80 + ((rng_value as u32 * 40) / 255) as u16; // 80-120%
    let varied_damage = ((base_damage as u32 * variance_percent as u32) / 100) as u16;

    // Calculate crit chance (base 5% + luck bonus)
    let crit_chance = 5 + (attacker_luck / 20); // Each 20 luck = +1% crit
    let crit_roll = (rng_value as u16 * 100) / 255;

    let (final_damage, combat_result) = if crit_roll < 2 {
        // 2% chance for Lucky Strike (200% damage, ignores DEF)
        (attacker_atk * 2, CombatResult::Lucky)
    } else if crit_roll < (2 + crit_chance) {
        // Critical hit (140% damage, ignores DEF)
        let crit_damage = ((attacker_atk as u32 * 140) / 100) as u16;
        (crit_damage, CombatResult::Critical)
    } else {
        // Normal hit with variance
        (varied_damage.max(1), CombatResult::Normal)
    };

    (final_damage.max(1), combat_result)
}

impl GameState {
    /// Roll for item drop based on enemy ID
    fn roll_item_drop(&self, enemy_id: u32, _rng_value: u8) -> Option<(u32, &'static str)> {
        // Simple item drop table based on enemy
        match enemy_id {
            1002 => Some((512, "Apple")),           // Poring drops Apple
            1007 => Some((705, "Clover")),          // Fabre drops Clover
            1004 => Some((518, "Honey")),           // Hornet drops Honey
            1051 => Some((955, "Worm Peeling")),   // Thief Bug drops Worm Peeling
            _ => None,
        }
    }

    /// Start farming with a new enemy
    pub fn start_farming(&mut self, enemy: Enemy) {
        if self.hero.use_sp(20) {
            self.current_enemy = Some(enemy);
            self.farm_state = FarmState::Fighting;
            self.farm_progress = 0;
            self.current_page = GamePage::Farm;
            // Reset animation to Idle when starting new farm
            self.monster_animation = MonsterAnimation::Idle;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;
            self.needs_redraw = true;
        }
    }

    /// Update farming progress
    pub fn update_farm_progress(&mut self, delta_ms: u32) {
        if self.farm_state == FarmState::Fighting {
            self.farm_progress += delta_ms;

            if self.farm_progress >= self.farm_duration_ms {
                self.complete_farming();
            }
        }
    }

    /// Complete farming and award rewards
    fn complete_farming(&mut self) {
        if let Some(enemy) = &self.current_enemy {
            let enemy_id = enemy.id;
            let zeny_earned = enemy.zeny_reward;

            self.hero.add_exp(enemy.base_exp);
            self.hero.add_zeny(zeny_earned);
            self.farm_state = FarmState::Victory;

            // Update quest progress - monster killed
            crate::tamagotchi::quest_system::update_quest_progress(
                self,
                QuestAction::MonsterKilled { enemy_id },
            );

            // Update quest progress - zeny earned
            crate::tamagotchi::quest_system::update_quest_progress(
                self,
                QuestAction::ZenyEarned {
                    amount: zeny_earned,
                },
            );

            // Roll for item drops
            let rng_value = (self.last_update_ms % 255) as u8;
            let drops = crate::tamagotchi::game_data::roll_drops(enemy_id, rng_value);

            // Clear previous drops and store new ones
            self.last_drops.clear();

            for (item_id, item_name, quantity) in drops.iter() {
                if self.hero.add_item(*item_id, item_name, *quantity) {
                    esp_println::println!("[DROPS] Got {} x{}", item_name, quantity);
                    self.last_drops.push((*item_id, item_name, *quantity)).ok();
                } else {
                    esp_println::println!("[DROPS] Inventory full! Lost {}", item_name);
                }
            }
        }
    }

    /// Reset farming state
    pub fn reset_farming(&mut self) {
        self.current_enemy = None;
        self.farm_state = FarmState::Idle;
        self.farm_progress = 0;
        // Set cooldown to prevent immediate re-touch (300ms)
        self.farm_touch_cooldown = 300;
        // Reset animation to Idle
        self.monster_animation = MonsterAnimation::Idle;
        self.monster_animation_frame = 0;
        self.monster_animation_started_ms = self.gif_animation_clock_ms;
    }

    /// Initialize rest state based on current HP/SP
    pub fn init_rest_state(&mut self) {
        // Check if already fully recovered
        if self.hero.sp >= self.hero.max_sp && self.hero.hp >= self.hero.max_hp {
            self.rest_state = RestState::FullSP;
        } else {
            self.rest_state = RestState::Resting;
        }
        self.rest_progress = 0;
    }

    /// Update rest progress
    pub fn update_rest_progress(&mut self, delta_ms: u32) {
        if self.rest_state == RestState::Resting {
            self.rest_progress += delta_ms;

            // Regenerate SP and HP every second
            if self.rest_progress >= 1000 {
                let seconds = self.rest_progress / 1000;

                // Regenerate SP (5 SP per second by default)
                self.hero
                    .regenerate_sp((seconds as u16) * self.sp_regen_rate);

                // Regenerate HP (10 HP per second)
                let hp_regen_rate = 10u16;
                self.hero.heal((seconds as u16) * hp_regen_rate);

                self.rest_progress %= 1000;

                // Check if both SP and HP are full
                if self.hero.sp >= self.hero.max_sp && self.hero.hp >= self.hero.max_hp {
                    self.rest_state = RestState::FullSP;
                }
            }
        }
    }

    /// Get farm progress percentage
    pub fn farm_progress_percent(&self) -> u8 {
        ((self.farm_progress as u64 * 100) / self.farm_duration_ms as u64) as u8
    }

    /// Start Whac-A-Mole battle
    pub fn start_battle(&mut self, enemy: Enemy) {
        if self.hero.use_sp(30) {
            // Battle costs 30 SP (more than farming)
            self.battle_enemy = Some(enemy);
            self.battle_state = BattleState::Playing;
            self.battle_circles = [None, None, None, None];
            self.battle_score = 0;
            self.battle_missed = 0;
            self.battle_combo = 0;
            self.battle_elapsed = 0;
            self.battle_next_spawn = self.last_update_ms + 500; // First spawn in 500ms
            self.current_page = GamePage::Battle; // Switch to battle page

            // Reset animation to Idle when starting new battle
            self.monster_animation = MonsterAnimation::Idle;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;
            self.needs_redraw = true;
        }
    }

    /// Spawn a new circle in the battle
    pub fn spawn_battle_circle(&mut self, rng_value: u8) {
        // Find empty slot
        for slot in &mut self.battle_circles {
            if slot.is_none() {
                // Random position in play area (avoid edges)
                let x = 40 + ((rng_value as i32 * 7) % 280);
                let y = 100 + ((rng_value as i32 * 13) % 220);

                // 70% chance for GoodTarget, 30% for BadTarget
                let circle_type = if rng_value % 10 < 7 {
                    CircleType::GoodTarget
                } else {
                    CircleType::BadTarget
                };

                *slot = Some(Circle::new(x, y, circle_type, self.last_update_ms));
                break;
            }
        }
    }

    /// Update battle state
    pub fn update_battle(&mut self, delta_ms: u32) {
        if self.battle_state != BattleState::Playing {
            return;
        }

        self.battle_elapsed += delta_ms;

        // Check if enemy is defeated
        if let Some(enemy) = &self.battle_enemy {
            if enemy.hp == 0 {
                self.complete_battle();
                return;
            }
        }

        // Check if battle time is up
        if self.battle_elapsed >= self.battle_duration {
            self.complete_battle();
            return;
        }

        // Spawn new circles
        if self.last_update_ms >= self.battle_next_spawn {
            let rng = (self.last_update_ms % 255) as u8;
            self.spawn_battle_circle(rng);
            self.battle_next_spawn = self.last_update_ms + self.battle_spawn_interval;
            self.needs_redraw = true; // Redraw when new circle spawns
        }

        // Check for expired circles
        for circle in &mut self.battle_circles {
            if let Some(c) = circle {
                if c.is_expired(self.last_update_ms) {
                    // Circle expired - if it was a BadTarget (enemy attack), hero takes damage
                    if c.circle_type == CircleType::BadTarget {
                        // Simple damage calculation: 10 base damage + level
                        let damage = if let Some(enemy) = &self.battle_enemy {
                            10 + enemy.level
                        } else {
                            10
                        };
                        self.hero.hp = self.hero.hp.saturating_sub(damage);
                        self.battle_missed += 1;
                        // Reset combo on missing red circle
                        self.battle_combo = 0;
                    } else {
                        // Missed green circle - counts as miss and resets combo
                        self.battle_missed += 1;
                        self.battle_combo = 0;
                    }
                    *circle = None;
                    self.needs_redraw = true; // Redraw when circle expires

                    // Check for defeat (hero HP reaches 0)
                    if self.hero.hp == 0 {
                        self.battle_state = BattleState::Defeat;
                        return;
                    }
                }
            }
        }
    }

    /// Handle circle click at position
    pub fn click_battle_circle(&mut self, x: i32, y: i32) -> bool {
        let mut enemy_defeated = false;

        for circle in &mut self.battle_circles {
            if let Some(c) = circle {
                if c.contains_point(x, y) {
                    match c.circle_type {
                        CircleType::GoodTarget => {
                            // Increase combo on green hit
                            self.battle_combo += 1;
                            self.battle_score += 1;

                            if let Some(enemy) = &mut self.battle_enemy {
                                // Base damage: 5 + hero level
                                let base_damage = 5 + self.hero.level;

                                // Combo multiplier: 1.0x at combo 1, increases by 0.2x per combo
                                // Caps at 3.0x (combo 11+)
                                let combo_multiplier =
                                    (1.0 + (self.battle_combo - 1) as f32 * 0.2).min(3.0);
                                let damage = (base_damage as f32 * combo_multiplier) as u16;

                                enemy.hp = enemy.hp.saturating_sub(damage);
                                esp_println::println!(
                                    "[BATTLE] Hit green! Combo: {}x ({}x multiplier) Dealt {} damage. Enemy HP: {}",
                                    self.battle_combo,
                                    combo_multiplier,
                                    damage,
                                    enemy.hp
                                );

                                // Check if enemy is defeated
                                if enemy.hp == 0 {
                                    enemy_defeated = true;
                                }
                            }
                        }
                        CircleType::BadTarget => {
                            // Blocked enemy attack - doesn't increase or decrease combo
                            self.battle_score += 1;
                            esp_println::println!(
                                "[BATTLE] Blocked red attack! Combo maintained at {}",
                                self.battle_combo
                            );
                        }
                    }
                    *circle = None;
                    self.needs_redraw = true; // Redraw when circle is clicked

                    // Complete battle after modifying circles
                    if enemy_defeated {
                        self.complete_battle();
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Complete battle and calculate rewards
    fn complete_battle(&mut self) {
        if let Some(enemy) = &self.battle_enemy {
            // Win only if enemy HP is 0 (defeated before timeout)
            if enemy.hp == 0 {
                self.battle_state = BattleState::Victory;
                // Award rewards based on score
                let exp_mult = (self.battle_score as u32).max(1);
                self.hero.add_exp(enemy.base_exp * exp_mult / 5);
                self.hero.add_zeny(enemy.zeny_reward * exp_mult / 5);

                // Roll for item drops
                let rng_value = (self.last_update_ms % 255) as u8;
                let drops = crate::tamagotchi::game_data::roll_drops(enemy.id, rng_value);

                // Clear previous drops and store new ones
                self.last_drops.clear();

                for (item_id, item_name, quantity) in drops.iter() {
                    if self.hero.add_item(*item_id, item_name, *quantity) {
                        esp_println::println!("[DROPS] Got {} x{}", item_name, quantity);
                        self.last_drops.push((*item_id, item_name, *quantity)).ok();
                    } else {
                        esp_println::println!("[DROPS] Inventory full! Lost {}", item_name);
                    }
                }
            } else {
                self.battle_state = BattleState::Defeat;
                // Clear drops on defeat
                self.last_drops.clear();
            }
            // Record when battle ended to prevent accidental clicks
            self.battle_end_time = self.last_update_ms;
        }
    }

    /// Reset battle state
    pub fn reset_battle(&mut self) {
        self.battle_enemy = None;
        self.battle_state = BattleState::Idle;
        self.battle_circles = [None, None, None, None];
        self.battle_score = 0;
        self.battle_missed = 0;
        self.battle_combo = 0;
        self.battle_elapsed = 0;
        self.battle_last_touch_x = 0;
        self.battle_last_touch_y = 0;
        self.battle_last_touch_time = 0;
        self.battle_end_time = 0;

        // Reset animation to Idle
        self.monster_animation = MonsterAnimation::Idle;
        self.monster_animation_frame = 0;
        self.monster_animation_started_ms = self.gif_animation_clock_ms;
    }

    /// Start JRPG battle with an enemy
    pub fn start_jrpg_battle(&mut self, enemy: Enemy) {
        esp_println::println!("[JRPG] Starting battle with {}", enemy.name);

        // Load skills for hero's job
        let hero_skills = JrpgSkill::get_skills_for_job(self.hero.job);

        // Get equipment bonuses
        let weapon = &self.hero.equipped_weapon;
        let armor = &self.hero.equipped_armor;
        let accessory = &self.hero.equipped_accessory;

        // Calculate total stats with equipment bonuses
        let total_str = self.hero.base_str as i16 + weapon.str_bonus + armor.str_bonus + accessory.str_bonus;
        let total_agi = self.hero.base_agi as i16 + weapon.agi_bonus + armor.agi_bonus + accessory.agi_bonus;
        let total_vit = self.hero.base_vit as i16 + weapon.vit_bonus + armor.vit_bonus + accessory.vit_bonus;
        let total_int = self.hero.base_int as i16 + weapon.int_bonus + armor.int_bonus + accessory.int_bonus;
        let total_dex = self.hero.base_dex as i16 + weapon.dex_bonus + armor.dex_bonus + accessory.dex_bonus;
        let total_luk = self.hero.base_luk as i16 + weapon.luk_bonus + armor.luk_bonus + accessory.luk_bonus;

        // Calculate ATK with equipment
        let weapon_atk = weapon.total_atk();
        let total_atk = 10 + (total_str.max(0) as u16 * 2) + weapon_atk;

        // Calculate DEF with equipment
        let armor_def = armor.total_def();
        let total_def = 5 + total_vit.max(0) as u16 + armor_def;

        // Calculate max HP/SP with equipment
        let equipment_hp = armor.hp_bonus;
        let equipment_sp = weapon.sp_bonus + accessory.sp_bonus;

        // Create hero combatant from current hero stats + equipment
        self.jrpg_hero_combatant = Some(JrpgCombatant {
            name: self.hero.name,
            level: self.hero.level,
            hp: self.hero.hp,
            max_hp: self.hero.max_hp + equipment_hp,
            sp: self.hero.sp,
            max_sp: self.hero.max_sp + equipment_sp,
            attack: total_atk,
            defense: total_def,
            // Stats with equipment bonuses
            agility: total_agi.max(0) as u16,
            luck: total_luk.max(0) as u16,
            intelligence: total_int.max(0) as u16,
            dexterity: total_dex.max(0) as u16,
            active_effects: heapless::Vec::new(),
            available_skills: hero_skills,
        });

        // Create enemy combatant
        self.jrpg_enemy_combatant = Some(JrpgCombatant {
            name: enemy.name,
            level: enemy.level,
            hp: enemy.hp,
            max_hp: enemy.max_hp,
            sp: 0,       // Enemies don't use SP for now
            max_sp: 0,
            attack: enemy.attack,
            defense: enemy.defense,
            // Enemy stats based on level
            agility: enemy.level,
            luck: enemy.level / 2,
            intelligence: enemy.level,
            dexterity: enemy.level,
            active_effects: heapless::Vec::new(),
            available_skills: heapless::Vec::new(), // Enemies don't use skills yet
        });

        // Store original enemy for rewards
        self.battle_enemy = Some(enemy);

        // Set initial state - start directly in PlayerTurn so menu is visible
        self.jrpg_battle_state = JrpgBattleState::PlayerTurn;
        self.jrpg_battle_menu = JrpgBattleMenu::Main;
        self.jrpg_menu_selection = 0;
        self.jrpg_battle_message = None; // No message at start
        self.jrpg_battle_message_timer = 0;
        self.jrpg_damage_dealt = 0;
        self.jrpg_action_animation_timer = 0;

        // Switch to JRPG battle page
        self.current_page = GamePage::JrpgBattle;
        self.needs_redraw = true;

        // Set animations to idle
        self.hero_animation = HeroAnimation::Idle;
        self.hero_animation_frame = 0;
        self.hero_animation_started_ms = self.gif_animation_clock_ms;
        self.monster_animation = MonsterAnimation::Idle;
        self.monster_animation_frame = 0;
        self.monster_animation_started_ms = self.gif_animation_clock_ms;
    }

    /// Execute player attack in JRPG battle
    pub fn jrpg_player_attack(&mut self) {
        if let (Some(hero), Some(enemy)) = (&self.jrpg_hero_combatant, &mut self.jrpg_enemy_combatant) {
            // Generate random value for damage calculation
            let rng_value = (self.last_update_ms.wrapping_add(self.gif_animation_clock_ms) % 255) as u8;

            // Check for double attack (AGI-based)
            let double_attack_chance = (hero.agility / 10).min(30); // Max 30% at AGI 300+
            let double_attack_roll = (rng_value as u16 * 100) / 255;
            let is_double_attack = double_attack_roll < double_attack_chance;

            // Calculate damage with variance, crits, lucky strikes, and miss chance
            let (damage, combat_result) = calculate_jrpg_damage(
                hero.attack,
                hero.luck,
                hero.dexterity,
                enemy.defense,
                enemy.agility,
                rng_value,
            );

            enemy.hp = enemy.hp.saturating_sub(damage);
            self.jrpg_damage_dealt = damage;
            self.jrpg_last_combat_result = combat_result;

            // Update combo counter
            if combat_result != CombatResult::Miss {
                self.jrpg_combo_count = self.jrpg_combo_count.saturating_add(1);
                if self.jrpg_combo_count >= 3 {
                    self.jrpg_combo_ready = true;
                }
            } else {
                self.jrpg_combo_count = 0;
                self.jrpg_combo_ready = false;
            }

            // Set damage animation position (near enemy at x=80, y=150)
            self.jrpg_damage_animation_timer = 1000; // 1 second animation
            self.jrpg_damage_x = 80 + 32; // Center of enemy GIF (64x64)
            self.jrpg_damage_y = 150 + 20; // Slightly below center

            let result_str = match combat_result {
                CombatResult::Critical => " CRITICAL!",
                CombatResult::Lucky => " LUCKY STRIKE!",
                CombatResult::Miss => " MISS!",
                CombatResult::Normal => "",
            };

            esp_println::println!("[JRPG] Hero dealt {} damage{} (Combo: {}). Enemy HP: {}/{}",
                damage, result_str, self.jrpg_combo_count, enemy.hp, enemy.max_hp);

            // Set attack animation
            self.hero_animation = HeroAnimation::Attacking;
            self.hero_animation_frame = 0;
            self.hero_animation_started_ms = self.gif_animation_clock_ms;

            // Enemy hit animation
            self.monster_attacked_animation = MonsterAttackedAnimation::Attacked;
            self.monster_attacked_frame = 0;
            self.monster_attacked_started_ms = self.gif_animation_clock_ms;

            self.needs_redraw = true;

            // Handle double attack
            if is_double_attack && enemy.hp > 0 {
                // Second hit with different RNG
                let rng_value2 = (rng_value.wrapping_add(self.jrpg_combo_count)) % 255;
                let (damage2, _combat_result2) = calculate_jrpg_damage(
                    hero.attack,
                    hero.luck,
                    hero.dexterity,
                    enemy.defense,
                    enemy.agility,
                    rng_value2,
                );

                enemy.hp = enemy.hp.saturating_sub(damage2);
                self.jrpg_damage_dealt += damage2; // Add to total damage display

                esp_println::println!("[JRPG] Double Attack! Hero dealt additional {} damage. Enemy HP: {}/{}",
                    damage2, enemy.hp, enemy.max_hp);
            }
        }
    }

    /// Execute enemy attack in JRPG battle
    pub fn jrpg_enemy_attack(&mut self) {
        if let (Some(enemy), Some(hero)) = (&self.jrpg_enemy_combatant, &mut self.jrpg_hero_combatant) {
            // Generate random value for damage calculation
            let rng_value = (self.last_update_ms.wrapping_add(self.gif_animation_clock_ms * 2) % 255) as u8;

            // Calculate damage with variance, crits, lucky strikes, and miss chance
            let (damage, combat_result) = calculate_jrpg_damage(
                enemy.attack,
                enemy.luck,
                enemy.dexterity,
                hero.defense,
                hero.agility,
                rng_value,
            );

            // Store combat result for UI display
            self.jrpg_last_combat_result = combat_result;

            // Only apply damage and reset combo if attack hit
            if combat_result != CombatResult::Miss {
                hero.hp = hero.hp.saturating_sub(damage);
                self.jrpg_damage_dealt = damage;

                // Reset combo on player damage
                self.jrpg_combo_count = 0;
                self.jrpg_combo_ready = false;

                // Hero hit animation (only when hit)
                self.hero_animation = HeroAnimation::Attacked;
                self.hero_animation_frame = 0;
                self.hero_animation_started_ms = self.gif_animation_clock_ms;
            } else {
                // Miss - no damage
                self.jrpg_damage_dealt = 0;
            }

            // Set damage animation position (near hero at x=240, y=150)
            self.jrpg_damage_animation_timer = 1000; // 1 second animation
            self.jrpg_damage_x = 240 + 32; // Center of hero GIF (64x64)
            self.jrpg_damage_y = 150 + 20; // Slightly below center

            let result_str = match combat_result {
                CombatResult::Critical => " CRITICAL!",
                CombatResult::Lucky => " LUCKY STRIKE!",
                CombatResult::Miss => " MISS!",
                CombatResult::Normal => "",
            };

            esp_println::println!("[JRPG] Enemy attack: {} damage{}. Hero HP: {}/{}",
                damage, result_str, hero.hp, hero.max_hp);

            // Set monster attack animation (always plays even on miss)
            self.monster_animation = MonsterAnimation::Attacking;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;

            self.needs_redraw = true;
        }
    }

    /// Execute player skill in JRPG battle
    pub fn jrpg_player_use_skill(&mut self, skill_index: usize) {
        // First, get skill and validate
        let skill = if let Some(hero) = &self.jrpg_hero_combatant {
            if skill_index >= hero.available_skills.len() {
                esp_println::println!("[JRPG] Invalid skill index");
                return;
            }

            let skill = hero.available_skills[skill_index];

            // Check SP cost
            if hero.sp < skill.sp_cost {
                esp_println::println!("[JRPG] Not enough SP! Need {}, have {}", skill.sp_cost, hero.sp);
                self.jrpg_battle_message = Some("Not enough SP!");
                self.jrpg_battle_message_timer = 2000;
                self.needs_redraw = true;
                return;
            }

            skill
        } else {
            return;
        };

        // Consume SP
        if let Some(hero_mut) = &mut self.jrpg_hero_combatant {
            hero_mut.sp = hero_mut.sp.saturating_sub(skill.sp_cost);
        }

        // Get hero stats needed for calculations (copied values)
        let (hero_attack, hero_luck, hero_intelligence) = if let Some(hero) = &self.jrpg_hero_combatant {
            (hero.attack, hero.luck, hero.intelligence)
        } else {
            return;
        };

        // Check if enemy exists
        if self.jrpg_enemy_combatant.is_none() {
            return;
        }

        // Generate random value for skill execution
        let rng_value = (self.last_update_ms.wrapping_add(self.gif_animation_clock_ms) % 255) as u8;

        esp_println::println!("[JRPG] Hero uses skill: {} (SP cost: {})", skill.name, skill.sp_cost);

        // Execute skill based on type
        match skill.skill_type {
            SkillType::Physical => {
                // Physical skill: use ATK with skill power multiplier
                let skill_damage = ((hero_attack as u32 * skill.power as u32) / 100) as u16;

                // Get enemy defense for damage calculation
                let enemy_def = if let Some(enemy) = &self.jrpg_enemy_combatant {
                    enemy.defense
                } else {
                    return;
                };

                // Skills never miss - calculate damage directly without miss check
                let base_damage = if skill_damage > enemy_def {
                    skill_damage - (enemy_def / 2)
                } else {
                    1
                };

                // Apply damage variance (±20%)
                let variance_percent = 80 + ((rng_value as u32 * 40) / 255) as u16;
                let varied_damage = ((base_damage as u32 * variance_percent as u32) / 100) as u16;

                // Calculate crit chance (skills can still crit)
                let crit_chance = 5 + (hero_luck / 20);
                let crit_roll = (rng_value as u16 * 100) / 255;

                let (damage, combat_result) = if crit_roll < 2 {
                    (skill_damage * 2, CombatResult::Lucky)
                } else if crit_roll < (2 + crit_chance) {
                    let crit_damage = ((skill_damage as u32 * 140) / 100) as u16;
                    (crit_damage, CombatResult::Critical)
                } else {
                    (varied_damage.max(1), CombatResult::Normal)
                };

                // Apply damage to enemy
                if let Some(enemy) = &mut self.jrpg_enemy_combatant {
                    enemy.hp = enemy.hp.saturating_sub(damage);
                    esp_println::println!("[JRPG] Skill dealt {} damage. Enemy HP: {}/{}", damage, enemy.hp, enemy.max_hp);
                }

                self.jrpg_damage_dealt = damage;
                self.jrpg_last_combat_result = combat_result;

                // Update combo
                if combat_result != CombatResult::Miss {
                    self.jrpg_combo_count = self.jrpg_combo_count.saturating_add(1);
                    if self.jrpg_combo_count >= 3 {
                        self.jrpg_combo_ready = true;
                    }
                } else {
                    self.jrpg_combo_count = 0;
                    self.jrpg_combo_ready = false;
                }
            },
            SkillType::Magic => {
                // Magic skill: use INT with skill power multiplier (ignores DEF)
                let magic_damage = ((hero_intelligence as u32 * skill.power as u32) / 100) as u16;
                // Apply variance
                let variance_percent = 80 + ((rng_value as u32 * 40) / 255) as u16;
                let damage = ((magic_damage as u32 * variance_percent as u32) / 100) as u16;

                // Apply damage to enemy
                if let Some(enemy) = &mut self.jrpg_enemy_combatant {
                    enemy.hp = enemy.hp.saturating_sub(damage);
                    esp_println::println!("[JRPG] Magic dealt {} damage. Enemy HP: {}/{}", damage, enemy.hp, enemy.max_hp);
                }

                self.jrpg_damage_dealt = damage;
                self.jrpg_last_combat_result = CombatResult::Normal;
            },
            SkillType::Healing => {
                // Heal skill: restore HP
                if let Some(hero_mut) = &mut self.jrpg_hero_combatant {
                    let heal_amount = ((hero_intelligence as u32 * skill.power as u32) / 100) as u16;
                    let old_hp = hero_mut.hp;
                    hero_mut.hp = (hero_mut.hp + heal_amount).min(hero_mut.max_hp);
                    let actual_heal = hero_mut.hp - old_hp;

                    self.jrpg_damage_dealt = actual_heal;
                    self.jrpg_last_combat_result = CombatResult::Normal;

                    esp_println::println!("[JRPG] Healed {} HP. Hero HP: {}/{}", actual_heal, hero_mut.hp, hero_mut.max_hp);
                }
            },
            SkillType::Buff | SkillType::Debuff | SkillType::Utility => {
                // Apply effect (buffs/debuffs/utility)
                esp_println::println!("[JRPG] Skill effect applied: {:?}", skill.effect);
                self.jrpg_damage_dealt = 0;
                self.jrpg_last_combat_result = CombatResult::Normal;
            },
        }

        // Set damage animation position
        if skill.skill_type == SkillType::Healing {
            // Heal animation on hero
            self.jrpg_damage_x = 240 + 32;
            self.jrpg_damage_y = 150 + 20;
        } else {
            // Damage animation on enemy
            self.jrpg_damage_x = 80 + 32;
            self.jrpg_damage_y = 150 + 20;
        }
        self.jrpg_damage_animation_timer = 1000;

        // Set attack animation
        self.hero_animation = HeroAnimation::Attacking;
        self.hero_animation_frame = 0;
        self.hero_animation_started_ms = self.gif_animation_clock_ms;

        // Enemy hit animation (if damage skill)
        if skill.skill_type == SkillType::Physical || skill.skill_type == SkillType::Magic {
            self.monster_attacked_animation = MonsterAttackedAnimation::Attacked;
            self.monster_attacked_frame = 0;
            self.monster_attacked_started_ms = self.gif_animation_clock_ms;
        }

        self.needs_redraw = true;
    }

    /// Try to run from battle (50% chance)
    pub fn jrpg_try_run(&mut self) -> bool {
        let rng = (self.last_update_ms % 100) as u8;
        let success = rng < 50; // 50% chance

        if success {
            self.jrpg_battle_state = JrpgBattleState::Escaped;
            esp_println::println!("[JRPG] Escaped successfully");
        } else {
            esp_println::println!("[JRPG] Failed to escape");
        }

        self.needs_redraw = true;
        success
    }

    /// End JRPG battle and return to map
    pub fn end_jrpg_battle(&mut self) {
        // Sync hero HP/SP back to main hero
        if let Some(hero_combatant) = &self.jrpg_hero_combatant {
            self.hero.hp = hero_combatant.hp;
            self.hero.sp = hero_combatant.sp;
        }

        // Award rewards on victory
        if self.jrpg_battle_state == JrpgBattleState::Victory {
            // Extract enemy data before borrowing self mutably
            let (enemy_id, base_exp, zeny_earned) = if let Some(enemy) = &self.battle_enemy {
                (enemy.id, enemy.base_exp, enemy.zeny_reward)
            } else {
                (0, 0, 0)
            };

            if enemy_id > 0 {
                self.hero.add_exp(base_exp);
                self.hero.add_zeny(zeny_earned);

                // Update quest progress - monster killed
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::MonsterKilled { enemy_id },
                );

                // Update quest progress - battle completed
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::BattleCompleted,
                );

                // Update quest progress - zeny earned
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::ZenyEarned {
                        amount: zeny_earned,
                    },
                );

                // Roll for item drops
                let rng_value = (self.last_update_ms % 255) as u8;
                let drop_rate = 30; // 30% drop chance
                if rng_value < drop_rate {
                    if let Some((item_id, item_name)) = self.roll_item_drop(enemy_id, rng_value) {
                        let quantity = 1;
                        self.hero.add_item(item_id, item_name, quantity);
                    }
                }

                esp_println::println!(
                    "[JRPG] Victory! Gained {} EXP, {} Zeny",
                    base_exp, zeny_earned
                );
            }
        }

        // Clean up battle state
        self.jrpg_hero_combatant = None;
        self.jrpg_enemy_combatant = None;
        self.battle_enemy = None;
        self.jrpg_battle_message = None;
        self.jrpg_menu_selection = 0;

        // Return to map
        self.current_page = GamePage::Map;
        self.needs_redraw = true;
    }
}
