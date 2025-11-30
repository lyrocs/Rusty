//! Cards Collection Page
//!
//! Displays all monster cards with owned/not owned status

use crate::display::Sh8601Driver;
use crate::game::{GameData, Card};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10, ascii::FONT_9X15},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, RoundedRectangle, CornerRadii},
    text::Text,
};
use std::error::Error;
use core::fmt::Write;

/// Card entry for display
#[derive(Debug, Clone)]
struct CardEntry {
    monster_id: u32,
    name: String,
    rarity: u8,
    atk_bonus: u32,
    def_bonus: u32,
    owned: bool,
    owned_count: usize,
}

/// Cards Collection Page
pub struct CardsPage {
    background_color: Rgb888,
    card_entries: Vec<CardEntry>,
    scroll_offset: usize,
}

impl CardsPage {
    /// Create a new cards collection page
    pub fn new(game_data: GameData, owned_cards: Vec<Card>) -> Result<Self, Box<dyn Error>> {
        // Get all enemies and their card data
        let mut card_entries = Vec::new();

        // Create a sorted list of enemy IDs
        let mut enemy_ids: Vec<u32> = game_data.enemies.keys().copied().collect();
        enemy_ids.sort();

        for enemy_id in enemy_ids {
            if let Some(enemy_data) = game_data.get_enemy(enemy_id) {
                // Count how many of this card the hero owns
                let owned_count = owned_cards.iter()
                    .filter(|c| c.monster_id == enemy_id)
                    .count();

                card_entries.push(CardEntry {
                    monster_id: enemy_id,
                    name: enemy_data.card.name.clone(),
                    rarity: enemy_data.card.rarity,
                    atk_bonus: enemy_data.card.atk_bonus,
                    def_bonus: enemy_data.card.def_bonus,
                    owned: owned_count > 0,
                    owned_count,
                });
            }
        }

        log::info!("Created card collection with {} total cards, {} owned",
            card_entries.len(),
            card_entries.iter().filter(|e| e.owned).count()
        );

        Ok(Self {
            background_color: Rgb888::new(10, 10, 15),
            card_entries,
            scroll_offset: 0,
        })
    }

    /// Handle touch for scrolling or returning to menu
    pub fn handle_touch(&mut self, _x: i32, y: i32) -> bool {
        // Top quarter: scroll up
        if y < 112 && self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
            false
        }
        // Bottom quarter: scroll down
        else if y > 336 {
            let max_scroll = self.card_entries.len().saturating_sub(4);
            if self.scroll_offset < max_scroll {
                self.scroll_offset += 1;
            }
            false
        }
        // Middle: return to menu
        else if y >= 112 && y <= 336 {
            true
        } else {
            false
        }
    }

    /// Draw a single card entry
    fn draw_card_entry(
        display: &mut Sh8601Driver,
        entry: &CardEntry,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        // Card background color based on rarity
        let rarity_color = if entry.owned {
            match entry.rarity {
                1 => Rgb888::new(100, 100, 100), // Gray - common
                2 => Rgb888::new(70, 140, 70),   // Green - uncommon
                3 => Rgb888::new(70, 100, 180),  // Blue - rare
                4 => Rgb888::new(140, 70, 180),  // Purple - epic
                5 => Rgb888::new(180, 140, 50),  // Gold - legendary
                _ => Rgb888::new(80, 80, 80),
            }
        } else {
            Rgb888::new(40, 40, 40) // Dark gray for not owned
        };

        // Card background
        RoundedRectangle::new(
            Rectangle::new(Point::new(10, y), Size::new(348, 90)),
            CornerRadii::new(Size::new(8, 8)),
        )
        .into_styled(PrimitiveStyle::with_fill(rarity_color))
        .draw(display)?;

        // Card name
        let name_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let mut card_name = heapless::String::<32>::new();
        write!(card_name, "{}", entry.name).ok();
        Text::new(&card_name, Point::new(20, y + 20), name_style).draw(display)?;

        // Rarity stars
        let mut stars = heapless::String::<16>::new();
        for _ in 0..entry.rarity {
            write!(stars, "★").ok();
        }
        for _ in entry.rarity..5 {
            write!(stars, "☆").ok();
        }
        let stars_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(255, 255, 200));
        Text::new(&stars, Point::new(20, y + 40), stars_style).draw(display)?;

        // Bonuses
        let bonus_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
        let mut bonus_text = heapless::String::<32>::new();
        if entry.atk_bonus > 0 && entry.def_bonus > 0 {
            write!(bonus_text, "+{} ATK, +{} DEF", entry.atk_bonus, entry.def_bonus).ok();
        } else if entry.atk_bonus > 0 {
            write!(bonus_text, "+{} ATK", entry.atk_bonus).ok();
        } else if entry.def_bonus > 0 {
            write!(bonus_text, "+{} DEF", entry.def_bonus).ok();
        }
        Text::new(&bonus_text, Point::new(20, y + 55), bonus_style).draw(display)?;

        // Owned status
        let owned_style = if entry.owned {
            MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 255, 150))
        } else {
            MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150))
        };

        let mut owned_text = heapless::String::<16>::new();
        if entry.owned {
            if entry.owned_count > 1 {
                write!(owned_text, "x{}", entry.owned_count).ok();
            } else {
                write!(owned_text, "OWNED").ok();
            }
        } else {
            write!(owned_text, "---").ok();
        }
        let owned_x = 330 - (owned_text.len() as i32 * 5);
        Text::new(&owned_text, Point::new(owned_x, y + 20), owned_style).draw(display)?;

        Ok(())
    }
}

impl Page for CardsPage {
    fn update(&mut self) -> bool {
        true // Keep page active
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Clear background
        display.clear(self.background_color)?;

        // Title
        let title_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 255, 100));
        Text::new("CARD COLLECTION", Point::new(100, 20), title_style).draw(display)?;

        // Collection stats
        let owned_count = self.card_entries.iter().filter(|e| e.owned).count();
        let total_count = self.card_entries.len();
        let mut stats = heapless::String::<32>::new();
        write!(stats, "{}/{} Collected", owned_count, total_count).ok();
        let stats_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
        Text::new(&stats, Point::new(120, 35), stats_style).draw(display)?;

        // Draw visible cards (4 at a time)
        let start_y = 50;
        let card_height = 95;
        let cards_to_show = 4;

        for i in 0..cards_to_show {
            let index = self.scroll_offset + i;
            if index < self.card_entries.len() {
                let card_y = start_y + (i as i32 * card_height);
                Self::draw_card_entry(display, &self.card_entries[index], card_y)?;
            }
        }

        // Scroll indicator
        if self.card_entries.len() > cards_to_show {
            let scroll_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));

            // Up arrow
            if self.scroll_offset > 0 {
                Text::new("▲", Point::new(180, 440), scroll_style).draw(display)?;
            }

            // Down arrow
            let max_scroll = self.card_entries.len().saturating_sub(cards_to_show);
            if self.scroll_offset < max_scroll {
                Text::new("▼", Point::new(180, 460), scroll_style).draw(display)?;
            }
        }

        // Footer hint
        let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));
        Text::new("[TAP CENTER] Back to Menu", Point::new(80, 455), hint_style).draw(display)?;

        display.flush()?;
        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No-op
    }

    fn needs_full_redraw(&self) -> bool {
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
