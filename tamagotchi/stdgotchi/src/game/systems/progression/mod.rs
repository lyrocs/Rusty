//! Progression System
//!
//! Handles monster growth: leveling, fusion (duplicates), stat upgrades, and zone unlocking.
//! Based on GDD sections 2.1.4, 2.1.5, 2.5.

pub mod leveling;
pub mod fusion;
pub mod upgrade;

pub use leveling::*;
pub use fusion::*;
pub use upgrade::*;
