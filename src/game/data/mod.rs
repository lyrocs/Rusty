//! Data Loading Module
//!
//! Handles loading and validation of all JSON game data files.
//! Provides a central GameData store for accessing game content.

pub mod loader;
pub mod game_data;

pub use loader::*;
pub use game_data::*;
