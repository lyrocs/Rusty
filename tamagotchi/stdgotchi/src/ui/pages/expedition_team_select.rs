//! Expedition Team Selection Page
//!
//! Allows players to select monsters and duration for an expedition.
//! Shows element requirements and validates team composition.

use crate::display::St7789pDriver;
use crate::game::core::Element;
use crate::game::systems::expedition::ExpeditionDuration;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action from team selection page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpeditionTeamAction {
    /// No action
    None,
    /// Go back to map selection
    Back,
    /// Toggle monster selection
    ToggleMonster(usize),
    /// Change duration
    SelectDuration(ExpeditionDuration),
    /// Start expedition
    StartExpedition,
}

/// Monster display data for team selection
#[derive(Clone)]
pub struct MonsterSelectData {
    pub id: String,
    pub name: String,
    pub level: u8,
    pub element: Element,
    pub is_available: bool, // Not in expedition
    pub is_selected: bool,
}

/// Expedition Team Selection Page
pub struct ExpeditionTeamSelectPage {
    map_id: String,
    map_name: String,
    required_elements: Vec<Element>,
    monsters: Vec<MonsterSelectData>,
    selected_duration: ExpeditionDuration,
    dirty: bool,

    // Touch areas
    back_area: Option<Rectangle>,
    monster_areas: Vec<Rectangle>,
    duration_areas: Vec<Rectangle>,
    start_area: Option<Rectangle>,
}

impl ExpeditionTeamSelectPage {
    pub fn new(
        map_id: String,
        map_name: String,
        required_elements: Vec<Element>,
        monsters: Vec<MonsterSelectData>,
    ) -> Self {
        Self {
            map_id,
            map_name,
            required_elements,
            monsters,
            selected_duration: ExpeditionDuration::Short,
            dirty: true,
            back_area: None,
            monster_areas: Vec::new(),
            duration_areas: Vec::new(),
            start_area: None,
        }
    }

    /// Get selected monster IDs
    pub fn selected_monster_ids(&self) -> Vec<String> {
        self.monsters.iter()
            .filter(|m| m.is_selected)
            .map(|m| m.id.clone())
            .collect()
    }

    /// Get selected duration
    pub fn selected_duration(&self) -> ExpeditionDuration {
        self.selected_duration
    }

    /// Get map ID
    pub fn map_id(&self) -> &str {
        &self.map_id
    }

    /// Check if element requirements are met
    pub fn requirements_met(&self) -> bool {
        let selected_elements: Vec<Element> = self.monsters.iter()
            .filter(|m| m.is_selected)
            .map(|m| m.element)
            .collect();

        self.required_elements.iter().all(|req| {
            selected_elements.contains(req)
        })
    }

    /// Check if team is valid (1-3 monsters, requirements met)
    pub fn can_start(&self) -> bool {
        let selected_count = self.monsters.iter().filter(|m| m.is_selected).count();
        selected_count >= 1 && selected_count <= 3 && self.requirements_met()
    }

    /// Toggle monster selection
    pub fn toggle_monster(&mut self, index: usize) {
        if index < self.monsters.len() && self.monsters[index].is_available {
            let selected_count = self.monsters.iter().filter(|m| m.is_selected).count();
            let is_currently_selected = self.monsters[index].is_selected;

            // Can deselect or select if under limit
            if is_currently_selected || selected_count < 3 {
                self.monsters[index].is_selected = !is_currently_selected;
                self.dirty = true;
            }
        }
    }

    /// Handle touch and return action
    pub fn handle_touch(&mut self, x: i32, y: i32) -> ExpeditionTeamAction {
        // Check start button
        if let Some(ref rect) = self.start_area {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                if self.can_start() {
                    return ExpeditionTeamAction::StartExpedition;
                }
            }
        }

        // Check monster selection
        for (i, rect) in self.monster_areas.iter().enumerate() {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                return ExpeditionTeamAction::ToggleMonster(i);
            }
        }

        // Check duration selection
        let durations = [
            ExpeditionDuration::Short,
            ExpeditionDuration::Medium,
            ExpeditionDuration::Long,
            ExpeditionDuration::Overnight,
        ];

        for (i, rect) in self.duration_areas.iter().enumerate() {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                if i < durations.len() {
                    self.selected_duration = durations[i];
                    self.dirty = true;
                    return ExpeditionTeamAction::SelectDuration(durations[i]);
                }
            }
        }

        ExpeditionTeamAction::None
    }

    fn element_color(element: &Element) -> Rgb888 {
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(150, 100, 50),
            Element::Wind => Rgb888::new(100, 200, 100),
            Element::Thunder => Rgb888::new(255, 255, 50),
            Element::Shadow => Rgb888::new(100, 50, 150),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(150, 150, 200),
            Element::Neutral => Rgb888::new(180, 180, 180),
        }
    }

    fn element_char(element: &Element) -> char {
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

    fn duration_text(duration: ExpeditionDuration) -> &'static str {
        // NOTE: Dev values. Change to "20min"/"1hr"/"4hr"/"8hr" for production
        match duration {
            ExpeditionDuration::Short => "1min",
            ExpeditionDuration::Medium => "2min",
            ExpeditionDuration::Long => "3min",
            ExpeditionDuration::Overnight => "4min",
        }
    }
}

impl Page for ExpeditionTeamSelectPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));

        // Title with margin
        Text::new("SELECT TEAM", Point::new(70, 18), title_style).draw(display)?;

        // Map name (truncate if needed)
        let map_name = if self.map_name.len() > 30 {
            &self.map_name[..30]
        } else {
            &self.map_name
        };
        Text::new(map_name, Point::new(10, 32), text_style).draw(display)?;

        // Element requirements
        let mut req_x = 10;
        Text::new("Need:", Point::new(req_x, 44), dim_style).draw(display)?;
        req_x += 36;

        for elem in &self.required_elements {
            let c = Self::element_char(elem);
            let elem_style = MonoTextStyle::new(&FONT_6X10, Self::element_color(elem));
            Text::new(&c.to_string(), Point::new(req_x, 44), elem_style).draw(display)?;
            req_x += 10;
        }

        // Status
        let status_color = if self.requirements_met() {
            Rgb888::new(50, 150, 50)
        } else {
            Rgb888::new(200, 80, 80)
        };
        let status_text = if self.requirements_met() { "OK" } else { "Need elem" };
        let status_style = MonoTextStyle::new(&FONT_6X10, status_color);
        Text::new(status_text, Point::new(170, 44), status_style).draw(display)?;

        // Monster list - rounded cards with light theme
        self.monster_areas.clear();
        let list_y = 52;
        let item_height = 28u32;

        for (i, monster) in self.monsters.iter().take(6).enumerate() {
            let y = list_y + (i as i32 * (item_height as i32 + 3));
            let rect = Rectangle::new(Point::new(10, y), Size::new(220, item_height));
            let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(6, 6)));

            let (bg_color, border_color) = if monster.is_selected {
                (Rgb888::new(180, 240, 180), Rgb888::new(100, 200, 100))
            } else if monster.is_available {
                (Rgb888::new(220, 225, 235), Rgb888::new(150, 160, 180))
            } else {
                (Rgb888::new(210, 210, 215), Rgb888::new(180, 180, 185))
            };

            // Fill
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(bg_color)
                .build())
                .draw(display)?;

            // Border
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(border_color)
                .stroke_width(2)
                .build())
                .draw(display)?;

            self.monster_areas.push(rect);

            // Element indicator
            let elem_style = MonoTextStyle::new(&FONT_6X10, Self::element_color(&monster.element));
            Text::new(&Self::element_char(&monster.element).to_string(), Point::new(16, y + 18), elem_style).draw(display)?;

            // Name and level (truncate name if needed)
            let name_color = if monster.is_available { Rgb888::BLACK } else { Rgb888::new(140, 140, 140) };
            let name_style = MonoTextStyle::new(&FONT_6X10, name_color);
            let truncated_name = if monster.name.len() > 15 {
                &monster.name[..15]
            } else {
                &monster.name
            };
            let info = format!("{} Lv.{}", truncated_name, monster.level);
            Text::new(&info, Point::new(30, y + 18), name_style).draw(display)?;

            // Selection indicator
            if monster.is_selected {
                Text::new("[X]", Point::new(200, y + 18), text_style).draw(display)?;
            } else if monster.is_available {
                Text::new("[ ]", Point::new(200, y + 18), dim_style).draw(display)?;
            } else {
                Text::new("---", Point::new(200, y + 18), dim_style).draw(display)?;
            }
        }

        // Duration selection - rounded tabs with light theme
        self.duration_areas.clear();
        let dur_y = 200;
        Text::new("Duration:", Point::new(10, dur_y), dim_style).draw(display)?;

        let durations = [
            ExpeditionDuration::Short,
            ExpeditionDuration::Medium,
            ExpeditionDuration::Long,
            ExpeditionDuration::Overnight,
        ];

        for (i, duration) in durations.iter().enumerate() {
            let x = 10 + (i as i32 * 54);
            let rect = Rectangle::new(Point::new(x, dur_y + 8), Size::new(52, 22));
            let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(6, 6)));

            let (bg_color, border_color) = if self.selected_duration == *duration {
                (Rgb888::new(100, 180, 240), Rgb888::new(60, 140, 200))
            } else {
                (Rgb888::new(200, 210, 220), Rgb888::new(150, 160, 170))
            };

            // Fill
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(bg_color)
                .build())
                .draw(display)?;

            // Border
            rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(border_color)
                .stroke_width(1)
                .build())
                .draw(display)?;

            self.duration_areas.push(rect);

            let dur_text = Self::duration_text(*duration);
            Text::new(dur_text, Point::new(x + 12, dur_y + 22), text_style).draw(display)?;
        }

        // Start button - full width rounded button
        let (start_bg, start_border) = if self.can_start() {
            (Rgb888::new(100, 200, 100), Rgb888::new(60, 160, 60))
        } else {
            (Rgb888::new(200, 200, 205), Rgb888::new(160, 160, 165))
        };
        let start_rect = Rectangle::new(Point::new(8, 240), Size::new(224, 42));
        let start_rounded = RoundedRectangle::new(start_rect, CornerRadii::new(Size::new(10, 10)));

        // Fill
        start_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(start_bg)
            .build())
            .draw(display)?;

        // Border
        start_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(start_border)
            .stroke_width(2)
            .build())
            .draw(display)?;

        let start_text_style = MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE);
        Text::new("START EXPEDITION", Point::new(48, 266), start_text_style).draw(display)?;
        self.start_area = Some(start_rect);
        self.back_area = None;

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
