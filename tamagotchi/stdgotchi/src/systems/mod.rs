/// Systems Module
///
/// ECS systems for stdgotchi Monster Tamer, organized by responsibility.

pub mod animation;
pub mod autosave;
pub mod battle;
pub mod battle_loading;
pub mod battle_result;
pub mod death;
pub mod dungeon_navigation;
pub mod expedition_navigation;
pub mod fps;
pub mod home_navigation;
pub mod input;
pub mod map_navigation;
pub mod menu;
pub mod monster_navigation;
pub mod render;
pub mod utility_navigation;

// Re-export all systems
pub use animation::{animation_cleanup_system, animation_init_system};
pub use autosave::{autosave_system, AutoSaveState};
pub use battle::battle_system;
pub use battle_loading::battle_loading_system;
pub use battle_result::battle_result_system;
pub use death::{death_detection_system, death_system};
pub use dungeon_navigation::{dungeon_combat_navigation_system, between_floors_navigation_system, dungeon_defeat_navigation_system};
pub use expedition_navigation::{expedition_navigation_system, create_expedition_map_page, check_expedition_completion};
pub use fps::fps_system;
pub use home_navigation::{home_navigation_system, update_home_page_data};
pub use input::button_system;
pub use map_navigation::map_navigation_system;
pub use menu::menu_system;
pub use monster_navigation::monster_navigation_system;
pub use render::render_system;
pub use utility_navigation::utility_navigation_system;
