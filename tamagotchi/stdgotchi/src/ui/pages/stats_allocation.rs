//! Stats Allocation Page
//!
//! Dedicated page for allocating stat points with reset functionality.

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
    Close,
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

/// Stats allocation page
pub struct StatsAllocationPage {
    touch_buttons: Vec<TouchButton>,
    background_color: Rgb888,
    needs_full_redraw: bool,
}

impl StatsAllocationPage {
    /// Create a new stats allocation page
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
        self.draw_combat_preview(display, hero)?;
        self.draw_bottom_buttons(display)?;

        display.flush()?;

        Ok(())
    }

    /// Draw header
    fn draw_header(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style_points = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 255, 100));

        use core::fmt::Write;

        Text::new("Stat Allocation", Point::new(15, 30), text_style_title).draw(display)?;

        // Available points
        let mut points_str = heapless::String::<32>::new();
        write!(points_str, "Points: {}", hero.stat_points).ok();
        Text::new(&points_str, Point::new(15, 55), text_style_points).draw(display)?;

        Ok(())
    }

    /// Draw stat allocation section
    fn draw_stats(&mut self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let start_y = 80;
        let line_height = 45;
        let margin = 15;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let button_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 200, 0));

        self.touch_buttons.clear();

        // Stat rows
        let stats = [
            ("STR", hero.stats.str, ButtonAction::IncreaseStr, ButtonAction::DecreaseStr),
            ("AGI", hero.stats.agi, ButtonAction::IncreaseAgi, ButtonAction::DecreaseAgi),
            ("VIT", hero.stats.vit, ButtonAction::IncreaseVit, ButtonAction::DecreaseVit),
            ("INT", hero.stats.int, ButtonAction::IncreaseInt, ButtonAction::DecreaseInt),
            ("DEX", hero.stats.dex, ButtonAction::IncreaseDex, ButtonAction::DecreaseDex),
            ("LUK", hero.stats.luk, ButtonAction::IncreaseLuk, ButtonAction::DecreaseLuk),
        ];

        use core::fmt::Write;

        for (i, (name, value, inc_action, dec_action)) in stats.iter().enumerate() {
            let y = start_y + (i as i32 * line_height);

            // Stat name and value
            let mut stat_str = heapless::String::<16>::new();
            write!(stat_str, "{}: {:>2}", name, value).ok();
            Text::new(&stat_str, Point::new(margin + 5, y + 30), text_style).draw(display)?;

            // [-] button
            let minus_x = 130;
            let button_width = 50;
            let button_height = 40;
            Rectangle::new(Point::new(minus_x, y), Size::new(button_width, button_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(120, 40, 40)))
                .draw(display)?;
            Text::new("-", Point::new(minus_x + 19, y + 28), button_text_style).draw(display)?;

            self.touch_buttons.push(TouchButton {
                bounds: (minus_x, y, button_width, button_height),
                action: *dec_action,
            });

            // [+] button
            let plus_x = 185;
            Rectangle::new(Point::new(plus_x, y), Size::new(button_width, button_height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 120, 60)))
                .draw(display)?;
            Text::new("+", Point::new(plus_x + 18, y + 28), button_text_style).draw(display)?;

            self.touch_buttons.push(TouchButton {
                bounds: (plus_x, y, button_width, button_height),
                action: *inc_action,
            });

            // Preview stat effect (right side)
            let effect_x = 250;
            let effect_text = match name {
                &"STR" => format_stat_effect("ATK", hero.stats.str, hero.stats.calculate_atk()),
                &"AGI" => format_stat_effect("FLEE", hero.stats.agi, hero.stats.calculate_flee(hero.level)),
                &"VIT" => format_stat_effect("HP", hero.stats.vit, hero.max_hp),
                &"INT" => format_stat_effect("SP", hero.stats.int, hero.max_sp),
                &"DEX" => format_stat_effect("HIT", hero.stats.dex, hero.stats.calculate_hit(hero.level)),
                &"LUK" => format_stat_effect("CRT", hero.stats.luk, (hero.stats.calculate_crit_rate() * 100.0) as u32),
                _ => heapless::String::<32>::new(),
            };

            let effect_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 200));
            Text::new(&effect_text, Point::new(effect_x, y + 30), effect_style).draw(display)?;
        }

        Ok(())
    }

    /// Draw combat stats preview
    fn draw_combat_preview(&self, display: &mut Sh8601Driver, hero: &Hero) -> Result<(), Box<dyn Error>> {
        let start_y = 360;
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        use core::fmt::Write;

        let mut preview = heapless::String::<64>::new();
        write!(
            preview,
            "Total: ATK:{} DEF:{} HP:{}/{}",
            hero.stats.calculate_atk(),
            hero.stats.calculate_def(),
            hero.current_hp,
            hero.max_hp
        ).ok();

        Text::new(&preview, Point::new(15, start_y), text_style).draw(display)?;

        Ok(())
    }

    /// Draw bottom buttons
    fn draw_bottom_buttons(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let button_y = 400;
        let button_height = 55;
        let button_width = 165;
        let margin = 15;
        let spacing = 8;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

        // Reset button (left)
        Rectangle::new(Point::new(margin, button_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(120, 40, 40)))
            .draw(display)?;
        Text::new("RESET", Point::new(margin + 50, button_y + 35), text_style).draw(display)?;

        self.touch_buttons.push(TouchButton {
            bounds: (margin, button_y, button_width, button_height),
            action: ButtonAction::ResetStats,
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

impl Page for StatsAllocationPage {
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

/// Helper to format stat effects
fn format_stat_effect(label: &str, _stat_value: u32, effect_value: u32) -> heapless::String<32> {
    use core::fmt::Write;
    let mut result = heapless::String::<32>::new();
    write!(result, "{}:{}", label, effect_value).ok();
    result
}
