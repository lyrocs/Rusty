//! Combat System
//!
//! Real-time combat with auto-attacks, skills, monster swapping, and elemental reactions.
//! Based on GDD section 2.4.

pub mod combat_state;
pub mod reactions;
pub mod auras;
pub mod swap;

pub use combat_state::*;
pub use reactions::*;
pub use auras::*;
pub use swap::*;
