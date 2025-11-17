//! Menu Page
//!
//! Main menu for navigating between game modes.

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;

use crate::display::Sh8601Driver;
use crate::ui::page::Page;

/// Menu button action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Map,
    Battle,
    Rustymon,
    Fragments,
    Quests,
}

/// Touch button area
struct TouchButton {
    bounds: (i32, i32, u32, u32), // x, y, width, height
    action: MenuAction,
}

impl TouchButton {
    fn contains(&self, x: i32, y: i32) -> bool {
        let (bx, by, bw, bh) = self.bounds;
        x >= bx && x < bx + bw as i32 && y >= by && y < by + bh as i32
    }
}

/// Menu page
pub struct MenuPage {
    background_color: Rgb888,
    touch_buttons: Vec<TouchButton>,
    needs_full_redraw: bool,
    has_active_battle: bool,
}

impl MenuPage {
    /// Create a new menu page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_buttons: Vec::new(),
            needs_full_redraw: true,
            has_active_battle: false,
        }
    }

    /// Set whether there's an active battle (affects Battle button visibility)
    pub fn set_has_active_battle(&mut self, has_battle: bool) {
        if self.has_active_battle != has_battle {
            self.has_active_battle = has_battle;
            self.needs_full_redraw = true; // Force redraw when battle state changes
        }
    }

    /// Handle touch input on menu
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<MenuAction> {
        for button in &self.touch_buttons {
            if button.contains(x, y) {
                log::info!("Menu button touched: {:?}", button.action);
                return Some(button.action);
            }
        }
        None
    }

    /// Draw the menu
    pub fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Default to no active battle - use draw_with_battle_state for conditional battle button
        self.draw_with_battle_state(display, full_redraw, false)
    }

    /// Draw the menu with battle state information
    pub fn draw_with_battle_state(&mut self, display: &mut Sh8601Driver, full_redraw: bool, has_active_battle: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        // Clear old touch buttons
        self.touch_buttons.clear();

        // Draw title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("MENU", Point::new(150, 40), title_style).draw(display)?;

        // Button dimensions
        let button_width = 280u32;
        let button_height = 50u32;
        let button_x = (368 - button_width) as i32 / 2; // Center horizontally
        let start_y = 70i32;
        let spacing = 68i32;

        let mut current_slot = 0;

        // Draw Map button
        self.draw_button(
            display,
            button_x,
            start_y + spacing * current_slot,
            button_width,
            button_height,
            "MAP",
            Rgb888::new(40, 80, 120),
            MenuAction::Map,
        )?;
        current_slot += 1;

        // Draw Battle button only if there's an active battle
        if has_active_battle {
            self.draw_button(
                display,
                button_x,
                start_y + spacing * current_slot,
                button_width,
                button_height,
                "BATTLE",
                Rgb888::new(120, 40, 40),
                MenuAction::Battle,
            )?;
            current_slot += 1;
        }

        // Draw Rustymon button
        self.draw_button(
            display,
            button_x,
            start_y + spacing * current_slot,
            button_width,
            button_height,
            "RUSTYMON",
            Rgb888::new(100, 40, 120),
            MenuAction::Rustymon,
        )?;
        current_slot += 1;

        // Draw Quests button
        self.draw_button(
            display,
            button_x,
            start_y + spacing * current_slot,
            button_width,
            button_height,
            "QUESTS",
            Rgb888::new(40, 120, 80),
            MenuAction::Quests,
        )?;
        current_slot += 1;

        // Draw Fragments button
        self.draw_button(
            display,
            button_x,
            start_y + spacing * current_slot,
            button_width,
            button_height,
            "FRAGMENTS",
            Rgb888::new(120, 100, 40),
            MenuAction::Fragments,
        )?;

        // Draw hint text at bottom
        let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        Text::new("Tap to select", Point::new(130, 435), hint_style).draw(display)?;

        display.flush()?;
        Ok(())
    }

    /// Draw a menu button
    fn draw_button(
        &mut self,
        display: &mut Sh8601Driver,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        title: &str,
        color: Rgb888,
        action: MenuAction,
    ) -> Result<(), Box<dyn Error>> {
        // Draw button background
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(display)?;

        // Draw button border
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 2))
            .draw(display)?;

        // Draw title centered vertically and horizontally
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let title_x = x + (width as i32 - (title.len() as i32 * 10)) / 2;
        // Font height is 20, text baseline is at bottom of glyph
        // Center vertically: y + (height/2) + (font_height/2 - descent)
        // For FONT_10X20, approximate vertical center adjustment is +7
        let title_y = y + (height as i32 / 2) + 7;
        Text::new(title, Point::new(title_x, title_y), title_style).draw(display)?;

        // Register touch button
        self.touch_buttons.push(TouchButton {
            bounds: (x, y, width, height),
            action,
        });

        Ok(())
    }
}

// Page trait implementation
impl Page for MenuPage {
    fn update(&mut self) -> bool {
        true // Always active
    }

    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        self.draw_with_battle_state(display, full_redraw, self.has_active_battle)
    }

    fn mark_dirty(&mut self) {
        self.needs_full_redraw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.needs_full_redraw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
