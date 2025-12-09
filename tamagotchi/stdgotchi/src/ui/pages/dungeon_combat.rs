//! Dungeon Combat Page
//!
//! Real-time combat UI for dungeon battles.
//! Displays enemy, player monster, bars, and action buttons.

use crate::display::St7789pDriver;
use crate::game::core::Element;
use crate::game::systems::combat::{CombatState, CombatEvent};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;
use std::time::Instant;

/// Action from combat page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonCombatAction {
    None,
    UseSkill,
    SwapMonster(u8),
    CombatEnded { victory: bool },
}

/// Dungeon combat page
pub struct DungeonCombatPage {
    combat_state: CombatState,
    last_update: Instant,
    dirty: bool,

    // Touch areas
    skill_button_area: Option<Rectangle>,
    swap_button_areas: [Option<Rectangle>; 3],

    // Damage feedback
    damage_popups: Vec<DamagePopup>,

    // Dungeon info
    dungeon_name: String,

    // End delay timer (seconds)
    end_delay: f32,
}

struct DamagePopup {
    damage: u16,
    is_player_damage: bool,  // true = damage to enemy, false = damage to player
    is_heal: bool,           // true = heal (green), false = damage (red/white)
    y_offset: f32,
    alpha: f32,
}

impl DungeonCombatPage {
    pub fn new(combat_state: CombatState, dungeon_name: String) -> Self {
        Self {
            combat_state,
            last_update: Instant::now(),
            dirty: true,
            skill_button_area: None,
            swap_button_areas: [None; 3],
            damage_popups: Vec::new(),
            dungeon_name,
            end_delay: 0.0,
        }
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> DungeonCombatAction {
        // Check skill button
        if let Some(rect) = self.skill_button_area {
            if rect.contains(Point::new(x, y)) {
                if self.combat_state.player_skl_bar >= 1.0 {
                    if let Some(event) = self.combat_state.use_skill() {
                        self.handle_combat_event(event);
                        self.dirty = true;
                        return DungeonCombatAction::UseSkill;
                    }
                }
            }
        }

        // Check swap buttons
        for (i, area) in self.swap_button_areas.iter().enumerate() {
            if let Some(rect) = area {
                if rect.contains(Point::new(x, y)) {
                    if let Some(event) = self.combat_state.swap_monster(i as u8) {
                        self.handle_combat_event(event);
                        self.dirty = true;
                        return DungeonCombatAction::SwapMonster(i as u8);
                    }
                }
            }
        }

        DungeonCombatAction::None
    }

    fn handle_combat_event(&mut self, event: CombatEvent) {
        match event {
            CombatEvent::PlayerAttack { damage, .. } => {
                self.damage_popups.push(DamagePopup {
                    damage,
                    is_player_damage: true,
                    is_heal: false,
                    y_offset: 0.0,
                    alpha: 1.0,
                });
            }
            CombatEvent::EnemyAttack { damage, .. } => {
                self.damage_popups.push(DamagePopup {
                    damage,
                    is_player_damage: false,
                    is_heal: false,
                    y_offset: 0.0,
                    alpha: 1.0,
                });
            }
            CombatEvent::PlayerSkill { damage, .. } => {
                self.damage_popups.push(DamagePopup {
                    damage,
                    is_player_damage: true,
                    is_heal: false,
                    y_offset: 0.0,
                    alpha: 1.0,
                });
            }
            CombatEvent::PlayerSkillHeal { heal_amount, .. } => {
                self.damage_popups.push(DamagePopup {
                    damage: heal_amount,
                    is_player_damage: false, // Show on player side
                    is_heal: true,
                    y_offset: 0.0,
                    alpha: 1.0,
                });
            }
            _ => {}
        }
    }

    fn element_color(element: Element) -> Rgb888 {
        match element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(150, 120, 50),
            Element::Wind => Rgb888::new(100, 220, 150),
            Element::Thunder => Rgb888::new(255, 255, 100),
            Element::Shadow => Rgb888::new(150, 50, 200),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(180, 180, 220),
        }
    }

    fn element_char(element: Element) -> char {
        match element {
            Element::Fire => 'F',
            Element::Water => 'W',
            Element::Earth => 'E',
            Element::Wind => 'N',
            Element::Thunder => 'T',
            Element::Shadow => 'S',
            Element::Holy => 'H',
            Element::Ghost => 'G',
        }
    }

