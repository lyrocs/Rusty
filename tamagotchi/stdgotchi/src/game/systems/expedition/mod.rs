//! Expedition System
//!
//! Passive exploration mode where monsters gather XP, resources, and capture new monsters.
//! Based on GDD section 2.2.

pub mod expedition;
pub mod rewards;
pub mod capture;

pub use expedition::*;
pub use rewards::*;
pub use capture::*;
