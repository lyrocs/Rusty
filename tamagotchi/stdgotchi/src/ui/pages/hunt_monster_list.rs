//! Hunt Monster List Page
//!
//! Page for selecting a monster to hunt from the current map.

use crate::display::Sh8601Driver;
use crate::game::{GameData, Hero};
use crate::game::data_loader::{EnemyData, MonsterType};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, RoundedRectangle, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action triggered by the hunt page
#[derive(Debug, Clone, PartialEq)]
pub enum HuntAction {
    /// Fight selected monster
    Fight(u32), // enemy_id
    /// Exit back to map
    Exit,
}

/// Monster info for display
#[derive(Debug, Clone)]
struct MonsterInfo {
    enemy_data: EnemyData,
    win_rate: f32,
    cards_owned: u32,
}

/// Hunt monster list page
pub struct HuntMonsterListPage {
    hero: Hero,
    game_data: GameData,
    map_id: u32,
    monsters: Vec<MonsterInfo>,
    selected_index: usize,
    scroll_offset: usize,
    pending_action: Option<HuntAction>,
    needs_redraw: bool,
    first_draw: bool,
}

impl HuntMonsterListPage {
    /// Create a new hunt monster list page
    pub fn new(hero: Hero, game_data: GameData, map_id: u32) -> Self {
        // Get map data to find monsters
        let mut monsters = Vec::new();

        if let Some(map_data) = game_data.get_map(map_id) {
            // Add regular monsters from map
            for &enemy_id in &map_data.enemies {
                if let Some(enemy_data) = game_data.get_enemy(enemy_id) {
                    let win_rate = Self::calculate_win_rate(&hero, enemy_data);
                    let cards_owned = hero.cards.iter()
                        .filter(|c| c.monster_id == enemy_id)
                        .count() as u32;

                    monsters.push(MonsterInfo {
                        enemy_data: enemy_data.clone(),
                        win_rate,
                        cards_owned,
                    });
                }
            }
        }

        // Add MVPs that spawn on this map
        for enemy_data in game_data.get_mvp_enemies() {
            if enemy_data.spawn_map_id == Some(map_id) {
                let win_rate = Self::calculate_win_rate(&hero, &enemy_data);
                let cards_owned = hero.cards.iter()
                    .filter(|c| c.monster_id == enemy_data.id)
                    .count() as u32;

                monsters.push(MonsterInfo {
                    enemy_data: enemy_data.clone(),
                    win_rate,
                    cards_owned,
                });
            }
        }

        log::info!("Hunt page: {} monsters available on map {}", monsters.len(), map_id);

        Self {
            hero,
            game_data,
            map_id,
            monsters,
            selected_index: 0,
            scroll_offset: 0,
            pending_action: None,
            needs_redraw: true,
            first_draw: true,
        }
    }

    /// Calculate estimated win rate (0.0 - 1.0)
    fn calculate_win_rate(hero: &Hero, enemy: &EnemyData) -> f32 {
        // Simple win rate calculation based on level difference and stats
        let level_diff = hero.level as i32 - enemy.level as i32;
        let level_factor = (level_diff as f32 * 0.1 + 0.5).clamp(0.1, 0.95);

        // Factor in attack vs defense
        let damage_ratio = if enemy.defense > 0 {
            hero.attack as f32 / (enemy.defense as f32 + 1.0)
        } else {
            2.0
        };
        let damage_factor = (damage_ratio * 0.3).clamp(0.1, 0.5);

        // Factor in HP ratio
        let hp_ratio = hero.max_health as f32 / enemy.hp as f32;
        let hp_factor = (hp_ratio * 0.2).clamp(0.05, 0.3);

        (level_factor + damage_factor + hp_factor).clamp(0.01, 0.99)
    }

    /// Handle tap
    pub fn handle_tap(&mut self, x: i32, y: i32) {
        log::info!("Hunt page tap at ({}, {})", x, y);

        // Check back button (top left)
        if x < 60 && y < 40 {
            log::info!("Back button tapped");
            self.pending_action = Some(HuntAction::Exit);
            return;
        }

        // Check monster items and fight buttons
        let item_height = 75;
        let start_y = 50;
        let visible_count = 5;

        for display_idx in 0..visible_count {
            let monster_idx = self.scroll_offset + display_idx;
            if monster_idx >= self.monsters.len() {
                break;
            }

            let item_y = start_y + (display_idx as i32 * item_height);

            // Check if tap is within this row
            if y >= item_y && y < item_y + item_height - 5 {
                // Check if Fight button (right side) was tapped
                if x >= 290 && x <= 355 {
                    let enemy_id = self.monsters[monster_idx].enemy_data.id;
                    log::info!("Fight button tapped for {}", self.monsters[monster_idx].enemy_data.name);
                    self.pending_action = Some(HuntAction::Fight(enemy_id));
                    return;
                }

                // Otherwise select this monster
                self.selected_index = monster_idx;
                self.needs_redraw = true;
            }
        }
    }

    /// Handle swipe up (scroll down list)
    pub fn handle_swipe_up(&mut self) {
        if self.scroll_offset + 5 < self.monsters.len() {
            self.scroll_offset += 1;
            self.needs_redraw = true;
        }
    }

