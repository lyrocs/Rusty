//! Expedition Setup Page
//!
//! Allows player to select expedition size and see risk assessment before starting

use crate::display::Sh8601Driver;
use crate::game::{Hero, Enemy, ExpeditionSize, ExpeditionResult, calculate_expedition};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10, ascii::FONT_9X15, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle, CornerRadii},
    text::Text,
};
use std::error::Error;
use core::fmt::Write;

/// Expedition Setup Page
pub struct ExpeditionSetupPage {
    background_color: Rgb888,
    hero: Hero,
    enemy: Enemy,
    selected_size: Option<ExpeditionSize>,
}

impl ExpeditionSetupPage {
    /// Create new expedition setup page
    pub fn new(hero: Hero, enemy: Enemy) -> Result<Self, Box<dyn Error>> {
        log::info!("Expedition setup: {} vs {}", hero.name, enemy.name);

        Ok(Self {
            background_color: Rgb888::new(15, 20, 30), // Dark blue-gray background
            hero,
            enemy,
            selected_size: None,
        })
    }

    /// Calculate expedition results for a given size
    fn calculate_for_size(&self, size: ExpeditionSize) -> ExpeditionResult {
        calculate_expedition(&self.hero, &self.enemy, size.count())
    }

    /// Handle touch input - returns selected expedition size if user tapped a valid button
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<ExpeditionSize> {
        // Size selection buttons (must match draw_size_selection positions)
        let button_y = 160;
        let button_h = 70;
        let sizes = ExpeditionSize::all();

        for (i, size) in sizes.iter().enumerate() {
            let button_x = 20 + (i as i32 * 88);
            let button_w = 83;

            if x >= button_x && x <= button_x + button_w
                && y >= button_y && y <= button_y + button_h
            {
                self.selected_size = Some(*size);
                return Some(*size);
            }
        }

        None
    }

    /// Get selected expedition size (after start button press)
    pub fn get_selection(&self) -> Option<ExpeditionSize> {
        self.selected_size
    }

