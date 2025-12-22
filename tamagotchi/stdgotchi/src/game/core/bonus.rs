//! Dungeon Bonus System
//!
//! Provides bonuses that can be selected between dungeon floors.

use serde::{Deserialize, Serialize};

/// Types of dungeon bonuses that can be offered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DungeonBonus {
    /// Heal all team monsters by a percentage of max HP
    HealTeam { percent: f32 },

    /// Boost a stat temporarily for a number of floors
    StatBoost {
        stat: StatBoostType,
        percent: f32,
        floors: u8,
    },

    /// Increase capture chance for a number of floors
    CaptureBoost { multiplier: f32, floors: u8 },

    /// Get extra crystals immediately
    ExtraCrystals { amount: u32 },

    /// Skip the next floor (advance without fighting)
    SkipFloor,

    /// Revive a dead monster with partial HP
    ReviveMonster { hp_percent: f32 },
}

/// Stat types that can be boosted
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatBoostType {
    Atk,
    Def,
    Spd,
    AllStats,
}

impl DungeonBonus {
    /// Get a display name for the bonus
    pub fn name(&self) -> &'static str {
        match self {
            DungeonBonus::HealTeam { .. } => "Team Heal",
            DungeonBonus::StatBoost { stat, .. } => match stat {
                StatBoostType::Atk => "ATK Boost",
                StatBoostType::Def => "DEF Boost",
                StatBoostType::Spd => "SPD Boost",
                StatBoostType::AllStats => "All Stats",
            },
            DungeonBonus::CaptureBoost { .. } => "Capture+",
            DungeonBonus::ExtraCrystals { .. } => "Crystals",
            DungeonBonus::SkipFloor => "Skip Floor",
            DungeonBonus::ReviveMonster { .. } => "Revive",
        }
    }

    /// Get a short description of the bonus
    pub fn description(&self) -> String {
        match self {
            DungeonBonus::HealTeam { percent } => {
                format!("Heal team {}%", (percent * 100.0) as u8)
            }
            DungeonBonus::StatBoost { stat, percent, floors } => {
                let stat_name = match stat {
                    StatBoostType::Atk => "ATK",
                    StatBoostType::Def => "DEF",
                    StatBoostType::Spd => "SPD",
                    StatBoostType::AllStats => "All stats",
                };
                format!("+{}% {} for {} floors", (percent * 100.0) as u8, stat_name, floors)
            }
            DungeonBonus::CaptureBoost { multiplier, floors } => {
                format!("{}x capture for {} floors", multiplier, floors)
            }
            DungeonBonus::ExtraCrystals { amount } => {
                format!("+{} crystals", amount)
            }
            DungeonBonus::SkipFloor => "Skip next floor".to_string(),
            DungeonBonus::ReviveMonster { hp_percent } => {
                format!("Revive at {}% HP", (hp_percent * 100.0) as u8)
            }
        }
    }

    /// Generate random bonus options for a given floor
    pub fn generate_options(floor: u16, has_dead_monster: bool) -> Vec<DungeonBonus> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut options = Vec::new();

        // Pool of possible bonuses based on floor
        let mut pool = Vec::new();

        // Always available
        pool.push(DungeonBonus::HealTeam {
            percent: 0.25 + (rng.gen::<f32>() * 0.25), // 25-50%
        });

        pool.push(DungeonBonus::ExtraCrystals {
            amount: 10 + (floor as u32 / 5) * 5 + rng.gen_range(0..10),
        });

        // Stat boosts (randomly pick one)
        let boost_stat = match rng.gen_range(0..4) {
            0 => StatBoostType::Atk,
            1 => StatBoostType::Def,
            2 => StatBoostType::Spd,
            _ => StatBoostType::AllStats,
        };
        let boost_percent = if boost_stat == StatBoostType::AllStats {
            0.10 + rng.gen::<f32>() * 0.10 // 10-20% for all stats
        } else {
            0.15 + rng.gen::<f32>() * 0.20 // 15-35% for single stat
        };
        pool.push(DungeonBonus::StatBoost {
            stat: boost_stat,
            percent: boost_percent,
            floors: 3 + rng.gen_range(0..3), // 3-5 floors
        });

        // Capture boost (less common)
        if rng.gen_bool(0.3) {
            pool.push(DungeonBonus::CaptureBoost {
                multiplier: 1.5 + rng.gen::<f32>() * 0.5, // 1.5x-2x
                floors: 2 + rng.gen_range(0..2),
            });
        }

        // Skip floor (rare, not available on boss floors or early floors)
        if floor >= 5 && floor % 10 != 9 && rng.gen_bool(0.15) {
            pool.push(DungeonBonus::SkipFloor);
        }

        // Revive (only if there's a dead monster)
        if has_dead_monster {
            pool.push(DungeonBonus::ReviveMonster {
                hp_percent: 0.30 + rng.gen::<f32>() * 0.20, // 30-50%
            });
        }

        // Shuffle and pick 3 unique options
        use rand::seq::SliceRandom;
        pool.shuffle(&mut rng);

        for bonus in pool.into_iter().take(3) {
            options.push(bonus);
        }

        // Ensure we always have 3 options
        while options.len() < 3 {
            options.push(DungeonBonus::HealTeam { percent: 0.20 });
        }

        options
    }
}

/// Active bonus effect being tracked during a dungeon run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBonus {
    pub bonus_type: ActiveBonusType,
    pub floors_remaining: u8,
}

/// Types of active bonuses (stat boosts and capture boosts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActiveBonusType {
    StatBoost { stat: StatBoostType, percent: f32 },
    CaptureBoost { multiplier: f32 },
}

impl ActiveBonus {
    /// Tick down floors remaining, returns true if still active
    pub fn tick_floor(&mut self) -> bool {
        if self.floors_remaining > 0 {
            self.floors_remaining -= 1;
        }
        self.floors_remaining > 0
    }
}
