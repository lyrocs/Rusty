// Button driver implementation

use crate::hal::{ButtonDriver, ButtonState};
use parking_lot::Mutex;
use std::sync::Arc;

/// ESP32 Button Driver
pub struct Esp32ButtonDriver {
    last_state: ButtonState,
}

impl Esp32ButtonDriver {
    pub fn new() -> Self {
        Self {
            last_state: ButtonState::Released,
        }
    }
}

impl ButtonDriver for Esp32ButtonDriver {
    fn read_button(&mut self) -> ButtonState {
        // TODO: Implement actual GPIO reading
        // For now, always return Released
        ButtonState::Released
    }
}

/// Thread-safe button wrapper
pub type SharedButton = Arc<Mutex<dyn ButtonDriver>>;
