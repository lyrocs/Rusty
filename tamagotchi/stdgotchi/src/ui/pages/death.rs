//! Death Page
//!
//! Displays when hero dies with 2-minute respawn timer

use crate::display::Sh8601Driver;
use crate::ui::page::Page;
use crate::ui::sprite::AnimatedSprite;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::{Duration, Instant};

/// Death page showing hero sit animation with respawn timer
pub struct DeathPage {
    background_color: Rgb888,
    sit_sprite: Option<AnimatedSprite>,
    death_time: Instant,
    respawn_duration: Duration,
    needs_full_redraw: bool,
    can_respawn: bool,
    last_remaining_seconds: Option<u64>, // Track last displayed time for refresh detection
}

impl DeathPage {
    /// Create a new death page
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Load novice sit sprite (16.gif) - use embedded only for simplicity
        let sit_data = include_bytes!("../../../../assets/images/novice/16.gif");

        let sit_sprite = AnimatedSprite::new(sit_data, (160, 200), Duration::from_millis(200), None)?;

        Ok(Self {
            background_color: Rgb888::new(20, 10, 10), // Dark red tint
            sit_sprite: Some(sit_sprite),
            death_time: Instant::now(),
            respawn_duration: Duration::from_secs(120), // 2 minutes
            needs_full_redraw: true,
            can_respawn: false,
            last_remaining_seconds: None,
        })
    }

    /// Check if respawn time has elapsed
    pub fn can_respawn(&self) -> bool {
        self.can_respawn
    }

    /// Get remaining time in seconds
    fn remaining_seconds(&self) -> u64 {
        let elapsed = self.death_time.elapsed();
        if elapsed >= self.respawn_duration {
            0
        } else {
            (self.respawn_duration - elapsed).as_secs()
        }
    }

    /// Get progress (0.0 to 1.0)
    fn progress(&self) -> f32 {
        let elapsed = self.death_time.elapsed().as_secs_f32();
        let total = self.respawn_duration.as_secs_f32();
        (elapsed / total).min(1.0)
    }

    /// Draw the death page
    pub fn draw_death_page(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        // Update sprite animation
        if let Some(ref mut sprite) = self.sit_sprite {
            sprite.update();
        }

        // Draw sit sprite
        if let Some(ref sprite) = self.sit_sprite {
            sprite.draw(display)?;
        }

        // Draw death message
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 100, 100));
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 200));

        Text::new("You have been", Point::new(95, 80), text_style_title).draw(display)?;
        Text::new("defeated!", Point::new(125, 105), text_style_title).draw(display)?;

        // Draw timer
        let remaining = self.remaining_seconds();

        // Check if timer value changed - if so, clear the text area
        let timer_changed = self.last_remaining_seconds != Some(remaining);

        if remaining > 0 {
            // Clear timer text area if it changed
            if timer_changed {
                Rectangle::new(Point::new(70, 320), Size::new(230, 30))
                    .into_styled(PrimitiveStyle::with_fill(self.background_color))
                    .draw(display)?;
            }

            let minutes = remaining / 60;
            let seconds = remaining % 60;

            use core::fmt::Write;
            let mut timer_str = heapless::String::<32>::new();
            write!(timer_str, "Respawn: {:02}:{:02}", minutes, seconds).ok();
            Text::new(&timer_str, Point::new(90, 340), text_style_info).draw(display)?;

            // Draw progress bar
            self.draw_progress_bar(display)?;
        } else {
            // Ready to respawn - clear timer area if transitioning from countdown
            if timer_changed {
                Rectangle::new(Point::new(70, 320), Size::new(230, 30))
                    .into_styled(PrimitiveStyle::with_fill(self.background_color))
                    .draw(display)?;
            }

            self.can_respawn = true;
            Text::new("Ready to respawn!", Point::new(70, 340), text_style_info).draw(display)?;

            // Draw respawn button
            self.draw_respawn_button(display)?;
        }

        // Update tracked remaining seconds
        self.last_remaining_seconds = Some(remaining);

        display.flush()?;
        Ok(())
    }

    /// Draw progress bar
    fn draw_progress_bar(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let bar_x = 35;
        let bar_y = 360;
        let bar_width = 300u32;
        let bar_height = 20u32;

        // Clear the entire progress bar area first
        Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width, bar_height))
            .into_styled(PrimitiveStyle::with_fill(self.background_color))
            .draw(display)?;

        // Background
        Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width, bar_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 20, 20)))
            .draw(display)?;

        // Progress fill
        let progress = self.progress();
        let filled_width = (bar_width as f32 * progress) as u32;

        if filled_width > 0 {
            Rectangle::new(Point::new(bar_x, bar_y), Size::new(filled_width, bar_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 200, 100)))
                .draw(display)?;
        }

        Ok(())
    }

    /// Draw respawn button
    fn draw_respawn_button(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let button_x = 90;
        let button_y = 390;
        let button_width = 190;
        let button_height = 60;

        // Clear the button area first
        Rectangle::new(Point::new(button_x, button_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_fill(self.background_color))
            .draw(display)?;

        Rectangle::new(Point::new(button_x, button_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 120, 60)))
            .draw(display)?;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("RESPAWN", Point::new(button_x + 45, button_y + 38), text_style).draw(display)?;

        Ok(())
    }

    /// Handle touch input
    pub fn handle_touch(&self, x: i32, y: i32) -> bool {
        if !self.can_respawn {
            return false;
        }

        // Check if touch is on respawn button
        let button_x = 90;
        let button_y = 390;
        let button_width = 190;
        let button_height = 60;

        x >= button_x
            && x < button_x + button_width
            && y >= button_y
            && y < button_y + button_height
    }
}

impl Page for DeathPage {
    fn update(&mut self) -> bool {
        // Update can_respawn status
        if !self.can_respawn && self.death_time.elapsed() >= self.respawn_duration {
            self.can_respawn = true;
        }

        // Continue running
        true
    }

    fn draw(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        self.draw_death_page(display, full_redraw)
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
