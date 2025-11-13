//! Rustymon - Pokemon-like creatures
//!
//! Rustymon are monsters that players collect and battle with.
//! Each has unique stats, elements, and levels.

use serde::{Deserialize, Serialize};

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

    /// Element type
    pub element: Element,

    // Base stats (randomly generated on capture)
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
        };

        // Calculate derived stats
        rustymon.recalculate_stats();
        rustymon.current_hp = rustymon.max_hp; // Start at full HP

        rustymon
    }

    /// Calculate experience needed for next level
    fn calculate_exp_to_next(level: u32) -> u32 {
        level.pow(2) * 100
    }

    /// Recalculate all derived stats based on base stats and level
    pub fn recalculate_stats(&mut self) {
        // HP calculation: Base + (VIT * 10) + (Level * 5)
        self.max_hp = 40 + (self.vit * 10) + (self.level * 5);

        // ATK calculation: Base + (STR * 2) + Level
        self.atk = 5 + (self.str * 2) + self.level;

        // DEF calculation: Base + VIT + (Level / 2)
        self.def = 2 + self.vit + (self.level / 2);

        // HIT calculation: Base + DEX + Level
        self.hit = 175 + self.dex + self.level;

        // FLEE calculation: Base + (DEX / 2) + Level
        self.flee = 100 + (self.dex / 2) + self.level;

        // CRIT calculation: Base + (LUK * 0.3)
        self.crit_rate = 5.0 + (self.luk as f32 * 0.3);

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

        // Every 5 levels, randomly increase one base stat
        if self.level % 5 == 0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let stat_choice = rng.gen_range(0..5);

            match stat_choice {
                0 => self.str += 1,
                1 => self.dex += 1,
                2 => self.vit += 1,
                3 => self.int += 1,
                4 => self.luk += 1,
                _ => {}
            }
        }

        // Recalculate stats
        let old_max_hp = self.max_hp;
        self.recalculate_stats();

        // Heal the HP gained from leveling
        let hp_gained = self.max_hp - old_max_hp;
        self.current_hp += hp_gained;

        log::info!("{} leveled up to {}!", self.name, self.level);
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
