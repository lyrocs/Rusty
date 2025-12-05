//! Battle Mode System
//!
//! Defines different battle modes available in the game.

use serde::{Deserialize, Serialize};

/// Different battle modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleMode {
    /// Auto-attack battle mode (existing AFK farm style)
    Auto,
    /// Semi-active turn-based battle with skill usage
    SemiActive,
}

impl Default for BattleMode {
    fn default() -> Self {
        BattleMode::Auto
    }
}

impl BattleMode {
    /// Get display name for the battle mode
    pub fn display_name(&self) -> &'static str {
        match self {
            BattleMode::Auto => "Auto Battle",
            BattleMode::SemiActive => "Skill Battle",
        }
    }

    /// Get description for the battle mode
    pub fn description(&self) -> &'static str {
        match self {
            BattleMode::Auto => "Automatic combat with no player input",
            BattleMode::SemiActive => "Turn-based combat with active skill usage",
        }
    }
}
