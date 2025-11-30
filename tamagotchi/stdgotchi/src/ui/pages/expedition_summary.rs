//! Expedition Summary Page
//!
//! Displays expedition results with loot reveal animation

use crate::display::Sh8601Driver;
use crate::game::{Hero, Card};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10, ascii::FONT_9X15, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, RoundedRectangle, CornerRadii},
    text::Text,
};
use std::error::Error;
use std::time::{Duration, Instant};
use core::fmt::Write;

/// Loot reveal state
#[derive(Debug, Clone, Copy, PartialEq)]
enum LootState {
    Hidden,      // Loot not revealed yet
    Revealing,   // Animation playing
    Revealed,    // Loot shown
}

/// Expedition Summary Page
pub struct ExpeditionSummaryPage {
    background_color: Rgb888,
    hero: Hero,
    initial_exp: u32,
    initial_level: u32,

    // Expedition results
    target_kills: u32,
    actual_kills: u32,
    exp_gained: u32,
    survived: bool,

    // Loot
    cards_dropped: Vec<Card>,
    loot_state: LootState,
    loot_reveal_time: Option<Instant>,

    // Timing
    start_time: Instant,
}

impl ExpeditionSummaryPage {
    /// Create expedition summary for successful completion
    pub fn new_success(
        hero: Hero,
        initial_level: u32,
        kills: u32,
        exp_gained: u32,
        cards: Vec<Card>,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Expedition SUCCESS: {} kills, {} XP, {} cards", kills, exp_gained, cards.len());

        Ok(Self {
            background_color: Rgb888::new(20, 40, 20), // Dark green
            initial_exp: hero.experience,
            initial_level,
            hero,
            target_kills: kills,
            actual_kills: kills,
            exp_gained,
            survived: true,
            cards_dropped: cards,
            loot_state: LootState::Hidden,
            loot_reveal_time: None,
            start_time: Instant::now(),
        })
    }

    /// Create expedition summary for death/failure
    pub fn new_failure(
        hero: Hero,
        initial_level: u32,
        target_kills: u32,
        actual_kills: u32,
        exp_gained: u32,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Expedition FAILED: {}/{} kills, {} XP gained (loot lost)", actual_kills, target_kills, exp_gained);

        Ok(Self {
            background_color: Rgb888::new(40, 20, 20), // Dark red
            initial_exp: hero.experience,
            initial_level,
            hero,
            target_kills,
            actual_kills,
            exp_gained,
            survived: false,
            cards_dropped: Vec::new(), // Loot lost on death
            loot_state: LootState::Revealed, // Skip loot reveal
            loot_reveal_time: None,
            start_time: Instant::now(),
        })
    }

    /// Handle touch input for loot reveal or continue
    pub fn handle_touch(&mut self, _x: i32, _y: i32) -> bool {
        match self.loot_state {
            LootState::Hidden => {
                // Start loot reveal animation
                self.loot_state = LootState::Revealing;
                self.loot_reveal_time = Some(Instant::now());
                false
            }
            LootState::Revealing => {
                // Wait for animation to finish
                false
            }
            LootState::Revealed => {
                // Continue tapped - page is done
                true
            }
        }
    }

    /// Check if user can continue (loot revealed or no loot)
    pub fn can_continue(&self) -> bool {
        self.loot_state == LootState::Revealed
    }

    /// Get updated hero with expedition rewards
    pub fn get_updated_hero(&self) -> Hero {
        self.hero.clone()
    }

    /// Get cards that were collected
    pub fn get_collected_cards(&self) -> Vec<Card> {
        if self.survived {
            self.cards_dropped.clone()
        } else {
            Vec::new() // Lost on death
        }
    }

    /// Draw EXP bar with gained XP
    fn draw_exp_bar(
        display: &mut Sh8601Driver,
        current_exp: u32,
        exp_to_level: u32,
        exp_gained: u32,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);

        // Label
        let mut exp_label = heapless::String::<32>::new();
        write!(exp_label, "XP: +{}", exp_gained).ok();
        Text::new(&exp_label, Point::new(20, y), small_style).draw(display)?;

        // Bar dimensions
        let bar_x = 20;
        let bar_y = y + 5;
        let bar_width = 250;
        let bar_height = 20;

        // Background
        Rectangle::new(
            Point::new(bar_x, bar_y),
            Size::new(bar_width, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        // Filled portion
        let fill_ratio = (current_exp as f32 / exp_to_level as f32).min(1.0);
        let filled_width = (bar_width as f32 * fill_ratio) as u32;

        if filled_width > 0 {
            Rectangle::new(
                Point::new(bar_x, bar_y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 200, 255)))
            .draw(display)?;
        }

        // Progress text
        let mut progress_text = heapless::String::<32>::new();
        write!(progress_text, "{}/{}", current_exp, exp_to_level).ok();
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let text_x = bar_x + ((bar_width / 2) as i32) - ((progress_text.len() as i32) * 3);
        Text::new(&progress_text, Point::new(text_x, bar_y + 13), text_style).draw(display)?;

        Ok(())
    }

