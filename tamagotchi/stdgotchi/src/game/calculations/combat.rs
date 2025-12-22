//! Combat Timing Calculations
//!
//! All combat timing calculations: ATK bar fill rate, SKL bar gain, swap cooldown.
//! This is the SINGLE SOURCE OF TRUTH for combat timing formulas.

/// Base SPD constant for speed scaling
/// With new formula, max speed advantage is capped at 2x
pub const BASE_SPD: f32 = 120.0;

/// SKL bar gain per attack (20%)
pub const SKL_GAIN_PER_ATTACK: f32 = 0.20;

/// Swap cooldown in seconds
pub const SWAP_COOLDOWN: f32 = 3.0;

/// Aura duration from auto-attack (seconds)
pub const AURA_DURATION_AUTO: f32 = 2.0;

/// Aura duration from skill (seconds)
pub const AURA_DURATION_SKILL: f32 = 4.0;

/// Stun duration from Electrocute reaction (seconds)
pub const STUN_DURATION_ELECTROCUTE: f32 = 1.0;

/// DoT tick interval (seconds)
pub const DOT_TICK_INTERVAL: f32 = 0.5;

/// Calculate ATK bar fill rate per second based on SPD
/// Uses diminishing returns to cap speed advantage at ~2x
/// SPD 30 = 0.75 attacks/sec, SPD 60 = 1.0 attacks/sec, SPD 120 = 1.5 attacks/sec
/// Max ratio between fastest and slowest is 2x (1.5 / 0.75)
pub fn atk_bar_fill_rate(spd: u16) -> f32 {
    // Base rate of 0.5 + scaled rate caps advantage at 2x
    0.5 + (spd as f32 / BASE_SPD)
}

/// Calculate ATK bar progress for a frame
pub fn update_atk_bar(current: f32, spd: u16, delta_time: f32) -> f32 {
    let fill_rate = atk_bar_fill_rate(spd);
    (current + fill_rate * delta_time).min(1.0)
}

/// Calculate SKL bar after an attack (gains 20% per attack)
pub fn update_skl_bar_after_attack(current: f32) -> f32 {
    (current + SKL_GAIN_PER_ATTACK).min(1.0)
}

/// Update swap cooldown (decreases over time)
pub fn update_swap_cooldown(current: f32, delta_time: f32) -> f32 {
    (current - delta_time).max(0.0)
}

/// Check if swap is available (cooldown expired)
pub fn can_swap(cooldown: f32) -> bool {
    cooldown <= 0.0
}

/// Check if skill can be used (SKL bar full)
pub fn can_use_skill(skl_bar: f32) -> bool {
    skl_bar >= 1.0
}

/// Calculate attacks per second for display
pub fn attacks_per_second(spd: u16) -> f32 {
    atk_bar_fill_rate(spd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atk_bar_fill_rate() {
        // New formula: 0.5 + (spd / 120.0)
        // SPD 30 = 0.75, SPD 60 = 1.0, SPD 120 = 1.5
        assert!((atk_bar_fill_rate(60) - 1.0).abs() < 0.001);
        assert!((atk_bar_fill_rate(120) - 1.5).abs() < 0.001);
        assert!((atk_bar_fill_rate(30) - 0.75).abs() < 0.001);

        // Max ratio is 2x (1.5 / 0.75)
        let slow = atk_bar_fill_rate(30);
        let fast = atk_bar_fill_rate(120);
        assert!((fast / slow - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_update_atk_bar() {
        // SPD 60 = 1.0 fill/second, after 0.5 seconds
        let new_bar = update_atk_bar(0.0, 60, 0.5);
        assert!((new_bar - 0.5).abs() < 0.001);

        // Capped at 1.0
        let new_bar = update_atk_bar(0.9, 60, 0.5);
        assert_eq!(new_bar, 1.0);
    }

    #[test]
    fn test_skl_bar() {
        let new_bar = update_skl_bar_after_attack(0.0);
        assert!((new_bar - 0.2).abs() < 0.001);

        // After 5 attacks = 100%
        let mut bar = 0.0;
        for _ in 0..5 {
            bar = update_skl_bar_after_attack(bar);
        }
        assert_eq!(bar, 1.0);
    }
}
