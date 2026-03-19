//! GPIO push-button driver with debounce and short/long-press detection.
//!
//! Extracted from rustymon.

use esp_idf_svc::hal::gpio::{Input, InputPin, OutputPin, PinDriver, Pull};
use std::time::Instant;

const LONG_PRESS_MS: u128 = 500;

/// A completed button press event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonEvent {
    ShortPress,
    LongPress,
}

/// GPIO push-button driver (active-low with internal pull-up).
pub struct ButtonDriver<'a, P: InputPin + OutputPin> {
    pin: PinDriver<'a, P, Input>,
    was_pressed: bool,
    press_start: Option<Instant>,
}

impl<'a, P: InputPin + OutputPin> ButtonDriver<'a, P> {
    pub fn new(gpio: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut pin = PinDriver::input(gpio)?;
        pin.set_pull(Pull::Up)?;
        Ok(ButtonDriver {
            pin,
            was_pressed: false,
            press_start: None,
        })
    }

    /// Poll once per tick. Returns `Some(event)` on button release.
    pub fn poll(&mut self) -> Option<ButtonEvent> {
        let pressed = self.pin.is_low();

        let event = match (self.was_pressed, pressed) {
            (false, true) => {
                self.press_start = Some(Instant::now());
                None
            }
            (true, false) => self.press_start.take().map(|t| {
                if t.elapsed().as_millis() < LONG_PRESS_MS {
                    ButtonEvent::ShortPress
                } else {
                    ButtonEvent::LongPress
                }
            }),
            _ => None,
        };

        self.was_pressed = pressed;
        event
    }
}
