//! Hunt Battle Result Page
//!
//! Shows battle results with options to continue hunting or stop.

use crate::display::Sh8601Driver;
use crate::game::{Hero, Card};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, RoundedRectangle, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action from result page
#[derive(Debug, Clone, PartialEq)]
pub enum HuntResultAction {
    /// Start next battle with same monster
    Next,
    /// Stop hunting, return to map
    Stop,
}

/// Hunt battle result page
pub struct HuntBattleResultPage {
    hero: Hero,
    enemy_id: u32,
    enemy_name: String,
    exp_gained: u32,
    exp_before: u32,
    exp_to_next: u32,
    level_before: u32,
    leveled_up: bool,
    card_dropped: Option<Card>,
    victory: bool,
    pending_action: Option<HuntResultAction>,
    needs_redraw: bool,
    first_draw: bool,
}

impl HuntBattleResultPage {
    /// Create a new hunt battle result page
    pub fn new(
        hero: Hero,
        enemy_id: u32,
        enemy_name: String,
        exp_gained: u32,
        exp_before: u32,
        exp_to_next: u32,
        level_before: u32,
        leveled_up: bool,
        card_dropped: Option<Card>,
        victory: bool,
    ) -> Self {
        log::info!(
            "Hunt result: {} vs {} - Victory={}, EXP={}, Card={:?}",
            hero.name, enemy_name, victory, exp_gained,
            card_dropped.as_ref().map(|c| &c.name)
        );

        Self {
            hero,
            enemy_id,
            enemy_name,
            exp_gained,
            exp_before,
            exp_to_next,
            level_before,
            leveled_up,
            card_dropped,
            victory,
            pending_action: None,
            needs_redraw: true,
            first_draw: true,
        }
    }

    /// Handle tap
    pub fn handle_tap(&mut self, x: i32, y: i32) {
        log::info!("Hunt result tap at ({}, {})", x, y);

        // Next button (left)
        if x >= 30 && x <= 170 && y >= 380 && y <= 430 {
            if self.victory && self.hero.current_health > 0 {
                log::info!("Next battle requested");
                self.pending_action = Some(HuntResultAction::Next);
            }
            return;
        }

        // Stop button (right)
        if x >= 195 && x <= 335 && y >= 380 && y <= 430 {
            log::info!("Stop hunting requested");
            self.pending_action = Some(HuntResultAction::Stop);
            return;
        }
    }

    /// Handle swipe left (stop)
    pub fn handle_swipe_left(&mut self) {
        self.pending_action = Some(HuntResultAction::Stop);
    }

    /// Take pending action
    pub fn take_action(&mut self) -> Option<HuntResultAction> {
        self.pending_action.take()
    }

    /// Get updated hero
    pub fn get_hero(&self) -> &Hero {
        &self.hero
    }

    /// Get enemy ID for next battle
    pub fn get_enemy_id(&self) -> u32 {
        self.enemy_id
    }

    /// Draw EXP bar
    fn draw_exp_bar(&self, display: &mut Sh8601Driver, x: i32, y: i32, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Background
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
            .draw(display)?;

        // Calculate fill based on current exp in level
        let current_exp = self.hero.experience;
        let exp_in_level = if self.leveled_up {
            // After level up, show from 0
            current_exp
        } else {
            current_exp - (self.exp_before - self.exp_gained) + self.exp_gained
        };

        let fill_ratio = (exp_in_level as f32 / self.hero.experience_to_next_level as f32).min(1.0);
        let fill_width = (fill_ratio * width as f32) as u32;

        // EXP fill (blue)
        Rectangle::new(Point::new(x, y), Size::new(fill_width.min(width), height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 200)))
            .draw(display)?;

        // Border
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 100, 100), 1))
            .draw(display)?;

        // Text
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let mut exp_text = heapless::String::<32>::new();
        write!(exp_text, "EXP: {}/{}", self.hero.experience, self.hero.experience_to_next_level).ok();
        Text::new(&exp_text, Point::new(x + 5, y + height as i32 / 2 + 3), small_style).draw(display)?;

        Ok(())
    }

    /// Draw HP bar
    fn draw_hp_bar(&self, display: &mut Sh8601Driver, x: i32, y: i32, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Background
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
            .draw(display)?;

        // Calculate fill
        let fill_ratio = (self.hero.current_health.max(0) as f32 / self.hero.max_health as f32).min(1.0);
        let fill_width = (fill_ratio * width as f32) as u32;

        // HP color based on percentage
        let hp_percent = fill_ratio * 100.0;
        let hp_color = if hp_percent > 50.0 {
            Rgb888::new(50, 200, 50)
        } else if hp_percent > 25.0 {
            Rgb888::new(200, 200, 50)
        } else {
            Rgb888::new(200, 50, 50)
        };

        Rectangle::new(Point::new(x, y), Size::new(fill_width.min(width), height))
            .into_styled(PrimitiveStyle::with_fill(hp_color))
            .draw(display)?;

        // Border
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 100, 100), 1))
            .draw(display)?;

        // Text
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let mut hp_text = heapless::String::<32>::new();
        write!(hp_text, "HP: {}/{}", self.hero.current_health.max(0), self.hero.max_health).ok();
        Text::new(&hp_text, Point::new(x + 5, y + height as i32 / 2 + 3), small_style).draw(display)?;

        Ok(())
    }
}

