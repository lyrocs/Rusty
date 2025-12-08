//! Monster Swapping
//!
//! Handles monster swapping during combat with cooldowns.

use crate::game::calculations::combat::SWAP_COOLDOWN;

/// Check if swap is available (cooldown expired)
pub fn can_swap(cooldown: f32) -> bool {
    cooldown <= 0.0
}

/// Perform a swap, returns the new cooldown value
pub fn perform_swap() -> f32 {
    SWAP_COOLDOWN
}

/// Update swap cooldowns for all team members
pub fn update_cooldowns(cooldowns: &mut [f32; 3], delta_time: f32) {
    for cooldown in cooldowns.iter_mut() {
        *cooldown = (*cooldown - delta_time).max(0.0);
    }
}

/// Get swap button state for UI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwapButtonState {
    /// Monster is available to swap to
    Available,
    /// Monster is on cooldown
    OnCooldown(f32), // Remaining seconds
    /// Monster is currently active
    Active,
    /// Monster is dead
    Dead,
}

/// Get swap button state for a monster index
pub fn get_swap_button_state(
    index: u8,
    active_index: u8,
    cooldown: f32,
    is_alive: bool,
) -> SwapButtonState {
    if !is_alive {
        SwapButtonState::Dead
    } else if index == active_index {
        SwapButtonState::Active
    } else if cooldown > 0.0 {
        SwapButtonState::OnCooldown(cooldown)
    } else {
        SwapButtonState::Available
    }
}
