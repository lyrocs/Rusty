//! FPS tracking system
//!
//! Tracks and updates frames per second for performance monitoring.

use bevy_ecs::prelude::*;
use std::time::Duration;

use crate::ecs::resources::AppState;

/// System to track FPS
pub fn fps_system(mut app_state: ResMut<AppState>) {
    app_state.frame_count += 1;

    let elapsed = app_state.last_fps_update.elapsed();

    // Update FPS every second
    if elapsed >= Duration::from_secs(1) {
        app_state.fps = app_state.frame_count as f32 / elapsed.as_secs_f32();
        app_state.frame_count = 0;
        app_state.last_fps_update = std::time::Instant::now();
    }
}
