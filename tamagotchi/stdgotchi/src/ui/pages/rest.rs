//! Rest Page
//!
//! Displays hero resting with HP regeneration over time

use crate::display::Sh8601Driver;
use crate::game::Hero;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::{Duration, Instant};

/// HP regeneration rate: 5% of max HP every 2 seconds
const HP_REGEN_INTERVAL: Duration = Duration::from_secs(2);
const HP_REGEN_PERCENT: f32 = 0.05; // 5%

/// Rest Page - Hero resting with HP regeneration
pub struct RestPage {
    background_color: Rgb888,
    hero: Hero,
    initial_hp: i32,
    last_regen: Instant,
    start_time: Instant,
}

impl RestPage {
    /// Create a new rest page with hero
    pub fn new(hero: Hero) -> Result<Self, Box<dyn Error>> {
        let initial_hp = hero.current_health;

        log::info!("Rest page created: {} with HP {}/{}", hero.name, hero.current_health, hero.max_health);

        Ok(Self {
            background_color: Rgb888::new(25, 35, 45),
            initial_hp,
            hero,
            last_regen: Instant::now(),
            start_time: Instant::now(),
        })
    }

    /// Process HP regeneration
    fn process_regen(&mut self) {
        if self.last_regen.elapsed() >= HP_REGEN_INTERVAL {
            self.last_regen = Instant::now();

            if self.hero.current_health < self.hero.max_health {
                let regen = ((self.hero.max_health as f32 * HP_REGEN_PERCENT) as i32)
                    .max(1)
                    .min(self.hero.max_health - self.hero.current_health);

                self.hero.heal(regen);

                log::debug!("{} regenerated {} HP ({}/{})",
                    self.hero.name, regen, self.hero.current_health, self.hero.max_health);
            }
        }
    }

    /// Check if HP is fully restored
    fn is_hp_full(&self) -> bool {
        self.hero.current_health >= self.hero.max_health
    }

    /// Draw HP bar with current/max HP
    fn draw_hp_bar(
        display: &mut Sh8601Driver,
        label: &str,
        current_hp: i32,
        max_hp: i32,
        x: i32,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        // Label
        let label_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        Text::new(label, Point::new(x, y), label_style).draw(display)?;

        // Bar dimensions
        let bar_width = 250;
        let bar_height = 20;
        let bar_y = y + 5;

        // Background bar (dark)
        Rectangle::new(
            Point::new(x, bar_y),
            Size::new(bar_width as u32, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        // HP bar (colored)
        let hp_percentage = (current_hp as f32 / max_hp as f32) * 100.0;
        let hp_color = if hp_percentage > 60.0 {
            Rgb888::new(0, 200, 0) // Green
        } else if hp_percentage > 30.0 {
            Rgb888::new(200, 200, 0) // Yellow
        } else {
            Rgb888::new(200, 0, 0) // Red
        };

        let filled_width = ((current_hp as f32 / max_hp as f32) * bar_width as f32) as u32;
        if filled_width > 0 {
            Rectangle::new(
                Point::new(x, bar_y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(hp_color))
            .draw(display)?;
        }

        // HP text on bar
        use core::fmt::Write;
        let mut hp_text = heapless::String::<32>::new();
        write!(hp_text, "{}/{}", current_hp, max_hp).ok();
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_x = x + (bar_width / 2) - (hp_text.len() as i32 * 5);
        Text::new(&hp_text, Point::new(text_x, bar_y + 15), text_style).draw(display)?;

        Ok(())
    }

    /// Get updated hero with HP regeneration
    pub fn get_updated_hero(&self) -> Hero {
        self.hero.clone()
    }

    /// Check if user touched the continue area (only active when HP is full)
    pub fn handle_touch(&self, x: i32, y: i32) -> bool {
        // Only allow continue when HP is fully restored
        if !self.is_hp_full() {
            return false;
        }

        // Continue button area at bottom center
        let button_x = 100;
        let button_y = 400;
        let button_w = 168;
        let button_h = 50;

        x >= button_x && x <= button_x + button_w && y >= button_y && y <= button_y + button_h
    }
}

impl Page for RestPage {
    fn update(&mut self) -> bool {
        // Process HP regeneration
        self.process_regen();

        // Always keep page active - user must manually continue
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Clear background
        display.clear(self.background_color)?;

        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 200, 255));
        Text::new("💤 Resting 💤", Point::new(120, 40), title_style).draw(display)?;

        // Hero info section (centered)
        let center_y = 200;

        // Hero name
        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let name_x = 184 - (self.hero.name.len() as i32 * 5);
        Text::new(&self.hero.name, Point::new(name_x, center_y - 60), name_style).draw(display)?;

        // Job info
        let job_name = self.hero.job.get_name();
        let job_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0)); // Gold color
        let job_x = 184 - (job_name.len() as i32 * 5);
        Text::new(job_name, Point::new(job_x, center_y - 30), job_style).draw(display)?;

        // Level info
        use core::fmt::Write;
        let mut level_text = heapless::String::<32>::new();
        write!(level_text, "Level {}", self.hero.level).ok();
        let level_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 200));
        let level_x = 184 - (level_text.len() as i32 * 5);
        Text::new(&level_text, Point::new(level_x, center_y), level_style).draw(display)?;

        // Draw HP bar (centered below hero info)
        Self::draw_hp_bar(
            display,
            "HP:",
            self.hero.current_health,
            self.hero.max_health,
            59, // Center the 250px bar: (368 - 250) / 2 = 59
            center_y + 40,
        )?;

        // Show regen rate
        let info_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));
        Text::new("+5% HP / 2s", Point::new(140, center_y + 80), info_style).draw(display)?;

        // HP restored indicator
        let restored = self.hero.current_health - self.initial_hp;
        if restored > 0 {
            let mut restored_text = heapless::String::<32>::new();
            write!(restored_text, "Restored: +{} HP", restored).ok();
            let restored_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(0, 255, 0));
            let restored_x = 184 - (restored_text.len() as i32 * 3);
            Text::new(&restored_text, Point::new(restored_x, center_y + 95), restored_style).draw(display)?;
        }

        // Draw continue button at bottom
        let button_x = 100;
        let button_y = 400;
        let button_w = 168;
        let button_h = 50;

        let hp_full = self.is_hp_full();
        let button_color = if hp_full {
            Rgb888::new(0, 150, 0) // Green when ready
        } else {
            Rgb888::new(100, 100, 100) // Gray while regenerating
        };

        Rectangle::new(
            Point::new(button_x, button_y),
            Size::new(button_w, button_h),
        )
        .into_styled(PrimitiveStyle::with_fill(button_color))
        .draw(display)?;

        let button_text = if hp_full {
            "Continue"
        } else {
            "Regenerating..."
        };
        let button_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_x = button_x + (button_w as i32 / 2) - (button_text.len() as i32 * 5);
        Text::new(button_text, Point::new(text_x, button_y + 30), button_style).draw(display)?;

        display.flush()?;
        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No-op
    }

    fn needs_full_redraw(&self) -> bool {
        true // Always redraw for HP updates
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
