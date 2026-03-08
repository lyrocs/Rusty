//! Expedition Detail Page
//!
//! Shows active expedition details: monsters, progress, time remaining, and cancel option.

use crate::game::core::Element;
use crate::game::systems::expedition::{Expedition, ExpeditionDuration};
use crate::ui::page::Page;
use crate::display::St7789pDriver;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Actions from expedition detail page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpeditionDetailAction {
    None,
    /// Cancel the expedition
    Cancel,
    /// Go back to home
    Back,
}

/// Data for a monster in the expedition
#[derive(Clone)]
pub struct ExpeditionMonsterData {
    pub name: String,
    pub species_id: String,
    pub level: u8,
    pub element: Element,
}

/// Expedition detail page
pub struct ExpeditionDetailPage {
    /// Expedition slot index (0 or 1)
    slot_index: usize,
    /// Map name
    map_name: String,
    /// Expedition duration
    duration: ExpeditionDuration,
    /// Started at timestamp
    started_at: u64,
    /// Monsters on this expedition
    monsters: Vec<ExpeditionMonsterData>,

    // Touch areas
    cancel_button: Option<Rectangle>,
    back_button: Option<Rectangle>,

    // State
    dirty: bool,
}

impl ExpeditionDetailPage {
    pub fn new(
        slot_index: usize,
        expedition: &Expedition,
        map_name: String,
        monsters: Vec<ExpeditionMonsterData>,
    ) -> Self {
        Self {
            slot_index,
            map_name,
            duration: expedition.duration,
            started_at: expedition.started_at,
            monsters,
            cancel_button: None,
            back_button: None,
            dirty: true,
        }
    }

    /// Get current time
    fn current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Calculate progress (0.0 to 1.0)
    fn progress(&self) -> f32 {
        let now = Self::current_time();
        let elapsed = now.saturating_sub(self.started_at);
        let total = self.duration.seconds();
        (elapsed as f32 / total as f32).min(1.0)
    }

    /// Get remaining time string
    fn remaining_time_string(&self) -> String {
        let now = Self::current_time();
        let end_time = self.started_at + self.duration.seconds();
        if now >= end_time {
            "Complete!".to_string()
        } else {
            let remaining = end_time - now;
            let mins = remaining / 60;
            let secs = remaining % 60;
            if mins > 60 {
                format!("{}h {}m remaining", mins / 60, mins % 60)
            } else {
                format!("{}m {}s remaining", mins, secs)
            }
        }
    }

    /// Check if expedition is complete
    fn is_complete(&self) -> bool {
        let now = Self::current_time();
        now >= self.started_at + self.duration.seconds()
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> ExpeditionDetailAction {
        let point = Point::new(x, y);

        // Check cancel button
        if let Some(ref rect) = self.cancel_button {
            if rect.contains(point) {
                return ExpeditionDetailAction::Cancel;
            }
        }

        // Check back button
        if let Some(ref rect) = self.back_button {
            if rect.contains(point) {
                return ExpeditionDetailAction::Back;
            }
        }

        ExpeditionDetailAction::None
    }

    /// Get the expedition slot index
    pub fn slot_index(&self) -> usize {
        self.slot_index
    }

    fn element_color(element: Element) -> Rgb888 {
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(100, 180, 80),
            Element::Wind => Rgb888::new(100, 200, 150),
            Element::Thunder => Rgb888::new(255, 255, 50),
            Element::Shadow => Rgb888::new(150, 100, 200),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(180, 180, 220),
            Element::Neutral => Rgb888::new(180, 180, 180),
        }
    }

    fn element_char(element: Element) -> char {
        match element {
            Element::Fire => 'F',
            Element::Water => 'W',
            Element::Earth => 'E',
            Element::Wind => 'A',
            Element::Thunder => 'T',
            Element::Shadow => 'S',
            Element::Holy => 'H',
            Element::Ghost => 'G',
            Element::Neutral => 'N',
        }
    }

    fn duration_name(duration: ExpeditionDuration) -> &'static str {
        match duration {
            ExpeditionDuration::Short => "Short",
            ExpeditionDuration::Medium => "Medium",
            ExpeditionDuration::Long => "Long",
            ExpeditionDuration::Overnight => "Overnight",
        }
    }
}

