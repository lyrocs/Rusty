/// Combat calculator for stats and timing
///
/// Calculates and caches combat stats, attack speeds, and damage values
use crate::combat::models::Enemy;
use crate::hero::models::Hero;

/// Caches combat stats and pre-calculated values for performance
#[derive(Debug, Clone)]
pub struct CombatCalculator {
    // Hero stats cache
    pub hero_attack_speed_ms: u32,
    pub hero_attack_power: u16,
    pub hero_hit_rate: f32,
    pub hero_defense: u16,
    pub hero_level: u16,
    pub hero_agi: u16,
    pub hero_dex: u16,
    pub hero_vit: u16,

    // Enemy stats cache
    pub enemy_attack_speed_ms: u32,
    pub enemy_attack_power: u16,
    pub enemy_hit_rate: f32,
    pub enemy_defense: u16,
    pub enemy_level: u16,

    // Pre-calculated timings
    pub hero_windup_ms: u32,
    pub hero_recovery_ms: u32,
    pub enemy_windup_ms: u32,
    pub enemy_recovery_ms: u32,

    // Dirty flags for recalculation
    hero_stats_dirty: bool,
    enemy_stats_dirty: bool,
}

impl CombatCalculator {
    /// Create a new combat calculator
    pub fn new() -> Self {
        Self {
            hero_attack_speed_ms: 2000,
            hero_attack_power: 1,
            hero_hit_rate: 80.0,
            hero_defense: 0,
            hero_level: 1,
            hero_agi: 1,
            hero_dex: 1,
            hero_vit: 1,

            enemy_attack_speed_ms: 3000,
            enemy_attack_power: 1,
            enemy_hit_rate: 85.0,
            enemy_defense: 0,
            enemy_level: 1,

            hero_windup_ms: 400,
            hero_recovery_ms: 200,
            enemy_windup_ms: 400,
            enemy_recovery_ms: 200,

            hero_stats_dirty: true,
            enemy_stats_dirty: true,
        }
    }

    /// Mark hero stats as dirty (needs recalculation)
    pub fn mark_hero_dirty(&mut self) {
        self.hero_stats_dirty = true;
    }

    /// Mark enemy stats as dirty (needs recalculation)
    pub fn mark_enemy_dirty(&mut self) {
        self.enemy_stats_dirty = true;
    }

    /// Update hero stats from current hero state
    pub fn update_hero_stats(&mut self, hero: &Hero) {
        if !self.hero_stats_dirty {
            return;
        }

        // Calculate total AGI with equipment bonuses
        let total_agi = (hero.base_agi as i32
            + hero.equipped_weapon.agi_bonus as i32
            + hero.equipped_armor.agi_bonus as i32
            + hero.equipped_shoes.agi_bonus as i32
            + hero.equipped_garment.agi_bonus as i32
            + hero.equipped_accessory1.agi_bonus as i32
            + hero.equipped_accessory2.agi_bonus as i32)
            .max(1) as u16;

        // Calculate total DEX with equipment bonuses
        let total_dex = (hero.base_dex as i32
            + hero.equipped_weapon.dex_bonus as i32
            + hero.equipped_accessory1.dex_bonus as i32
            + hero.equipped_accessory2.dex_bonus as i32)
            .max(1) as u16;

        // Calculate total VIT with equipment bonuses
        let total_vit = (hero.base_vit as i32
            + hero.equipped_armor.vit_bonus as i32
            + hero.equipped_garment.vit_bonus as i32
            + hero.equipped_shoes.vit_bonus as i32)
            .max(1) as u16;

        // Calculate ASPD bonus from equipment
        let aspd_bonus = hero.equipped_weapon.aspd_bonus
            + hero.equipped_armor.aspd_bonus
            + hero.equipped_shoes.aspd_bonus
            + hero.equipped_garment.aspd_bonus
            + hero.equipped_accessory1.aspd_bonus
            + hero.equipped_accessory2.aspd_bonus;

        // Calculate attack speed based on AGI and ASPD bonus
        self.hero_attack_speed_ms = calculate_attack_speed_ms(total_agi, aspd_bonus as i16);

        // Calculate attack power
        self.hero_attack_power = hero.base_str * 2 + hero.equipped_weapon.atk_bonus;

        // Calculate defense
        self.hero_defense = (hero.base_vit / 2)
            + hero.equipped_armor.def_bonus
            + hero.equipped_garment.def_bonus
            + hero.equipped_shoes.def_bonus;

        // Store stats
        self.hero_level = hero.level;
        self.hero_agi = total_agi;
        self.hero_dex = total_dex;
        self.hero_vit = total_vit;

        // Calculate windup and recovery times based on weapon type
        // Faster weapons have shorter windups
        let weapon_speed_factor = if self.hero_attack_speed_ms < 1000 {
            0.75 // Fast weapon: 300ms windup
        } else if self.hero_attack_speed_ms < 1500 {
            1.0 // Normal weapon: 400ms windup
        } else {
            1.25 // Slow weapon: 500ms windup
        };

        self.hero_windup_ms = (400.0 * weapon_speed_factor) as u32;
        self.hero_recovery_ms = 200;

        self.hero_stats_dirty = false;
    }

