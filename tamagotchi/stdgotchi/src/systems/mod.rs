/// Systems Module
///
///ECS systems for stdgotchi, organized by responsibility.

pub mod input;
pub mod render;

// Re-export all systems
pub use input::{button_system, touch_system};
pub use render::render_system;
