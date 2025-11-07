//! Animation management system
//!
//! Handles initialization and cleanup of GIF animation state.

use bevy_ecs::prelude::*;
use log::info;
use std::time::{Duration, Instant};

use crate::ecs::resources::{AppMode, AppState, GifAnimationState};

/// Embedded GIF animation data
const MAP_GIF_DATA: &[u8] = include_bytes!("../../assets/images/map/1.gif");
const HORNET_GIF_DATA: &[u8] = include_bytes!("../../assets/images/hornet/38.gif");

/// System to initialize GIF animation when entering GifPlaying mode
pub fn animation_init_system(world: &mut World) {
    // Query app state
    let app_state = world.resource::<AppState>();
    let current_mode = app_state.current_mode;

    // Check if animation already exists
    let has_animation = world.get_non_send_resource::<GifAnimationState>().is_some();

    // Check if we need to initialize animation
    if current_mode == AppMode::GifPlaying && !has_animation {
        info!("Initializing GIF animation state...");

        // Load map as static image (first frame only)
        let map_image = match crate::display::StaticImage::new(MAP_GIF_DATA) {
            Ok(image) => image,
            Err(e) => {
                log::error!("Failed to load map image: {:?}", e);
                return;
            }
        };

        // Load hornet as animated GIF
        let mut hornet_player = match crate::display::GifPlayer::new(HORNET_GIF_DATA) {
            Ok(player) => player,
            Err(e) => {
                log::error!("Failed to load hornet GIF: {:?}", e);
                return;
            }
        };

        let (map_width, map_height) = map_image.dimensions();
        let (hornet_width, hornet_height) = hornet_player.dimensions();

        info!(
            "Map image: {}x{} (static background) - Display: 368x448",
            map_width, map_height
        );

        if map_width > 368 || map_height > 448 {
            log::warn!(
                "Map GIF is larger than display! Will be clipped. Map: {}x{}, Display: 368x448",
                map_width,
                map_height
            );
        }
        info!(
            "Hornet GIF: {}x{}, {} frames",
            hornet_width,
            hornet_height,
            hornet_player.frame_count()
        );

        // Calculate positions
        const DISPLAY_WIDTH: i32 = 368;
        const DISPLAY_HEIGHT: i32 = 448;

        // Center map horizontally (it's 384 wide, display is 368 wide)
        // This will crop 8 pixels on each side instead of 16 on one side
        let map_x = (DISPLAY_WIDTH - map_width as i32) / 2;
        let map_y = 0; // Top of screen
        let map_pos = (map_x, map_y);

        // Center hornet
        let hornet_x = (DISPLAY_WIDTH - hornet_width as i32) / 2;
        let hornet_y = (DISPLAY_HEIGHT - hornet_height as i32) / 2;
        let hornet_pos = (hornet_x, hornet_y);

        info!("Map at {:?}, Hornet at {:?}", map_pos, hornet_pos);

        // Determine total frames (3 loops of hornet only, map is static)
        let hornet_frame_count = hornet_player.frame_count();
        let total_frames = (hornet_frame_count * 3) as u32;

        // Frame delay (approx 50ms for 20 FPS - 2x speed)
        let frame_delay = Duration::from_millis(20);

        // Create animation state
        let animation_state = GifAnimationState {
            map_image,
            hornet_player,
            current_frame: 0,
            hornet_frame_index: 0,
            hornet_frame_count,
            total_frames,
            last_frame_time: Instant::now(),
            frame_delay,
            map_pos,
            hornet_pos,
        };

        // Insert as NonSend resource
        world.insert_non_send_resource(animation_state);

        info!("GIF animation initialized ({} total frames)", total_frames);
    }
}

/// System to cleanup GIF animation when exiting GifPlaying mode
pub fn animation_cleanup_system(world: &mut World) {
    // Query app state
    let app_state = world.resource::<AppState>();
    let current_mode = app_state.current_mode;

    // Check if animation exists
    let has_animation = world.get_non_send_resource::<GifAnimationState>().is_some();

    // If we have animation state but are no longer in GifPlaying mode, clean up
    if current_mode != AppMode::GifPlaying && has_animation {
        info!("Cleaning up GIF animation state...");
        world.remove_non_send_resource::<GifAnimationState>();
    }
}