    /// Update enemy stats from current enemy
    pub fn update_enemy_stats(&mut self, enemy: &Enemy) {
        if !self.enemy_stats_dirty {
            return;
        }

        // Enemy attack speed based on level
        // Higher level enemies attack faster
        self.enemy_attack_speed_ms = (5000 - (enemy.level as u32 * 30)).max(3000).min(5000);

        self.enemy_attack_power = enemy.attack;
        self.enemy_defense = enemy.defense;
        self.enemy_level = enemy.level;

        // Enemy hit rate (85% base - 0.5% per level above 1)
        self.enemy_hit_rate = (85.0 - (enemy.level as f32 - 1.0) * 0.5).max(75.0);

        // Enemy timings (slightly different from hero)
        self.enemy_windup_ms = 400;
        self.enemy_recovery_ms = 200;

        self.enemy_stats_dirty = false;
    }

    /// Calculate hero hit rate against current enemy
    pub fn calculate_hero_hit_rate(&self) -> f32 {
        // Hit rate formula: 80% + (DEX / 5) + (Hero Level - Enemy Level)
        // Final hit rate clamped between 20% and 95%
        let base_hit_rate = 80.0;
        let dex_bonus = self.hero_dex as f32 / 5.0;
        let level_diff = self.hero_level as i32 - self.enemy_level as i32;
        (base_hit_rate + dex_bonus + level_diff as f32)
            .max(20.0)
            .min(95.0)
    }

    /// Calculate hero damage against current enemy (before miss check)
    pub fn calculate_hero_damage(&self, skill_multiplier: u16) -> u16 {
        let base_damage = if self.hero_attack_power > self.enemy_defense {
            self.hero_attack_power - self.enemy_defense
        } else {
            1
        };

        base_damage * skill_multiplier
    }

    /// Calculate enemy damage against hero (before miss check)
    pub fn calculate_enemy_damage(&self) -> u16 {
        if self.enemy_attack_power > self.hero_defense {
            self.enemy_attack_power - self.hero_defense
        } else {
            1
        }
    }

    /// Check if hero attack hits (returns true for hit, false for miss)
    pub fn roll_hero_hit(&self, rng_value: u8) -> bool {
        let hit_rate = self.calculate_hero_hit_rate();
        let miss_chance = 100.0 - hit_rate;
        (rng_value as f32) >= miss_chance
    }

    /// Check if enemy attack hits (returns true for hit, false for miss)
    pub fn roll_enemy_hit(&self, rng_value: u8) -> bool {
        let miss_chance = 100.0 - self.enemy_hit_rate;
        (rng_value as f32) >= miss_chance
    }

