//! Hero system
//!
//! Manages hero stats, level, job progression, and HP/SP

use super::stats::Stats;
use super::inventory::Inventory;
use super::equipment::EquippedItems;
use serde::{Deserialize, Serialize};

/// Hero job types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Job {
    Novice,
    Swordsman,
    Knight,
}

impl Job {
    /// Get the next job in progression
    pub fn next_job(&self) -> Option<Job> {
        match self {
            Job::Novice => Some(Job::Swordsman),
            Job::Swordsman => Some(Job::Knight),
            Job::Knight => None,
        }
    }

    /// Get the level requirement for this job
    pub fn min_level(&self) -> u32 {
        match self {
            Job::Novice => 1,
            Job::Swordsman => 10,
            Job::Knight => 40,
        }
    }

    /// Get job name as string
    pub fn name(&self) -> &'static str {
        match self {
            Job::Novice => "Novice",
            Job::Swordsman => "Swordsman",
            Job::Knight => "Knight",
        }
    }

    /// Get base stats for this job
    pub fn base_stats(&self) -> Stats {
        match self {
            Job::Novice => Stats {
                str: 5,
                agi: 5,
                vit: 5,
                int: 5,
                dex: 5,
                luk: 5,
            },
            Job::Swordsman => Stats {
                str: 10,
                agi: 7,
                vit: 10,
                int: 5,
                dex: 7,
                luk: 5,
            },
            Job::Knight => Stats {
                str: 15,
                agi: 10,
                vit: 15,
                int: 7,
                dex: 10,
                luk: 7,
            },
        }
    }
}

/// Hero character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hero {
    pub name: String,
    pub job: Job,
    pub level: u32,
    pub exp: u64,
    pub exp_to_next_level: u64,
    pub stats: Stats,
    pub stat_points: u32, // Available stat points to allocate

    // Current HP/SP
    pub current_hp: u32,
    pub max_hp: u32,
    pub current_sp: u32,
    pub max_sp: u32,

    // Combat stats (base + equipment)
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub crit_rate: f32,

    // Inventory and equipment
    pub inventory: Inventory,
    pub equipped_items: EquippedItems,
    pub gold: u32,
}

impl Hero {
    /// Create a new hero starting as Novice level 1
    pub fn new() -> Self {
        let stats = Stats::new();
        let level = 1;
        let base_hp = 50; // Novice base HP
        let base_sp = 20; // Novice base SP
        let max_hp = stats.calculate_max_hp(base_hp, level);
        let max_sp = stats.calculate_max_sp(base_sp, level);

        Self {
            name: "Hero".to_string(),
            job: Job::Novice,
            level,
            exp: 0,
            exp_to_next_level: Self::calculate_exp_for_level(2),
            stats,
            stat_points: 0, // Start with 0 stat points, gain 3 per level
            current_hp: max_hp,
            max_hp,
            current_sp: max_sp,
            max_sp,
            atk: stats.calculate_atk(),
            def: stats.calculate_def(),
            hit: stats.calculate_hit(level),
            flee: stats.calculate_flee(level),
            crit_rate: stats.calculate_crit_rate(),
            inventory: Inventory::new(),
            equipped_items: EquippedItems::new(),
            gold: 100, // Starting gold
        }
    }

    /// Calculate EXP required for a given level
    pub fn calculate_exp_for_level(level: u32) -> u64 {
        // Simple exponential formula: level^3 * 10
        ((level as u64).pow(3)) * 10
    }

    /// Gain experience points
    pub fn gain_exp(&mut self, exp: u64) {
        self.exp += exp;
        
        // Check for level up
        while self.exp >= self.exp_to_next_level && self.level < 99 {
            self.level_up();
        }
    }

    /// Level up the hero
    fn level_up(&mut self) {
        self.level += 1;
        self.exp -= self.exp_to_next_level;
        self.exp_to_next_level = Self::calculate_exp_for_level(self.level + 1);

        // Grant 3 stat points per level
        self.stat_points += 3;

        // Increase stats based on job
        self.apply_stat_growth();

        // Recalculate derived stats
        self.recalculate_stats();
        
        // Restore HP/SP on level up
        self.current_hp = self.max_hp;
        self.current_sp = self.max_sp;
        
        // Check for automatic job change
        self.check_job_change();
        
        log::info!("Level up! Now level {}", self.level);
    }

    /// Apply stat growth based on current job
    fn apply_stat_growth(&mut self) {
        match self.job {
            Job::Novice => {
                self.stats.str += 1;
                self.stats.agi += 1;
                self.stats.vit += 1;
                self.stats.int += 1;
                self.stats.dex += 1;
                self.stats.luk += 1;
            }
            Job::Swordsman => {
                self.stats.str += 2;
                self.stats.agi += 1;
                self.stats.vit += 2;
                self.stats.int += 0;
                self.stats.dex += 1;
                self.stats.luk += 1;
            }
            Job::Knight => {
                self.stats.str += 3;
                self.stats.agi += 2;
                self.stats.vit += 3;
                self.stats.int += 1;
                self.stats.dex += 2;
                self.stats.luk += 1;
            }
        }
    }

