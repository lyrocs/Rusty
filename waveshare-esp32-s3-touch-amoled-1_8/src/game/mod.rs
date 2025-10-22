pub mod logic;
pub mod draw;

pub use logic::*;
pub use draw::*;

// Conway's Game of Life grid configuration
pub const GRID_WIDTH: usize = 52; // 368 / 7 ≈ 52
pub const GRID_HEIGHT: usize = 64; // 448 / 7 ≈ 64
pub const RESET_AFTER_GENERATIONS: usize = 300;
