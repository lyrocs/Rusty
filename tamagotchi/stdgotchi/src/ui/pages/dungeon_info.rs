//! Dungeon Info Page
//!
//! Shows detailed information about a specific dungeon:
//! - Description
//! - Monsters present (from enemy pools)
//! - Level range
//! - Crystal/XP rewards
//! - Boss information
//! - Start button

use crate::display::St7789pDriver;
use crate::game::core::{Element, Dungeon};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action from dungeon info page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DungeonInfoAction {
    /// No action
    None,
    /// Go back to dungeon list
    Back,
    /// Start dungeon from checkpoint
    StartDungeon { checkpoint: u16 },
}

/// Monster info for display
#[derive(Clone)]
pub struct MonsterDisplayInfo {
    pub name: String,
    pub element: Element,
    pub is_boss: bool,
}

/// Dungeon info page
pub struct DungeonInfoPage {
    dungeon_id: String,
    dungeon_name: String,
    description: String,
    elements: Vec<Element>,
    monsters: Vec<MonsterDisplayInfo>,
    level_min: u8,
    level_max: u8,
    crystal_reward: u32,
    xp_reward: u32,
    boss_names: Vec<String>,
    available_checkpoints: Vec<u16>,
    selected_checkpoint: usize,
    highest_floor: u16,

    // Touch areas
    back_area: Option<Rectangle>,
    start_button: Option<Rectangle>,
    checkpoint_left: Option<Rectangle>,
    checkpoint_right: Option<Rectangle>,

    scroll_offset: i32,
    dirty: bool,
}

impl DungeonInfoPage {
    pub fn new(
        dungeon: &Dungeon,
        monsters: Vec<MonsterDisplayInfo>,
        boss_names: Vec<String>,
        level_min: u8,
        level_max: u8,
        highest_floor: u16,
    ) -> Self {
        let available_checkpoints = dungeon.available_checkpoints(highest_floor);

        Self {
            dungeon_id: dungeon.id.clone(),
            dungeon_name: dungeon.name.clone(),
            description: dungeon.description.clone(),
            elements: dungeon.dominant_elements.clone(),
            monsters,
            level_min,
            level_max,
            crystal_reward: dungeon.base_crystal_reward,
            xp_reward: dungeon.base_xp_reward,
            boss_names,
            available_checkpoints,
            selected_checkpoint: 0,
            highest_floor,
            back_area: None,
            start_button: None,
            checkpoint_left: None,
            checkpoint_right: None,
            scroll_offset: 0,
            dirty: true,
        }
    }

    pub fn dungeon_id(&self) -> &str {
        &self.dungeon_id
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> DungeonInfoAction {
        let point = Point::new(x, y);

        // Check back button
        if let Some(ref rect) = self.back_area {
            if rect.contains(point) {
                return DungeonInfoAction::Back;
            }
        }

        // Check start button
        if let Some(ref rect) = self.start_button {
            if rect.contains(point) {
                let checkpoint = self.available_checkpoints.get(self.selected_checkpoint)
                    .copied()
                    .unwrap_or(1);
                return DungeonInfoAction::StartDungeon { checkpoint };
            }
        }

        // Check checkpoint arrows
        if let Some(ref rect) = self.checkpoint_left {
            if rect.contains(point) && self.selected_checkpoint > 0 {
                self.selected_checkpoint -= 1;
                self.dirty = true;
            }
        }

        if let Some(ref rect) = self.checkpoint_right {
            if rect.contains(point) && self.selected_checkpoint < self.available_checkpoints.len() - 1 {
                self.selected_checkpoint += 1;
                self.dirty = true;
            }
        }

        DungeonInfoAction::None
    }

    /// Handle swipe for scrolling monster list
    pub fn handle_swipe(&mut self, up: bool) {
        let max_scroll = (self.monsters.len() as i32 - 3).max(0) * 18;
        if up {
            self.scroll_offset = (self.scroll_offset + 36).min(max_scroll);
        } else {
            self.scroll_offset = (self.scroll_offset - 36).max(0);
        }
        self.dirty = true;
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
}

impl Page for DungeonInfoPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let label_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(80, 80, 80));

        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        // Header with dungeon name
        let header_rect = Rectangle::new(Point::new(0, 0), Size::new(240, 32));
        display.fill_solid(&header_rect, Rgb888::new(180, 80, 60))?;

        // Truncate name if too long
        let name = if self.dungeon_name.len() > 20 {
            &self.dungeon_name[..20]
        } else {
            &self.dungeon_name
        };
        let name_x = (240 - name.len() as i32 * 7) / 2;
        Text::new(name, Point::new(name_x, 20), title_style).draw(display)?;

        // Back button area
        self.back_area = Some(Rectangle::new(Point::new(0, 0), Size::new(50, 32)));
        Text::new("<", Point::new(10, 20), title_style).draw(display)?;

        let mut y = 40;

