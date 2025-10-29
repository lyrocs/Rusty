/// Hero stat management
///
/// Handles stat allocation, progression, and calculations.

/// Stat management methods that can be implemented on Hero
pub trait StatsExt {
    /// Add experience and handle level up
    fn add_exp(&mut self, amount: u32);

    /// Level up the hero
    fn level_up(&mut self);

    /// Add a stat point to a specific stat
    fn increase_stat(&mut self, stat_name: &str) -> bool;

    /// Reset all stats (refund all spent stat points)
    fn reset_stats(&mut self);

    /// Get HP percentage
    fn hp_percent(&self) -> u8;

    /// Get SP percentage
    fn sp_percent(&self) -> u8;

    /// Get EXP percentage
    fn exp_percent(&self) -> u8;
}

// Note: The actual implementation of StatsExt for Hero is in models.rs
// to avoid circular dependencies between hero modules.
