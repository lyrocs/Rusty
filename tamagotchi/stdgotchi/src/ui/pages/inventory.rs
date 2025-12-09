//! Inventory Page
//!
//! Shows player resources: crystals and elemental essences.

use crate::display::St7789pDriver;
use crate::game::core::Element;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    text::Text,
};
use std::error::Error;

/// Action from inventory page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAction {
    /// No action
    None,
    /// Go back
    Back,
}

/// Inventory page
pub struct InventoryPage {
    crystals: u32,
    essences: [(Element, u16); 8],

    // Touch areas
    back_area: Option<Rectangle>,

    dirty: bool,
}

impl InventoryPage {
    pub fn new(crystals: u32, essences: [(Element, u16); 8]) -> Self {
        Self {
            crystals,
            essences,
            back_area: None,
            dirty: true,
        }
    }

    /// Create from player data
    pub fn from_player(crystals: u32, fire: u16, water: u16, earth: u16, wind: u16, thunder: u16, shadow: u16, holy: u16, ghost: u16) -> Self {
        Self::new(crystals, [
            (Element::Fire, fire),
            (Element::Water, water),
            (Element::Earth, earth),
            (Element::Wind, wind),
            (Element::Thunder, thunder),
            (Element::Shadow, shadow),
            (Element::Holy, holy),
            (Element::Ghost, ghost),
        ])
    }

    /// Handle touch input
    pub fn handle_touch(&self, x: i32, y: i32) -> InventoryAction {
        let point = Point::new(x, y);

        if let Some(rect) = self.back_area {
            if rect.contains(point) {
                return InventoryAction::Back;
            }
        }

        InventoryAction::None
    }

    fn element_name(element: Element) -> &'static str {
        match element {
            Element::Fire => "Fire",
            Element::Water => "Water",
            Element::Earth => "Earth",
            Element::Wind => "Wind",
            Element::Thunder => "Thunder",
            Element::Shadow => "Shadow",
            Element::Holy => "Holy",
            Element::Ghost => "Ghost",
        }
    }

    fn element_char(element: Element) -> char {
        match element {
            Element::Fire => 'F',
            Element::Water => 'W',
            Element::Earth => 'E',
            Element::Wind => 'N',
            Element::Thunder => 'T',
            Element::Shadow => 'S',
            Element::Holy => 'H',
            Element::Ghost => 'G',
        }
    }

    fn element_color(element: Element) -> Rgb888 {
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(150, 120, 50),
            Element::Wind => Rgb888::new(100, 220, 150),
            Element::Thunder => Rgb888::new(255, 255, 100),
            Element::Shadow => Rgb888::new(150, 50, 200),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(180, 180, 220),
        }
    }
}

impl Page for InventoryPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));

        if full_redraw {
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        // Header
        Text::new("INVENTORY", Point::new(125, 35), title_style).draw(display)?;

        // Crystals section
        let crystal_y = 70;
        Text::new("RESOURCES", Point::new(30, crystal_y), dim_style).draw(display)?;

        let crystal_box = Rectangle::new(Point::new(30, crystal_y + 10), Size::new(308, 50));
        display.fill_solid(&crystal_box, Rgb888::new(35, 40, 50))?;
        Rectangle::new(Point::new(30, crystal_y + 10), Size::new(308, 50))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 180, 255), 2))
            .draw(display)?;

        let crystal_text = format!("Crystals: {}", self.crystals);
        Text::new(&crystal_text, Point::new(50, crystal_y + 42), text_style).draw(display)?;

        // Essences section
        let essence_y = 150;
        Text::new("ESSENCES", Point::new(30, essence_y), dim_style).draw(display)?;

        let essence_box_height = 220u32;
        let essence_box = Rectangle::new(Point::new(30, essence_y + 10), Size::new(308, essence_box_height));
        display.fill_solid(&essence_box, Rgb888::new(35, 40, 50))?;

        // Draw each essence
        let mut y_offset = essence_y + 35;
        let col1_x = 50;
        let col2_x = 200;

        for (i, (element, count)) in self.essences.iter().enumerate() {
            let x = if i % 2 == 0 { col1_x } else { col2_x };
            let y = y_offset;

            if i % 2 == 1 {
                y_offset += 50;
            }

            // Element icon background
            let icon_bg = Rectangle::new(Point::new(x - 5, y - 15), Size::new(130, 35));
            display.fill_solid(&icon_bg, Rgb888::new(40, 45, 55))?;

            // Element icon
            let elem_color = Self::element_color(*element);
            let elem_char = Self::element_char(*element);
            let elem_style = MonoTextStyle::new(&FONT_9X15, elem_color);

            Text::new(&format!("[{}]", elem_char), Point::new(x, y), elem_style).draw(display)?;

            // Element name and count
            let name = Self::element_name(*element);
            let essence_text = format!("{}: {}", name, count);
            Text::new(&essence_text, Point::new(x + 35, y), text_style).draw(display)?;
        }

        // Usage hint
        let hint_y = 390;
        Text::new("Essences are used for major upgrades", Point::new(50, hint_y), dim_style).draw(display)?;

        // Back button
        let back_rect = Rectangle::new(Point::new(15, 410), Size::new(80, 30));
        display.fill_solid(&back_rect, Rgb888::new(80, 60, 60))?;
        Text::new("< BACK", Point::new(25, 430), text_style).draw(display)?;
        self.back_area = Some(back_rect);

        display.flush()?;
        self.dirty = false;
        Ok(())
    }

    fn update(&mut self) -> bool {
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
