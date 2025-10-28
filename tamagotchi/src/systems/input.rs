// Input processing system

use bevy_ecs::prelude::*;
use crossbeam_channel::Receiver;
use crate::types::InputEvent;

/// Resource to receive input events from input thread
#[derive(Resource)]
pub struct InputEventReceiver(pub Receiver<InputEvent>);

/// Process input events and update game state
pub fn process_input_system(
    input_rx: Res<InputEventReceiver>,
) {
    // Drain all pending input events
    while let Ok(event) = input_rx.0.try_recv() {
        match event {
            InputEvent::Touch(x, y) => {
                log::debug!("Touch at ({}, {})", x, y);
                // TODO: Handle touch input
            }
            InputEvent::TouchRelease => {
                log::debug!("Touch released");
            }
            InputEvent::Button(btn) => {
                log::debug!("Button pressed: {:?}", btn);
            }
            InputEvent::Gesture(gesture) => {
                log::debug!("Gesture detected: {:?}", gesture);
            }
        }
    }
}
