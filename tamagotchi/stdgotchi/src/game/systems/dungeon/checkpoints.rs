//! Checkpoint System
//!
//! Manages dungeon checkpoints and starting points.

/// Checkpoint interval (every 5 floors)
pub const CHECKPOINT_INTERVAL: u16 = 5;

/// Get the highest checkpoint unlocked for a floor
pub fn highest_checkpoint_for_floor(floor: u16) -> u16 {
    (floor / CHECKPOINT_INTERVAL) * CHECKPOINT_INTERVAL
}

/// Get available starting checkpoints based on highest floor reached
pub fn available_checkpoints(highest_floor: u16) -> Vec<u16> {
    let mut checkpoints = vec![1]; // Always can start from floor 1
    let mut checkpoint = CHECKPOINT_INTERVAL;

    while checkpoint <= highest_floor {
        checkpoints.push(checkpoint);
        checkpoint += CHECKPOINT_INTERVAL;
    }

    checkpoints
}

/// Get reward multiplier for starting floor
/// Based on GDD section 2.3.3
pub fn reward_multiplier_for_start_floor(start_floor: u16) -> f32 {
    match start_floor {
        0..=9 => 1.0,    // Base rewards
        10..=19 => 1.5,  // x1.5
        20..=29 => 2.0,  // x2.0
        _ => 2.5,        // x2.5 (floor 30+)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoints() {
        assert_eq!(highest_checkpoint_for_floor(7), 5);
        assert_eq!(highest_checkpoint_for_floor(10), 10);
        assert_eq!(highest_checkpoint_for_floor(23), 20);
    }

    #[test]
    fn test_available_checkpoints() {
        assert_eq!(available_checkpoints(3), vec![1]);
        assert_eq!(available_checkpoints(12), vec![1, 5, 10]);
        assert_eq!(available_checkpoints(25), vec![1, 5, 10, 15, 20, 25]);
    }

    #[test]
    fn test_reward_multiplier() {
        assert_eq!(reward_multiplier_for_start_floor(1), 1.0);
        assert_eq!(reward_multiplier_for_start_floor(10), 1.5);
        assert_eq!(reward_multiplier_for_start_floor(20), 2.0);
        assert_eq!(reward_multiplier_for_start_floor(35), 2.5);
    }
}
