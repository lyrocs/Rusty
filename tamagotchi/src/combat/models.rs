/// Combat models
///
/// Core combat entities including Enemy and basic combat states.
use crate::tamagotchi::get_enemy_data;

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
    pub zeny_reward: u32, // Calculated zeny (base_exp / 10)
}

impl Enemy {
    /// Get a random enemy based on hero level (from enemies.json)
    /// Uses generated data from build.rs
    pub fn random_for_level(_hero_level: u16, rng_value: u8) -> Self {
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

/// Rest state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestState {
    Resting,
    FullSP,
    Complete,
}

/// Farm duration options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarmDuration {
    OneMinute,   // 60 seconds, 20 SP, 1.0x multiplier per minute
    FiveMinutes, // 300 seconds, 90 SP, 0.90x multiplier per minute (4.5x total)
    TenMinutes,  // 600 seconds, 180 SP, 0.85x multiplier per minute (8.5x total)
}

impl FarmDuration {
    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        match self {
            FarmDuration::OneMinute => 60_000,
            FarmDuration::FiveMinutes => 300_000,
            FarmDuration::TenMinutes => 600_000,
        }
    }

    /// Get SP cost
    pub fn sp_cost(&self) -> u16 {
        match self {
            FarmDuration::OneMinute => 20,
            FarmDuration::FiveMinutes => 90,
            FarmDuration::TenMinutes => 180,
        }
    }

    /// Get efficiency multiplier per minute (for reward penalty on longer farms)
    pub fn multiplier_per_minute(&self) -> f32 {
        match self {
            FarmDuration::OneMinute => 1.0,
            FarmDuration::FiveMinutes => 0.90,
            FarmDuration::TenMinutes => 0.85,
        }
    }

    /// Get total multiplier for the entire duration
    pub fn total_multiplier(&self) -> f32 {
        match self {
            FarmDuration::OneMinute => 1.0,
            FarmDuration::FiveMinutes => 4.5,  // 0.90 × 5
            FarmDuration::TenMinutes => 8.5,   // 0.85 × 10
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            FarmDuration::OneMinute => "1 MIN",
            FarmDuration::FiveMinutes => "5 MIN",
            FarmDuration::TenMinutes => "10 MIN",
        }
    }
}

/// Farm efficiency rating based on hero power vs enemy power
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfficiencyRating {
    Excellent, // ≥2.0 ratio, 2.5x multiplier, ★★★
    Good,      // 1.5-2.0 ratio, 1.5x multiplier, ★★
    Fair,      // 1.0-1.5 ratio, 1.0x multiplier, ★
    Risky,     // 0.7-1.0 ratio, 0.7x multiplier, ⚠
    Impossible, // <0.7 ratio, blocked, ✗
}

impl EfficiencyRating {
    /// Calculate rating from power ratio
    pub fn from_power_ratio(ratio: f32) -> Self {
        if ratio >= 2.0 {
            EfficiencyRating::Excellent
        } else if ratio >= 1.5 {
            EfficiencyRating::Good
        } else if ratio >= 1.0 {
            EfficiencyRating::Fair
        } else if ratio >= 0.7 {
            EfficiencyRating::Risky
        } else {
            EfficiencyRating::Impossible
        }
    }

    /// Get efficiency multiplier for rewards
    pub fn multiplier(&self) -> f32 {
        match self {
            EfficiencyRating::Excellent => 2.5,
            EfficiencyRating::Good => 1.5,
            EfficiencyRating::Fair => 1.0,
            EfficiencyRating::Risky => 0.7,
            EfficiencyRating::Impossible => 0.0,
        }
    }

    /// Get display icon
    pub fn icon(&self) -> &'static str {
        match self {
            EfficiencyRating::Excellent => "***",
            EfficiencyRating::Good => "**",
            EfficiencyRating::Fair => "*",
            EfficiencyRating::Risky => "!",
            EfficiencyRating::Impossible => "X",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            EfficiencyRating::Excellent => "Excellent",
            EfficiencyRating::Good => "Good",
            EfficiencyRating::Fair => "Fair",
            EfficiencyRating::Risky => "Risky",
            EfficiencyRating::Impossible => "Impossible",
        }
    }

    /// Check if farming is allowed
    pub fn is_allowed(&self) -> bool {
        !matches!(self, EfficiencyRating::Impossible)
    }
}

/// Combat result type (for JRPG battles)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResult {
    Normal,
    Critical,
    Lucky,
    Miss,
}