    /// Handle swipe down (scroll up list)
    pub fn handle_swipe_down(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.needs_redraw = true;
        }
    }

    /// Handle swipe left (exit)
    pub fn handle_swipe_left(&mut self) {
        self.pending_action = Some(HuntAction::Exit);
    }

    /// Take pending action
    pub fn take_action(&mut self) -> Option<HuntAction> {
        self.pending_action.take()
    }

    /// Get monster type color
    fn get_type_color(monster_type: MonsterType) -> Rgb888 {
        match monster_type {
            MonsterType::Normal => Rgb888::new(100, 100, 100),
            MonsterType::MiniMvp => Rgb888::new(180, 120, 50),
            MonsterType::Mvp => Rgb888::new(200, 50, 50),
        }
    }

    /// Get win rate color
    fn get_win_rate_color(rate: f32) -> Rgb888 {
        if rate >= 0.7 {
            Rgb888::new(50, 200, 50)  // Green - easy
        } else if rate >= 0.4 {
            Rgb888::new(200, 200, 50) // Yellow - medium
        } else {
            Rgb888::new(200, 50, 50)  // Red - hard
        }
    }
}

impl Page for HuntMonsterListPage {
    fn update(&mut self) -> bool {
        // Page stays open until action is taken
        self.pending_action.is_none()
    }

    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.first_draw || self.needs_redraw {
            // Clear screen
            Rectangle::new(Point::zero(), Size::new(480, 480))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 25, 35)))
                .draw(display)?;

            use core::fmt::Write;
            let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));

            // Back button
            RoundedRectangle::new(
                Rectangle::new(Point::new(5, 5), Size::new(50, 30)),
                CornerRadii::new(Size::new(5, 5)),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 80, 80)))
            .draw(display)?;
            Text::new("< Back", Point::new(10, 25), small_style).draw(display)?;

            // Title
            Text::new("HUNT", Point::new(150, 30), title_style).draw(display)?;

            if self.monsters.is_empty() {
                Text::new("No monsters on this map", Point::new(80, 200), title_style).draw(display)?;
            } else {
                // Draw monster list
                let item_height = 75;
                let start_y = 50;
                let visible_count = 5;

                for display_idx in 0..visible_count {
                    let monster_idx = self.scroll_offset + display_idx;
                    if monster_idx >= self.monsters.len() {
                        break;
                    }

                    let info = &self.monsters[monster_idx];
                    let y = start_y + (display_idx as i32 * item_height);
                    let is_selected = monster_idx == self.selected_index;

                    // Background
                    let bg_color = if is_selected {
                        Rgb888::new(50, 50, 70)
                    } else {
                        Rgb888::new(35, 38, 48)
                    };

                    RoundedRectangle::new(
                        Rectangle::new(Point::new(5, y), Size::new(358, item_height as u32 - 5)),
                        CornerRadii::new(Size::new(5, 5)),
                    )
                    .into_styled(PrimitiveStyle::with_fill(bg_color))
                    .draw(display)?;

                    // Monster type indicator (left bar)
                    let type_color = Self::get_type_color(info.enemy_data.monster_type);
                    Rectangle::new(Point::new(5, y), Size::new(4, item_height as u32 - 5))
                        .into_styled(PrimitiveStyle::with_fill(type_color))
                        .draw(display)?;

                    // Monster name
                    let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
                    let name_short: String = info.enemy_data.name.chars().take(12).collect();
                    Text::new(&name_short, Point::new(15, y + 20), name_style).draw(display)?;

                    // Level
                    let mut level_text = heapless::String::<16>::new();
                    write!(level_text, "Lv.{}", info.enemy_data.level).ok();
                    Text::new(&level_text, Point::new(15, y + 38), small_style).draw(display)?;

                    // Drop rate
                    let mut drop_text = heapless::String::<16>::new();
                    write!(drop_text, "Drop:{:.2}%", info.enemy_data.drop_rate * 100.0).ok();
                    Text::new(&drop_text, Point::new(15, y + 52), small_style).draw(display)?;

                    // Cards owned
                    let cards_color = if info.cards_owned > 0 {
                        Rgb888::new(100, 200, 100)
                    } else {
                        Rgb888::new(100, 100, 100)
                    };
                    let mut cards_text = heapless::String::<16>::new();
                    write!(cards_text, "Cards:{}", info.cards_owned).ok();
                    Text::new(&cards_text, Point::new(15, y + 66), MonoTextStyle::new(&FONT_6X10, cards_color)).draw(display)?;

                    // Win rate (right side)
                    let win_color = Self::get_win_rate_color(info.win_rate);
                    let mut win_text = heapless::String::<16>::new();
                    write!(win_text, "{:.0}%", info.win_rate * 100.0).ok();
                    Text::new(&win_text, Point::new(200, y + 35), MonoTextStyle::new(&FONT_10X20, win_color)).draw(display)?;
                    Text::new("Win", Point::new(205, y + 50), small_style).draw(display)?;

                    // Fight button
                    RoundedRectangle::new(
                        Rectangle::new(Point::new(290, y + 15), Size::new(65, 40)),
                        CornerRadii::new(Size::new(5, 5)),
                    )
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 50, 50)))
                    .draw(display)?;

                    let btn_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
                    Text::new("Fight", Point::new(297, y + 42), btn_style).draw(display)?;
                }

                // Scroll indicator
                if self.monsters.len() > visible_count {
                    let mut scroll_text = heapless::String::<32>::new();
                    write!(scroll_text, "{}-{} of {}",
                        self.scroll_offset + 1,
                        (self.scroll_offset + visible_count).min(self.monsters.len()),
                        self.monsters.len()
                    ).ok();
                    Text::new(&scroll_text, Point::new(130, 435), small_style).draw(display)?;
                }
            }

            self.first_draw = false;
            self.needs_redraw = false;
        }

        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering hunt monster list page");
    }

    fn on_exit(&mut self) {
        log::info!("Exiting hunt monster list page");
    }

    fn needs_clear(&self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {
        self.needs_redraw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.first_draw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
