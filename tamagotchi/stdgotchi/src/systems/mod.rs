/// Systems Module
///
/// ECS systems for stdgotchi, organized by responsibility.

pub mod animation;
pub mod fps;
pub mod input;
pub mod render;

// Re-export all systems
pub use animation::{animation_cleanup_system, animation_init_system};
pub use fps::fps_system;
pub use input::{button_system, touch_system};
pub use render::render_system;
