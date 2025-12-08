//! Expedition Team Selection Page
//!
//! Allows players to select monsters and duration for an expedition.
//! Shows element requirements and validates team composition.

use crate::display::Sh8601Driver;
use crate::game::core::Element;
use crate::game::systems::expedition::ExpeditionDuration;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Rectangle,
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
        // Check back button
        if let Some(ref rect) = self.back_area {
            if x >= rect.top_left.x && x < rect.top_left.x + rect.size.width as i32
                && y >= rect.top_left.y && y < rect.top_left.y + rect.size.height as i32
            {
                return ExpeditionTeamAction::Back;
            }
        }

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
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));

        // Title
        Text::new("SELECT TEAM", Point::new(110, 25), title_style).draw(display)?;

        // Map name
        Text::new(&self.map_name, Point::new(15, 45), text_style).draw(display)?;

        // Element requirements
        let mut req_x = 15;
        Text::new("Required:", Point::new(req_x, 60), dim_style).draw(display)?;
        req_x += 90; // "Required:" is 9 chars * 9px + padding

        for elem in &self.required_elements {
            let c = Self::element_char(elem);
            let elem_style = MonoTextStyle::new(&FONT_9X15, Self::element_color(elem));
            Text::new(&c.to_string(), Point::new(req_x, 60), elem_style).draw(display)?;
            req_x += 15;
        }

        // Status
        let status_color = if self.requirements_met() {
            Rgb888::new(100, 200, 100)
        } else {
            Rgb888::new(200, 100, 100)
        };
        let status_text = if self.requirements_met() { "OK" } else { "Need elements" };
        let status_style = MonoTextStyle::new(&FONT_9X15, status_color);
        Text::new(status_text, Point::new(200, 60), status_style).draw(display)?;

        // Monster list
        self.monster_areas.clear();
        let list_y = 75;
        let item_height = 40u32;

        for (i, monster) in self.monsters.iter().take(6).enumerate() {
            let y = list_y + (i as i32 * (item_height as i32 + 3));
            let rect = Rectangle::new(Point::new(15, y), Size::new(338, item_height));

            let bg_color = if monster.is_selected {
                Rgb888::new(50, 80, 50)
            } else if monster.is_available {
                Rgb888::new(35, 40, 50)
            } else {
                Rgb888::new(30, 30, 35)
            };

            display.fill_solid(&rect, bg_color)?;
            self.monster_areas.push(rect);

            // Element indicator
            let elem_style = MonoTextStyle::new(&FONT_9X15, Self::element_color(&monster.element));
            Text::new(&Self::element_char(&monster.element).to_string(), Point::new(25, y + 27), elem_style).draw(display)?;

            // Name and level
            let name_color = if monster.is_available { Rgb888::WHITE } else { Rgb888::new(100, 100, 100) };
            let name_style = MonoTextStyle::new(&FONT_9X15, name_color);
            let info = format!("{} Lv.{}", monster.name, monster.level);
            Text::new(&info, Point::new(50, y + 27), name_style).draw(display)?;

            // Selection indicator
            if monster.is_selected {
                Text::new("[X]", Point::new(300, y + 27), text_style).draw(display)?;
            } else if monster.is_available {
                Text::new("[ ]", Point::new(300, y + 27), dim_style).draw(display)?;
            } else {
                Text::new("---", Point::new(300, y + 27), dim_style).draw(display)?;
            }
        }

        // Duration selection
        self.duration_areas.clear();
        let dur_y = 340;
        Text::new("Duration:", Point::new(15, dur_y), dim_style).draw(display)?;

        let durations = [
            ExpeditionDuration::Short,
            ExpeditionDuration::Medium,
            ExpeditionDuration::Long,
            ExpeditionDuration::Overnight,
        ];

        for (i, duration) in durations.iter().enumerate() {
            let x = 15 + (i as i32 * 85);
            let rect = Rectangle::new(Point::new(x, dur_y + 10), Size::new(80, 25));

            let bg_color = if self.selected_duration == *duration {
                Rgb888::new(60, 100, 60)
            } else {
                Rgb888::new(40, 45, 55)
            };

            display.fill_solid(&rect, bg_color)?;
            self.duration_areas.push(rect);

            let dur_text = Self::duration_text(*duration);
            Text::new(dur_text, Point::new(x + 20, dur_y + 28), text_style).draw(display)?;
        }

        // Back button
        let back_rect = Rectangle::new(Point::new(15, 410), Size::new(80, 30));
        display.fill_solid(&back_rect, Rgb888::new(80, 60, 60))?;
        Text::new("< BACK", Point::new(25, 430), text_style).draw(display)?;
        self.back_area = Some(back_rect);

        // Start button
        let start_color = if self.can_start() {
            Rgb888::new(60, 120, 60)
        } else {
            Rgb888::new(50, 50, 55)
        };
        let start_rect = Rectangle::new(Point::new(250, 410), Size::new(100, 30));
        display.fill_solid(&start_rect, start_color)?;
        Text::new("START", Point::new(275, 430), text_style).draw(display)?;
        self.start_area = Some(start_rect);

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
