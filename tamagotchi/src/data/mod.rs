/// Data Module
///
/// Centralized game data management organized by domain.
/// Loads data from JSON files at compile-time and provides accessor functions.

pub mod common;
pub mod enemies;
pub mod maps;
pub mod items;
pub mod npcs;
pub mod drops;

// Re-export commonly used items
pub use common::*;
pub use enemies::{get_enemy_data, get_all_enemies, EnemyData};
pub use maps::{get_map_data, get_all_maps, get_map_name, get_map_connections,
               get_map_enemies, get_map_enemy_data, is_city, MapData, MAP_PRONTERA_ID};
pub use items::get_item_name;
pub use npcs::{get_npc_name, get_city_npcs};
pub use drops::{roll_drops, DropEntry};
