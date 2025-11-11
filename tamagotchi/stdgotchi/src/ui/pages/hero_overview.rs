//! Hero Overview Page
//!
//! Displays hero stats (read-only).

use crate::display::Sh8601Driver;
use crate::game::Hero;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
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
    AllocateStats,
    Close,
}

/// Hero overview page
pub struct HeroOverviewPage {
    touch_buttons: Vec<TouchButton>,
    background_color: Rgb888,
    needs_full_redraw: bool,
}

impl HeroOverviewPage {
    /// Create a new hero overview page
    pub fn new() -> Self {
        Self {
            touch_buttons: Vec::new(),
            background_color: Rgb888::new(15, 20, 30),
            needs_full_redraw: true,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<ButtonAction> {
        for button in &self.touch_buttons {
            if button.contains(x, y) {
                return Some(button.action);
            }
        }
        None
    }

    /// Draw header with job and level
    fn draw_header(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        use core::fmt::Write;

        // Hero Overview title
        Text::new("Hero Overview", Point::new(15, 25), text_style_title).draw(display)?;

        // Job name
        let mut job_str = heapless::String::<32>::new();
        write!(job_str, "{}", hero.job.name()).ok();
        Text::new(&job_str, Point::new(15, 50), text_style_info).draw(display)?;

        // Level and EXP
        let mut level_str = heapless::String::<48>::new();
        write!(level_str, "Lv {} ({}/{})", hero.level, hero.exp, hero.exp_to_next_level).ok();
        Text::new(&level_str, Point::new(15, 75), text_style_info).draw(display)?;

        // Gold
        let mut gold_str = heapless::String::<32>::new();
        write!(gold_str, "Gold: {}", hero.gold).ok();
        Text::new(&gold_str, Point::new(15, 100), text_style_info).draw(display)?;

        Ok(())
    }

    /// Draw stats section (read-only, 2-column layout)
    fn draw_stats(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let start_y = 130;
        let line_height = 28;
        let margin = 15;
        let col2_x = 195;

        let text_style_label = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 255));
        let text_style_value = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        use core::fmt::Write;

        // Title
        Text::new("Stats", Point::new(margin, start_y), text_style_label).draw(display)?;

        // Available points
        let mut points_str = heapless::String::<32>::new();
        write!(points_str, "Points: {}", hero.stat_points).ok();
        let points_color = if hero.stat_points > 0 {
            Rgb888::new(100, 255, 100)
        } else {
            Rgb888::new(150, 150, 150)
        };
        let points_style = MonoTextStyle::new(&FONT_10X20, points_color);
        Text::new(&points_str, Point::new(col2_x, start_y), points_style).draw(display)?;

        // Base stats (left column)
        let y_offset = start_y + 25;
        let stats = [
            ("STR", hero.stats.str),
            ("AGI", hero.stats.agi),
            ("VIT", hero.stats.vit),
            ("INT", hero.stats.int),
            ("DEX", hero.stats.dex),
            ("LUK", hero.stats.luk),
        ];

        for (i, (name, value)) in stats.iter().enumerate() {
            let y = y_offset + (i as i32 * line_height);
            let mut stat_str = heapless::String::<16>::new();
            write!(stat_str, "{}: {:>2}", name, value).ok();
            Text::new(&stat_str, Point::new(margin + 5, y), text_style_value).draw(display)?;
        }

        Ok(())
    }