    /// Draw card reveal box
    fn draw_card_reveal(
        display: &mut Sh8601Driver,
        cards: &[Card],
        state: LootState,
    ) -> Result<(), Box<dyn Error>> {
        let y = 310; // Adjusted to account for hero section at top
        let label_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);

        Text::new("LOOT:", Point::new(20, y), label_style).draw(display)?;

        match state {
            LootState::Hidden => {
                // Mystery box
                let box_style = PrimitiveStyle::with_fill(Rgb888::new(100, 80, 50));
                RoundedRectangle::new(
                    Rectangle::new(Point::new(20, y + 10), Size::new(80, 80)),
                    CornerRadii::new(Size::new(8, 8)),
                )
                .into_styled(box_style)
                .draw(display)?;

                let mystery_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 200));
                Text::new("?", Point::new(48, y + 60), mystery_style).draw(display)?;

                let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
                Text::new("[TAP to reveal]", Point::new(110, y + 50), hint_style).draw(display)?;
            }
            LootState::Revealing | LootState::Revealed => {
                if cards.is_empty() {
                    // No loot
                    let no_loot_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
                    Text::new("... nothing", Point::new(20, y + 30), no_loot_style).draw(display)?;
                } else {
                    // Show cards
                    for (i, card) in cards.iter().enumerate() {
                        let card_y = y + 10 + (i as i32 * 90);

                        // Card background
                        let rarity_color = match card.rarity {
                            1 => Rgb888::new(150, 150, 150), // Gray - common
                            2 => Rgb888::new(100, 200, 100), // Green - uncommon
                            3 => Rgb888::new(100, 150, 255), // Blue - rare
                            4 => Rgb888::new(200, 100, 255), // Purple - epic
                            5 => Rgb888::new(255, 180, 50),  // Gold - legendary
                            _ => Rgb888::new(100, 100, 100),
                        };

                        RoundedRectangle::new(
                            Rectangle::new(Point::new(20, card_y), Size::new(328, 80)),
                            CornerRadii::new(Size::new(8, 8)),
                        )
                        .into_styled(PrimitiveStyle::with_fill(rarity_color))
                        .draw(display)?;

                        // Card name
                        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
                        let mut card_name = heapless::String::<32>::new();
                        write!(card_name, "{}", card.name).ok();
                        Text::new(&card_name, Point::new(30, card_y + 25), name_style).draw(display)?;

                        // Rarity stars
                        let mut stars = heapless::String::<16>::new();
                        for _ in 0..card.rarity {
                            write!(stars, "★").ok();
                        }
                        for _ in card.rarity..5 {
                            write!(stars, "☆").ok();
                        }
                        let stars_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 255, 200));
                        Text::new(&stars, Point::new(30, card_y + 45), stars_style).draw(display)?;

                        // Bonuses
                        let bonus_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 255, 255));
                        let mut bonus_text = heapless::String::<32>::new();
                        if card.atk_bonus > 0 && card.def_bonus > 0 {
                            write!(bonus_text, "+{} ATK, +{} DEF", card.atk_bonus, card.def_bonus).ok();
                        } else if card.atk_bonus > 0 {
                            write!(bonus_text, "+{} ATK", card.atk_bonus).ok();
                        } else if card.def_bonus > 0 {
                            write!(bonus_text, "+{} DEF", card.def_bonus).ok();
                        }
                        Text::new(&bonus_text, Point::new(30, card_y + 65), bonus_style).draw(display)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Page for ExpeditionSummaryPage {
    fn update(&mut self) -> bool {
        // Check if loot reveal animation should complete
        if self.loot_state == LootState::Revealing {
            if let Some(reveal_time) = self.loot_reveal_time {
                if reveal_time.elapsed() > Duration::from_millis(500) {
                    self.loot_state = LootState::Revealed;
                }
            }
        }

        // Keep page active - expedition_summary_system will close it when user taps Continue
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Clear background
        display.clear(self.background_color)?;

        // Title
        if self.survived {
            let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 255, 150));
            Text::new("EXPEDITION COMPLETE!", Point::new(20, 20), title_style).draw(display)?;
        } else {
            let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 150, 150));
            Text::new("EXPEDITION FAILED", Point::new(30, 20), title_style).draw(display)?;
        }

        // Hero Section Box
        let hero_box_y = 40;
        let hero_box_bg = if self.survived {
            Rgb888::new(30, 50, 40) // Dark green tint for success
        } else {
            Rgb888::new(50, 30, 30) // Dark red tint for failure
        };

        Rectangle::new(
            Point::new(10, hero_box_y),
            Size::new(348, 85),
        )
        .into_styled(PrimitiveStyle::with_fill(hero_box_bg))
        .draw(display)?;

        // Hero name
        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new(&self.hero.name, Point::new(20, hero_box_y + 20), name_style).draw(display)?;

        // Job and Level
        let job_name = self.hero.job.get_name();
        let mut job_level = heapless::String::<64>::new();
        write!(job_level, "{} Lv.{}", job_name, self.hero.level).ok();
        let job_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
        Text::new(&job_level, Point::new(20, hero_box_y + 35), job_style).draw(display)?;

        // HP bar in hero section
        let hp_bar_y = hero_box_y + 45;
        let bar_width = 250;
        let bar_height = 20;

        // HP label
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        Text::new("HP:", Point::new(20, hp_bar_y), small_style).draw(display)?;

        // Bar background
        Rectangle::new(
            Point::new(20, hp_bar_y + 5),
            Size::new(bar_width as u32, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        // Filled portion
        let hp_percentage = (self.hero.current_health as f32 / self.hero.max_health as f32) * 100.0;
        let hp_color = if hp_percentage > 60.0 {
            Rgb888::new(0, 200, 0) // Green
        } else if hp_percentage > 30.0 {
            Rgb888::new(200, 200, 0) // Yellow
        } else {
            Rgb888::new(200, 0, 0) // Red
        };

        let filled_width = ((self.hero.current_health as f32 / self.hero.max_health as f32) * bar_width as f32) as u32;
        if filled_width > 0 {
            Rectangle::new(
                Point::new(20, hp_bar_y + 5),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(hp_color))
            .draw(display)?;
        }

        // HP text on bar
        let mut hp_text = heapless::String::<32>::new();
        write!(hp_text, "{}/{}", self.hero.current_health, self.hero.max_health).ok();
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_x = 20 + (bar_width / 2) - (hp_text.len() as i32 * 5);
        Text::new(&hp_text, Point::new(text_x, hp_bar_y + 20), text_style).draw(display)?;

        // Level up notification - PROMINENT (if leveled up)
        let mut y = 140; // Start below hero section (hero_box_y + 85 + 15)
        if self.hero.level > self.initial_level {
            let levels_gained = self.hero.level - self.initial_level;

            // Draw background highlight for level up
            Rectangle::new(
                Point::new(20, y),
                Size::new(328, 40),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50)))
            .draw(display)?;

            let mut level_up_text = heapless::String::<48>::new();
            if levels_gained == 1 {
                write!(level_up_text, "*** LEVEL UP! ***").ok();
            } else {
                write!(level_up_text, "*** +{} LEVELS UP! ***", levels_gained).ok();
            }
            let level_up_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 100));
            let level_up_x = 184 - (level_up_text.len() as i32 * 5);
            Text::new(&level_up_text, Point::new(level_up_x, y + 25), level_up_style).draw(display)?;

            // Show level transition
            let mut level_transition = heapless::String::<32>::new();
            write!(level_transition, "Level {} -> {}", self.initial_level, self.hero.level).ok();
            let transition_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 255, 200));
            let transition_x = 184 - (level_transition.len() as i32 * 3);
            Text::new(&level_transition, Point::new(transition_x, y + 40), transition_style).draw(display)?;

            y += 60; // Move stats section down
        }

        // Stats section
        let stats_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);

        // Kills
        let mut kills_text = heapless::String::<32>::new();
        if self.survived {
            write!(kills_text, "Kills: {}", self.actual_kills).ok();
        } else {
            write!(kills_text, "Kills: {} / {}", self.actual_kills, self.target_kills).ok();
        }
        Text::new(&kills_text, Point::new(20, y), stats_style).draw(display)?;

        // XP bar
        Self::draw_exp_bar(
            display,
            self.hero.experience,
            self.hero.experience_to_next_level,
            self.exp_gained,
            y + 20,
        )?;

        // Loot section
        if self.survived {
            Self::draw_card_reveal(display, &self.cards_dropped, self.loot_state)?;
        } else {
            // Lost loot message
            let y = 310; // Match loot section position
            let lost_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(255, 100, 100));
            Text::new("Loot: LOST!", Point::new(20, y), lost_style).draw(display)?;

            // KO recovery time
            let recovery_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(200, 200, 200));
            Text::new("Recovery: 10:00", Point::new(20, y + 30), recovery_style).draw(display)?;
        }

        // Continue button (only if can continue)
        if self.can_continue() {
            let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
            Text::new("[TAP] Continue", Point::new(120, 450), hint_style).draw(display)?;
        }

        // Flush to display
        display.flush()?;

        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No internal dirty tracking
    }

    fn needs_full_redraw(&self) -> bool {
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
