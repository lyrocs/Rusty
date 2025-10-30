/// UI Pages Module
///
/// Individual page rendering functions organized by page type.

pub mod overview;
pub mod stats;
pub mod equipment;
pub mod farm;
pub mod rest;
pub mod battle;
pub mod map;
pub mod menu;
pub mod inventory;
pub mod quests;
pub mod settings;
pub mod jrpg_battle;
pub mod crafting;

// Re-export all page drawing functions
pub use overview::draw_overview_page;
pub use stats::draw_stats_page;
pub use equipment::draw_equipment_page;
pub use farm::draw_farm_page;
pub use rest::draw_rest_page;
pub use battle::draw_battle_page;
pub use map::draw_map_page;
pub use menu::draw_menu;
pub use inventory::draw_inventory;
pub use quests::draw_quests_page;
pub use settings::draw_settings_page;
pub use jrpg_battle::draw_jrpg_battle_page;
pub use crafting::draw_crafting_page;