    /// Draw combat stats section (right side)
    fn draw_combat_stats(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let start_x = 195;
        let start_y = 180;
        let line_height = 28;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        use core::fmt::Write;

        // Combat stats
        let combat_stats = [
            ("ATK", hero.stats.calculate_atk()),
            ("DEF", hero.stats.calculate_def()),
            ("HIT", hero.stats.calculate_hit(hero.level)),
            ("FLE", hero.stats.calculate_flee(hero.level)),
        ];

        for (i, (name, value)) in combat_stats.iter().enumerate() {
            let y = start_y + (i as i32 * line_height);
            let mut stat_str = heapless::String::<24>::new();
            write!(stat_str, "{}:{}", name, value).ok();
            Text::new(&stat_str, Point::new(start_x, y), text_style).draw(display)?;
        }

        // Crit rate
        let crit_y = start_y + (combat_stats.len() as i32 * line_height);
        let mut crit_str = heapless::String::<24>::new();
        let crit_rate = hero.stats.calculate_crit_rate();
        write!(crit_str, "CRT:{:.1}%", crit_rate * 100.0).ok();
        Text::new(&crit_str, Point::new(start_x, crit_y), text_style).draw(display)?;

        Ok(())
    }

    /// Draw HP/SP bars
    fn draw_hp_sp_bars(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let start_y = 295;
        let bar_width = 338;
        let bar_height = 12;
        let margin = 15;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        use core::fmt::Write;

        // HP Bar
        let hp_percent = (hero.current_hp as f32 / hero.max_hp as f32).clamp(0.0, 1.0);
        let hp_filled = (bar_width as f32 * hp_percent) as u32;

        Rectangle::new(Point::new(margin, start_y), Size::new(bar_width, bar_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 20, 20)))
            .draw(display)?;

        if hp_filled > 0 {
            Rectangle::new(Point::new(margin, start_y), Size::new(hp_filled, bar_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(200, 50, 50)))
                .draw(display)?;
        }

        let mut hp_str = heapless::String::<32>::new();
        write!(hp_str, "HP: {}/{}", hero.current_hp, hero.max_hp).ok();
        Text::new(&hp_str, Point::new(margin, start_y + 23), text_style).draw(display)?;

        // SP Bar
        let sp_y = start_y + 38;
        let sp_percent = (hero.current_sp as f32 / hero.max_sp as f32).clamp(0.0, 1.0);
        let sp_filled = (bar_width as f32 * sp_percent) as u32;

        Rectangle::new(Point::new(margin, sp_y), Size::new(bar_width, bar_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 20, 40)))
            .draw(display)?;

        if sp_filled > 0 {
            Rectangle::new(Point::new(margin, sp_y), Size::new(sp_filled, bar_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 50, 200)))
                .draw(display)?;
        }

        let mut sp_str = heapless::String::<32>::new();
        write!(sp_str, "SP: {}/{}", hero.current_sp, hero.max_sp).ok();
        Text::new(&sp_str, Point::new(margin, sp_y + 23), text_style).draw(display)?;

        Ok(())
    }

    /// Draw bottom buttons
    fn draw_buttons(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let button_y = 400;
        let button_height = 55;
        let button_width = 165;
        let margin = 15;
        let spacing = 8;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        // Allocate Stats button (left)
        Rectangle::new(Point::new(margin, button_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 100, 120)))
            .draw(display)?;
        Text::new("STATS", Point::new(margin + 50, button_y + 35), text_style).draw(display)?;

        self.touch_buttons.push(TouchButton {
            bounds: (margin, button_y, button_width, button_height),
            action: ButtonAction::AllocateStats,
        });

        // Close button (right)
        let close_x = margin + button_width as i32 + spacing;
        Rectangle::new(Point::new(close_x, button_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 80, 80)))
            .draw(display)?;
        Text::new("BACK", Point::new(close_x + 57, button_y + 35), text_style).draw(display)?;

        self.touch_buttons.push(TouchButton {
            bounds: (close_x, button_y, button_width, button_height),
            action: ButtonAction::Close,
        });

        Ok(())
    }
}

impl Page for HeroOverviewPage {
    fn update(&mut self) -> bool {
        true
    }

    fn draw(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }
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

        self.touch_buttons.clear();
        self.draw_header(display, hero)?;
        self.draw_stats(display, hero)?;
        self.draw_combat_stats(display, hero)?;
        self.draw_hp_sp_bars(display, hero)?;
        self.draw_buttons(display)?;

        display.flush()?;

        Ok(())
    }
}
