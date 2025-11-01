// Embassy task modules
pub mod channels;
pub mod game;
pub mod input;
pub mod render;
pub mod storage;

// Re-export commonly used types
pub use channels::{InputEvent, RenderCommand, SaveCommand};
