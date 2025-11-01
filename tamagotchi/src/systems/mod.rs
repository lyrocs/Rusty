/// Systems Module
///
/// ECS systems for game logic, organized by responsibility.

pub mod animations;
pub mod input;
pub mod render;
pub mod save;
pub mod update;
pub mod idle_farm_update;

// Re-export all systems
pub use animations::*;
pub use input::{tamagotchi_button_system, tamagotchi_touch_system};
pub use render::tamagotchi_render_system;
pub use save::tamagotchi_save_system;
pub use update::tamagotchi_update_system;
pub use idle_farm_update::*;
