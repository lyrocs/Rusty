//! Expedition System
//!
//! Pre-calculated combat system where hero is sent on expeditions to farm monsters.
//! All combat is calculated upfront - battle screen only shows animations.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use super::enemy::Enemy;
use super::hero::Hero;

/// Simplified card that drops from monsters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Card {
    pub monster_id: u32,
    pub name: String,
    pub rarity: u8,        // 1-5 stars
    pub atk_bonus: u32,
    pub def_bonus: u32,
}

impl Card {
    /// Create a card from monster data
    pub fn new(monster_id: u32, name: String, rarity: u8, atk_bonus: u32, def_bonus: u32) -> Self {
        Self {
            monster_id,
            name,
            rarity,
            atk_bonus,
            def_bonus,
        }
    }
}

/// Hero state for expedition management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HeroState {
    /// Ready for action
    Ready,
    /// Currently on expedition (stores when it will end)
    OnExpedition {
        end_time: u64, // Unix timestamp (seconds since epoch)
    },
    /// Knocked out and recovering (stores when recovery completes)
    KO {
        recovery_time: u64, // Unix timestamp (seconds since epoch)
    },
}

impl Default for HeroState {
    fn default() -> Self {
        HeroState::Ready
    }
}

impl HeroState {
    /// Check if hero is ready for a new expedition
    pub fn is_ready(&self) -> bool {
        matches!(self, HeroState::Ready)
    }

    /// Check if currently on expedition
    pub fn is_on_expedition(&self) -> bool {
        matches!(self, HeroState::OnExpedition { .. })
    }

    /// Check if knocked out
    pub fn is_ko(&self) -> bool {
        matches!(self, HeroState::KO { .. })
    }

    /// Get remaining time for current state (in seconds)
    pub fn remaining_time(&self) -> Option<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();

        match self {
            HeroState::OnExpedition { end_time } => {
                if *end_time > now {
                    Some(*end_time - now)
                } else {
                    Some(0)
                }
            },
            HeroState::KO { recovery_time } => {
                if *recovery_time > now {
                    Some(*recovery_time - now)
                } else {
                    Some(0)
                }
            },
            HeroState::Ready => None,
        }
    }
}

/// Result of expedition calculation
#[derive(Debug, Clone)]
pub struct ExpeditionResult {
    /// Total duration in seconds
    pub duration_seconds: f32,
    /// Total damage hero will take
    pub total_damage: f32,
    /// Whether hero survives the expedition
    pub survives: bool,
    /// Number of kills completed (may be less than target if hero dies)
    pub kills_completed: u32,
    /// Time per kill (for animation timing)
    pub time_per_kill: f32,
    /// Damage per kill (for HP bar animation)
    pub damage_per_kill: f32,
}

/// Expedition size options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpeditionSize {
    Small = 1,
    Medium = 5,
    Large = 20,
    Huge = 50,
}

impl ExpeditionSize {
    /// Get all available sizes
    pub fn all() -> [ExpeditionSize; 4] {
        [
            ExpeditionSize::Small,
            ExpeditionSize::Medium,
            ExpeditionSize::Large,
            ExpeditionSize::Huge,
        ]
    }

    /// Get the count value
    pub fn count(&self) -> u32 {
        *self as u32
    }

    /// Get drop rate multiplier for this expedition size
    pub fn drop_multiplier(&self) -> f32 {
        match self {
            ExpeditionSize::Small => 1.0,
            ExpeditionSize::Medium => 1.1,
            ExpeditionSize::Large => 1.25,
            ExpeditionSize::Huge => 1.5,
        }
    }

    /// Get risk indicator emoji
    /// Conservative thresholds to account for rounding and calculation errors
    pub fn risk_indicator(damage_ratio: f32) -> &'static str {
        if damage_ratio < 0.4 {
            "✓ Safe"
        } else if damage_ratio < 0.7 {
            "⚠️ Risky"
        } else if damage_ratio < 0.95 {
            "☠️ Dangerous"
        } else {
            "💀 Will die"
        }
    }
}

// ============================================================================
// COMBAT FORMULAS
// ============================================================================

/// Calculate damage per second (DPS)
/// Formula: (ATK - DEF/2) minimum 1
pub fn calculate_dps(attacker_atk: u32, defender_def: u32) -> f32 {
    let damage = attacker_atk.saturating_sub(defender_def / 2);
    damage.max(1) as f32
}

