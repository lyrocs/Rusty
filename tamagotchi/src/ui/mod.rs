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
pub mod helpers;
pub mod pages;
pub mod farming_header;

// Re-export colors, components, and pages for convenience
pub use colors::*;
pub use components::*;
pub use pages::*;
pub use farming_header::*;

// Re-export legacy items
pub use gif::*;
pub use text::*;
pub use battery::*;
