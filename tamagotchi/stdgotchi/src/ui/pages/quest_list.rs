//! Quest List Page
//!
//! Displays active quests and their progress

use crate::display::Sh8601Driver;
use crate::game::{ActiveQuest, GameData, QuestData, QuestManager, QuestStatus};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use std::error::Error;

/// Actions from quest list page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestListAction {
    ClaimReward(u32), // Claim reward for quest ID
    ScrollUp,
    ScrollDown,
    Close,
}

/// Touch area
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: QuestListAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Quest List page
pub struct QuestListPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
    scroll_offset: usize,
}

impl QuestListPage {
    const ITEMS_PER_PAGE: usize = 4;

    /// Create new quest list page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
            scroll_offset: 0,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<QuestListAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Quest list action: {:?}", area.action);
                return Some(area.action);
            }
        }
        None
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.needs_full_redraw = true;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self, total_items: usize) {
        if self.scroll_offset + Self::ITEMS_PER_PAGE < total_items {
            self.scroll_offset += 1;
            self.needs_full_redraw = true;
        }
    }

    /// Draw quest list screen
    pub fn draw_quest_list(
        &mut self,
        display: &mut Sh8601Driver,
        quest_manager: &QuestManager,
        game_data: &GameData,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.touch_areas.clear();

        // Draw title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0)); // Gold
        Text::new("Daily Quests", Point::new(10, 25), title_style).draw(display)?;

        // Get active quests
        let active_quests: Vec<(&u32, &ActiveQuest)> = quest_manager
            .active_quests
            .iter()
            .filter(|(quest_id, _aq)| {
                if let Some(data) = game_data.get_quest(**quest_id) {
                    data.is_daily()
                } else {
                    false
                }
            })
            .collect();

        // Draw count
        let count_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));
        let mut count_str = heapless::String::<32>::new();
        write!(count_str, "{} active", active_quests.len()).ok();
        Text::new(&count_str, Point::new(250, 25), count_style).draw(display)?;

        if active_quests.is_empty() {
            // Draw empty state
            let empty_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));
            Text::new("No active quests!", Point::new(80, 200), empty_style).draw(display)?;
            Text::new("Quests reset daily", Point::new(60, 230), empty_style).draw(display)?;
        } else {
            // Draw quest entries
            let start_y = 50;
            let item_height = 95;

            let visible_quests: Vec<_> = active_quests
                .iter()
                .skip(self.scroll_offset)
                .take(Self::ITEMS_PER_PAGE)
                .collect();

            for (i, (quest_id, active_quest)) in visible_quests.iter().enumerate() {
                let y_pos = start_y + (i as i32 * item_height);

                if let Some(quest_data) = game_data.get_quest(**quest_id) {
                    self.draw_quest_entry(display, **quest_id, active_quest, quest_data, y_pos)?;
                }
            }

            // Draw scroll indicators if needed
            if self.scroll_offset > 0 {
                let up_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 200, 100));
                Text::new("^ UP", Point::new(160, 45), up_style).draw(display)?;

                // Add touch area for scroll up
                self.touch_areas.push(TouchArea {
                    bounds: (130, 30, 100, 30),
                    action: QuestListAction::ScrollUp,
                });
            }

            if self.scroll_offset + Self::ITEMS_PER_PAGE < active_quests.len() {
                let down_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 200, 100));
                Text::new("v DOWN", Point::new(150, 435), down_style).draw(display)?;

                // Add touch area for scroll down
                self.touch_areas.push(TouchArea {
                    bounds: (130, 420, 100, 30),
                    action: QuestListAction::ScrollDown,
                });
            }
        }

        // Draw close button
        let close_bounds = (280, 5, 80, 30);
        let close_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(100, 50, 50))
            .stroke_color(Rgb888::new(150, 100, 100))
            .stroke_width(1)
            .build();

        Rectangle::new(
            Point::new(close_bounds.0, close_bounds.1),
            Size::new(close_bounds.2, close_bounds.3),
        )
        .into_styled(close_style)
        .draw(display)?;

        let close_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new(
            "CLOSE",
            Point::new(close_bounds.0 + 8, close_bounds.1 + 20),
            close_text_style,
        )
        .draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: close_bounds,
            action: QuestListAction::Close,
        });

        display.flush()?;
        Ok(())
    }

    /// Draw a single quest entry
    fn draw_quest_entry(
        &mut self,
        display: &mut Sh8601Driver,
        quest_id: u32,
        active_quest: &ActiveQuest,
        quest_data: &QuestData,
        y_pos: i32,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Background for quest entry
        let bg_color = match active_quest.status {
            QuestStatus::Completed => Rgb888::new(30, 50, 30), // Green tint
            QuestStatus::InProgress => Rgb888::new(25, 30, 40),
            _ => Rgb888::new(20, 25, 35),
        };

        Rectangle::new(Point::new(10, y_pos), Size::new(348, 85))
            .into_styled(PrimitiveStyle::with_fill(bg_color))
            .draw(display)?;

        // Quest name
        let name_color = match active_quest.status {
            QuestStatus::Completed => Rgb888::new(150, 255, 150), // Bright green
            _ => Rgb888::new(220, 220, 220),
        };
        let name_style = MonoTextStyle::new(&FONT_10X20, name_color);
        Text::new(&quest_data.name, Point::new(15, y_pos + 20), name_style).draw(display)?;

        // Quest description (truncated)
        let desc_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));
        let desc = if quest_data.description.len() > 32 {
            &quest_data.description[..32]
        } else {
            &quest_data.description
        };
        Text::new(desc, Point::new(15, y_pos + 42), desc_style).draw(display)?;

        // Progress bar
        let bar_x = 15;
        let bar_y = y_pos + 52;
        let bar_width = 250u32;
        let bar_height = 16u32;

        // Background bar
        Rectangle::new(
            Point::new(bar_x, bar_y),
            Size::new(bar_width, bar_height),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 50, 50)))
        .draw(display)?;

        // Progress fill
        if let Some(progress) = active_quest.progress.first() {
            let fill_width =
                ((progress.current as f32 / progress.target as f32) * bar_width as f32) as u32;
            let fill_color = if progress.is_complete() {
                Rgb888::new(100, 200, 100) // Green
            } else {
                Rgb888::new(100, 150, 200) // Blue
            };

            if fill_width > 0 {
                Rectangle::new(
                    Point::new(bar_x, bar_y),
                    Size::new(fill_width.min(bar_width), bar_height),
                )
                .into_styled(PrimitiveStyle::with_fill(fill_color))
                .draw(display)?;
            }

            // Progress text
            let progress_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            let mut progress_str = heapless::String::<32>::new();
            write!(progress_str, "{}/{}", progress.current, progress.target).ok();
            Text::new(&progress_str, Point::new(bar_x + 90, bar_y + 13), progress_style)
                .draw(display)?;
        }

        // Reward info
        let reward_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0)); // Gold
        let mut reward_str = heapless::String::<32>::new();
        write!(reward_str, "+{} EXP", quest_data.rewards.exp).ok();
        Text::new(&reward_str, Point::new(15, y_pos + 80), reward_style).draw(display)?;

        // Claim button (if completed)
        if active_quest.status == QuestStatus::Completed {
            let btn_bounds = (275, y_pos + 50, 75, 30);
            let btn_style = PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(50, 150, 50))
                .stroke_color(Rgb888::new(100, 200, 100))
                .stroke_width(2)
                .build();

            Rectangle::new(
                Point::new(btn_bounds.0, btn_bounds.1),
                Size::new(btn_bounds.2, btn_bounds.3),
            )
            .into_styled(btn_style)
            .draw(display)?;

            let btn_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            Text::new(
                "CLAIM",
                Point::new(btn_bounds.0 + 8, btn_bounds.1 + 20),
                btn_text_style,
            )
            .draw(display)?;

            self.touch_areas.push(TouchArea {
                bounds: (btn_bounds.0, btn_bounds.1, btn_bounds.2, btn_bounds.3 as u32),
                action: QuestListAction::ClaimReward(quest_id),
            });
        }

        Ok(())
    }

    /// Mark for full redraw
    pub fn mark_redraw(&mut self) {
        self.needs_full_redraw = true;
    }
}

impl Page for QuestListPage {
    fn update(&mut self) -> bool {
        true // Keep page active
    }

    fn draw(&mut self, _display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Drawing is handled by draw_quest_list with game data
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

impl Default for QuestListPage {
    fn default() -> Self {
        Self::new()
    }
}
