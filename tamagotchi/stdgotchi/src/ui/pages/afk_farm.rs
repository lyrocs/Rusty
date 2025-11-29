//! AFK Farm Page
//!
//! Passive farming mode where hero gains EXP over time without taking damage
//! EXP gain rate is based on hero stats vs enemy stats

use crate::display::Sh8601Driver;
use crate::game::{Enemy, GameData, Hero};
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

/// EXP update interval: gain EXP every 3 seconds
const EXP_GAIN_INTERVAL: Duration = Duration::from_secs(3);

/// AFK Farm Page - Passive EXP farming
pub struct AfkFarmPage {
    background_color: Rgb888,
    hero: Hero,
    initial_exp: u32,
    initial_level: u32,
    last_exp_gain: Instant,
    start_time: Instant,
    exp_per_cycle: u32,
    total_exp_gained: u32,
    enemy_name: String,
    game_data: GameData,
}

impl AfkFarmPage {
    /// Create a new AFK farm page
    /// EXP per cycle is calculated based on hero stats vs enemy difficulty
    pub fn new(
        hero: Hero,
        enemy_ids: &[u32],
        game_data: GameData,
    ) -> Result<Self, Box<dyn Error>> {
        let initial_exp = hero.experience;
        let initial_level = hero.level;

        // Get enemy name for display
        let enemy_name = if let Some(first_enemy_id) = enemy_ids.first() {
            game_data.get_enemy(*first_enemy_id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "Enemy".to_string())
        } else {
            "Enemy".to_string()
        };

        // Calculate EXP per cycle based on hero and enemy stats
        let exp_per_cycle = Self::calculate_exp_per_cycle(&hero, enemy_ids, &game_data);
        log::info!("💰 AFK farming EXP per cycle ({} seconds): {} EXP",
            EXP_GAIN_INTERVAL.as_secs(), exp_per_cycle);

        Ok(Self {
            background_color: Rgb888::new(15, 25, 35),
            hero,
            initial_exp,
            initial_level,
            last_exp_gain: Instant::now(),
            start_time: Instant::now(),
            exp_per_cycle,
            total_exp_gained: 0,
            enemy_name,
            game_data,
        })
    }

    /// Calculate EXP gain per cycle based on hero stats vs enemy difficulty
    /// Formula: (Hero level × Enemy level × Enemy base_exp) / 50 per cycle
    fn calculate_exp_per_cycle(hero: &Hero, enemy_ids: &[u32], game_data: &GameData) -> u32 {
        if enemy_ids.is_empty() {
            return 5; // Minimum EXP
        }

        // Calculate hero power (level + stats)
        let hero_power = hero.level + (hero.attack as u32 / 10) + (hero.max_health as u32 / 50);

        // Get average enemy difficulty
        let mut total_enemy_exp = 0;
        let mut enemy_count = 0;
        for enemy_id in enemy_ids {
            if let Some(enemy_data) = game_data.get_enemy(*enemy_id) {
                total_enemy_exp += enemy_data.base_exp as u32;
                enemy_count += 1;
            }
        }

        if enemy_count == 0 {
            return 5;
        }

        let avg_enemy_exp = total_enemy_exp / enemy_count;

        // Calculate EXP per cycle: (hero_power × avg_enemy_exp) / 50
        let exp_per_cycle = (hero_power * avg_enemy_exp) / 50;

        // Ensure minimum 5 EXP and maximum 100 EXP per cycle
        exp_per_cycle.max(5).min(100)
    }

    /// Process EXP gain
    fn process_exp_gain(&mut self) {
        if self.last_exp_gain.elapsed() >= EXP_GAIN_INTERVAL {
            self.last_exp_gain = Instant::now();

            // Gain EXP
            let leveled_up = self.hero.gain_experience(self.exp_per_cycle);
            self.total_exp_gained += self.exp_per_cycle;

            if leveled_up {
                log::info!("🎉 {} leveled up to {}!", self.hero.name, self.hero.level);
            }

            log::debug!("💰 Gained {} EXP (Total: +{})",
                self.exp_per_cycle, self.total_exp_gained);
        }
    }

    /// Get elapsed farming time
    fn get_elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Format duration as MM:SS
    fn format_duration(duration: Duration) -> heapless::String<16> {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;

        let mut result = heapless::String::new();
        use core::fmt::Write;
        write!(result, "{:02}:{:02}", minutes, seconds).ok();
        result
    }

    /// Get updated hero with EXP gains
    pub fn get_updated_hero(&self) -> Hero {
        self.hero.clone()
    }

    /// Check if user touched the stop button
    pub fn handle_touch(&self, x: i32, y: i32) -> bool {
        // Stop button area at bottom center
        let button_x = 100;
        let button_y = 400;
        let button_w = 168;
        let button_h = 50;

        x >= button_x && x <= button_x + button_w && y >= button_y && y <= button_y + button_h
    }
}

