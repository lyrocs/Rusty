//! Monster Tamer Game System Module
//!
//! Implements core RPG mechanics for the Ragnarok Monster Tamer game:
//! - Monster collection and capture
//! - Expeditions (passive exploration)
//! - Dungeons (active real-time combat)
//! - Progression (leveling, fusion, upgrades)
//!
//! # Architecture
//! - `core`: Core data structures (Monster, Species, Skill, Element, Team, Player)
//! - `calculations`: Pure calculation functions (stats, xp, damage, combat)
//! - `systems`: Game systems (expedition, dungeon, combat, progression)
//! - `data`: Data loading from JSON files

// ============================================
// NEW ARCHITECTURE MODULES
// ============================================
pub mod core;
pub mod calculations;
pub mod systems;
pub mod data;

// Re-export core types
pub use core::{Element, Monster, MonsterStatus, Skill, SkillEffectType, Species, Team, Player};
pub use core::{MAX_TEAM_SIZE, MAX_MONSTERS};
pub use calculations::{stats, xp, damage, combat};
pub use data::TamerGameData;

// ============================================
// RETAINED MODULES (still needed for infrastructure)
// ============================================
pub mod enemy;          // Enemy data loading - will be refactored to use Species
pub mod battle;         // Battle logic - will be refactored for real-time combat
pub mod kill_tracker;   // Still useful for tracking
pub mod map;            // Map/Zone system - will be extended for expeditions
pub mod save;           // Save system - will be updated for new data
pub mod data_loader;    // Data loading - will load new JSON files
pub mod element_system; // Element advantages - to be merged with core::element

pub use enemy::*;
pub use battle::*;
pub use kill_tracker::*;
pub use map::*;
pub use save::*;
pub use data_loader::*;
pub use element_system::*;
