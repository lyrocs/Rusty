//! Hero Overview Page
//!
//! Displays hero stats and allows stat point allocation.

use crate::display::Sh8601Driver;
use crate::game::{Hero, Stats};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;

/// Touch area for buttons
#[derive(Debug, Clone)]
struct TouchButton {
    bounds: (i32, i32, u32, u32), // (x, y, width, height)
    action: ButtonAction,
}

impl TouchButton {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Button actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    IncreaseStr,
    DecreaseStr,
    IncreaseAgi,
    DecreaseAgi,
    IncreaseVit,
    DecreaseVit,
    IncreaseInt,
    DecreaseInt,
    IncreaseDex,
    DecreaseDex,
    IncreaseLuk,
    DecreaseLuk,
    ResetStats,
}

/// Stat type for allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatType {
    Str,
    Agi,
    Vit,
    Int,
    Dex,
    Luk,
}

/// Hero overview page
pub struct HeroOverviewPage {
    touch_buttons: Vec<TouchButton>,
    background_color: Rgb888,
    needs_full_redraw: bool,
    pending_allocation: Stats, // Stats with pending allocations (not yet committed)
    allocated_points: u32,     // Total points allocated
}

impl HeroOverviewPage {
    /// Create a new hero overview page
    pub fn new() -> Self {
        Self {
            touch_buttons: Vec::new(),
            background_color: Rgb888::new(15, 20, 30),
            needs_full_redraw: true,
            pending_allocation: Stats::default(),
            allocated_points: 0,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32, hero: &mut Hero) -> bool {
        for button in &self.touch_buttons {
            if button.contains(x, y) {
                self.handle_button_action(button.action, hero);
                return true;
            }
        }
        false
    }

    /// Handle button action
    fn handle_button_action(&mut self, action: ButtonAction, hero: &mut Hero) {
        match action {
            ButtonAction::IncreaseStr => self.allocate_point(StatType::Str, hero),
            ButtonAction::DecreaseStr => self.deallocate_point(StatType::Str, hero),
            ButtonAction::IncreaseAgi => self.allocate_point(StatType::Agi, hero),
            ButtonAction::DecreaseAgi => self.deallocate_point(StatType::Agi, hero),
            ButtonAction::IncreaseVit => self.allocate_point(StatType::Vit, hero),
            ButtonAction::DecreaseVit => self.deallocate_point(StatType::Vit, hero),
            ButtonAction::IncreaseInt => self.allocate_point(StatType::Int, hero),
            ButtonAction::DecreaseInt => self.deallocate_point(StatType::Int, hero),
            ButtonAction::IncreaseDex => self.allocate_point(StatType::Dex, hero),
            ButtonAction::DecreaseDex => self.deallocate_point(StatType::Dex, hero),
            ButtonAction::IncreaseLuk => self.allocate_point(StatType::Luk, hero),
            ButtonAction::DecreaseLuk => self.deallocate_point(StatType::Luk, hero),
            ButtonAction::ResetStats => self.reset_allocation(hero),
        }
        self.needs_full_redraw = true;
    }

    /// Allocate a stat point
    fn allocate_point(&mut self, stat: StatType, hero: &mut Hero) {
        // Check if we have points available
        let available_points = hero.stat_points.saturating_sub(self.allocated_points);
        if available_points == 0 {
            log::warn!("No stat points available");
            return;
        }

        // Check max stat limit (99)
        let current_value = match stat {
            StatType::Str => hero.stats.str,
            StatType::Agi => hero.stats.agi,
            StatType::Vit => hero.stats.vit,
            StatType::Int => hero.stats.int,
            StatType::Dex => hero.stats.dex,
            StatType::Luk => hero.stats.luk,
        };

        if current_value >= 99 {
            log::warn!("Stat already at maximum (99)");
            return;
        }

        // Allocate the point
        match stat {
            StatType::Str => hero.stats.str += 1,
            StatType::Agi => hero.stats.agi += 1,
            StatType::Vit => hero.stats.vit += 1,
            StatType::Int => hero.stats.int += 1,
            StatType::Dex => hero.stats.dex += 1,
            StatType::Luk => hero.stats.luk += 1,
        }

        self.allocated_points += 1;
        hero.stat_points -= 1;

        // Update hero HP/SP based on new stats
        hero.recalculate_max_hp_sp();

        log::info!("Allocated point to {:?}, remaining: {}", stat, hero.stat_points);
    }

    /// Deallocate a stat point (undo allocation)
    fn deallocate_point(&mut self, stat: StatType, hero: &mut Hero) {
        // Get base stat from job
        let base_value = match stat {
            StatType::Str => hero.job.base_stats().str,
            StatType::Agi => hero.job.base_stats().agi,
            StatType::Vit => hero.job.base_stats().vit,
            StatType::Int => hero.job.base_stats().int,
            StatType::Dex => hero.job.base_stats().dex,
            StatType::Luk => hero.job.base_stats().luk,
        };

        // Check if we can decrease (can't go below base)
        let current_value = match stat {
            StatType::Str => hero.stats.str,
            StatType::Agi => hero.stats.agi,
            StatType::Vit => hero.stats.vit,
            StatType::Int => hero.stats.int,
            StatType::Dex => hero.stats.dex,
            StatType::Luk => hero.stats.luk,
        };

        if current_value <= base_value {
            log::warn!("Cannot decrease stat below base value");
            return;
        }

        // Deallocate the point
        match stat {
            StatType::Str => hero.stats.str -= 1,
            StatType::Agi => hero.stats.agi -= 1,
            StatType::Vit => hero.stats.vit -= 1,
            StatType::Int => hero.stats.int -= 1,
            StatType::Dex => hero.stats.dex -= 1,
            StatType::Luk => hero.stats.luk -= 1,
        }

        self.allocated_points = self.allocated_points.saturating_sub(1);
        hero.stat_points += 1;

        // Update hero HP/SP
        hero.recalculate_max_hp_sp();

        log::info!("Deallocated point from {:?}, remaining: {}", stat, hero.stat_points);
    }

    /// Reset all stat allocations
    fn reset_allocation(&mut self, hero: &mut Hero) {
        log::info!("Resetting stats to base values");

        // Reset to base stats
        hero.stats = hero.job.base_stats();

        // Restore all stat points (3 per level, adjusted for base level 1)
        hero.stat_points = (hero.level.saturating_sub(1)) * 3;

        self.allocated_points = 0;

        // Recalculate HP/SP
        hero.recalculate_max_hp_sp();

        log::info!("Stats reset, {} points available", hero.stat_points);
    }

    /// Draw header with job and level
    fn draw_header(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let text_style_title = MonoTextStyle::new(&FONT_6X10, Rgb888::new(255, 255, 200));
        let text_style_info = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 180, 180));