    /// Get total time for hero attack cycle (attack speed)
    pub fn get_hero_attack_cycle_ms(&self) -> u32 {
        self.hero_attack_speed_ms
    }

    /// Get total time for enemy attack cycle (attack speed)
    pub fn get_enemy_attack_cycle_ms(&self) -> u32 {
        self.enemy_attack_speed_ms
    }
}

impl Default for CombatCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate attack speed in milliseconds based on AGI and ASPD bonus
///
/// Formula:
/// - Base delay: 2000ms
/// - Each AGI point reduces delay by 10ms
/// - Each ASPD bonus reduces delay by 5ms
/// - Min delay: 500ms, Max delay: 5000ms
pub fn calculate_attack_speed_ms(agi: u16, aspd_bonus: i16) -> u32 {
    const BASE_DELAY: i32 = 2000;
    const AGI_REDUCTION: i32 = 10;
    const ASPD_REDUCTION: i32 = 5;
    const MIN_DELAY: u32 = 500;
    const MAX_DELAY: u32 = 5000;

    let agi_reduction = (agi as i32) * AGI_REDUCTION;
    let aspd_reduction = (aspd_bonus as i32) * ASPD_REDUCTION;
    let final_delay = BASE_DELAY - agi_reduction - aspd_reduction;

    (final_delay.max(MIN_DELAY as i32).min(MAX_DELAY as i32)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_speed_calculation() {
        // Base case: 1 AGI, 0 ASPD bonus = 2000ms
        assert_eq!(calculate_attack_speed_ms(1, 0), 1990);

        // High AGI: 50 AGI = 2000 - 500 = 1500ms
        assert_eq!(calculate_attack_speed_ms(50, 0), 1500);

        // Max speed: 200 AGI = 2000 - 2000 = 500ms (min)
        assert_eq!(calculate_attack_speed_ms(200, 0), 500);

        // Low AGI: 1 AGI, -100 ASPD = 2000 - 10 + 500 = 2490ms
        assert_eq!(calculate_attack_speed_ms(1, -100), 2490);

        // ASPD bonus: 20 AGI, 50 ASPD = 2000 - 200 - 250 = 1550ms
        assert_eq!(calculate_attack_speed_ms(20, 50), 1550);

        // Test clamping to max
        assert_eq!(calculate_attack_speed_ms(1, -1000), 5000);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let mut calc = CombatCalculator::new();
        calc.hero_level = 10;
        calc.hero_dex = 20;
        calc.enemy_level = 10;

        // Base 80% + 20 DEX / 5 = 80% + 4% = 84%
        let hit_rate = calc.calculate_hero_hit_rate();
        assert!((hit_rate - 84.0).abs() < 0.1);

        // Level advantage
        calc.hero_level = 15;
        calc.enemy_level = 10;
        let hit_rate = calc.calculate_hero_hit_rate();
        assert!((hit_rate - 89.0).abs() < 0.1); // 84% + 5% level diff

        // Test clamping to min
        calc.hero_level = 1;
        calc.hero_dex = 1;
        calc.enemy_level = 50;
        let hit_rate = calc.calculate_hero_hit_rate();
        assert_eq!(hit_rate, 20.0); // Should clamp to minimum
    }

    #[test]
    fn test_damage_calculation() {
        let mut calc = CombatCalculator::new();
        calc.hero_attack_power = 100;
        calc.enemy_defense = 30;

        // Normal damage: 100 - 30 = 70
        assert_eq!(calc.calculate_hero_damage(1), 70);

        // Skill damage: 70 * 2 = 140
        assert_eq!(calc.calculate_hero_damage(2), 140);

        // Min damage: when attack <= defense
        calc.hero_attack_power = 20;
        calc.enemy_defense = 30;
        assert_eq!(calc.calculate_hero_damage(1), 1);
    }
}