    /// Check if hero should change jobs automatically
    fn check_job_change(&mut self) {
        if self.level >= Job::Knight.min_level() && self.job == Job::Swordsman {
            self.change_job(Job::Knight);
        } else if self.level >= Job::Swordsman.min_level() && self.job == Job::Novice {
            self.change_job(Job::Swordsman);
        }
    }

    /// Change to a new job
    fn change_job(&mut self, new_job: Job) {
        log::info!("Job change: {} -> {}", self.job.name(), new_job.name());
        self.job = new_job;
        
        // Give bonus stats on job change
        match new_job {
            Job::Swordsman => {
                self.stats.str += 5;
                self.stats.vit += 5;
            }
            Job::Knight => {
                self.stats.str += 10;
                self.stats.vit += 10;
                self.stats.agi += 5;
            }
            _ => {}
        }
        
        self.recalculate_stats();
    }

    /// Recalculate all derived stats
    pub fn recalculate_stats(&mut self) {
        let base_hp = match self.job {
            Job::Novice => 50,
            Job::Swordsman => 100,
            Job::Knight => 200,
        };
        let base_sp = match self.job {
            Job::Novice => 20,
            Job::Swordsman => 30,
            Job::Knight => 50,
        };
        
        self.max_hp = self.stats.calculate_max_hp(base_hp, self.level);
        self.max_sp = self.stats.calculate_max_sp(base_sp, self.level);
        self.atk = self.stats.calculate_atk();
        self.def = self.stats.calculate_def();
        self.hit = self.stats.calculate_hit(self.level);
        self.flee = self.stats.calculate_flee(self.level);
        self.crit_rate = self.stats.calculate_crit_rate();
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: u32) {
        if damage >= self.current_hp {
            self.current_hp = 0;
        } else {
            self.current_hp -= damage;
        }
    }

