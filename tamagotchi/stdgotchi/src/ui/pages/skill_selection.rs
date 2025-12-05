//! Skill Selection Page
//!
//! Allows players to equip cards with skills to their 3 skill slots.
//! Only cards that unlock skills can be equipped here.

use crate::display::Sh8601Driver;
use crate::game::{Hero, GameData};
use crate::game::expedition::Card;
use crate::game::skill::SkillData;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X10, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, RoundedRectangle, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Card with skill info for display
#[derive(Debug, Clone)]
pub struct SkillCardInfo {
    pub card: Card,
    pub skill: SkillData,
}

/// Page state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageState {
    /// Viewing the 3 equipment slots
    SlotView,
    /// Selecting a card for a specific slot
    CardSelection { slot_index: usize },
}

/// Action from skill selection page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillSelectionAction {
    /// Exit the page
    Exit,
    /// Card was equipped
    CardEquipped,
    /// Card was unequipped
    CardUnequipped,
}

/// Skill Selection Page
pub struct SkillSelectionPage {
    // Hero reference for equipped skills
    hero: Hero,
    game_data: GameData,

    // Available cards with skills
    available_cards: Vec<SkillCardInfo>,

    // UI state
    state: PageState,
    selected_slot: usize,
    selected_card_index: usize,
    scroll_offset: usize,

    // Pending action
    pending_action: Option<SkillSelectionAction>,

    // Display state
    needs_redraw: bool,
    first_draw: bool,
}

impl SkillSelectionPage {
    /// Create a new skill selection page
    pub fn new(hero: Hero, game_data: GameData) -> Self {
        // Build list of available skill cards
        log::info!("Skill selection: Hero has {} total cards", hero.cards.len());
        let mut available_cards = Vec::new();
        for card in &hero.cards {
            log::info!("  Card: {} (id={}) unlocks_skill={:?}", card.name, card.monster_id, card.unlocks_skill);
            if let Some(skill_id) = card.unlocks_skill {
                if let Some(skill_data) = game_data.get_skill(skill_id) {
                    log::info!("    -> Found skill: {} (id={})", skill_data.name, skill_id);
                    available_cards.push(SkillCardInfo {
                        card: card.clone(),
                        skill: skill_data.clone(),
                    });
                } else {
                    log::warn!("    -> Skill {} not found in game data!", skill_id);
                }
            }
        }

        log::info!("Skill selection page: {} cards with skills available", available_cards.len());

        Self {
            hero,
            game_data,
            available_cards,
            state: PageState::SlotView,
            selected_slot: 0,
            selected_card_index: 0,
            scroll_offset: 0,
            pending_action: None,
            needs_redraw: true,
            first_draw: true,
        }
    }

    /// Get the updated hero
    pub fn get_hero(&self) -> &Hero {
        &self.hero
    }

    /// Take pending action
    pub fn take_action(&mut self) -> Option<SkillSelectionAction> {
        self.pending_action.take()
    }

    /// Handle swipe left (exit)
    pub fn handle_swipe_left(&mut self) {
        match self.state {
            PageState::SlotView => {
                self.pending_action = Some(SkillSelectionAction::Exit);
            }
            PageState::CardSelection { .. } => {
                self.state = PageState::SlotView;
                self.needs_redraw = true;
            }
        }
    }

    /// Handle swipe up (scroll or select previous)
    pub fn handle_swipe_up(&mut self) {
        match self.state {
            PageState::SlotView => {
                if self.selected_slot > 0 {
                    self.selected_slot -= 1;
                    self.needs_redraw = true;
                }
            }
            PageState::CardSelection { .. } => {
                if self.selected_card_index > 0 {
                    self.selected_card_index -= 1;
                    if self.selected_card_index < self.scroll_offset {
                        self.scroll_offset = self.selected_card_index;
                    }
                    self.needs_redraw = true;
                }
            }
        }
    }

    /// Handle swipe down (scroll or select next)
    pub fn handle_swipe_down(&mut self) {
        match self.state {
            PageState::SlotView => {
                if self.selected_slot < 2 {
                    self.selected_slot += 1;
                    self.needs_redraw = true;
                }
            }
            PageState::CardSelection { .. } => {
                // +1 for "Unequip" option
                let max_index = self.available_cards.len();
                if self.selected_card_index < max_index {
                    self.selected_card_index += 1;
                    // Scroll if needed (show 5 items at a time)
                    if self.selected_card_index >= self.scroll_offset + 5 {
                        self.scroll_offset = self.selected_card_index.saturating_sub(4);
                    }
                    self.needs_redraw = true;
                }
            }
        }
    }

