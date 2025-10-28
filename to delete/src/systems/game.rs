// Game logic system

use bevy_ecs::prelude::*;

/// Simple game state for testing
#[derive(Resource)]
pub struct GameState {
    pub frame_count: u64,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            frame_count: 0,
        }
    }
}

/// Update game logic
pub fn game_update_system(
    mut game_state: ResMut<GameState>,
) {
    game_state.frame_count += 1;

    if game_state.frame_count % 60 == 0 {
        log::info!("Frame: {}", game_state.frame_count);
    }
}
