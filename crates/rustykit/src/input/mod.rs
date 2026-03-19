//! Input subsystem: touch, buttons, and the input polling thread.

pub mod button;
pub mod thread;
pub mod touch;

/// High-level input events delivered to the application.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Tap at screen coordinates (sent on finger-up if no swipe detected).
    Tap { x: u16, y: u16 },
    /// Touch started/continuing at coordinates.
    TouchDown { x: u16, y: u16 },
    /// Touch released.
    TouchUp,
    /// Swipe gesture (sent once per touch, consumes the tap).
    Swipe(SwipeDirection),
    /// BOOT button (GPIO9) pressed.
    BootPressed,
    /// BOOT button released.
    BootReleased,
    /// Power button (GPIO18) pressed.
    PowerPressed,
    /// Power button released.
    PowerReleased,
}

/// Swipe directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}
