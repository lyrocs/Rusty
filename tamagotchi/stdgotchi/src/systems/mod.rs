/// Systems Module
///
/// ECS systems for stdgotchi, organized by responsibility.

pub mod animation;
pub mod autosave;
pub mod battle;
pub mod battle_loading;
pub mod crafting;
pub mod death;
pub mod equipment;
pub mod fps;
pub mod input;
pub mod inventory;
pub mod map_navigation;
pub mod menu;
pub mod render;
pub mod stats_allocation;

// Re-export all systems
pub use animation::{animation_cleanup_system, animation_init_system};
pub use autosave::{autosave_system, AutoSaveState};
pub use battle::battle_system;
pub use battle_loading::battle_loading_system;
pub use crafting::crafting_system;
pub use death::{death_detection_system, death_system};
pub use equipment::equipment_system;
pub use fps::fps_system;
pub use input::button_system;
pub use inventory::inventory_system;
pub use map_navigation::{hero_overview_system, map_navigation_system};
pub use menu::menu_system;
pub use render::render_system;
pub use stats_allocation::stats_allocation_system;
