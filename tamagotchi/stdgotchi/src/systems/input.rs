//! Input handling systems
//!
//! Button input processing.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, ButtonResource, InputEventChannel, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle button input from input thread
/// This system runs FIRST and collects all events from the channel.
/// It processes power button events and boot button events globally,
/// and forwards touch events to PendingInputEvents for mode-specific handling.
pub fn button_system(
    input_channel: Res<InputEventChannel>,
    mut button_res: NonSendMut<ButtonResource>,
    mut app_state: ResMut<AppState>,
    mut pending_events: ResMut<PendingInputEvents>,
) {
    // Clear any leftover events from previous frame
    pending_events.events.clear();

    // Collect all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::PowerPressed => {
                button_res.pwr_last_state = true;
            }
            InputEvent::PowerReleased => {
                // PWR button toggles screen on/off on release (rising edge)
                if button_res.pwr_last_state {
                    app_state.screen_on = !app_state.screen_on;
                    if app_state.screen_on {
                        app_state.needs_redraw = true;
                    }
                }
                button_res.pwr_last_state = false;
            }
            InputEvent::BootPressed => {
                // Handle boot button globally - opens Home (if screen is on)
                if app_state.screen_on {
                    log::info!("BOOT button pressed - Opening Home");
                    app_state.current_mode = AppMode::Home;
                    app_state.needs_redraw = true;
                }
            }
            // Forward touch events for mode-specific handling
            other_event => {
                pending_events.events.push(other_event);
            }
        }
    }
}
