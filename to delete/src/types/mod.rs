// Common types and events for inter-thread communication

use serde::{Deserialize, Serialize};

/// Input events sent from input thread to game logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    Touch(u16, u16),
    TouchRelease,
    Button(ButtonType),
    Gesture(GestureType),
}

/// Button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonType {
    Boot,
    Power,
}

/// Gesture types from touch controller
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureType {
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    DoubleTap,
}

/// Render commands sent from game logic to render thread
#[derive(Debug, Clone)]
pub enum RenderCommand {
    Clear,
    DrawSprite { sprite_id: String, x: u16, y: u16, frame: usize },
    DrawRect { x: u16, y: u16, width: u16, height: u16, color: Color },
    DrawText { text: String, x: u16, y: u16, color: Color },
    Present,
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// Size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

/// Rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}
