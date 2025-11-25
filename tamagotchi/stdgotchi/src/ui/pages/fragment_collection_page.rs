//! Fragment Collection Page
//!
//! Displays monster fragments and allows summoning or evolving when enough are collected

use crate::display::Sh8601Driver;
use crate::game::{EnemyData, FragmentCollection, GameData, Rustymon};
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

/// Actions from fragment collection page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentCollectionAction {
    Summon(u32), // Summon monster with this ID
    ScrollUp,
    ScrollDown,
    Close,
}

/// Touch area
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: FragmentCollectionAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Fragment Collection page
pub struct FragmentCollectionPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
    scroll_offset: usize,
}

impl FragmentCollectionPage {
    const ITEMS_PER_PAGE: usize = 5;

    /// Create new fragment collection page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
            scroll_offset: 0,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<FragmentCollectionAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Fragment collection action: {:?}", area.action);
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

    /// Draw fragment collection screen
    pub fn draw_fragment_collection(
        &mut self,
        display: &mut Sh8601Driver,
        fragment_collection: &FragmentCollection,
        rustymon_collection: &[Rustymon],
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
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 200));
        Text::new("Fragments", Point::new(10, 20), title_style).draw(display)?;

        // Draw total count
        let count_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));
        let mut count_str = heapless::String::<32>::new();
        write!(
            count_str,
            "{} types",
            fragment_collection.get_unique_monster_count()
        )
        .ok();
        Text::new(&count_str, Point::new(250, 20), count_style).draw(display)?;

        // Get list of monsters with fragments
        let fragment_list = fragment_collection.get_fragment_list();

        if fragment_list.is_empty() {
            // Draw empty state
            let empty_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));
            Text::new(
                "No fragments yet!",
                Point::new(80, 200),
                empty_style,
            )
            .draw(display)?;
            Text::new(
                "Defeat monsters",
                Point::new(60, 230),
                empty_style,
            )
            .draw(display)?;
            Text::new(
                "in battle to",
                Point::new(80, 250),
                empty_style,
            )
            .draw(display)?;
            Text::new(
                "collect fragments!",
                Point::new(60, 270),
                empty_style,
            )
            .draw(display)?;
        } else {
            // Draw list items
            let start_y = 50;
            let item_height = 70;
            let visible_end = (self.scroll_offset + Self::ITEMS_PER_PAGE).min(fragment_list.len());

            for (list_idx, (monster_id, fragment_count)) in fragment_list
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .take(Self::ITEMS_PER_PAGE)
            {
                let y = start_y + ((list_idx - self.scroll_offset) as i32 * item_height as i32);

                // Get monster data
                let enemy_data = game_data.get_enemy(*monster_id);
                if enemy_data.is_none() {
                    continue;
                }
                let enemy_data = enemy_data.unwrap();

                // Draw item background
                let bg_color = if (list_idx - self.scroll_offset) % 2 == 0 {
                    Rgb888::new(20, 25, 35)
                } else {
                    Rgb888::new(25, 30, 40)
                };

                Rectangle::new(Point::new(10, y), Size::new(348, item_height as u32))
                    .into_styled(PrimitiveStyle::with_fill(bg_color))
                    .draw(display)?;

                // Check if player already has this species (for evolution vs summon)
                let existing_rustymon = rustymon_collection.iter()
                    .find(|r| r.species_id == *monster_id);

                // Calculate fragment requirement (base for summon, Fibonacci for evolution)
                let (required_fragments, button_text, is_evolution) = if let Some(existing) = existing_rustymon {
                    // Evolution: Calculate next evolution requirement
                    let next_evolution = existing.evolution_level + 1;
                    let required = crate::game::fragment_collection::calculate_evolution_fragments(
                        enemy_data.fragments_required,
                        next_evolution
                    );
                    let mut text = heapless::String::<16>::new();
                    write!(text, "Evo +{}", next_evolution).ok();
                    (required, text, true)
                } else {
                    // Summon: Use base fragment requirement
                    let mut text = heapless::String::<16>::new();
                    write!(text, "Summon").ok();
                    (enemy_data.fragments_required, text, false)
                };

                // Check if can summon/evolve
                let can_action = *fragment_count >= required_fragments;

                // Draw element indicator
                let element = enemy_data.get_element();
                let element_color = get_element_color(element);
                Rectangle::new(Point::new(20, y + 10), Size::new(30, 50))
                    .into_styled(PrimitiveStyle::with_fill(element_color))
                    .draw(display)?;

                // Draw monster name with evolution level if owned
                let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
                let mut name_str = heapless::String::<24>::new();
                if let Some(existing) = existing_rustymon {
                    // Show evolution level for owned monsters
                    if enemy_data.name.len() > 8 {
                        write!(name_str, "{}..+{}", &enemy_data.name[..7], existing.evolution_level).ok();
                    } else {
                        write!(name_str, "{} +{}", enemy_data.name, existing.evolution_level).ok();
                    }
                } else {
                    // Regular name for unowned
                    if enemy_data.name.len() > 12 {
                        write!(name_str, "{}...", &enemy_data.name[..9]).ok();
                    } else {
                        write!(name_str, "{}", enemy_data.name).ok();
                    }
                }
                Text::new(&name_str, Point::new(60, y + 20), name_style).draw(display)?;

                // Draw element name
                let elem_style = MonoTextStyle::new(&FONT_10X20, element_color);
                let elem_str = element.as_str();
                Text::new(elem_str, Point::new(60, y + 40), elem_style).draw(display)?;

                // Draw fragment count
                let count_style = if can_action {
                    MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 255, 100))
                } else {
                    MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 100))
                };

                let mut frag_str = heapless::String::<24>::new();
                write!(
                    frag_str,
                    "{}/{}",
                    fragment_count,
                    required_fragments
                )
                .ok();
                Text::new(&frag_str, Point::new(200, y + 30), count_style).draw(display)?;

                // Draw progress bar
                self.draw_progress_bar(
                    display,
                    (60, y + 50),
                    *fragment_count,
                    required_fragments,
                    200,
                    can_action,
                )?;

                // Draw summon/evolve button if ready
                if can_action {
                    let button_color = if is_evolution {
                        Rgb888::new(40, 80, 120) // Blue for evolution
                    } else {
                        Rgb888::new(80, 40, 120) // Purple for summon
                    };

                    Rectangle::new(Point::new(280, y + 20), Size::new(70, 30))
                        .into_styled(
                            PrimitiveStyleBuilder::new()
                                .fill_color(button_color)
                                .stroke_color(Rgb888::new(160, 200, 255))
                                .stroke_width(2)
                                .build(),
                        )
                        .draw(display)?;

                    let btn_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 100));
                    Text::new(&button_text, Point::new(285, y + 40), btn_style).draw(display)?;

                    self.touch_areas.push(TouchArea {
                        bounds: (280, y + 20, 70, 30),
                        action: FragmentCollectionAction::Summon(*monster_id),
                    });
                }
            }

            // Draw scroll indicators
            if self.scroll_offset > 0 {
                let arrow_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 255));
                Text::new("▲", Point::new(330, 40), arrow_style).draw(display)?;

                self.touch_areas.push(TouchArea {
                    bounds: (320, 30, 40, 20),
                    action: FragmentCollectionAction::ScrollUp,
                });
            }

            if visible_end < fragment_list.len() {
                let arrow_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 255));
                Text::new("▼", Point::new(330, 420), arrow_style).draw(display)?;

                self.touch_areas.push(TouchArea {
                    bounds: (320, 410, 40, 20),
                    action: FragmentCollectionAction::ScrollDown,
                });
            }
        }

        // Draw "Back" button
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
            action: FragmentCollectionAction::Close,
        });

        display.flush()?;
        Ok(())
    }

    /// Draw progress bar for fragment collection
    fn draw_progress_bar(
        &self,
        display: &mut Sh8601Driver,
        position: (i32, i32),
        current: u32,
        required: u32,
        width: u32,
        can_summon: bool,
    ) -> Result<(), Box<dyn Error>> {
        let (x, y) = position;
        let height = 10;

        // Background
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
            .draw(display)?;

        // Fill
        let fill_width = if required > 0 {
            ((current.min(required) as f32 / required as f32) * width as f32) as u32
        } else {
            width
        };

        let fill_color = if can_summon {
            Rgb888::new(100, 255, 100) // Green when ready
        } else {
            Rgb888::new(200, 200, 100) // Yellow while collecting
        };

        Rectangle::new(Point::new(x, y), Size::new(fill_width, height))
            .into_styled(PrimitiveStyle::with_fill(fill_color))
            .draw(display)?;

        // Border
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
            .draw(display)?;

        Ok(())
    }
}

impl Default for FragmentCollectionPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for FragmentCollectionPage {
    fn update(&mut self) -> bool {
        true // Stay active until explicitly closed
    }

    fn draw(
        &mut self,
        _display: &mut Sh8601Driver,
        _full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // This page requires external data
        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering fragment collection page");
        self.needs_full_redraw = true;
        self.scroll_offset = 0;
    }

    fn on_exit(&mut self) {
        log::info!("Exiting fragment collection page");
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
