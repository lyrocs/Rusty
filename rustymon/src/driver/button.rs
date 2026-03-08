//! BOOT Button Driver for ESP-IDF
//!
//! Generic GPIO push-button driver with debounce and short/long-press detection.
//! On the Waveshare ESP32-C6-Touch-LCD-1.83 the BOOT button is wired to GPIO9
//! (active-low, requires internal pull-up).
//!
//! # Usage
//! ```no_run
//! let mut btn = ButtonDriver::new(pins.gpio9)?;
//! loop {
//!     if let Some(event) = btn.poll() {
//!         match event {
//!             ButtonEvent::ShortPress => { /* toggle */ }
//!             ButtonEvent::LongPress  => { /* confirm */ }
//!         }
//!     }
//! }
//! ```

use esp_idf_svc::hal::gpio::{Input, InputPin, OutputPin, PinDriver, Pull};
use std::time::Instant;

// Threshold separating a short press from a long press (milliseconds).
const LONG_PRESS_MS: u128 = 500;

/// A completed button press event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonEvent {
    /// Button released in less than `LONG_PRESS_MS`.
    ShortPress,
    /// Button held for at least `LONG_PRESS_MS`.
    LongPress,
}

/// GPIO push-button driver (active-low with internal pull-up).
///
/// Generic over any GPIO pin that supports both input and output modes so that
/// the ESP-IDF HAL's `set_pull` call compiles (it requires `InputPin + OutputPin`).
pub struct ButtonDriver<'a, P: InputPin + OutputPin> {
    pin: PinDriver<'a, P, Input>,
    was_pressed: bool,
    press_start: Option<Instant>,
}

impl<'a, P: InputPin + OutputPin> ButtonDriver<'a, P> {
    /// Initialise the button: configures the pin as input with pull-up.
    pub fn new(gpio: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut pin = PinDriver::input(gpio)?;
        pin.set_pull(Pull::Up)?;
        Ok(ButtonDriver {
            pin,
            was_pressed: false,
            press_start: None,
        })
    }

    /// Call once per game loop tick.
    ///
    /// Returns `Some(event)` on the falling edge (release) of a press, or
    /// `None` if nothing of interest happened this tick.
    pub fn poll(&mut self) -> Option<ButtonEvent> {
        let pressed = self.pin.is_low();

        let event = match (self.was_pressed, pressed) {
            // Rising edge – start timing
            (false, true) => {
                self.press_start = Some(Instant::now());
                None
            }
            // Falling edge – classify the press duration
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