    /// Handle tap (select slot or card)
    pub fn handle_tap(&mut self, x: i32, y: i32) {
        log::info!("Skill selection tap at ({}, {}), state={:?}", x, y, self.state);
        match self.state {
            PageState::SlotView => {
                // Check which slot was tapped (slots are at y=80, 200, 320)
                let slot_height = 120;
                let slot_start_y = 80;
                for i in 0..3 {
                    let slot_y = slot_start_y + (i as i32 * slot_height);
                    if y >= slot_y && y < slot_y + 100 && x >= 20 && x <= 460 {
                        self.selected_slot = i;
                        log::info!("Selected slot {}", i);
                        self.state = PageState::CardSelection { slot_index: i };
                        self.selected_card_index = 0;
                        self.scroll_offset = 0;
                        self.needs_redraw = true;
                        return;
                    }
                }
            }
            PageState::CardSelection { slot_index } => {
                // Check which item was tapped
                let item_height = 70;
                let start_y = 70;

                // Check Unequip option (always at index 0)
                if y >= start_y && y < start_y + item_height - 5 && x >= 20 && x <= 460 {
                    log::info!("Tapped Unequip option");
                    self.hero.unequip_skill(slot_index);
                    self.pending_action = Some(SkillSelectionAction::CardUnequipped);
                    self.state = PageState::SlotView;
                    self.needs_redraw = true;
                    return;
                }

                // Check card items
                let visible_count = 5;
                for display_idx in 0..visible_count {
                    let card_idx = self.scroll_offset + display_idx;
                    if card_idx >= self.available_cards.len() {
                        break;
                    }

                    let item_y = start_y + ((display_idx + 1) as i32 * item_height);
                    if y >= item_y && y < item_y + item_height - 5 && x >= 20 && x <= 460 {
                        let card_info = &self.available_cards[card_idx];
                        log::info!("Tapped card: {} (skill {})", card_info.card.name, card_info.skill.name);

                        // Check if already equipped in another slot
                        if self.hero.is_card_equipped(card_info.card.monster_id) {
                            log::warn!("Card already equipped in another slot");
                            return;
                        }

                        // Equip the card
                        if let Err(e) = self.hero.equip_skill(
                            slot_index,
                            card_info.card.monster_id,
                            card_info.skill.id,
                        ) {
                            log::error!("Failed to equip skill: {}", e);
                        } else {
                            log::info!("Equipped {} to slot {}", card_info.skill.name, slot_index);
                            self.pending_action = Some(SkillSelectionAction::CardEquipped);
                            self.state = PageState::SlotView;
                            self.needs_redraw = true;
                        }
                        return;
                    }
                }
            }
        }
    }