impl Page for AfkFarmPage {
    fn update(&mut self) -> bool {
        // Process EXP gain
        self.process_exp_gain();

        // Always keep page active - user must manually stop
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Clear background
        display.clear(self.background_color)?;

        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        Text::new("⚡ AFK Farming ⚡", Point::new(90, 30), title_style).draw(display)?;

        // Farming info section
        let info_y = 70;
        let info_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));

        let mut farming_text = heapless::String::<64>::new();
        write!(farming_text, "Farming: {}", self.enemy_name).ok();
        Text::new(&farming_text, Point::new(20, info_y), info_style).draw(display)?;

        let mut rate_text = heapless::String::<64>::new();
        write!(rate_text, "Rate: {} EXP / {}s", self.exp_per_cycle, EXP_GAIN_INTERVAL.as_secs()).ok();
        Text::new(&rate_text, Point::new(20, info_y + 15), info_style).draw(display)?;

        // Elapsed time
        let elapsed = Self::format_duration(self.get_elapsed_time());
        let mut time_text = heapless::String::<32>::new();
        write!(time_text, "Time: {}", elapsed).ok();
        Text::new(&time_text, Point::new(20, info_y + 30), info_style).draw(display)?;

        // Hero info section (centered)
        let center_y = 180;

        // Hero name
        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let name_x = 184 - (self.hero.name.len() as i32 * 5);
        Text::new(&self.hero.name, Point::new(name_x, center_y - 40), name_style).draw(display)?;

        // Job and Level
        let job_name = self.hero.job.get_name();
        let mut job_level = heapless::String::<64>::new();
        write!(job_level, "{} - Level {}", job_name, self.hero.level).ok();
        let job_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let job_x = 184 - (job_level.len() as i32 * 5);
        Text::new(&job_level, Point::new(job_x, center_y - 10), job_style).draw(display)?;

        // Level up indicator
        if self.hero.level > self.initial_level {
            let levels_gained = self.hero.level - self.initial_level;
            let mut level_up_text = heapless::String::<32>::new();
            if levels_gained == 1 {
                write!(level_up_text, "🎉 Leveled Up! 🎉").ok();
            } else {
                write!(level_up_text, "🎉 +{} Levels! 🎉", levels_gained).ok();
            }
            let level_up_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(0, 255, 0));
            let level_up_x = 184 - (level_up_text.len() as i32 * 5);
            Text::new(&level_up_text, Point::new(level_up_x, center_y + 20), level_up_style).draw(display)?;
        }

        // EXP Progress bar
        let exp_label_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        Text::new("EXP:", Point::new(59, center_y + 50), exp_label_style).draw(display)?;

        let bar_width = 250;
        let bar_height = 20;
        let bar_x = 59;
        let bar_y = center_y + 55;

        // Background bar
        Rectangle::new(
            Point::new(bar_x, bar_y),
            Size::new(bar_width as u32, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        // EXP bar (filled)
        let exp_percentage = (self.hero.experience as f32 / self.hero.experience_to_next_level as f32).min(1.0);
        let filled_width = (exp_percentage * bar_width as f32) as u32;
        if filled_width > 0 {
            Rectangle::new(
                Point::new(bar_x, bar_y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 200, 255)))
            .draw(display)?;
        }

        // EXP text
        let mut exp_text = heapless::String::<32>::new();
        write!(exp_text, "{}/{}", self.hero.experience, self.hero.experience_to_next_level).ok();
        let exp_text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let exp_text_x = bar_x + (bar_width / 2) - (exp_text.len() as i32 * 3);
        Text::new(&exp_text, Point::new(exp_text_x, bar_y + 13), exp_text_style).draw(display)?;

        // Stats gained section
        let stats_y = center_y + 100;
        let stats_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 255, 150));

        let mut total_gained = heapless::String::<32>::new();
        write!(total_gained, "Total EXP Gained: +{}", self.total_exp_gained).ok();
        let total_x = 184 - (total_gained.len() as i32 * 3);
        Text::new(&total_gained, Point::new(total_x, stats_y), stats_style).draw(display)?;

        // Draw stop button at bottom
        let button_x = 100;
        let button_y = 400;
        let button_w = 168;
        let button_h = 50;

        Rectangle::new(
            Point::new(button_x, button_y),
            Size::new(button_w, button_h),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(200, 0, 0)))
        .draw(display)?;

        let button_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("Stop Farming", Point::new(button_x + 20, button_y + 30), button_style).draw(display)?;

        display.flush()?;
        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No-op
    }

    fn needs_full_redraw(&self) -> bool {
        true // Always redraw for EXP updates
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