    /// Draw stat comparison section
    fn draw_stats(
        display: &mut Sh8601Driver,
        hero: &Hero,
        enemy: &Enemy,
    ) -> Result<(), Box<dyn Error>> {
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let medium_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let enemy_name_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 150, 150));

        // Enemy info
        let mut enemy_title = heapless::String::<32>::new();
        write!(enemy_title, "{}", enemy.name).ok();
        Text::new(&enemy_title, Point::new(20, 40), enemy_name_style).draw(display)?;

        let mut enemy_stats = heapless::String::<64>::new();
        write!(
            enemy_stats,
            "ATK:{} DEF:{} HP:{}",
            enemy.atk, enemy.def, enemy.max_hp
        )
        .ok();
        Text::new(&enemy_stats, Point::new(20, 58), small_style).draw(display)?;

        // Divider line
        Rectangle::new(Point::new(20, 70), Size::new(328, 2))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 60, 60)))
            .draw(display)?;

        // Hero info
        let mut hero_title = heapless::String::<32>::new();
        write!(hero_title, "{} (Lv{})", hero.name, hero.level).ok();
        let hero_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 255, 150));
        Text::new(&hero_title, Point::new(20, 90), hero_style).draw(display)?;

        let mut hero_stats = heapless::String::<64>::new();
        write!(
            hero_stats,
            "ATK:{} DEF:{} HP:{}/{}",
            hero.attack, hero.defense, hero.current_health, hero.max_health
        )
        .ok();
        Text::new(&hero_stats, Point::new(20, 108), small_style).draw(display)?;

        Ok(())
    }

    /// Draw expedition size selection buttons with risk indicators
    fn draw_size_selection(
        display: &mut Sh8601Driver,
        hero: &Hero,
        enemy: &Enemy,
        selected: Option<ExpeditionSize>,
    ) -> Result<(), Box<dyn Error>> {
        let label_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 255));

        Text::new("HUNT HOW MANY?", Point::new(70, 140), title_style).draw(display)?;

        let button_y = 160;
        let button_h = 70;
        let sizes = ExpeditionSize::all();

        for (i, size) in sizes.iter().enumerate() {
            let button_x = 20 + (i as i32 * 88);
            let button_w = 83;

            // Calculate expedition for this size
            let result = calculate_expedition(hero, enemy, size.count());
            let damage_ratio = result.total_damage / hero.current_health as f32;
            let risk_indicator = ExpeditionSize::risk_indicator(damage_ratio);

            // Button background color based on risk (matching risk_indicator thresholds)
            let bg_color = if damage_ratio >= 0.95 {
                Rgb888::new(60, 20, 20) // Will die - red
            } else if damage_ratio >= 0.7 {
                Rgb888::new(60, 40, 20) // Dangerous - orange
            } else if damage_ratio >= 0.4 {
                Rgb888::new(60, 60, 20) // Risky - yellow
            } else {
                Rgb888::new(20, 60, 20) // Safe - green
            };

            // Highlight selected button
            let final_bg = if Some(*size) == selected {
                Rgb888::new(
                    bg_color.r().saturating_add(40),
                    bg_color.g().saturating_add(40),
                    bg_color.b().saturating_add(40),
                )
            } else {
                bg_color
            };

            // Draw button
            RoundedRectangle::new(
                Rectangle::new(
                    Point::new(button_x, button_y),
                    Size::new(button_w as u32, button_h as u32),
                ),
                CornerRadii::new(Size::new(8, 8)),
            )
            .into_styled(PrimitiveStyle::with_fill(final_bg))
            .draw(display)?;

            // Size count
            let count_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            let mut count_text = heapless::String::<8>::new();
            write!(count_text, "[{}]", size.count()).ok();
            let text_x = button_x + 15;
            Text::new(&count_text, Point::new(text_x, button_y + 25), count_style).draw(display)?;

            // Risk indicator (split into two lines if needed)
            let risk_parts: Vec<&str> = risk_indicator.split(' ').collect();
            if risk_parts.len() == 2 {
                // Two-word indicator (e.g., "Will die", "Not Very")
                Text::new(risk_parts[0], Point::new(button_x + 8, button_y + 45), label_style)
                    .draw(display)?;
                Text::new(risk_parts[1], Point::new(button_x + 8, button_y + 58), label_style)
                    .draw(display)?;
            } else {
                // Single word or emoji
                Text::new(risk_indicator, Point::new(button_x + 5, button_y + 50), label_style)
                    .draw(display)?;
            }
        }

        Ok(())
    }

    /// Draw expedition details for selected size
    fn draw_expedition_details(
        display: &mut Sh8601Driver,
        hero: &Hero,
        result: &ExpeditionResult,
    ) -> Result<(), Box<dyn Error>> {
        let info_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let value_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 150));

        let y = 250;

        // Time estimate
        let mut time_text = heapless::String::<32>::new();
        let minutes = (result.duration_seconds / 60.0) as u32;
        let seconds = (result.duration_seconds % 60.0) as u32;
        if minutes > 0 {
            write!(time_text, "Time: ~{}m{}s", minutes, seconds).ok();
        } else {
            write!(time_text, "Time: ~{}s", seconds).ok();
        }
        Text::new(&time_text, Point::new(20, y), info_style).draw(display)?;

        // Damage estimate (percentage of current HP)
        let damage_percent = ((result.total_damage / hero.current_health as f32) * 100.0) as u32;
        let mut damage_text = heapless::String::<32>::new();
        write!(damage_text, "HP loss: ~{}%", damage_percent.min(100)).ok();
        Text::new(&damage_text, Point::new(20, y + 20), info_style).draw(display)?;

        Ok(())
    }
}

impl Page for ExpeditionSetupPage {
    fn update(&mut self) -> bool {
        // Keep page active (return true)
        // The expedition_setup_system will handle closing the page when START is pressed
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Clear background
        display.clear(self.background_color)?;

        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 255, 200));
        Text::new("EXPEDITION SETUP", Point::new(50, 20), title_style).draw(display)?;

        // Stats comparison
        Self::draw_stats(display, &self.hero, &self.enemy)?;

        // Size selection buttons
        Self::draw_size_selection(display, &self.hero, &self.enemy, self.selected_size)?;

        // Show details if size selected
        if let Some(size) = self.selected_size {
            let result = self.calculate_for_size(size);
            Self::draw_expedition_details(display, &self.hero, &result)?;

            // Start button
            let button_style = PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50));
            RoundedRectangle::new(
                Rectangle::new(Point::new(100, 290), Size::new(168, 45)),
                CornerRadii::new(Size::new(8, 8)),
            )
            .into_styled(button_style)
            .draw(display)?;

            let start_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            Text::new("START", Point::new(135, 320), start_style).draw(display)?;
        }

        // Instructions
        let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));
        let hint_text = if self.selected_size.is_some() {
            "[TAP START] to begin"
        } else {
            "[TAP SIZE] to select"
        };
        Text::new(hint_text, Point::new(80, 450), hint_style).draw(display)?;

        // Flush to display
        display.flush()?;

        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No internal dirty tracking needed
    }

    fn needs_full_redraw(&self) -> bool {
        // Always full redraw for this page
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