    /// Draw slot view
    fn draw_slot_view(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("SKILL EQUIPMENT", Point::new(130, 30), title_style).draw(display)?;

        // Instructions
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));
        Text::new("Tap slot to change  |  Swipe left to exit", Point::new(80, 50), small_style).draw(display)?;

        // Draw 3 slots
        for i in 0..3 {
            let y = 80 + (i as i32 * 120);
            let is_selected = i == self.selected_slot;

            // Slot background
            let bg_color = if is_selected {
                Rgb888::new(60, 60, 100)
            } else {
                Rgb888::new(40, 40, 40)
            };

            RoundedRectangle::new(
                Rectangle::new(Point::new(20, y), Size::new(440, 100)),
                CornerRadii::new(Size::new(10, 10)),
            )
            .into_styled(PrimitiveStyle::with_fill(bg_color))
            .draw(display)?;

            // Selection indicator
            if is_selected {
                RoundedRectangle::new(
                    Rectangle::new(Point::new(20, y), Size::new(440, 100)),
                    CornerRadii::new(Size::new(10, 10)),
                )
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 2))
                .draw(display)?;
            }

            // Slot label
            use core::fmt::Write;
            let mut label = heapless::String::<16>::new();
            write!(label, "Slot {}", i + 1).ok();
            Text::new(&label, Point::new(30, y + 25), title_style).draw(display)?;

            // Check if slot has a skill equipped
            let slot = &self.hero.equipped_skill_slots[i];
            if let Some(skill_id) = slot.skill_id {
                if let Some(skill_data) = self.game_data.get_skill(skill_id) {
                    // Show skill name
                    let skill_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 200, 100));
                    Text::new(&skill_data.name, Point::new(30, y + 55), skill_style).draw(display)?;

                    // Show skill description
                    Text::new(&skill_data.description, Point::new(30, y + 80), small_style).draw(display)?;

                    // Show cooldown
                    let mut cd_text = heapless::String::<16>::new();
                    write!(cd_text, "CD: {:.1}s", skill_data.cooldown_seconds).ok();
                    Text::new(&cd_text, Point::new(350, y + 55), small_style).draw(display)?;
                }
            } else {
                // Empty slot
                let empty_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 100, 100));
                Text::new("Empty - Tap to equip", Point::new(30, y + 55), empty_style).draw(display)?;
            }
        }

        Ok(())
    }

    /// Draw card selection view
    fn draw_card_selection(&self, display: &mut Sh8601Driver, slot_index: usize) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let mut title = heapless::String::<32>::new();
        write!(title, "Select Card for Slot {}", slot_index + 1).ok();
        Text::new(&title, Point::new(100, 30), title_style).draw(display)?;

        // Instructions
        let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));
        Text::new("Swipe up/down to scroll  |  Tap to select", Point::new(80, 50), small_style).draw(display)?;

        let visible_count = 5;
        let item_height = 70;

        // Draw "Unequip" option first
        {
            let y = 70;
            let is_selected = self.selected_card_index == 0;

            let bg_color = if is_selected {
                Rgb888::new(100, 50, 50)
            } else {
                Rgb888::new(40, 40, 40)
            };

            RoundedRectangle::new(
                Rectangle::new(Point::new(20, y), Size::new(440, item_height as u32 - 5)),
                CornerRadii::new(Size::new(5, 5)),
            )
            .into_styled(PrimitiveStyle::with_fill(bg_color))
            .draw(display)?;

            if is_selected {
                RoundedRectangle::new(
                    Rectangle::new(Point::new(20, y), Size::new(440, item_height as u32 - 5)),
                    CornerRadii::new(Size::new(5, 5)),
                )
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 2))
                .draw(display)?;
            }

            let unequip_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 100, 100));
            Text::new("UNEQUIP SLOT", Point::new(150, y + 40), unequip_style).draw(display)?;
        }

        // Draw available cards
        let start = self.scroll_offset;
        let end = (start + visible_count - 1).min(self.available_cards.len());

        for (display_idx, card_idx) in (start..end).enumerate() {
            let card_info = &self.available_cards[card_idx];
            let y = 70 + ((display_idx + 1) as i32 * item_height);
            let list_index = card_idx + 1; // +1 because index 0 is "Unequip"
            let is_selected = list_index == self.selected_card_index;

            // Check if already equipped
            let is_equipped = self.hero.is_card_equipped(card_info.card.monster_id);

            let bg_color = if is_selected {
                Rgb888::new(60, 60, 100)
            } else if is_equipped {
                Rgb888::new(60, 40, 40)
            } else {
                Rgb888::new(40, 40, 40)
            };

            RoundedRectangle::new(
                Rectangle::new(Point::new(20, y), Size::new(440, item_height as u32 - 5)),
                CornerRadii::new(Size::new(5, 5)),
            )
            .into_styled(PrimitiveStyle::with_fill(bg_color))
            .draw(display)?;

            if is_selected {
                RoundedRectangle::new(
                    Rectangle::new(Point::new(20, y), Size::new(440, item_height as u32 - 5)),
                    CornerRadii::new(Size::new(5, 5)),
                )
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 2))
                .draw(display)?;
            }

            // Card name
            Text::new(&card_info.card.name, Point::new(30, y + 20), title_style).draw(display)?;

            // Skill name and description
            let skill_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 200, 100));
            let mut skill_text = heapless::String::<64>::new();
            write!(skill_text, "-> {} (CD: {:.1}s)", card_info.skill.name, card_info.skill.cooldown_seconds).ok();
            Text::new(&skill_text, Point::new(30, y + 40), skill_style).draw(display)?;

            // Show if already equipped
            if is_equipped {
                let equipped_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(255, 100, 100));
                Text::new("[EQUIPPED]", Point::new(350, y + 20), equipped_style).draw(display)?;
            }

            // Rarity stars
            let mut stars = heapless::String::<8>::new();
            for _ in 0..card_info.card.rarity {
                stars.push('*').ok();
            }
            Text::new(&stars, Point::new(400, y + 40), small_style).draw(display)?;
        }

        // Scroll indicator
        if self.available_cards.len() > visible_count - 1 {
            let mut scroll_text = heapless::String::<16>::new();
            write!(scroll_text, "{}/{}", start + 1, self.available_cards.len()).ok();
            Text::new(&scroll_text, Point::new(400, 450), small_style).draw(display)?;
        }

        Ok(())
    }
}

impl Page for SkillSelectionPage {
    fn update(&mut self) -> bool {
        // Page stays open until action is taken
        self.pending_action.is_none()
    }

    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.first_draw || self.needs_redraw {
            // Clear screen
            Rectangle::new(Point::zero(), Size::new(480, 480))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 20, 30)))
                .draw(display)?;

            match self.state {
                PageState::SlotView => {
                    self.draw_slot_view(display)?;
                }
                PageState::CardSelection { slot_index } => {
                    self.draw_card_selection(display, slot_index)?;
                }
            }

            self.first_draw = false;
            self.needs_redraw = false;
        }

        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering skill selection page");
        self.needs_redraw = true;
    }

    fn on_exit(&mut self) {
        log::info!("Exiting skill selection page");
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
