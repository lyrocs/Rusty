//! Rustymon - Pokemon-like creatures
//!
//! Rustymon are monsters that players collect and battle with.
//! Each has unique stats, elements, and levels.

use serde::{Deserialize, Serialize};
use super::skill::RustymonSkills;
use super::data_loader::get_exp_to_next_level;

/// Element types for Rustymon and enemies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Element {
    Neutral,
    Water,
    Earth,
    Fire,
    Wind,
    Poison,
    Holy,
    Shadow,
    Ghost,
    Undead,
}

impl Element {
    /// Parse element from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "neutral" => Some(Element::Neutral),
            "water" => Some(Element::Water),
            "earth" => Some(Element::Earth),
            "fire" => Some(Element::Fire),
            "wind" => Some(Element::Wind),
            "poison" => Some(Element::Poison),
            "holy" => Some(Element::Holy),
            "shadow" => Some(Element::Shadow),
            "ghost" => Some(Element::Ghost),
            "undead" => Some(Element::Undead),
            _ => None,
        }
    }

    /// Get element name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Element::Neutral => "Neutral",
            Element::Water => "Water",
            Element::Earth => "Earth",
            Element::Fire => "Fire",
            Element::Wind => "Wind",
            Element::Poison => "Poison",
            Element::Holy => "Holy",
            Element::Shadow => "Shadow",
            Element::Ghost => "Ghost",
            Element::Undead => "Undead",
        }
    }
}

/// A Rustymon creature that the player can collect and battle with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rustymon {
    /// Unique instance ID (UUID)
    pub id: String,

    /// Species ID (monster type: 1002=Poring, 1007=Fabre, etc.)
    pub species_id: u32,

    /// Species name
    pub name: String,

    /// Current level (1-99)
    pub level: u32,

    /// Current experience points
    pub exp: u32,

    /// Experience needed for next level
    pub exp_to_next: u32,

    /// Evolution level (0 = base, 1+ = evolved)
    /// Each evolution costs fragments based on Fibonacci sequence
    #[serde(default)]
    pub evolution_level: u32,

    /// Element type
    pub element: Element,

    // Base stats (FIXED from enemy data - same for all instances of this species)
    // Stats increase randomly by +1 per level (one random stat chosen each level)
    /// Strength - affects ATK
    pub str: u32,

    /// Dexterity - affects HIT and FLEE
    pub dex: u32,

    /// Vitality - affects HP and DEF
    pub vit: u32,

    /// Intelligence - affects future magic system
    pub int: u32,

    /// Luck - affects CRIT
    pub luk: u32,

    // Current battle stats (calculated from base stats)
    /// Current HP in battle
    pub current_hp: u32,

    /// Maximum HP
    pub max_hp: u32,

    /// Attack power
    pub atk: u32,

    /// Defense
    pub def: u32,

    /// Hit rate
    pub hit: u32,

    /// Flee/evasion
    pub flee: u32,

    /// Critical hit rate (percentage)
    pub crit_rate: f32,

    /// Skills system
    #[serde(default)]
    pub skills: RustymonSkills,
}

impl Rustymon {
    /// Create a new Rustymon with given parameters
    pub fn new(
        id: String,
        species_id: u32,
        name: String,
        level: u32,
        element: Element,
        str: u32,
        dex: u32,
        vit: u32,
        int: u32,
        luk: u32,
    ) -> Self {
        let mut rustymon = Self {
            id,
            species_id,
            name,
            level,
            exp: 0,
            exp_to_next: Self::calculate_exp_to_next(level),
            evolution_level: 0,
            element,
            str,
            dex,
            vit,
            int,
            luk,
            current_hp: 0,
            max_hp: 0,
            atk: 0,
            def: 0,
            hit: 0,
            flee: 0,
            crit_rate: 0.0,
            skills: RustymonSkills::new(),
        };

        // Calculate derived stats
        rustymon.recalculate_stats();
        rustymon.current_hp = rustymon.max_hp; // Start at full HP

        rustymon
    }

    /// Calculate experience needed for next level
    fn calculate_exp_to_next(level: u32) -> u32 {
        get_exp_to_next_level(level)
    }

    /// Recalculate all derived stats based on base stats and level
    /// Evolution bonus: +5% to all stats per evolution level
    pub fn recalculate_stats(&mut self) {
        // Calculate evolution multiplier (1.0 for level 0, 1.05 for level 1, 1.10 for level 2, etc.)
        let evolution_multiplier = 1.0 + (self.evolution_level as f32 * 0.05);

        // HP calculation: Base + (VIT * 10) + (Level * 5)
        let base_hp = 40 + (self.vit * 10) + (self.level * 5);
        self.max_hp = (base_hp as f32 * evolution_multiplier) as u32;

        // ATK calculation: Base + (STR * 2) + Level
        let base_atk = 5 + (self.str * 2) + self.level;
        self.atk = (base_atk as f32 * evolution_multiplier) as u32;

        // DEF calculation: Base + VIT + (Level / 2)
        let base_def = 2 + self.vit + (self.level / 2);
        self.def = (base_def as f32 * evolution_multiplier) as u32;

        // HIT calculation: Base + DEX + Level
        let base_hit = 175 + self.dex + self.level;
        self.hit = (base_hit as f32 * evolution_multiplier) as u32;

        // FLEE calculation: Base + (DEX / 2) + Level
        let base_flee = 100 + (self.dex / 2) + self.level;
        self.flee = (base_flee as f32 * evolution_multiplier) as u32;

        // CRIT calculation: Base + (LUK * 0.3)
        let base_crit = 5.0 + (self.luk as f32 * 0.3);
        self.crit_rate = base_crit * evolution_multiplier;

        // Update exp to next level
        self.exp_to_next = Self::calculate_exp_to_next(self.level);
    }