impl Page for ExpeditionDetailPage {
    fn draw(&mut self, display: &mut St7789pDriver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));

        // Background
        let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
        display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;

        // Header
        let header_rect = Rectangle::new(Point::new(2, 2), Size::new(236, 28));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 150, 200))
            .build())
            .draw(display)?;

        Text::new("Expedition Details", Point::new(50, 22), title_style).draw(display)?;

        // Map name section
        let map_section_y = 38;
        let map_rect = Rectangle::new(Point::new(10, map_section_y), Size::new(220, 45));
        let map_rounded = RoundedRectangle::new(map_rect, CornerRadii::new(Size::new(6, 6)));
        map_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(220, 225, 235))
            .build())
            .draw(display)?;

        // Map name
        let map_display = if self.map_name.len() > 22 {
            &self.map_name[..22]
        } else {
            &self.map_name
        };
        Text::new(map_display, Point::new(18, map_section_y + 16), title_style).draw(display)?;

        // Duration
        let duration_text = format!("Duration: {}", Self::duration_name(self.duration));
        Text::new(&duration_text, Point::new(18, map_section_y + 32), dim_style).draw(display)?;

        // Progress section
        let progress_y = 90;
        Text::new("Progress:", Point::new(10, progress_y), text_style).draw(display)?;

        // Progress bar
        let progress = self.progress();
        let progress_percent = (progress * 100.0) as u8;
        let is_complete = self.is_complete();

        let bar_y = progress_y + 8;
        let bar_width = 180u32;
        let bar_height = 16u32;

        // Bar background
        let bar_bg = Rectangle::new(Point::new(10, bar_y), Size::new(bar_width, bar_height));
        let bar_bg_rounded = RoundedRectangle::new(bar_bg, CornerRadii::new(Size::new(4, 4)));
        bar_bg_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(180, 180, 190))
            .build())
            .draw(display)?;

        // Bar fill
        let fill_width = ((bar_width as f32 * progress).round() as u32).max(1);
        let fill_color = if is_complete {
            Rgb888::new(80, 180, 80)
        } else {
            Rgb888::new(60, 120, 200)
        };
        let bar_fill = Rectangle::new(Point::new(10, bar_y), Size::new(fill_width, bar_height));
        let bar_fill_rounded = RoundedRectangle::new(bar_fill, CornerRadii::new(Size::new(4, 4)));
        bar_fill_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(fill_color)
            .build())
            .draw(display)?;

        // Progress percentage
        let percent_text = format!("{}%", progress_percent);
        Text::new(&percent_text, Point::new(195, bar_y + 12), text_style).draw(display)?;

        // Time remaining
        let time_text = self.remaining_time_string();
        let time_color = if is_complete {
            Rgb888::new(50, 150, 50)
        } else {
            Rgb888::new(80, 80, 80)
        };
        let time_style = MonoTextStyle::new(&FONT_6X10, time_color);
        Text::new(&time_text, Point::new(10, bar_y + 30), time_style).draw(display)?;

        // Monsters section
        let monsters_y = 145;
        Text::new("Monsters:", Point::new(10, monsters_y), text_style).draw(display)?;

        let monster_card_y = monsters_y + 8;
        let monster_card_w = 70u32;
        let monster_card_h = 55u32;
        let monster_spacing = 75;

        for (i, monster) in self.monsters.iter().enumerate() {
            let x = 10 + (i as i32 * monster_spacing);

            // Monster card background
            let card_rect = Rectangle::new(Point::new(x, monster_card_y), Size::new(monster_card_w, monster_card_h));
            let card_rounded = RoundedRectangle::new(card_rect, CornerRadii::new(Size::new(6, 6)));
            card_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(220, 225, 235))
                .build())
                .draw(display)?;

            // Element indicator
            let elem_color = Self::element_color(monster.element);
            let elem_rect = Rectangle::new(Point::new(x + 4, monster_card_y + 4), Size::new(20, 18));
            display.fill_solid(&elem_rect, elem_color)?;

            let elem_char = Self::element_char(monster.element);
            let elem_text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
            Text::new(&elem_char.to_string(), Point::new(x + 10, monster_card_y + 16), elem_text_style).draw(display)?;

            // Level
            let level_text = format!("Lv{}", monster.level);
            Text::new(&level_text, Point::new(x + 28, monster_card_y + 16), dim_style).draw(display)?;

            // Name (truncated)
            let name = if monster.name.len() > 9 {
                &monster.name[..9]
            } else {
                &monster.name
            };
            Text::new(name, Point::new(x + 4, monster_card_y + 32), text_style).draw(display)?;

            // Status
            let status = if is_complete { "Ready!" } else { "Exploring" };
            let status_color = if is_complete {
                Rgb888::new(50, 150, 50)
            } else {
                Rgb888::new(100, 100, 100)
            };
            let status_style = MonoTextStyle::new(&FONT_6X10, status_color);
            Text::new(status, Point::new(x + 4, monster_card_y + 46), status_style).draw(display)?;
        }

        // Buttons section
        let button_y = 220;
        let button_h = 40u32;

        // Cancel button (only show if not complete)
        if !is_complete {
            let cancel_rect = Rectangle::new(Point::new(10, button_y), Size::new(105, button_h));
            let cancel_rounded = RoundedRectangle::new(cancel_rect, CornerRadii::new(Size::new(8, 8)));
            cancel_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(220, 100, 100))
                .build())
                .draw(display)?;
            cancel_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(180, 60, 60))
                .stroke_width(2)
                .build())
                .draw(display)?;

            let cancel_text_style = MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE);
            Text::new("CANCEL", Point::new(32, button_y + 26), cancel_text_style).draw(display)?;

            self.cancel_button = Some(cancel_rect);
        } else {
            self.cancel_button = None;
        }

        // Back button
        let back_x = if is_complete { 65 } else { 125 };
        let back_w = if is_complete { 110u32 } else { 105u32 };
        let back_rect = Rectangle::new(Point::new(back_x, button_y), Size::new(back_w, button_h));
        let back_rounded = RoundedRectangle::new(back_rect, CornerRadii::new(Size::new(8, 8)));
        back_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 150, 200))
            .build())
            .draw(display)?;
        back_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(60, 100, 160))
            .stroke_width(2)
            .build())
            .draw(display)?;

        let back_text_style = MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE);
        let back_text_x = if is_complete { back_x + 30 } else { back_x + 25 };
        Text::new("BACK", Point::new(back_text_x, button_y + 26), back_text_style).draw(display)?;

        self.back_button = Some(back_rect);

        // Instructions
        let instruction = if is_complete {
            "Tap expedition on Home to collect!"
        } else {
            "Cancel returns monsters without rewards"
        };
        let instruction_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(120, 120, 120));
        Text::new(instruction, Point::new(10, 275), instruction_style).draw(display)?;

        display.flush()?;
        self.dirty = false;
        Ok(())
    }

    fn update(&mut self) -> bool {
        // Refresh display every second to update timer
        self.dirty = true;
        true
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.dirty
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
