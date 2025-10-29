/// Core game domain module
///
/// Contains fundamental game state, types, and constants shared across all domains.

pub mod game_state;
pub mod types;
pub mod constants;
pub mod farming;
pub mod rest;

pub use game_state::*;
pub use types::*;
pub use constants::*;