    /// Heal HP
    pub fn heal(&mut self, amount: u32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    /// Check if hero is alive
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    /// Get attack interval in milliseconds
    pub fn get_attack_interval(&self) -> u64 {
        self.stats.calculate_attack_interval()
    }

    /// Get HP percentage
    pub fn hp_percentage(&self) -> f32 {
        (self.current_hp as f32 / self.max_hp as f32) * 100.0
    }

    /// Get SP percentage
    pub fn sp_percentage(&self) -> f32 {
        (self.current_sp as f32 / self.max_sp as f32) * 100.0
    }

    /// Recalculate max HP and SP based on current stats
    pub fn recalculate_max_hp_sp(&mut self) {
        let base_hp = match self.job {
            Job::Novice => 50,
            Job::Swordsman => 100,
            Job::Knight => 200,
        };
        let base_sp = match self.job {
            Job::Novice => 20,
            Job::Swordsman => 30,
            Job::Knight => 50,
        };

        self.max_hp = self.stats.calculate_max_hp(base_hp, self.level);
        self.max_sp = self.stats.calculate_max_sp(base_sp, self.level);

        // Ensure current HP/SP don't exceed new max
        self.current_hp = self.current_hp.min(self.max_hp);
        self.current_sp = self.current_sp.min(self.max_sp);
    }

    /// Equip an item from inventory to an equipment slot
    /// Returns error if item doesn't exist, wrong type, or level too low
    pub fn equip_item(&mut self, unique_id: u64, item_database: &std::collections::HashMap<u32, super::item::ItemData>) -> Result<(), String> {
        // Get the item from inventory
        let item = self.inventory.get_equipment(unique_id)
            .ok_or("Item not found in inventory")?;

        // Get item data to check slot and requirements
        let item_data = item_database.get(&item.item_id)
            .ok_or("Item data not found")?;

        // Check if it's equipment
        let slot = item_data.slot.ok_or("Item is not equipment")?;

        // Check level requirement
        if let Some(required_level) = item_data.required_level {
            if self.level < required_level {
                return Err(format!("Requires level {}", required_level));
            }
        }

        // If slot already has equipment, unequip it first
        if let Some(old_unique_id) = self.equipped_items.get_slot(slot) {
            // The old item stays in inventory, just unequipped
            log::info!("Unequipping previous item from {:?}", slot);
        }

        // Equip the new item
        self.equipped_items.equip(slot, unique_id);
        log::info!("Equipped {} to {:?}", item_data.name, slot);

        // Recalculate stats
        self.recalculate_combat_stats(item_database);

        Ok(())
    }

    /// Unequip an item from an equipment slot back to inventory
    pub fn unequip_item(&mut self, slot: super::item::EquipmentSlot, item_database: &std::collections::HashMap<u32, super::item::ItemData>) -> Result<(), String> {
        let unique_id = self.equipped_items.unequip(slot)
            .ok_or("No item equipped in that slot")?;

        log::info!("Unequipped item from {:?}", slot);

        // Recalculate stats
        self.recalculate_combat_stats(item_database);

        Ok(())
    }

    /// Recalculate combat stats including equipment bonuses
    pub fn recalculate_combat_stats(&mut self, item_database: &std::collections::HashMap<u32, super::item::ItemData>) {
        // Calculate base stats from hero stats
        self.atk = self.stats.calculate_atk();
        self.def = self.stats.calculate_def();
        self.hit = self.stats.calculate_hit(self.level);
        self.flee = self.stats.calculate_flee(self.level);
        self.crit_rate = self.stats.calculate_crit_rate();

        // Add equipment bonuses
        let equipment_stats = super::equipment::EquipmentStats::calculate(
            &self.equipped_items,
            self.inventory.items(),
            item_database,
        );

        self.atk += equipment_stats.atk;
        self.def += equipment_stats.def;
        self.hit += equipment_stats.hit;
        self.flee += equipment_stats.flee;

        log::debug!(
            "Stats recalculated - ATK:{} DEF:{} HIT:{} FLEE:{}",
            self.atk,
            self.def,
            self.hit,
            self.flee
        );
    }

    /// Upgrade equipment with materials
    /// Returns Ok(true) on success, Ok(false) on failure, Err on invalid attempt
    pub fn upgrade_equipment(
        &mut self,
        unique_id: u64,
        item_database: &std::collections::HashMap<u32, super::item::ItemData>,
        upgrade_recipes: &std::collections::HashMap<String, Vec<super::item::UpgradeRecipe>>,
    ) -> Result<bool, String> {
        // Get the item from inventory
        let item = self.inventory.get_equipment_mut(unique_id)
            .ok_or("Item not found in inventory")?;

        // Check if it's equipment
        if !item.is_equipment() {
            return Err("Item is not equipment".to_string());
        }

        let current_level = item.get_upgrade_level();
        if current_level >= 10 {
            return Err("Maximum upgrade level reached (+10)".to_string());
        }

        // Get item data to determine equipment type
        let item_data = item_database.get(&item.item_id)
            .ok_or("Item data not found")?;

        let slot = item_data.slot.ok_or("Item has no equipment slot")?;

        // Determine recipe category based on slot
        let recipe_category = match slot {
            super::item::EquipmentSlot::Weapon => "weapon_upgrades",
            super::item::EquipmentSlot::Armor => "armor_upgrades",
            super::item::EquipmentSlot::Shoes => "shoes_upgrades",
            super::item::EquipmentSlot::Garment => "garment_upgrades",
            super::item::EquipmentSlot::Accessory => "accessory_upgrades",
            super::item::EquipmentSlot::Headgear => "headgear_upgrades",
        };

        // Get upgrade recipe
        let recipes = upgrade_recipes.get(recipe_category)
            .ok_or(format!("No upgrade recipes for {}", recipe_category))?;

        let recipe = recipes.iter()
            .find(|r| r.from_level == current_level)
            .ok_or(format!("No upgrade recipe from level {}", current_level))?;

        // Check gold
        if self.gold < recipe.gold_cost {
            return Err(format!("Not enough gold (need: {}, have: {})", recipe.gold_cost, self.gold));
        }

        // Check materials
        for material in &recipe.materials {
            let has = self.inventory.get_material_quantity(material.item_id);
            if has < material.quantity {
                return Err(format!(
                    "Not enough {} (need: {}, have: {})",
                    material.name,
                    material.quantity,
                    has
                ));
            }
        }

        // Deduct gold
        self.gold -= recipe.gold_cost;

        // Consume materials
        for material in &recipe.materials {
            self.inventory.remove_material(material.item_id, material.quantity)?;
        }

        // Roll for success
        let roll = rand::random::<u32>() % 100;
        let success = roll < recipe.success_rate;

        // Get the item again (mutable borrow)
        let item = self.inventory.get_equipment_mut(unique_id)
            .ok_or("Item disappeared during upgrade")?;

        if success {
            // Upgrade succeeded
            item.upgrade()?;
            log::info!("Upgrade SUCCESS! {} is now +{}", item_data.name, item.get_upgrade_level());

            // Recalculate combat stats if item is equipped
            if self.equipped_items.get_slot(slot).map(|id| id == unique_id).unwrap_or(false) {
                self.recalculate_combat_stats(item_database);
            }

            Ok(true)
        } else {
            // Upgrade failed
            log::warn!("Upgrade FAILED! {} remains at +{}", item_data.name, current_level);

            // On failure at higher levels, equipment may break or downgrade
            // For now, just keep it at the same level (no downgrade)
            // Could add downgrade logic here if desired

            Ok(false)
        }
    }
}

impl Default for Hero {
    fn default() -> Self {
        Self::new()
    }
}
