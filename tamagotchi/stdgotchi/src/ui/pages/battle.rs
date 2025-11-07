//! Battle Page
//!
//! Displays a map background with animated monster(s).

use crate::display::Sh8601Driver;
use crate::ui::page::Page;
use crate::ui::sprite::{AnimatedSprite, Background};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::Duration;

/// Battle page showing map and monster(s)
pub struct BattlePage {
    background: Background,
    monsters: Vec<AnimatedSprite>,
    fps: f32,
    first_draw: bool,
}

impl BattlePage {
    /// Create a new battle page
    ///
    /// # Arguments
    /// * `map_data` - GIF data for the background map
    /// * `map_position` - Position of the map
    pub fn new(map_data: &[u8], map_position: (i32, i32)) -> Result<Self, Box<dyn Error>> {
        let background = Background::new(map_data, map_position)?;

        Ok(Self {
            background,
            monsters: Vec::new(),
            fps: 0.0,
            first_draw: true,
        })
    }

    /// Add a left-centered monster (positioned in left half of screen)
    ///
    /// # Arguments
    /// * `monster_data` - GIF data for the monster animation
    /// * `frame_delay` - Time between animation frames
    /// * `loops` - Number of times to loop (None for infinite)
    pub fn add_left_centered_monster(
        &mut self,
        monster_data: &[u8],
        frame_delay: Duration,
        loops: Option<u32>,
    ) -> Result<(), Box<dyn Error>> {
        // Create sprite to get dimensions
        let sprite = AnimatedSprite::new(monster_data, (0, 0), frame_delay, loops)?;
        let (width, height) = sprite.dimensions();

        // Calculate left-centered position (assuming 368x448 display)
        const DISPLAY_WIDTH: i32 = 368;
        const DISPLAY_HEIGHT: i32 = 448;
        const HALF_WIDTH: i32 = DISPLAY_WIDTH / 2;

        let x = (HALF_WIDTH - width as i32) / 2;
        let y = (DISPLAY_HEIGHT - height as i32) / 2;

        // Recreate sprite with correct position
        let sprite = AnimatedSprite::new(monster_data, (x, y), frame_delay, loops)?;

        log::info!(
            "Added left-centered monster at ({}, {}): {}x{}",
            x,
            y,
            width,
            height
        );

        self.monsters.push(sprite);
        Ok(())
    }

    /// Add a right-centered monster/hero (positioned in right half of screen)
    ///
    /// # Arguments
    /// * `monster_data` - GIF data for the monster/hero animation
    /// * `frame_delay` - Time between animation frames
    /// * `loops` - Number of times to loop (None for infinite)
    pub fn add_right_centered_monster(
        &mut self,
        monster_data: &[u8],
        frame_delay: Duration,
        loops: Option<u32>,
    ) -> Result<(), Box<dyn Error>> {
        // Create sprite to get dimensions
        let sprite = AnimatedSprite::new(monster_data, (0, 0), frame_delay, loops)?;
        let (width, height) = sprite.dimensions();

        // Calculate right-centered position (assuming 368x448 display)
        const DISPLAY_WIDTH: i32 = 368;
        const DISPLAY_HEIGHT: i32 = 448;
        const HALF_WIDTH: i32 = DISPLAY_WIDTH / 2;

        let x = HALF_WIDTH + (HALF_WIDTH - width as i32) / 2;
        let y = (DISPLAY_HEIGHT - height as i32) / 2;

        // Recreate sprite with correct position
        let sprite = AnimatedSprite::new(monster_data, (x, y), frame_delay, loops)?;

        log::info!(
            "Added right-centered hero/monster at ({}, {}): {}x{}",
            x,
            y,
            width,
            height
        );

        self.monsters.push(sprite);
        Ok(())
    }

    /// Set FPS for display
    pub fn set_fps(&mut self, fps: f32) {
        self.fps = fps;
    }

    /// Draw FPS overlay (without flushing)
    fn draw_fps_overlay(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Draw semi-transparent background box for FPS
        Rectangle::new(Point::new(5, 2), Size::new(70, 15))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        // Draw FPS text
        let mut fps_str = heapless::String::<16>::new();
        write!(fps_str, "FPS: {:.1}", self.fps).ok();

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);
        Text::new(&fps_str, Point::new(10, 10), text_style).draw(display)?;

        Ok(())
    }
}

impl Page for BattlePage {
    fn update(&mut self) -> bool {
        // Update all monster animations
        for monster in &mut self.monsters {
            monster.update();
        }

        // Continue running (return false to exit page)
        true
    }

    fn draw(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Draw full background only on first frame
        if full_redraw {
            self.background.draw(display)?;
        } else {
            // For subsequent frames, only clear and redraw sprite zones
            for monster in &self.monsters {
                // Get sprite bounds
                let bounds = monster.bounds();

                // Clear this sprite's area by redrawing background region
                self.background.draw_region(display, bounds)?;
            }
        }

        // Draw all animated monsters (they change every frame)
        for monster in &self.monsters {
            monster.draw(display)?;
        }

        // Draw FPS overlay (no flush)
        self.draw_fps_overlay(display)?;

        // Flush to display once at the end
        display.flush()?;

        // Mark that we've done the first draw
        if self.first_draw {
            self.first_draw = false;
        }

        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering battle page");
        self.first_draw = true; // Force full redraw when entering
    }

    fn on_exit(&mut self) {
        log::info!("Exiting battle page");
    }

    fn mark_dirty(&mut self) {
        self.first_draw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.first_draw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