        // Job name
        use core::fmt::Write;
        let mut job_str = heapless::String::<32>::new();
        write!(job_str, "{}", hero.job.name()).ok();
        Text::new(&job_str, Point::new(10, 15), text_style_title).draw(display)?;

        // Level and EXP
        let mut level_str = heapless::String::<32>::new();
        write!(level_str, "Lv {} ({}/{})", hero.level, hero.exp, hero.exp_to_next_level).ok();
        Text::new(&level_str, Point::new(10, 28), text_style_info).draw(display)?;

        Ok(())
    }

    /// Draw stat allocation section
    fn draw_stats(&mut self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let start_y = 45;
        let line_height = 22;

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let button_text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(255, 200, 0));

        self.touch_buttons.clear();

        // Title
        Text::new("Base Stats", Point::new(10, start_y), text_style).draw(display)?;

        // Available points
        use core::fmt::Write;
        let mut points_str = heapless::String::<32>::new();
        write!(points_str, "Points: {}", hero.stat_points).ok();
        Text::new(&points_str, Point::new(250, start_y), text_style).draw(display)?;

        // Stat rows
        let stats = [
            ("STR", hero.stats.str, ButtonAction::IncreaseStr, ButtonAction::DecreaseStr),
            ("AGI", hero.stats.agi, ButtonAction::IncreaseAgi, ButtonAction::DecreaseAgi),
            ("VIT", hero.stats.vit, ButtonAction::IncreaseVit, ButtonAction::DecreaseVit),
            ("INT", hero.stats.int, ButtonAction::IncreaseInt, ButtonAction::DecreaseInt),
            ("DEX", hero.stats.dex, ButtonAction::IncreaseDex, ButtonAction::DecreaseDex),
            ("LUK", hero.stats.luk, ButtonAction::IncreaseLuk, ButtonAction::DecreaseLuk),
        ];

        for (i, (name, value, inc_action, dec_action)) in stats.iter().enumerate() {
            let y = start_y + 15 + (i as i32 * line_height);

            // Stat name and value
            let mut stat_str = heapless::String::<16>::new();
            write!(stat_str, "{}: {:>3}", name, value).ok();
            Text::new(&stat_str, Point::new(15, y), text_style).draw(display)?;

            // [-] button
            let minus_x = 150;
            let minus_bounds = (minus_x, y - 8, 20, 12);
            Rectangle::new(Point::new(minus_x, y - 8), Size::new(20, 12))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 40, 40)))
                .draw(display)?;
            Text::new("-", Point::new(minus_x + 7, y), button_text_style).draw(display)?;
            self.touch_buttons.push(TouchButton {
                bounds: minus_bounds,
                action: *dec_action,
            });

            // [+] button
            let plus_x = 180;
            let plus_bounds = (plus_x, y - 8, 20, 12);
            Rectangle::new(Point::new(plus_x, y - 8), Size::new(20, 12))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 100, 40)))
                .draw(display)?;
            Text::new("+", Point::new(plus_x + 6, y), button_text_style).draw(display)?;
            self.touch_buttons.push(TouchButton {
                bounds: plus_bounds,
                action: *inc_action,
            });
        }

        Ok(())
    }

    /// Draw combat stats section
    fn draw_combat_stats(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let start_y = 200;

        let text_style_title = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 255));
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 180, 180));

        Text::new("Combat Stats", Point::new(10, start_y), text_style_title).draw(display)?;

        use core::fmt::Write;

        // ATK and DEF
        let mut atk_str = heapless::String::<32>::new();
        write!(atk_str, "ATK: {}", hero.stats.calculate_atk()).ok();
        Text::new(&atk_str, Point::new(15, start_y + 15), text_style).draw(display)?;

        let mut def_str = heapless::String::<32>::new();
        write!(def_str, "DEF: {}", hero.stats.calculate_def()).ok();
        Text::new(&def_str, Point::new(150, start_y + 15), text_style).draw(display)?;

        // HIT and FLEE
        let mut hit_str = heapless::String::<32>::new();
        write!(hit_str, "HIT: {}", hero.stats.calculate_hit(hero.level)).ok();
        Text::new(&hit_str, Point::new(15, start_y + 30), text_style).draw(display)?;

        let mut flee_str = heapless::String::<32>::new();
        write!(flee_str, "FLEE: {}", hero.stats.calculate_flee(hero.level)).ok();
        Text::new(&flee_str, Point::new(150, start_y + 30), text_style).draw(display)?;

        // CRIT
        let mut crit_str = heapless::String::<32>::new();
        let crit_rate = hero.stats.calculate_crit_rate();
        write!(crit_str, "CRIT: {:.1}%", crit_rate * 100.0).ok();
        Text::new(&crit_str, Point::new(15, start_y + 45), text_style).draw(display)?;

        Ok(())
    }

    /// Draw HP/SP bars
    fn draw_hp_sp_bars(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let start_y = 270;
        let bar_width = 200;
        let bar_height = 8;

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 180, 180));

        use core::fmt::Write;

        // HP Bar
        let hp_percent = (hero.current_hp as f32 / hero.max_hp as f32).clamp(0.0, 1.0);
        let hp_filled = (bar_width as f32 * hp_percent) as u32;

        Rectangle::new(Point::new(15, start_y), Size::new(bar_width, bar_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 20, 20)))
            .draw(display)?;

        if hp_filled > 0 {
            Rectangle::new(Point::new(15, start_y), Size::new(hp_filled, bar_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(200, 50, 50)))
                .draw(display)?;
        }

        let mut hp_str = heapless::String::<32>::new();
        write!(hp_str, "HP: {}/{}", hero.current_hp, hero.max_hp).ok();
        Text::new(&hp_str, Point::new(15, start_y + 20), text_style).draw(display)?;

        // SP Bar
        let sp_percent = (hero.current_sp as f32 / hero.max_sp as f32).clamp(0.0, 1.0);
        let sp_filled = (bar_width as f32 * sp_percent) as u32;

        Rectangle::new(Point::new(15, start_y + 30), Size::new(bar_width, bar_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 20, 40)))
            .draw(display)?;

        if sp_filled > 0 {
            Rectangle::new(Point::new(15, start_y + 30), Size::new(sp_filled, bar_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 50, 200)))
                .draw(display)?;
        }

        let mut sp_str = heapless::String::<32>::new();
        write!(sp_str, "SP: {}/{}", hero.current_sp, hero.max_sp).ok();
        Text::new(&sp_str, Point::new(15, start_y + 50), text_style).draw(display)?;

        Ok(())
    }

    /// Draw reset button
    fn draw_reset_button(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let button_x = 100;
        let button_y = 380;
        let button_width = 160;
        let button_height = 30;

        Rectangle::new(Point::new(button_x, button_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 40, 40)))
            .draw(display)?;

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        Text::new("Reset Stats", Point::new(button_x + 40, button_y + 18), text_style).draw(display)?;

        self.touch_buttons.push(TouchButton {
            bounds: (button_x, button_y, button_width, button_height),
            action: ButtonAction::ResetStats,
        });

        Ok(())
    }

    /// Draw help text
    fn draw_help_text(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        Text::new("Tap +/- to allocate stats", Point::new(10, 430), text_style).draw(display)?;
        Ok(())
    }
}

impl Page for HeroOverviewPage {
    fn update(&mut self) -> bool {
        // Page always active
        true
    }

    fn draw(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Note: Hero is passed separately via GameManager
        // This is a placeholder - actual drawing happens through GameManager
        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        // Drawing logic moved to draw_with_hero method
        Ok(())
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

impl HeroOverviewPage {
    /// Draw the page with hero data
    pub fn draw_with_hero(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.draw_header(display, hero)?;
        self.draw_stats(display, hero)?;
        self.draw_combat_stats(display, hero)?;
        self.draw_hp_sp_bars(display, hero)?;
        self.draw_reset_button(display)?;
        self.draw_help_text(display)?;

        display.flush()?;

        Ok(())
    }
}