/// Calculate time to kill one monster (in seconds)
/// Formula: monster HP / hero DPS
pub fn time_to_kill(hero: &Hero, monster: &Enemy) -> f32 {
    let hero_dps = calculate_dps(hero.attack as u32, monster.def);
    monster.max_hp as f32 / hero_dps
}

/// Calculate damage hero takes per monster killed
/// Formula: monster DPS × time to kill
pub fn damage_per_kill(hero: &Hero, monster: &Enemy) -> f32 {
    let monster_dps = calculate_dps(monster.atk, hero.defense as u32);
    let ttk = time_to_kill(hero, monster);
    monster_dps * ttk
}

/// Calculate full expedition results
pub fn calculate_expedition(hero: &Hero, monster: &Enemy, count: u32) -> ExpeditionResult {
    let time_per_kill = time_to_kill(hero, monster);
    let damage_per_kill = damage_per_kill(hero, monster);

    let total_time = time_per_kill * count as f32;
    let total_damage = damage_per_kill * count as f32;

    // Hero survives if total damage is less than current HP
    // If damage equals or exceeds HP, hero dies
    let survives = total_damage < hero.current_health as f32;

    let kills_before_death = if survives {
        count
    } else {
        // Calculate how many kills before running out of HP
        // Use floor to get complete kills - if you can do 16.9 kills,
        // you'll die during the 17th kill, so you complete 16 kills
        let kills_possible = (hero.current_health as f32 / damage_per_kill).floor() as u32;
        kills_possible
    };

    let damage_ratio = total_damage / hero.current_health as f32;

    log::info!(
        "Expedition calc: {} vs {} x{} | HP:{} DMG/kill:{:.1} Total DMG:{:.1} Ratio:{:.2} Survives:{} Kills:{}/{}",
        hero.name, monster.name, count,
        hero.current_health, damage_per_kill, total_damage, damage_ratio,
        survives, kills_before_death, count
    );

    ExpeditionResult {
        duration_seconds: total_time,
        total_damage,
        survives,
        kills_completed: kills_before_death,
        time_per_kill,
        damage_per_kill,
    }
}

/// Calculate card drops from an expedition
/// Returns vector of dropped cards
pub fn calculate_drops(
    monster: &Enemy,
    card_template: &Card,
    base_drop_rate: f32,
    kills: u32,
    expedition_size: ExpeditionSize,
) -> Vec<Card> {
    let multiplier = expedition_size.drop_multiplier();
    let effective_rate = base_drop_rate * multiplier;

    let mut drops = Vec::new();
    for _ in 0..kills {
        if rand::random::<f32>() < effective_rate {
            drops.push(card_template.clone());
        }
    }
    drops
}

/// Calculate XP per kill
pub fn xp_per_kill(monster_level: u32) -> u32 {
    monster_level * 10
}

/// Calculate XP needed for next level
pub fn xp_for_next_level(current_level: u32) -> u32 {
    current_level * 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dps_calculation() {
        // ATK 100, DEF 20 -> 100 - 10 = 90
        assert_eq!(calculate_dps(100, 20), 90.0);

        // ATK 50, DEF 60 -> 50 - 30 = 20
        assert_eq!(calculate_dps(50, 60), 20.0);

        // ATK 10, DEF 100 -> minimum 1
        assert_eq!(calculate_dps(10, 100), 1.0);
    }

    #[test]
    fn test_expedition_size_multipliers() {
        assert_eq!(ExpeditionSize::Small.drop_multiplier(), 1.0);
        assert_eq!(ExpeditionSize::Medium.drop_multiplier(), 1.1);
        assert_eq!(ExpeditionSize::Large.drop_multiplier(), 1.25);
        assert_eq!(ExpeditionSize::Huge.drop_multiplier(), 1.5);
    }

    #[test]
    fn test_risk_indicators() {
        assert_eq!(ExpeditionSize::risk_indicator(0.3), "✓ Safe");
        assert_eq!(ExpeditionSize::risk_indicator(0.5), "⚠️ Risky");
        assert_eq!(ExpeditionSize::risk_indicator(0.85), "☠️ Dangerous");
        assert_eq!(ExpeditionSize::risk_indicator(1.5), "💀 Will die");
    }

    #[test]
    fn test_hero_state() {
        let state = HeroState::Ready;
        assert!(state.is_ready());
        assert!(!state.is_on_expedition());
        assert!(!state.is_ko());
    }
}
