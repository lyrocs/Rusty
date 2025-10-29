/// UI Module
///
/// Provides UI components and page rendering for the game.

// Legacy exports (keep for compatibility with old code)
pub mod gif;
pub mod text;
pub mod battery;

// New component-based architecture
pub mod colors;
pub mod components;
pub mod pages;

// Re-export colors and components for convenience
pub use colors::*;
pub use components::*;

// Re-export legacy items
pub use gif::*;
pub use text::*;
pub use battery::*;
