//! Battle Result Page
//!
//! Displays battle victory with EXP gains and HP regeneration

use crate::display::Sh8601Driver;
use crate::game::{GameData, Hero};
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

/// Battle Result Page - Shows victory and EXP gains
pub struct BattleResultPage {
    background_color: Rgb888,
    hero: Hero,
    initial_hp: i32,
    initial_exp: u32,
    initial_level: u32,
    exp_gained: u32,
    game_data: GameData,
    last_regen: Instant,
    start_time: Instant,
}

impl BattleResultPage {
    /// Create a new battle result page
    pub fn new(
        mut hero: Hero,
        exp_gained: u32,
        game_data: GameData,
    ) -> Result<Self, Box<dyn Error>> {
        let initial_hp = hero.current_health;
        let initial_exp = hero.experience;
        let initial_level = hero.level;

        // Apply EXP gain and level up
        let leveled_up = hero.gain_experience(exp_gained);

        if leveled_up {
            log::info!("🎉 {} leveled up to {}!", hero.name, hero.level);
        }

        log::info!("📊 Battle Victory: {} gained {} EXP (Level {})",
            hero.name, exp_gained, hero.level);

        Ok(Self {
            background_color: Rgb888::new(20, 30, 40),
            hero,
            initial_hp,
            initial_exp,
            initial_level,
            exp_gained,
            game_data,
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

    /// Draw EXP bar with gained exp highlight
    fn draw_exp_bar(
        display: &mut Sh8601Driver,
        label: &str,
        current_exp: u32,
        exp_to_next: u32,
        exp_gained: u32,
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

        // Calculate filled width for current EXP
        let exp_percentage = (current_exp as f32 / exp_to_next as f32).min(1.0);
        let filled_width = (exp_percentage * bar_width as f32) as u32;

        // EXP bar (blue)
        if filled_width > 0 {
            Rectangle::new(
                Point::new(x, bar_y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 200, 255)))
            .draw(display)?;
        }

        // EXP gain text (centered, green)
        use core::fmt::Write;
        let mut exp_text = heapless::String::<32>::new();
        write!(exp_text, "+{} EXP", exp_gained).ok();
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(0, 255, 0));
        let text_x = x + (bar_width / 2) - (exp_text.len() as i32 * 5);
        Text::new(&exp_text, Point::new(text_x, bar_y + 15), text_style).draw(display)?;

        Ok(())
    }

    /// Get updated hero with HP regeneration
    pub fn get_updated_hero(&self) -> Hero {
        self.hero.clone()
    }

    /// Check if user touched the continue button
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

impl Page for BattleResultPage {
    fn update(&mut self) -> bool {
        // Process HP regeneration
        self.process_regen();

        // Always keep page active - user must manually continue
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Clear background
        display.clear(self.background_color)?;

        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        Text::new("⚔️ VICTORY! ⚔️", Point::new(90, 30), title_style).draw(display)?;

        // Hero Section Box (near top)
        let hero_box_y = 55;
        Rectangle::new(
            Point::new(10, hero_box_y),
            Size::new(348, 85),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 40, 50)))
        .draw(display)?;

        // Hero name
        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new(&self.hero.name, Point::new(20, hero_box_y + 20), name_style).draw(display)?;

        // Job and Level
        let job_name = self.hero.job.get_name();
        let mut job_level = heapless::String::<64>::new();
        write!(job_level, "{} Lv.{}", job_name, self.hero.level).ok();
        let job_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
        Text::new(&job_level, Point::new(20, hero_box_y + 35), job_style).draw(display)?;

        // HP bar in hero section
        Self::draw_hp_bar(
            display,
            "HP:",
            self.hero.current_health,
            self.hero.max_health,
            20,
            hero_box_y + 45,
        )?;

        // Results section
        let center_y = 220;

        // Level up indicator - PROMINENT
        if self.hero.level > self.initial_level {
            let levels_gained = self.hero.level - self.initial_level;

            // Draw background highlight for level up
            Rectangle::new(
                Point::new(20, center_y - 10),
                Size::new(328, 40),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50)))
            .draw(display)?;

            let mut level_up_text = heapless::String::<48>::new();
            if levels_gained == 1 {
                write!(level_up_text, "*** LEVEL UP! ***").ok();
            } else {
                write!(level_up_text, "*** +{} LEVELS UP! ***", levels_gained).ok();
            }
            let level_up_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 100));
            let level_up_x = 184 - (level_up_text.len() as i32 * 5);
            Text::new(&level_up_text, Point::new(level_up_x, center_y + 15), level_up_style).draw(display)?;

            // Show level transition
            let mut level_transition = heapless::String::<32>::new();
            write!(level_transition, "Level {} -> {}", self.initial_level, self.hero.level).ok();
            let transition_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 255, 200));
            let transition_x = 184 - (level_transition.len() as i32 * 3);
            Text::new(&level_transition, Point::new(transition_x, center_y + 30), transition_style).draw(display)?;
        }

        // HP restored indicator (if any HP was restored)
        let hp_restored = self.hero.current_health - self.initial_hp;
        if hp_restored > 0 {
            let mut restored_text = heapless::String::<32>::new();
            write!(restored_text, "Restored: +{} HP", hp_restored).ok();
            let restored_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 255, 150));
            let restored_x = 184 - (restored_text.len() as i32 * 3);
            Text::new(&restored_text, Point::new(restored_x, center_y + 15), restored_style).draw(display)?;
        }

        // EXP bar (centered)
        Self::draw_exp_bar(
            display,
            "EXP:",
            self.hero.experience,
            self.hero.experience_to_next_level,
            self.exp_gained,
            59,
            center_y + 30,
        )?;

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
