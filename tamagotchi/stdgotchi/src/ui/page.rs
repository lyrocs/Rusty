//! Page System
//!
//! Flexible page management for different screens/modes.

use crate::display::Sh8601Driver;
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
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>>;

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

/// Page manager to handle current page
pub struct PageManager {
    current_page: Option<Box<dyn Page>>,
}

impl PageManager {
    /// Create a new page manager
    pub fn new() -> Self {
        Self {
            current_page: None,
        }
    }

    /// Set the current page
    pub fn set_page(&mut self, mut page: Box<dyn Page>) {
        // Exit old page
        if let Some(old_page) = &mut self.current_page {
            old_page.on_exit();
        }

        // Enter new page
        page.on_enter();
        self.current_page = Some(page);
    }

    /// Update the current page
    /// Returns true if page is still active, false if done
    pub fn update(&mut self) -> bool {
        if let Some(page) = &mut self.current_page {
            page.update()
        } else {
            true
        }
    }

    /// Draw the current page
    pub fn draw(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        if let Some(page) = &mut self.current_page {
            let full_redraw = page.needs_full_redraw();
            page.draw(display, full_redraw)
        } else {
            Ok(())
        }
    }

    /// Check if current page needs clear
    pub fn needs_clear(&self) -> bool {
        self.current_page.as_ref().map_or(false, |p| p.needs_clear())
    }

    /// Get mutable reference to current page
    pub fn current_page_mut(&mut self) -> Option<&mut Box<dyn Page>> {
        self.current_page.as_mut()
    }
}

impl Default for PageManager {
    fn default() -> Self {
        Self::new()
    }
}
