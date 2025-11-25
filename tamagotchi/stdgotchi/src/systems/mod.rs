/// Systems Module
///
/// ECS systems for stdgotchi, organized by responsibility.

pub mod afk;
pub mod animation;
pub mod autosave;
pub mod battle;
pub mod battle_loading;
pub mod battle_3v3_loading;
pub mod battle_result;
pub mod death;
pub mod fps;
pub mod input;
pub mod map_navigation;
pub mod menu;
pub mod render;
pub mod rest;
pub mod rustymon_navigation;
pub mod quest_navigation;

// Re-export all systems
pub use afk::afk_system;
pub use animation::{animation_cleanup_system, animation_init_system};
pub use autosave::{autosave_system, AutoSaveState};
pub use battle::battle_system;
pub use battle_loading::battle_loading_system;
pub use battle_3v3_loading::battle_3v3_loading_system;
pub use battle_result::battle_result_system;
pub use death::{death_detection_system, death_system};
pub use fps::fps_system;
pub use input::button_system;
pub use map_navigation::map_navigation_system;
pub use menu::menu_system;
pub use render::render_system;
pub use rest::rest_system;
pub use rustymon_navigation::{
    fragment_collection_system, rustymon_detail_system, rustymon_list_system,
    rustymon_skills_system, rustymon_summon_system,
};
pub use quest_navigation::quest_navigation_system;
