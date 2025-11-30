//! Expedition In Progress Page
//!
//! Shows expedition progress with timer and cancel option

use crate::display::Sh8601Driver;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10, ascii::FONT_9X15, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use core::fmt::Write;

/// Expedition In Progress Page
pub struct ExpeditionInProgressPage {
    background_color: Rgb888,

    // Expedition details
    enemy_name: String,
    target_kills: u32,
    duration_seconds: f32,

    // Timing
    start_time: u64,      // Unix timestamp
    end_time: u64,        // Unix timestamp
}

impl ExpeditionInProgressPage {
    /// Create new expedition in progress page
    pub fn new(
        enemy_name: String,
        target_kills: u32,
        duration_seconds: f32,
        start_time: u64,
        end_time: u64,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Expedition started: {} x{} (~{:.0}s)", enemy_name, target_kills, duration_seconds);

        Ok(Self {
            background_color: Rgb888::new(15, 20, 30),
            enemy_name,
            target_kills,
            duration_seconds,
            start_time,
            end_time,
        })
    }

    /// Get current unix timestamp
    fn current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Check if expedition is complete
    pub fn is_complete(&self) -> bool {
        Self::current_time() >= self.end_time
    }

    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> u64 {
        let now = Self::current_time();
        if now >= self.end_time {
            0
        } else {
            self.end_time - now
        }
    }

    /// Get progress ratio (0.0 to 1.0)
    fn progress_ratio(&self) -> f32 {
        let now = Self::current_time();
        if now >= self.end_time {
            return 1.0;
        }

        let total_duration = self.end_time - self.start_time;
        let elapsed = now - self.start_time;

        (elapsed as f32 / total_duration as f32).min(1.0)
    }

    /// Draw progress bar
    fn draw_progress_bar(
        display: &mut Sh8601Driver,
        progress: f32,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        let bar_x = 20;
        let bar_width = 328;
        let bar_height = 30;

        // Background
        Rectangle::new(
            Point::new(bar_x, y),
            Size::new(bar_width, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        // Filled portion
        let filled_width = (bar_width as f32 * progress) as u32;
        if filled_width > 0 {
            Rectangle::new(
                Point::new(bar_x, y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 150, 255)))
            .draw(display)?;
        }

        // Progress percentage
        let mut progress_text = heapless::String::<16>::new();
        write!(progress_text, "{:.0}%", progress * 100.0).ok();
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_x = bar_x + ((bar_width / 2) as i32) - 20;
        Text::new(&progress_text, Point::new(text_x, y + 20), text_style).draw(display)?;

        Ok(())
    }

    /// Draw timer display
    fn draw_timer(
        display: &mut Sh8601Driver,
        remaining_seconds: u64,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        let minutes = remaining_seconds / 60;
        let seconds = remaining_seconds % 60;

        let timer_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 150));
        let mut timer_text = heapless::String::<32>::new();
        write!(timer_text, "Time left: {}:{:02}", minutes, seconds).ok();
        Text::new(&timer_text, Point::new(80, y), timer_style).draw(display)?;

        Ok(())
    }
}

impl Page for ExpeditionInProgressPage {
    fn update(&mut self) -> bool {
        // Page stays active until expedition completes
        // The expedition system will check is_complete() and switch modes
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Clear background
        display.clear(self.background_color)?;

        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 255, 200));
        Text::new("EXPEDITION IN PROGRESS", Point::new(30, 25), title_style).draw(display)?;

        // Enemy and target
        let info_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let mut enemy_text = heapless::String::<64>::new();
        write!(enemy_text, "Hunting: {} x{}", self.enemy_name, self.target_kills).ok();
        Text::new(&enemy_text, Point::new(20, 60), info_style).draw(display)?;

        // Progress bar
        let progress = self.progress_ratio();
        Self::draw_progress_bar(display, progress, 100)?;

        // Estimated kills so far
        let kills_so_far = (self.target_kills as f32 * progress) as u32;
        let mut kills_text = heapless::String::<32>::new();
        write!(kills_text, "Kills: {}/{}", kills_so_far, self.target_kills).ok();
        let kills_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(200, 200, 200));
        Text::new(&kills_text, Point::new(20, 160), kills_style).draw(display)?;

        // Timer
        let remaining = self.remaining_seconds();
        Self::draw_timer(display, remaining, 200)?;

        // Status message
        let status_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 200, 255));
        if progress >= 0.9 {
            Text::new("Almost done...", Point::new(100, 250), status_style).draw(display)?;
        } else if progress >= 0.5 {
            Text::new("Halfway there...", Point::new(90, 250), status_style).draw(display)?;
        } else if progress >= 0.2 {
            Text::new("Making progress...", Point::new(80, 250), status_style).draw(display)?;
        } else {
            Text::new("Just getting started...", Point::new(60, 250), status_style).draw(display)?;
        }

        // Instructions
        let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));
        Text::new("Your hero is away on expedition", Point::new(55, 420), hint_style).draw(display)?;
        Text::new("[SWIPE RIGHT] to return to map", Point::new(60, 435), hint_style).draw(display)?;
        Text::new("(expedition continues in background)", Point::new(45, 450), hint_style).draw(display)?;

        // Flush to display
        display.flush()?;

        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No internal dirty tracking
    }

    fn needs_full_redraw(&self) -> bool {
        // Always redraw to update timer
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
