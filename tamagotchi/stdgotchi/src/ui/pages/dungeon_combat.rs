//! Dungeon Combat Page
//!
//! Real-time combat UI for dungeon battles.
//! Displays enemy, player monster, bars, and action buttons.

use crate::display::Sh8601Driver;
use crate::game::core::Element;
use crate::game::systems::combat::{CombatState, CombatEvent};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
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
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));

        if full_redraw {
            // Clear screen
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(15, 18, 25))?;
        }

        // ═══════════════════════════════════════
        // HEADER: Dungeon name, floor, wave, crystals
        // ═══════════════════════════════════════
        let header_text = format!(
            "{} Fl.{}",
            self.dungeon_name,
            self.combat_state.current_floor,
        );
        Text::new(&header_text, Point::new(15, 25), dim_style).draw(display)?;

        // Wave indicator (show wave N/M)
        let wave_text = format!(
            "Wave {}/{}  +{}",
            self.combat_state.current_wave,
            self.combat_state.total_waves,
            self.combat_state.crystals_earned,
        );
        Text::new(&wave_text, Point::new(200, 25), dim_style).draw(display)?;

        // ═══════════════════════════════════════
        // ENEMY SECTION
        // ═══════════════════════════════════════
        let enemy = &self.combat_state.enemy;
        let enemy_y = 50;

        // Enemy name and element
        let elem_color = Self::element_color(enemy.element);
        let elem_style = MonoTextStyle::new(&FONT_9X15, elem_color);
        let enemy_info = format!(
            "{} {} Lv.{}",
            Self::element_char(enemy.element),
            enemy.name,
            enemy.level
        );
        Text::new(&enemy_info, Point::new(15, enemy_y + 20), elem_style).draw(display)?;

        // Enemy HP bar
        let hp_bar_x = 15;
        let hp_bar_y = enemy_y + 30;
        let bar_width = 250u32;
        let bar_height = 15u32;

        // HP bar background
        let hp_bg = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(bar_width, bar_height));
        display.fill_solid(&hp_bg, Rgb888::new(60, 30, 30))?;

        // HP bar fill
        let hp_percent = enemy.hp_current as f32 / enemy.hp_max as f32;
        let hp_fill_width = ((bar_width as f32) * hp_percent) as u32;
        if hp_fill_width > 0 {
            let hp_fill = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(hp_fill_width, bar_height));
            display.fill_solid(&hp_fill, Rgb888::new(200, 60, 60))?;
        }

        // HP text
        let hp_text = format!("{}/{}", enemy.hp_current, enemy.hp_max);
        Text::new(&hp_text, Point::new(hp_bar_x + bar_width as i32 + 10, hp_bar_y + 12), dim_style).draw(display)?;

        // Enemy SKL bar
        let skl_bar_y = enemy_y + 50;
        let skl_bg = Rectangle::new(Point::new(hp_bar_x, skl_bar_y), Size::new(150, 8));
        display.fill_solid(&skl_bg, Rgb888::new(40, 40, 60))?;

        let skl_fill_width = (150.0 * self.combat_state.enemy_skl_bar) as u32;
        if skl_fill_width > 0 {
            let skl_fill = Rectangle::new(Point::new(hp_bar_x, skl_bar_y), Size::new(skl_fill_width, 8));
            display.fill_solid(&skl_fill, Rgb888::new(150, 100, 200))?;
        }
        Text::new("SKL", Point::new(hp_bar_x + 155, skl_bar_y + 7), dim_style).draw(display)?;

        // Enemy aura indicator
        if let Some((aura_elem, _)) = self.combat_state.enemy_aura {
            let aura_color = Self::element_color(aura_elem);
            let aura_style = MonoTextStyle::new(&FONT_9X15, aura_color);
            Text::new(&format!("[{}]", Self::element_char(aura_elem)), Point::new(280, enemy_y + 20), aura_style).draw(display)?;
        }

        // ═══════════════════════════════════════
        // PLAYER SECTION
        // ═══════════════════════════════════════
        let player_y = 180;

        if let Some(monster) = self.combat_state.active_monster() {
            // Player monster name and element
            let elem_color = Self::element_color(monster.element);
            let elem_style = MonoTextStyle::new(&FONT_9X15, elem_color);
            let player_info = format!(
                "{} {} Lv.{}",
                Self::element_char(monster.element),
                monster.name,
                monster.level
            );
            Text::new(&player_info, Point::new(15, player_y + 20), elem_style).draw(display)?;

            // Player HP bar
            let hp_bar_y = player_y + 30;
            let hp_bg = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(bar_width, bar_height));
            display.fill_solid(&hp_bg, Rgb888::new(30, 60, 30))?;

            let hp_percent = monster.hp_current as f32 / monster.hp_max as f32;
            let hp_fill_width = ((bar_width as f32) * hp_percent) as u32;
            if hp_fill_width > 0 {
                let hp_fill = Rectangle::new(Point::new(hp_bar_x, hp_bar_y), Size::new(hp_fill_width, bar_height));
                display.fill_solid(&hp_fill, Rgb888::new(60, 200, 60))?;
            }

            let hp_text = format!("{}/{}", monster.hp_current, monster.hp_max);
            Text::new(&hp_text, Point::new(hp_bar_x + bar_width as i32 + 10, hp_bar_y + 12), dim_style).draw(display)?;

            // Player ATK bar
            let atk_bar_y = player_y + 50;
            let atk_bg = Rectangle::new(Point::new(hp_bar_x, atk_bar_y), Size::new(200, 10));
            display.fill_solid(&atk_bg, Rgb888::new(40, 40, 40))?;

            let atk_fill_width = (200.0 * self.combat_state.player_atk_bar) as u32;
            if atk_fill_width > 0 {
                let atk_fill = Rectangle::new(Point::new(hp_bar_x, atk_bar_y), Size::new(atk_fill_width, 10));
                display.fill_solid(&atk_fill, Rgb888::new(255, 200, 100))?;
            }
            Text::new("ATK", Point::new(hp_bar_x + 205, atk_bar_y + 9), dim_style).draw(display)?;

            // Player SKL bar
            let skl_bar_y = player_y + 65;
            let skl_bg = Rectangle::new(Point::new(hp_bar_x, skl_bar_y), Size::new(200, 10));
            display.fill_solid(&skl_bg, Rgb888::new(40, 40, 60))?;

            let skl_fill_width = (200.0 * self.combat_state.player_skl_bar) as u32;
            if skl_fill_width > 0 {
                let skl_color = if self.combat_state.player_skl_bar >= 1.0 {
                    Rgb888::new(255, 200, 255) // Ready to use
                } else {
                    Rgb888::new(150, 100, 200)
                };
                let skl_fill = Rectangle::new(Point::new(hp_bar_x, skl_bar_y), Size::new(skl_fill_width, 10));
                display.fill_solid(&skl_fill, skl_color)?;
            }
            Text::new("SKL", Point::new(hp_bar_x + 205, skl_bar_y + 9), dim_style).draw(display)?;
        }

        // ═══════════════════════════════════════
        // ACTION BUTTONS
        // ═══════════════════════════════════════
        let button_y = 320;
        let button_width = 100u32;
        let button_height = 40u32;

        // Swap buttons for team monsters
        for (i, monster) in self.combat_state.player_monsters.iter().take(3).enumerate() {
            let x = 15 + (i as i32 * 115);
            let is_active = i == self.combat_state.active_index as usize;
            let is_dead = !monster.is_alive();
            let on_cooldown = self.combat_state.swap_cooldowns[i] > 0.0;

            let bg_color = if is_active {
                Rgb888::new(60, 100, 60) // Active
            } else if is_dead {
                Rgb888::new(50, 30, 30) // Dead
            } else if on_cooldown {
                Rgb888::new(50, 50, 50) // Cooldown
            } else {
                Rgb888::new(40, 60, 80) // Available
            };

            let rect = Rectangle::new(Point::new(x, button_y), Size::new(button_width, button_height));
            display.fill_solid(&rect, bg_color)?;

            // Element icon and name
            let elem_color = Self::element_color(monster.element);
            let elem_style = MonoTextStyle::new(&FONT_9X15, elem_color);
            Text::new(&Self::element_char(monster.element).to_string(), Point::new(x + 10, button_y + 25), elem_style).draw(display)?;

            // Status text
            let status = if is_active {
                "ACTIVE"
            } else if is_dead {
                "DEAD"
            } else if on_cooldown {
                &format!("{:.0}s", self.combat_state.swap_cooldowns[i])
            } else {
                "SWAP"
            };
            Text::new(status, Point::new(x + 30, button_y + 25), dim_style).draw(display)?;

            if !is_active && !is_dead {
                self.swap_button_areas[i] = Some(rect);
            } else {
                self.swap_button_areas[i] = None;
            }
        }

        // Skill button
        let skill_x = 15;
        let skill_y = 380;
        let skill_ready = self.combat_state.player_skl_bar >= 1.0;

        let skill_color = if skill_ready {
            Rgb888::new(100, 60, 150) // Ready
        } else {
            Rgb888::new(40, 40, 50) // Not ready
        };

        let skill_rect = Rectangle::new(Point::new(skill_x, skill_y), Size::new(200, 45));
        display.fill_solid(&skill_rect, skill_color)?;

        let skill_text = if let Some(monster) = self.combat_state.active_monster() {
            if skill_ready {
                format!("SKILL: {}", monster.skill.name)
            } else {
                format!("{} ({}%)", monster.skill.name, (self.combat_state.player_skl_bar * 100.0) as u8)
            }
        } else {
            "NO SKILL".to_string()
        };
        Text::new(&skill_text, Point::new(skill_x + 10, skill_y + 28), text_style).draw(display)?;

        self.skill_button_area = Some(skill_rect);

        // Draw damage/heal popups
        for popup in &self.damage_popups {
            let popup_color = if popup.is_heal {
                Rgb888::new(100, 255, 100) // Green for heal
            } else if popup.is_player_damage {
                Rgb888::WHITE // White for damage to enemy
            } else {
                Rgb888::new(255, 100, 100) // Red for damage to player
            };

            let popup_style = MonoTextStyle::new(&FONT_10X20, popup_color);
            let popup_x = if popup.is_player_damage { 280 } else { 200 }; // Enemy side vs player side
            let popup_y = if popup.is_player_damage {
                (80.0 - popup.y_offset) as i32 // Enemy area
            } else {
                (220.0 - popup.y_offset) as i32 // Player area
            };

            let popup_text = if popup.is_heal {
                format!("+{}", popup.damage)
            } else {
                format!("-{}", popup.damage)
            };
            Text::new(&popup_text, Point::new(popup_x, popup_y), popup_style).draw(display)?;
        }

        // Wave transition message
        if self.combat_state.is_wave_transitioning {
            let wave_msg = format!(
                "Wave {} cleared!",
                self.combat_state.current_wave
            );
            let wave_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 220, 100));
            Text::new(&wave_msg, Point::new(100, 130), wave_style).draw(display)?;

            let next_msg = "Next wave incoming...";
            let next_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(180, 180, 180));
            Text::new(next_msg, Point::new(100, 155), next_style).draw(display)?;
        }

        // Combat ended message
        if self.combat_state.combat_ended {
            let msg = if self.combat_state.player_won {
                "VICTORY!"
            } else {
                "DEFEAT"
            };
            let msg_color = if self.combat_state.player_won {
                Rgb888::new(100, 255, 100)
            } else {
                Rgb888::new(255, 100, 100)
            };
            let msg_style = MonoTextStyle::new(&FONT_10X20, msg_color);
            Text::new(msg, Point::new(140, 150), msg_style).draw(display)?;
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
