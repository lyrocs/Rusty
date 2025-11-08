//! Animation management system
//!
//! Handles initialization and cleanup of page-based animation.
//! NOTE: These systems are no longer used with the new menu system.
//! Kept for backwards compatibility.

use bevy_ecs::prelude::*;

/// System to initialize page when entering GifPlaying mode
/// NOTE: No longer used with new menu system
pub fn animation_init_system(_world: &mut World) {
    // Animation init system is no longer used with new menu system
}

/// System to cleanup page when exiting GifPlaying mode
/// NOTE: No longer used with new menu system
pub fn animation_cleanup_system(_world: &mut World) {
    // Animation cleanup system is no longer used with new menu system
}