    /// Get combat result if combat ended
    pub fn combat_result(&self) -> Option<(bool, u32, u32)> {
        if self.combat_state.combat_ended {
            Some((
                self.combat_state.player_won,
                self.combat_state.crystals_earned,
                self.combat_state.xp_earned,
            ))
        } else {
            None
        }
    }

    /// Get reference to combat state
    pub fn combat_state(&self) -> &CombatState {
        &self.combat_state
    }
}

impl Page for DungeonCombatPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));

        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        // Header card
        let header_rect = Rectangle::new(Point::new(5, 2), Size::new(230, 20));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(4, 4)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(180, 140, 140))
            .build())
            .draw(display)?;

        let dungeon_name = if self.dungeon_name.len() > 10 { &self.dungeon_name[..10] } else { &self.dungeon_name };
        let header_text = format!("{} Fl.{}", dungeon_name, self.combat_state.current_floor);
        Text::new(&header_text, Point::new(10, 16), text_style).draw(display)?;

        let wave_text = format!("W{}/{} +{}", self.combat_state.current_wave, self.combat_state.total_waves, self.combat_state.crystals_earned);
        Text::new(&wave_text, Point::new(160, 16), text_style).draw(display)?;

        // Enemy section card
        let enemy = &self.combat_state.enemy;
        let enemy_y = 26;

        let enemy_card = Rectangle::new(Point::new(5, enemy_y), Size::new(230, 50));
        let enemy_rounded = RoundedRectangle::new(enemy_card, CornerRadii::new(Size::new(6, 6)));
        enemy_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(255, 235, 235))
            .build())
            .draw(display)?;
        enemy_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(200, 150, 150))
            .stroke_width(1)
            .build())
            .draw(display)?;

        // Enemy name
        let elem_color = Self::element_color(enemy.element);
        let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
        let enemy_name = if enemy.name.len() > 12 { &enemy.name[..12] } else { &enemy.name };
        let enemy_info = format!("{} {} Lv.{}", Self::element_char(enemy.element), enemy_name, enemy.level);
        Text::new(&enemy_info, Point::new(12, enemy_y + 14), elem_style).draw(display)?;

        // Enemy HP bar
        let hp_bar_x = 12;
        let hp_bar_y = enemy_y + 20;
        let bar_width = 150u32;
        let bar_height = 10u32;

        let hp_bg = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(bar_width, bar_height));
        display.fill_solid(&hp_bg, Rgb888::new(200, 180, 180))?;

        let hp_percent = enemy.hp_current as f32 / enemy.hp_max as f32;
        let hp_fill_width = ((bar_width as f32) * hp_percent) as u32;
        if hp_fill_width > 0 {
            let hp_fill = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(hp_fill_width, bar_height));
            display.fill_solid(&hp_fill, Rgb888::new(220, 80, 80))?;
        }

        let hp_text = format!("{}/{}", enemy.hp_current, enemy.hp_max);
        Text::new(&hp_text, Point::new(hp_bar_x + bar_width as i32 + 5, hp_bar_y + 8), dim_style).draw(display)?;

        // Enemy SKL bar
        let skl_bar_y = enemy_y + 34;
        let skl_bg = Rectangle::new(Point::new(hp_bar_x, skl_bar_y), Size::new(100, 6));
        display.fill_solid(&skl_bg, Rgb888::new(200, 200, 220))?;

        let skl_fill_width = (100.0 * self.combat_state.enemy_skl_bar) as u32;
        if skl_fill_width > 0 {
            let skl_fill = Rectangle::new(Point::new(hp_bar_x, skl_bar_y), Size::new(skl_fill_width, 6));
            display.fill_solid(&skl_fill, Rgb888::new(150, 100, 200))?;
        }
        Text::new("SKL", Point::new(hp_bar_x + 105, skl_bar_y + 5), dim_style).draw(display)?;

        // Enemy aura
        if let Some((aura_elem, _)) = self.combat_state.enemy_aura {
            let aura_color = Self::element_color(aura_elem);
            let aura_style = MonoTextStyle::new(&FONT_6X10, aura_color);
            Text::new(&format!("[{}]", Self::element_char(aura_elem)), Point::new(200, enemy_y + 14), aura_style).draw(display)?;
        }

        // Player section card
        let player_y = 80;

        if let Some(monster) = self.combat_state.active_monster() {
            let player_card = Rectangle::new(Point::new(5, player_y), Size::new(230, 58));
            let player_rounded = RoundedRectangle::new(player_card, CornerRadii::new(Size::new(6, 6)));
            player_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(235, 255, 235))
                .build())
                .draw(display)?;
            player_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(150, 200, 150))
                .stroke_width(1)
                .build())
                .draw(display)?;

            let elem_color = Self::element_color(monster.element);
            let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
            let monster_name = if monster.name.len() > 12 { &monster.name[..12] } else { &monster.name };
            let player_info = format!("{} {} Lv.{}", Self::element_char(monster.element), monster_name, monster.level);
            Text::new(&player_info, Point::new(12, player_y + 14), elem_style).draw(display)?;

            // Player HP bar
            let hp_bar_y = player_y + 20;
            let hp_bg = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(bar_width, bar_height));
            display.fill_solid(&hp_bg, Rgb888::new(180, 200, 180))?;

            let hp_percent = monster.hp_current as f32 / monster.hp_max as f32;
            let hp_fill_width = ((bar_width as f32) * hp_percent) as u32;
            if hp_fill_width > 0 {
                let hp_fill = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(hp_fill_width, bar_height));
                display.fill_solid(&hp_fill, Rgb888::new(80, 200, 80))?;
            }

            let hp_text = format!("{}/{}", monster.hp_current, monster.hp_max);
            Text::new(&hp_text, Point::new(hp_bar_x + bar_width as i32 + 5, hp_bar_y + 8), dim_style).draw(display)?;

            // ATK bar
            let atk_bar_y = player_y + 34;
            let atk_bg = Rectangle::new(Point::new(hp_bar_x, atk_bar_y), Size::new(130, 6));
            display.fill_solid(&atk_bg, Rgb888::new(200, 200, 200))?;

            let atk_fill_width = (130.0 * self.combat_state.player_atk_bar) as u32;
            if atk_fill_width > 0 {
                let atk_fill = Rectangle::new(Point::new(hp_bar_x, atk_bar_y), Size::new(atk_fill_width, 6));
                display.fill_solid(&atk_fill, Rgb888::new(255, 180, 80))?;
            }
            Text::new("ATK", Point::new(hp_bar_x + 135, atk_bar_y + 5), dim_style).draw(display)?;

            // SKL bar
            let skl_bar_y = player_y + 44;
            let skl_bg = Rectangle::new(Point::new(hp_bar_x, skl_bar_y), Size::new(130, 6));
            display.fill_solid(&skl_bg, Rgb888::new(200, 200, 220))?;

            let skl_fill_width = (130.0 * self.combat_state.player_skl_bar) as u32;
            if skl_fill_width > 0 {
                let skl_color = if self.combat_state.player_skl_bar >= 1.0 {
                    Rgb888::new(220, 150, 255)
                } else {
                    Rgb888::new(150, 100, 200)
                };
                let skl_fill = Rectangle::new(Point::new(hp_bar_x, skl_bar_y), Size::new(skl_fill_width, 6));
                display.fill_solid(&skl_fill, skl_color)?;
            }
            Text::new("SKL", Point::new(hp_bar_x + 135, skl_bar_y + 5), dim_style).draw(display)?;
        }

        // Swap buttons
        let button_y = 145;
        let button_width = 70u32;
        let button_height = 28u32;

        for (i, monster) in self.combat_state.player_monsters.iter().take(3).enumerate() {
            let x = 8 + (i as i32 * 76);
            let is_active = i == self.combat_state.active_index as usize;
            let is_dead = !monster.is_alive();
            let on_cooldown = self.combat_state.swap_cooldowns[i] > 0.0;

            let (bg_color, border_color) = if is_active {
                (Rgb888::new(180, 230, 180), Rgb888::new(100, 180, 100))
            } else if is_dead {
                (Rgb888::new(230, 200, 200), Rgb888::new(180, 140, 140))
            } else if on_cooldown {
                (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
            } else {
                (Rgb888::new(200, 220, 240), Rgb888::new(140, 170, 200))
            };

            let rect = Rectangle::new(Point::new(x, button_y), Size::new(button_width, button_height));
            let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(6, 6)));
            rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(bg_color).build()).draw(display)?;
            rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(border_color).stroke_width(1).build()).draw(display)?;

            let elem_color = Self::element_color(monster.element);
            let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
            Text::new(&Self::element_char(monster.element).to_string(), Point::new(x + 6, button_y + 18), elem_style).draw(display)?;

            let status = if is_active { "ACT" } else if is_dead { "KO" } else if on_cooldown { &format!("{:.0}s", self.combat_state.swap_cooldowns[i]) } else { "SWAP" };
            Text::new(status, Point::new(x + 20, button_y + 18), dim_style).draw(display)?;

            if !is_active && !is_dead { self.swap_button_areas[i] = Some(rect); } else { self.swap_button_areas[i] = None; }
        }

        // Skill button
        let skill_y = 180;
        let skill_ready = self.combat_state.player_skl_bar >= 1.0;

        let (skill_bg, skill_border) = if skill_ready {
            (Rgb888::new(220, 200, 240), Rgb888::new(150, 100, 200))
        } else {
            (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
        };

        let skill_rect = Rectangle::new(Point::new(8, skill_y), Size::new(224, 28));
        let skill_rounded = RoundedRectangle::new(skill_rect, CornerRadii::new(Size::new(6, 6)));
        skill_rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(skill_bg).build()).draw(display)?;
        skill_rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(skill_border).stroke_width(2).build()).draw(display)?;

        let skill_text = if let Some(monster) = self.combat_state.active_monster() {
            let skill_name = if monster.skill.name.len() > 14 { &monster.skill.name[..14] } else { &monster.skill.name };
            if skill_ready { format!("SKILL: {}", skill_name) } else { format!("{} ({}%)", skill_name, (self.combat_state.player_skl_bar * 100.0) as u8) }
        } else { "NO SKILL".to_string() };
        Text::new(&skill_text, Point::new(16, skill_y + 18), text_style).draw(display)?;

        self.skill_button_area = Some(skill_rect);

        // Damage popups
        for popup in &self.damage_popups {
            let popup_color = if popup.is_heal { Rgb888::new(50, 200, 50) } else if popup.is_player_damage { Rgb888::new(50, 50, 50) } else { Rgb888::new(220, 80, 80) };
            let popup_style = MonoTextStyle::new(&FONT_7X13, popup_color);
            let popup_x = if popup.is_player_damage { 180 } else { 140 };
            let popup_y = if popup.is_player_damage { (55.0 - popup.y_offset) as i32 } else { (115.0 - popup.y_offset) as i32 };
            let popup_text = if popup.is_heal { format!("+{}", popup.damage) } else { format!("-{}", popup.damage) };
            Text::new(&popup_text, Point::new(popup_x, popup_y), popup_style).draw(display)?;
        }

        // Wave transition
        if self.combat_state.is_wave_transitioning {
            let wave_style = MonoTextStyle::new(&FONT_7X13, Rgb888::new(200, 150, 50));
            Text::new(&format!("Wave {} cleared!", self.combat_state.current_wave), Point::new(60, 220), wave_style).draw(display)?;
            Text::new("Next wave...", Point::new(80, 240), dim_style).draw(display)?;
        }

        // Combat ended
        if self.combat_state.combat_ended {
            let (msg, msg_color) = if self.combat_state.player_won { ("VICTORY!", Rgb888::new(50, 180, 50)) } else { ("DEFEAT", Rgb888::new(200, 80, 80)) };
            let msg_style = MonoTextStyle::new(&FONT_7X13, msg_color);
            Text::new(msg, Point::new(90, 230), msg_style).draw(display)?;
        }

        display.flush()?;
        self.dirty = false;
        Ok(())
    }

    fn update(&mut self) -> bool {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Cap delta to avoid huge jumps
        let delta = delta.min(0.1);

        // If combat ended, wait for delay before closing
        if self.combat_state.combat_ended {
            self.end_delay += delta;
            self.dirty = true;
            // Wait 2 seconds after combat ends
            return self.end_delay < 2.0;
        }

        // Update combat state
        let events = self.combat_state.update(delta);

        // Handle events
        for event in events {
            self.handle_combat_event(event);
        }

        // Update damage popups
        self.damage_popups.retain_mut(|popup| {
            popup.y_offset += delta * 50.0;
            popup.alpha -= delta * 2.0;
            popup.alpha > 0.0
        });

        // Always need redraw for real-time combat
        self.dirty = true;

        // Return true while combat is active
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