impl Page for HuntBattleResultPage {
    fn update(&mut self) -> bool {
        self.pending_action.is_none()
    }

    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.first_draw || self.needs_redraw {
            use core::fmt::Write;

            // Clear screen
            let bg_color = if self.victory {
                Rgb888::new(20, 35, 25) // Greenish for victory
            } else {
                Rgb888::new(35, 20, 20) // Reddish for defeat
            };
            Rectangle::new(Point::zero(), Size::new(480, 480))
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 180, 180));

            // Result title
            let result_text = if self.victory { "VICTORY!" } else { "DEFEAT" };
            let result_color = if self.victory {
                Rgb888::new(100, 255, 100)
            } else {
                Rgb888::new(255, 100, 100)
            };
            Text::new(result_text, Point::new(130, 40), MonoTextStyle::new(&FONT_10X20, result_color)).draw(display)?;

            // Enemy name
            let mut enemy_text = heapless::String::<32>::new();
            write!(enemy_text, "vs {}", self.enemy_name).ok();
            Text::new(&enemy_text, Point::new(130, 70), small_style).draw(display)?;

            // EXP gained section
            let mut exp_text = heapless::String::<32>::new();
            write!(exp_text, "+{} EXP", self.exp_gained).ok();
            Text::new(&exp_text, Point::new(130, 120), MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 150, 255))).draw(display)?;

            // Level up notification
            if self.leveled_up {
                let mut level_text = heapless::String::<32>::new();
                write!(level_text, "LEVEL UP! {} -> {}", self.level_before, self.hero.level).ok();
                Text::new(&level_text, Point::new(100, 150), MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0))).draw(display)?;
            }

            // EXP bar
            Text::new("Experience:", Point::new(30, 185), small_style).draw(display)?;
            self.draw_exp_bar(display, 30, 195, 300, 25)?;

            // HP bar
            Text::new("Health:", Point::new(30, 245), small_style).draw(display)?;
            self.draw_hp_bar(display, 30, 255, 300, 25)?;

            // Card drop section
            Text::new("Loot:", Point::new(30, 310), title_style).draw(display)?;

            if let Some(ref card) = self.card_dropped {
                // Card dropped!
                RoundedRectangle::new(
                    Rectangle::new(Point::new(30, 325), Size::new(300, 40)),
                    CornerRadii::new(Size::new(5, 5)),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 60, 20)))
                .draw(display)?;

                let card_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
                Text::new(&card.name, Point::new(40, 352), card_style).draw(display)?;

                // Rarity stars
                let mut stars = heapless::String::<8>::new();
                for _ in 0..card.rarity {
                    stars.push('*').ok();
                }
                Text::new(&stars, Point::new(250, 352), MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0))).draw(display)?;
            } else {
                Text::new("No card dropped", Point::new(40, 350), MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 100, 100))).draw(display)?;
            }

            // Buttons
            let can_continue = self.victory && self.hero.current_health > 0;

            // Next button
            let next_color = if can_continue {
                Rgb888::new(50, 150, 50)
            } else {
                Rgb888::new(60, 60, 60)
            };
            RoundedRectangle::new(
                Rectangle::new(Point::new(30, 380), Size::new(140, 50)),
                CornerRadii::new(Size::new(8, 8)),
            )
            .into_styled(PrimitiveStyle::with_fill(next_color))
            .draw(display)?;

            let next_text_color = if can_continue { Rgb888::WHITE } else { Rgb888::new(100, 100, 100) };
            Text::new("Next", Point::new(75, 412), MonoTextStyle::new(&FONT_10X20, next_text_color)).draw(display)?;

            // Stop button
            RoundedRectangle::new(
                Rectangle::new(Point::new(195, 380), Size::new(140, 50)),
                CornerRadii::new(Size::new(8, 8)),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 50, 50)))
            .draw(display)?;
            Text::new("Stop", Point::new(240, 412), title_style).draw(display)?;

            self.first_draw = false;
            self.needs_redraw = false;
        }

        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering hunt battle result page");
    }

    fn on_exit(&mut self) {
        log::info!("Exiting hunt battle result page");
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
