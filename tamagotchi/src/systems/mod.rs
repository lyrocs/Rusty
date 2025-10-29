/// Systems Module
///
/// ECS systems for game logic, organized by responsibility.
/// Currently re-exports from tamagotchi/systems.rs for backward compatibility.

pub mod input;

// Re-export input systems
pub use input::{tamagotchi_button_system, tamagotchi_touch_system};

// Re-export other systems from tamagotchi (will be extracted in future refactoring)
pub use crate::tamagotchi::systems::{
    tamagotchi_update_system,
    tamagotchi_render_system,
    tamagotchi_save_system,
};
