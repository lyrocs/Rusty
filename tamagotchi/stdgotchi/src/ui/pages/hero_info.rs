//! Hero Info Page
//!
//! Displays detailed hero information including stats, job, cards, and state

use crate::display::Sh8601Driver;
use crate::game::Hero;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10, ascii::FONT_9X15, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, RoundedRectangle, CornerRadii},
    text::Text,
};
use std::error::Error;
use core::fmt::Write;

/// Hero Info Page
pub struct HeroInfoPage {
    background_color: Rgb888,
    hero: Hero,
}

impl HeroInfoPage {
    /// Create new hero info page
    pub fn new(hero: Hero) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            background_color: Rgb888::new(15, 20, 30),
            hero,
        })
    }

    /// Update hero data (called when hero changes)
    pub fn update_hero(&mut self, hero: Hero) {
        self.hero = hero;
    }

    /// Draw hero header (name, level, job)
    fn draw_header(
        display: &mut Sh8601Driver,
        hero: &Hero,
    ) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 255, 200));
        let info_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);

        // Hero name and level
        let mut name_text = heapless::String::<32>::new();
        write!(name_text, "{}", hero.name).ok();
        Text::new(&name_text, Point::new(20, 25), title_style).draw(display)?;

        let mut level_text = heapless::String::<16>::new();
        write!(level_text, "Lv {}", hero.level).ok();
        Text::new(&level_text, Point::new(280, 25), title_style).draw(display)?;

        // Job class
        let mut job_text = heapless::String::<32>::new();
        write!(job_text, "Job: {:?}", hero.job).ok();
        Text::new(&job_text, Point::new(20, 50), info_style).draw(display)?;

        Ok(())
    }

    /// Draw stats section
    fn draw_stats(
        display: &mut Sh8601Driver,
        hero: &Hero,
    ) -> Result<(), Box<dyn Error>> {
        let label_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(200, 200, 200));
        let value_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 150));

        let y_start = 80;
        let spacing = 30;

        // HP
        let mut hp_text = heapless::String::<32>::new();
        write!(hp_text, "HP: {}/{}", hero.current_health, hero.max_health).ok();
        Text::new(&hp_text, Point::new(20, y_start), value_style).draw(display)?;

        // HP Bar
        let bar_y = y_start + 5;
        let bar_width = 250;
        let bar_height = 15;

        Rectangle::new(
            Point::new(20, bar_y),
            Size::new(bar_width, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        let hp_ratio = hero.current_health as f32 / hero.max_health as f32;
        let filled_width = (bar_width as f32 * hp_ratio) as u32;

        if filled_width > 0 {
            let hp_color = if hp_ratio > 0.5 {
                Rgb888::new(100, 255, 100)
            } else if hp_ratio > 0.25 {
                Rgb888::new(255, 255, 100)
            } else {
                Rgb888::new(255, 100, 100)
            };

            Rectangle::new(
                Point::new(20, bar_y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(hp_color))
            .draw(display)?;
        }

        let mut y = y_start + spacing + 10;

        // Attack
        let mut atk_text = heapless::String::<32>::new();
        write!(atk_text, "ATK: {}", hero.attack).ok();
        Text::new(&atk_text, Point::new(20, y), label_style).draw(display)?;
        y += spacing;

        // Defense
        let mut def_text = heapless::String::<32>::new();
        write!(def_text, "DEF: {}", hero.defense).ok();
        Text::new(&def_text, Point::new(20, y), label_style).draw(display)?;
        y += spacing;

        // Attack Speed
        let mut aspd_text = heapless::String::<32>::new();
        write!(aspd_text, "ASPD: {:.1}", hero.aspd).ok();
        Text::new(&aspd_text, Point::new(20, y), label_style).draw(display)?;

        Ok(())
    }

    /// Draw experience bar
    fn draw_exp_bar(
        display: &mut Sh8601Driver,
        hero: &Hero,
    ) -> Result<(), Box<dyn Error>> {
        let label_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let y = 250;

        // Label
        let mut exp_label = heapless::String::<32>::new();
        write!(exp_label, "EXP: {}/{}", hero.experience, hero.experience_to_next_level).ok();
        Text::new(&exp_label, Point::new(20, y), label_style).draw(display)?;

        // Bar
        let bar_y = y + 5;
        let bar_width = 328;
        let bar_height = 20;

        Rectangle::new(
            Point::new(20, bar_y),
            Size::new(bar_width, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        let exp_ratio = hero.experience as f32 / hero.experience_to_next_level as f32;
        let filled_width = (bar_width as f32 * exp_ratio) as u32;

        if filled_width > 0 {
            Rectangle::new(
                Point::new(20, bar_y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 200, 255)))
            .draw(display)?;
        }

        Ok(())
    }

    /// Draw hero state
    fn draw_state(
        display: &mut Sh8601Driver,
        hero: &Hero,
    ) -> Result<(), Box<dyn Error>> {
        let y = 290;
        let state_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(200, 200, 255));

        let state_text = match &hero.state {
            crate::game::HeroState::Ready => "Status: Ready for adventure!",
            crate::game::HeroState::OnExpedition { end_time: _ } => {
                if let Some(remaining) = hero.state.remaining_time() {
                    let minutes = remaining / 60;
                    let seconds = remaining % 60;
                    let mut text = heapless::String::<64>::new();
                    write!(text, "Status: On expedition ({}:{:02} left)", minutes, seconds).ok();
                    Text::new(&text, Point::new(20, y), state_style).draw(display)?;
                    return Ok(());
                } else {
                    "Status: On expedition (finishing...)"
                }
            }
            crate::game::HeroState::KO { recovery_time: _ } => {
                if let Some(remaining) = hero.state.remaining_time() {
                    let minutes = remaining / 60;
                    let seconds = remaining % 60;
                    let mut text = heapless::String::<64>::new();
                    write!(text, "Status: KO! (recovery {}:{:02})", minutes, seconds).ok();
                    let ko_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 150, 150));
                    Text::new(&text, Point::new(20, y), ko_style).draw(display)?;
                    return Ok(());
                } else {
                    "Status: KO! (recovering...)"
                }
            }
        };

        Text::new(state_text, Point::new(20, y), state_style).draw(display)?;
        Ok(())
    }

    /// Draw card collection summary
    fn draw_cards_summary(
        display: &mut Sh8601Driver,
        hero: &Hero,
    ) -> Result<(), Box<dyn Error>> {
        let y = 330;
        let label_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);

        let mut cards_text = heapless::String::<32>::new();
        write!(cards_text, "Cards: {}", hero.cards.len()).ok();
        Text::new(&cards_text, Point::new(20, y), label_style).draw(display)?;

        // Count cards by rarity
        let mut rarity_counts = [0u32; 6]; // 0-5 stars
        for card in &hero.cards {
            if card.rarity <= 5 {
                rarity_counts[card.rarity as usize] += 1;
            }
        }

        // Show rarity breakdown
        let mut breakdown_y = y + 25;
        for (rarity, count) in rarity_counts.iter().enumerate().skip(1) {
            if *count > 0 {
                let star_color = match rarity {
                    1 => Rgb888::new(150, 150, 150), // Gray
                    2 => Rgb888::new(100, 200, 100), // Green
                    3 => Rgb888::new(100, 150, 255), // Blue
                    4 => Rgb888::new(200, 100, 255), // Purple
                    5 => Rgb888::new(255, 180, 50),  // Gold
                    _ => Rgb888::WHITE,
                };

                let mut stars = heapless::String::<16>::new();
                for _ in 0..rarity {
                    write!(stars, "★").ok();
                }
                let star_style = MonoTextStyle::new(&FONT_6X10, star_color);
                Text::new(&stars, Point::new(30, breakdown_y), star_style).draw(display)?;

                let mut count_text = heapless::String::<16>::new();
                write!(count_text, " x{}", count).ok();
                let count_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
                Text::new(&count_text, Point::new(100, breakdown_y), count_style).draw(display)?;

                breakdown_y += 15;
            }
        }

        Ok(())
    }
}

impl Page for HeroInfoPage {
    fn update(&mut self) -> bool {
        // Keep page active
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Clear background
        display.clear(self.background_color)?;

        // Draw all sections
        Self::draw_header(display, &self.hero)?;
        Self::draw_stats(display, &self.hero)?;
        Self::draw_exp_bar(display, &self.hero)?;
        Self::draw_state(display, &self.hero)?;
        Self::draw_cards_summary(display, &self.hero)?;

        // Instructions
        let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));
        Text::new("[SWIPE RIGHT] to return to menu", Point::new(65, 450), hint_style).draw(display)?;

        // Flush to display
        display.flush()?;

        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No internal dirty tracking
    }

    fn needs_full_redraw(&self) -> bool {
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
