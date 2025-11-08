/// Systems Module
///
/// ECS systems for stdgotchi, organized by responsibility.

pub mod animation;
pub mod autosave;
pub mod fps;
pub mod input;
pub mod map_navigation;
pub mod menu;
pub mod render;

// Re-export all systems
pub use animation::{animation_cleanup_system, animation_init_system};
pub use autosave::{autosave_system, AutoSaveState};
pub use fps::fps_system;
pub use input::button_system;
pub use map_navigation::{hero_overview_system, map_navigation_system};
pub use menu::menu_system;
pub use render::render_system;