        // Description section
        let desc_rect = Rectangle::new(Point::new(8, y), Size::new(224, 40));
        let rounded = RoundedRectangle::new(desc_rect, CornerRadii::new(Size::new(6, 6)));
        rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(250, 250, 255))
            .build())
            .draw(display)?;

        // Wrap description text
        let desc = if self.description.len() > 36 {
            format!("{}...", &self.description[..36])
        } else {
            self.description.clone()
        };
        Text::new(&desc, Point::new(14, y + 15), dim_style).draw(display)?;

        // Elements display
        let mut elem_x = 14;
        for elem in &self.elements {
            let elem_style = MonoTextStyle::new(&FONT_6X10, Self::element_color(elem));
            Text::new(&Self::element_char(elem).to_string(), Point::new(elem_x, y + 32), elem_style).draw(display)?;
            elem_x += 12;
        }

        y += 48;

        // Stats row: Level range, Crystals, XP
        Text::new("Level:", Point::new(12, y + 12), label_style).draw(display)?;
        Text::new(&format!("{}-{}", self.level_min, self.level_max), Point::new(52, y + 12), text_style).draw(display)?;

        Text::new("Crystals:", Point::new(90, y + 12), label_style).draw(display)?;
        Text::new(&format!("{}", self.crystal_reward), Point::new(145, y + 12), text_style).draw(display)?;

        Text::new("XP:", Point::new(175, y + 12), label_style).draw(display)?;
        Text::new(&format!("{}", self.xp_reward), Point::new(195, y + 12), text_style).draw(display)?;

        y += 20;

        // Progress info
        if self.highest_floor > 0 {
            Text::new(&format!("Record: Floor {}", self.highest_floor), Point::new(12, y + 12), dim_style).draw(display)?;
        } else {
            Text::new("Not yet explored", Point::new(12, y + 12), dim_style).draw(display)?;
        }

        y += 20;

        // Monsters section
        let _monsters_header_y = y;
        Text::new("MONSTERS", Point::new(12, y + 12), label_style).draw(display)?;
        y += 18;

        // Monster list area with clipping
        let monster_list_height = 54;
        let monster_area = Rectangle::new(Point::new(8, y), Size::new(224, monster_list_height as u32));
        display.fill_solid(&monster_area, Rgb888::new(245, 245, 250))?;

        let mut monster_y = y + 14 - self.scroll_offset;
        for monster in &self.monsters {
            if monster_y >= y && monster_y < y + monster_list_height {
                // Element indicator
                let elem_style = MonoTextStyle::new(&FONT_6X10, Self::element_color(&monster.element));
                Text::new(&Self::element_char(&monster.element).to_string(), Point::new(14, monster_y), elem_style).draw(display)?;

                // Monster name
                let name_style = if monster.is_boss {
                    MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 50, 50))
                } else {
                    text_style
                };
                let name = if monster.name.len() > 18 { &monster.name[..18] } else { &monster.name };
                Text::new(name, Point::new(28, monster_y), name_style).draw(display)?;

                if monster.is_boss {
                    Text::new("BOSS", Point::new(150, monster_y), MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 50, 50))).draw(display)?;
                }
            }
            monster_y += 18;
        }

        // Scroll indicator if needed
        if self.monsters.len() > 3 {
            let scroll_ratio = self.scroll_offset as f32 / ((self.monsters.len() as i32 - 3).max(1) * 18) as f32;
            let indicator_y = y + (scroll_ratio * (monster_list_height - 20) as f32) as i32;
            let indicator = Rectangle::new(Point::new(228, indicator_y), Size::new(3, 20));
            display.fill_solid(&indicator, Rgb888::new(180, 180, 185))?;
        }

        y += monster_list_height + 8;

        // Bosses section (if any)
        if !self.boss_names.is_empty() {
            Text::new("BOSSES:", Point::new(12, y + 12), label_style).draw(display)?;
            let boss_text: String = self.boss_names.iter()
                .take(3)
                .map(|n| if n.len() > 10 { &n[..10] } else { n })
                .collect::<Vec<&str>>()
                .join(", ");
            Text::new(&boss_text, Point::new(60, y + 12), MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 50, 50))).draw(display)?;
            y += 18;
        }

        // Checkpoint selector
        y += 4;
        let _checkpoint_y = y;
        Text::new("Start from:", Point::new(12, y + 12), label_style).draw(display)?;

        // Left arrow
        self.checkpoint_left = Some(Rectangle::new(Point::new(80, y), Size::new(20, 20)));
        if self.selected_checkpoint > 0 {
            Text::new("<", Point::new(86, y + 14), text_style).draw(display)?;
        }

        // Current checkpoint
        let checkpoint_floor = self.available_checkpoints.get(self.selected_checkpoint)
            .copied()
            .unwrap_or(1);
        let checkpoint_text = format!("F{}", checkpoint_floor);
        Text::new(&checkpoint_text, Point::new(106, y + 14), text_style).draw(display)?;

        // Right arrow
        self.checkpoint_right = Some(Rectangle::new(Point::new(140, y), Size::new(20, 20)));
        if self.selected_checkpoint < self.available_checkpoints.len() - 1 {
            Text::new(">", Point::new(146, y + 14), text_style).draw(display)?;
        }

        y += 28;

        // Start button
        let button_rect = Rectangle::new(Point::new(40, y), Size::new(160, 36));
        let button_rounded = RoundedRectangle::new(button_rect, CornerRadii::new(Size::new(8, 8)));
        button_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(80, 180, 80))
            .build())
            .draw(display)?;
        button_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(60, 140, 60))
            .stroke_width(2)
            .build())
            .draw(display)?;

        Text::new("START", Point::new(95, y + 22), MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE)).draw(display)?;
        self.start_button = Some(button_rect);

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
