//! Game Calculations Module
//!
//! Contains all pure calculation functions for the game.
//! This is the SINGLE SOURCE OF TRUTH for all game math.
//!
//! # Design Principles
//! - All functions are pure (no side effects)
//! - All formulas match the GDD specifications
//! - Easy to unit test
//! - Easy to balance (change formulas in one place)

pub mod stats;
pub mod xp;
pub mod damage;
pub mod combat;

pub use stats::*;
pub use xp::*;
pub use damage::*;
pub use combat::*;