    /// Gain experience points, returns true if leveled up
    pub fn gain_exp(&mut self, exp: u32) -> bool {
        self.exp += exp;

        if self.exp >= self.exp_to_next {
            self.level_up();
            return true;
        }

        false
    }

    /// Level up the Rustymon
    fn level_up(&mut self) {
        if self.level >= 99 {
            return; // Max level reached
        }

        self.level += 1;
        self.exp = 0;

        // Every level, randomly increase one base stat by 1
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let stat_choice = rng.gen_range(0..5);

        let stat_name = match stat_choice {
            0 => { self.str += 1; "STR" },
            1 => { self.dex += 1; "DEX" },
            2 => { self.vit += 1; "VIT" },
            3 => { self.int += 1; "INT" },
            4 => { self.luk += 1; "LUK" },
            _ => "UNKNOWN"
        };

        // Recalculate stats
        let old_max_hp = self.max_hp;
        self.recalculate_stats();

        // Heal the HP gained from leveling
        let hp_gained = self.max_hp - old_max_hp;
        self.current_hp += hp_gained;

        log::info!("{} leveled up to {}! {} +1", self.name, self.level, stat_name);
    }

    /// Evolve the Rustymon to next evolution level
    /// Increases evolution_level by 1 and applies +5% stat bonus
    pub fn evolve(&mut self) {
        let old_evolution = self.evolution_level;
        self.evolution_level += 1;

        // Recalculate stats with new evolution multiplier
        let old_max_hp = self.max_hp;
        self.recalculate_stats();

        // Heal HP gained from evolution
        let hp_gained = self.max_hp.saturating_sub(old_max_hp);
        self.current_hp = self.current_hp.saturating_add(hp_gained).min(self.max_hp);

        log::info!("✨ {} evolved from level {} → {}! All stats +5%",
            self.name, old_evolution, self.evolution_level);
        log::info!("  New stats: HP={}, ATK={}, DEF={}, HIT={}, FLEE={}, CRIT={:.1}%",
            self.max_hp, self.atk, self.def, self.hit, self.flee, self.crit_rate);
    }

    /// Take damage in battle
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

    /// Fully restore HP
    pub fn full_heal(&mut self) {
        self.current_hp = self.max_hp;
    }

    /// Check if Rustymon is alive
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    /// Check if Rustymon is fainted
    pub fn is_fainted(&self) -> bool {
        self.current_hp == 0
    }

    /// Get HP percentage (0.0 to 1.0)
    pub fn hp_percentage(&self) -> f32 {
        if self.max_hp == 0 {
            0.0
        } else {
            self.current_hp as f32 / self.max_hp as f32
        }
    }

    /// Get EXP percentage (0.0 to 1.0)
    pub fn exp_percentage(&self) -> f32 {
        if self.exp_to_next == 0 {
            0.0
        } else {
            self.exp as f32 / self.exp_to_next as f32
        }
    }

    /// Check and learn skills for current level
    /// Returns list of newly learned skill IDs
    pub fn check_and_learn_skills(&mut self, learnable_skills: &[super::skill::LearnableSkill]) -> Vec<u32> {
        let mut newly_learned = Vec::new();

        for learnable in learnable_skills {
            // Check if Rustymon meets level requirement and hasn't learned this skill yet
            if self.level >= learnable.learn_level && !self.skills.learned_skills.contains(&learnable.skill_id) {
                if self.skills.learn_skill(learnable.skill_id) {
                    newly_learned.push(learnable.skill_id);
                    log::info!("{} learned skill ID {}!", self.name, learnable.skill_id);
                }
            }
        }

        newly_learned
    }

    /// Auto-enable the first available passive skill
    pub fn auto_enable_first_passive(&mut self, skill_data: &std::collections::HashMap<u32, super::skill::Skill>) {
        // Find first learned passive skill that isn't enabled
        for skill_id in &self.skills.learned_skills {
            if let Some(skill) = skill_data.get(skill_id) {
                if skill.is_passive() && !self.skills.get_enabled_skills().contains(skill_id) {
                    // Try to enable in first available slot
                    for slot in 0..3 {
                        if self.skills.enabled_skills[slot].is_none() {
                            self.skills.enable_skill(*skill_id, slot);
                            log::info!("{} auto-enabled passive skill: {}", self.name, skill.name);
                            return;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_from_str() {
        assert_eq!(Element::from_str("water"), Some(Element::Water));
        assert_eq!(Element::from_str("FIRE"), Some(Element::Fire));
        assert_eq!(Element::from_str("invalid"), None);
    }

    #[test]
    fn test_rustymon_creation() {
        let rustymon = Rustymon::new(
            "test-id".to_string(),
            1002,
            "Poring".to_string(),
            1,
            Element::Water,
            10, 10, 10, 10, 10,
        );

        assert_eq!(rustymon.level, 1);
        assert!(rustymon.max_hp > 0);
        assert_eq!(rustymon.current_hp, rustymon.max_hp);
    }

    #[test]
    fn test_level_up() {
        let mut rustymon = Rustymon::new(
            "test-id".to_string(),
            1002,
            "Poring".to_string(),
            1,
            Element::Water,
            10, 10, 10, 10, 10,
        );

        let old_level = rustymon.level;
        let exp_needed = rustymon.exp_to_next;

        let leveled_up = rustymon.gain_exp(exp_needed);
        assert!(leveled_up);
        assert_eq!(rustymon.level, old_level + 1);
    }
}
