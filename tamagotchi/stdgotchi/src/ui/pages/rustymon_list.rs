//! Rustymon List Page
//!
//! Displays all owned Rustymon with indicators for team membership

use crate::display::Sh8601Driver;
use crate::game::{Element, Rustymon, RustymonTeam};
use crate::game::element_system::get_element_color;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, PrimitiveStyleBuilder},
    text::Text,
};
use std::error::Error;

/// Actions that can be triggered from Rustymon list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustymonListAction {
    SelectRustymon(usize), // Index in collection
    ScrollUp,
    ScrollDown,
    Close,
}

/// Touch area
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: RustymonListAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Rustymon List page
pub struct RustymonListPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
    scroll_offset: usize, // Current scroll position
}

impl RustymonListPage {
    const ITEMS_PER_PAGE: usize = 6;

    /// Create new Rustymon list page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
            scroll_offset: 0,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<RustymonListAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Rustymon list action: {:?}", area.action);
                return Some(area.action);
            }
        }
        None
    }

    /// Scroll up in the list
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.needs_full_redraw = true;
        }
    }

    /// Scroll down in the list
    pub fn scroll_down(&mut self, total_rustymon: usize) {
        if self.scroll_offset + Self::ITEMS_PER_PAGE < total_rustymon {
            self.scroll_offset += 1;
            self.needs_full_redraw = true;
        }
    }

    /// Draw Rustymon list screen
    pub fn draw_rustymon_list(
        &mut self,
        display: &mut Sh8601Driver,
        rustymon_collection: &[Rustymon],
        rustymon_team: &RustymonTeam,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.touch_areas.clear();

        // Draw title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 200));
        Text::new("Rustymon", Point::new(10, 20), title_style).draw(display)?;

        // Draw count
        let count_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));
        let mut count_str = heapless::String::<32>::new();
        write!(
            count_str,
            "{}/{}",
            rustymon_collection.len(),
            100 // Max collection size
        )
        .ok();
        Text::new(&count_str, Point::new(250, 20), count_style).draw(display)?;

        // Draw list items
        let start_y = 50;
        let item_height = 60;
        let visible_end = (self.scroll_offset + Self::ITEMS_PER_PAGE).min(rustymon_collection.len());

        for (idx, rustymon) in rustymon_collection
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(Self::ITEMS_PER_PAGE)
        {
            let list_idx = idx - self.scroll_offset;
            let y = start_y + (list_idx as i32 * item_height as i32);

            // Draw item background
            let bg_color = if list_idx % 2 == 0 {
                Rgb888::new(20, 25, 35)
            } else {
                Rgb888::new(25, 30, 40)
            };

            Rectangle::new(Point::new(10, y), Size::new(348, item_height as u32))
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            // Check if in active team
            let in_team = rustymon_team.is_in_team(&rustymon.id);

            // Draw team indicator
            if in_team {
                Rectangle::new(Point::new(10, y), Size::new(5, item_height as u32))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 200, 100)))
                    .draw(display)?;
            }

            // Draw element indicator (colored bar)
            let element_color = get_element_color(rustymon.element);
            Rectangle::new(Point::new(20, y + 5), Size::new(30, 50))
                .into_styled(PrimitiveStyle::with_fill(element_color))
                .draw(display)?;

            // Draw Rustymon name
            let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            let mut name_str = heapless::String::<20>::new();
            // Truncate name if too long
            if rustymon.name.len() > 12 {
                write!(name_str, "{}...", &rustymon.name[..9]).ok();
            } else {
                write!(name_str, "{}", rustymon.name).ok();
            }
            Text::new(&name_str, Point::new(60, y + 20), name_style).draw(display)?;

            // Draw level
            let level_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 100));
            let mut level_str = heapless::String::<16>::new();
            write!(level_str, "Lv {}", rustymon.level).ok();
            Text::new(&level_str, Point::new(60, y + 40), level_style).draw(display)?;

            // Draw HP
            let hp_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 255, 100));
            let mut hp_str = heapless::String::<24>::new();
            write!(hp_str, "HP:{}", rustymon.max_hp).ok();
            Text::new(&hp_str, Point::new(200, y + 20), hp_style).draw(display)?;

            // Draw element name
            let elem_style = MonoTextStyle::new(&FONT_10X20, element_color);
            let elem_str = rustymon.element.as_str();
            Text::new(elem_str, Point::new(200, y + 40), elem_style).draw(display)?;

            // Add touch area for this item
            self.touch_areas.push(TouchArea {
                bounds: (10, y, 348, item_height as u32),
                action: RustymonListAction::SelectRustymon(idx),
            });
        }

        // Draw scroll indicators
        if self.scroll_offset > 0 {
            // Up arrow
            let arrow_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 255));
            Text::new("▲", Point::new(330, 40), arrow_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (320, 30, 40, 20),
                action: RustymonListAction::ScrollUp,
            });
        }

        if visible_end < rustymon_collection.len() {
            // Down arrow
            let arrow_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 255));
            Text::new("▼", Point::new(330, 420), arrow_style).draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (320, 410, 40, 20),
                action: RustymonListAction::ScrollDown,
            });
        }

        // Draw "Back" button at bottom
        Rectangle::new(Point::new(10, 420), Size::new(100, 30))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb888::new(60, 60, 80))
                    .stroke_color(Rgb888::new(120, 120, 160))
                    .stroke_width(2)
                    .build(),
            )
            .draw(display)?;

        let back_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("Back", Point::new(30, 440), back_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (10, 420, 100, 30),
            action: RustymonListAction::Close,
        });

        // Draw empty state if no Rustymon
        if rustymon_collection.is_empty() {
            let empty_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));
            Text::new(
                "No Rustymon yet!",
                Point::new(80, 200),
                empty_style,
            )
            .draw(display)?;
            Text::new(
                "Collect fragments",
                Point::new(60, 230),
                empty_style,
            )
            .draw(display)?;
            Text::new(
                "to summon your",
                Point::new(60, 250),
                empty_style,
            )
            .draw(display)?;
            Text::new(
                "first Rustymon!",
                Point::new(60, 270),
                empty_style,
            )
            .draw(display)?;
        }

        display.flush()?;
        Ok(())
    }
}

impl Default for RustymonListPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for RustymonListPage {
    fn update(&mut self) -> bool {
        true // Stay active until explicitly closed
    }

    fn draw(
        &mut self,
        _display: &mut Sh8601Driver,
        _full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // This page requires external data, so drawing is done via draw_rustymon_list
        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering Rustymon list page");
        self.needs_full_redraw = true;
        self.scroll_offset = 0;
    }

    fn on_exit(&mut self) {
        log::info!("Exiting Rustymon list page");
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
