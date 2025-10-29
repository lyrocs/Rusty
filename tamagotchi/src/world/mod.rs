/// World Module
///
/// Manages game world, locations, and navigation.

pub mod location;
pub mod navigation;

// Re-export commonly used items
pub use location::*;
pub use navigation::*;
