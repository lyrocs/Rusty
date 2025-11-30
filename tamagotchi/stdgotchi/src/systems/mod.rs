/// Systems Module
///
/// ECS systems for stdgotchi, organized by responsibility.

pub mod afk;
pub mod animation;
pub mod autosave;
pub mod battle;
pub mod battle_loading;
pub mod battle_result;
pub mod cards;
pub mod death;
pub mod fps;
pub mod input;
pub mod map_navigation;
pub mod menu;
pub mod pokemon_info;
pub mod render;
pub mod rest;
pub mod quest_navigation;
pub mod expedition_setup;
pub mod expedition_in_progress;
pub mod expedition_summary;
pub mod hero_info;
pub mod hero_state;

// Re-export all systems
pub use afk::afk_system;
pub use animation::{animation_cleanup_system, animation_init_system};
pub use autosave::{autosave_system, AutoSaveState};
pub use battle::battle_system;
pub use battle_loading::battle_loading_system;
pub use battle_result::battle_result_system;
pub use cards::cards_system;
pub use death::{death_detection_system, death_system};
pub use fps::fps_system;
pub use input::button_system;
pub use map_navigation::map_navigation_system;
pub use menu::menu_system;
pub use pokemon_info::pokemon_info_system;
pub use render::render_system;
pub use rest::rest_system;
pub use quest_navigation::quest_navigation_system;
pub use expedition_setup::expedition_setup_system;
pub use expedition_in_progress::expedition_in_progress_system;
pub use expedition_summary::expedition_summary_system;
pub use hero_info::hero_info_system;
pub use hero_state::hero_state_system;
