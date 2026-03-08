//! Page System
//!
//! Flexible page management for different screens/modes.

use crate::display::St7789pDriver;
use std::error::Error;

/// Trait for pages that can be rendered
pub trait Page {
    /// Update page state (called every frame)
    /// Returns true if the page should continue, false if it's done
    fn update(&mut self) -> bool;

    /// Draw the page
    ///
    /// # Arguments
    /// * `display` - Display driver
    /// * `full_redraw` - If true, redraw everything (background + sprites).
    ///                   If false, only redraw changed elements (sprites only)
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>>;

    /// Called when entering this page
    fn on_enter(&mut self) {
        // Default: do nothing
    }

    /// Called when exiting this page
    fn on_exit(&mut self) {
        // Default: do nothing
    }

    /// Whether this page needs a full screen clear before drawing
    fn needs_clear(&self) -> bool {
        true // Default: clear screen
    }

    /// Mark page as needing full redraw (for external changes)
    fn mark_dirty(&mut self);

    /// Check if page needs full redraw
    fn needs_full_redraw(&self) -> bool;

    /// Get mutable reference as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
