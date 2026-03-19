//! # Rustykit
//!
//! A Rust framework for ESP32-C6 touch LCD devices.
//!
//! Rustykit abstracts away all hardware drivers (display, touch, SD card, WiFi)
//! and provides a high-level API for building applications on the
//! Waveshare ESP32-C6-Touch-LCD-1.83 board.
//!
//! # Quick Start
//!
//! ```ignore
//! use rustykit::prelude::*;
//!
//! fn main() {
//!     esp_idf_svc::sys::link_patches();
//!     esp_idf_svc::log::EspLogger::initialize_default();
//!
//!     let app = App::init(AppConfig::default()).unwrap();
//!     app.run(|ctx| {
//!         for event in &ctx.input {
//!             if let InputEvent::Tap { x, y } = event {
//!                 log::info!("Tap at ({}, {})", x, y);
//!             }
//!         }
//!         ctx.canvas.clear(Color::BLACK);
//!         ctx.canvas.draw_text("Hello!", 20, 30, FontSize::Large, Color::YELLOW);
//!     });
//! }
//! ```

pub mod app;
pub mod display;
pub mod error;
pub mod hw;
pub mod input;
pub mod net;
pub mod sprite;
pub mod storage;
pub mod ui;

/// Import everything you need with `use rustykit::prelude::*`.
pub mod prelude {
    pub use crate::app::{App, AppConfig, AppContext};
    pub use crate::display::{Canvas, Color, ColorMode, FontSize};
    pub use crate::error::{Result, RustyError};
    pub use crate::input::{InputEvent, SwipeDirection};
    pub use crate::net::{Network, WifiConfig};
    pub use crate::sprite::{AnimationPlayer, Sprite};
    pub use crate::storage::Storage;
    pub use crate::ui::widgets::*;
    pub use crate::ui::{View, Widget};
}
