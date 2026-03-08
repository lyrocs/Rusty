//! Input handling systems
//!
//! Button input processing.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, ButtonResource, DisplayResource, InputEventChannel, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle button input from input thread
/// This system runs FIRST and collects all events from the channel.
/// It processes boot button and power button events globally,
/// and forwards touch events to PendingInputEvents for mode-specific handling.
pub fn button_system(
    input_channel: Res<InputEventChannel>,
    mut button_res: NonSendMut<ButtonResource>,
    mut app_state: ResMut<AppState>,
    mut pending_events: ResMut<PendingInputEvents>,
    mut display_res: NonSendMut<DisplayResource>,
) {
    // Clear any leftover events from previous frame
    pending_events.events.clear();

    // Collect all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::PowerPressed => {
                log::info!("[SYSTEM] PWR button pressed - toggling screen");
            }
            InputEvent::PowerReleased => {
                // PWR button toggles screen on/off on release (rising edge)
                app_state.screen_on = !app_state.screen_on;
                log::info!("[SYSTEM] Screen toggled: {}", if app_state.screen_on { "ON" } else { "OFF" });

                if app_state.screen_on {
                    // Turn screen on
                    if let Err(e) = display_res.display.display_on() {
                        log::error!("Failed to turn display on: {:?}", e);
                    }
                    app_state.needs_redraw = true;
                } else {
                    // Turn screen off
                    if let Err(e) = display_res.display.display_off() {
                        log::error!("Failed to turn display off: {:?}", e);
                    }
                }
            }
            InputEvent::BootPressed => {
                // Handle boot button globally - opens Home (if screen is on)
                if app_state.screen_on {
                    log::info!("BOOT button pressed - Opening Home");
                    app_state.current_mode = AppMode::Home;
                    app_state.needs_redraw = true;
                }
            }
            InputEvent::BootReleased => {
                // Ignore boot release for now
            }
            // Forward touch events for mode-specific handling
            other_event => {
                pending_events.events.push(other_event);
            }
        }
    }
}
