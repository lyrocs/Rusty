//! Dungeon Combat Page
//!
//! Real-time combat UI for dungeon battles with GIF animations.
//! Uses shared canvas for memory efficiency - only ONE canvas buffer in RAM.
//!
//! Layout:
//! - Top: Enemy stats (left) | Player monster stats (right)
//! - Middle: Enemy GIF (left) | Player GIF (right)
//! - Bottom: Swap buttons | Skill button

use crate::display::{St7789pDriver, GifMeta, SharedCanvas};
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

/// Which monster is performing an action animation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveAnim {
    None,
    Enemy,
    Player,
}

/// Type of animation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimType {
    Idle,
    Attack,
    Hurt,
    Death,
}

// Static GIF data - stays in flash, never copied to RAM
static PORING_IDLE: &[u8] = include_bytes!("../../../assets/images/poring/6.gif");
static PORING_ATTACK: &[u8] = include_bytes!("../../../assets/images/poring/22.gif");
static PORING_HURT: &[u8] = include_bytes!("../../../assets/images/poring/30.gif");
static PORING_DEATH: &[u8] = include_bytes!("../../../assets/images/poring/38.gif");

static FABRE_IDLE: &[u8] = include_bytes!("../../../assets/images/fabre/6.gif");
static FABRE_ATTACK: &[u8] = include_bytes!("../../../assets/images/fabre/22.gif");
static FABRE_HURT: &[u8] = include_bytes!("../../../assets/images/fabre/30.gif");
static FABRE_DEATH: &[u8] = include_bytes!("../../../assets/images/fabre/38.gif");

static HORNET_IDLE: &[u8] = include_bytes!("../../../assets/images/hornet/6.gif");
static HORNET_ATTACK: &[u8] = include_bytes!("../../../assets/images/hornet/22.gif");
static HORNET_HURT: &[u8] = include_bytes!("../../../assets/images/hornet/30.gif");
static HORNET_DEATH: &[u8] = include_bytes!("../../../assets/images/hornet/38.gif");

static THIEF_BUG_IDLE: &[u8] = include_bytes!("../../../assets/images/thief_bug/6.gif");
static THIEF_BUG_ATTACK: &[u8] = include_bytes!("../../../assets/images/thief_bug/22.gif");
static THIEF_BUG_HURT: &[u8] = include_bytes!("../../../assets/images/thief_bug/30.gif");
static THIEF_BUG_DEATH: &[u8] = include_bytes!("../../../assets/images/thief_bug/38.gif");

/// Get GIF data for a species and animation type (returns static reference to flash)
fn get_gif_data(species_id: &str, anim_type: AnimType) -> Option<&'static [u8]> {
    match (species_id, anim_type) {
        ("poring", AnimType::Idle) => Some(PORING_IDLE),
        ("poring", AnimType::Attack) => Some(PORING_ATTACK),
        ("poring", AnimType::Hurt) => Some(PORING_HURT),
        ("poring", AnimType::Death) => Some(PORING_DEATH),
        ("fabre", AnimType::Idle) => Some(FABRE_IDLE),
        ("fabre", AnimType::Attack) => Some(FABRE_ATTACK),
        ("fabre", AnimType::Hurt) => Some(FABRE_HURT),
        ("fabre", AnimType::Death) => Some(FABRE_DEATH),
        ("hornet", AnimType::Idle) => Some(HORNET_IDLE),
        ("hornet", AnimType::Attack) => Some(HORNET_ATTACK),
        ("hornet", AnimType::Hurt) => Some(HORNET_HURT),
        ("hornet", AnimType::Death) => Some(HORNET_DEATH),
        ("thief_bug", AnimType::Idle) => Some(THIEF_BUG_IDLE),
        ("thief_bug", AnimType::Attack) => Some(THIEF_BUG_ATTACK),
        ("thief_bug", AnimType::Hurt) => Some(THIEF_BUG_HURT),
        ("thief_bug", AnimType::Death) => Some(THIEF_BUG_DEATH),
        _ => None,
    }
}

/// Load GIF metadata (lightweight, no canvas allocation)
fn load_gif_meta(species_id: &str, anim_type: AnimType) -> Option<GifMeta> {
    get_gif_data(species_id, anim_type).and_then(|data| GifMeta::new(data).ok())
}

/// Action from combat page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonCombatAction {
    None,
    UseSkill,
    SwapMonster(u8),
    CombatEnded { victory: bool },
}

/// Dungeon combat page with memory-efficient animations
///
/// Memory layout:
/// - SharedCanvas: ONE buffer ~25KB (80×80×4)
/// - GifMeta: lightweight metadata per GIF (~100 bytes each)
/// - Total: ~25KB instead of ~75KB with multiple GifPlayers
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

    // Species IDs for loading animations
    enemy_species: String,
    player_species: String,

    // SHARED canvas - only ONE buffer in RAM!
    shared_canvas: SharedCanvas,

    // Lightweight GIF metadata (no canvas allocation)
    enemy_gif: Option<GifMeta>,
    player_gif: Option<GifMeta>,

    // Current animation state
    enemy_anim_type: AnimType,
    player_anim_type: AnimType,

    // Frame tracking
    enemy_frame: usize,
    player_frame: usize,
    frame_timer: f32,

    // Action animation state
    action_target: ActiveAnim,
    action_timer: f32,
}

struct DamagePopup {
    damage: u16,
    is_player_damage: bool,
    is_heal: bool,
    y_offset: f32,
    alpha: f32,
}

impl DungeonCombatPage {
    pub fn new(combat_state: CombatState, dungeon_name: String) -> Self {
        let enemy_species = combat_state.enemy.species_id.clone();
        let player_species = combat_state.active_monster()
            .map(|m| m.species_id.clone())
            .unwrap_or_else(|| "poring".to_string());

        // Create ONE shared canvas - the only significant RAM allocation!
        let shared_canvas = SharedCanvas::new(80, 80);

        // Load lightweight GIF metadata (no canvas allocation)
        let enemy_gif = load_gif_meta(&enemy_species, AnimType::Idle);
        let player_gif = load_gif_meta(&player_species, AnimType::Idle);

        Self {
            combat_state,
            last_update: Instant::now(),
            dirty: true,
            skill_button_area: None,
            swap_button_areas: [None; 3],
            damage_popups: Vec::new(),
            dungeon_name,
            end_delay: 0.0,
            enemy_species,
            player_species,
            shared_canvas,
            enemy_gif,
            player_gif,
            enemy_anim_type: AnimType::Idle,
            player_anim_type: AnimType::Idle,
            enemy_frame: 0,
            player_frame: 0,
            frame_timer: 0.0,
            action_target: ActiveAnim::None,
            action_timer: 0.0,
        }
    }

    /// Switch animation type for a monster (reloads GIF metadata, not canvas)
    fn set_animation(&mut self, target: ActiveAnim, anim_type: AnimType) {
        match target {
            ActiveAnim::Enemy => {
                if self.enemy_anim_type != anim_type {
                    self.enemy_gif = load_gif_meta(&self.enemy_species, anim_type);
                    self.enemy_anim_type = anim_type;
                    self.enemy_frame = 0;
                }
            }
            ActiveAnim::Player => {
                if self.player_anim_type != anim_type {
                    self.player_gif = load_gif_meta(&self.player_species, anim_type);
                    self.player_anim_type = anim_type;
                    self.player_frame = 0;
                }
            }
            _ => {}
        }
        self.action_target = target;
        self.action_timer = 0.0;
    }

    /// End action animation and return to idle
    fn end_action(&mut self) {
        match self.action_target {
            ActiveAnim::Enemy => {
                self.enemy_gif = load_gif_meta(&self.enemy_species, AnimType::Idle);
                self.enemy_anim_type = AnimType::Idle;
                self.enemy_frame = 0;
            }
            ActiveAnim::Player => {
                self.player_gif = load_gif_meta(&self.player_species, AnimType::Idle);
                self.player_anim_type = AnimType::Idle;
                self.player_frame = 0;
            }
            _ => {}
        }
        self.action_target = ActiveAnim::None;
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> DungeonCombatAction {
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
                // Player attacks - show player attack animation
                if self.action_target == ActiveAnim::None {
                    self.set_animation(ActiveAnim::Player, AnimType::Attack);
                }
                self.damage_popups.push(DamagePopup {
                    damage,
                    is_player_damage: true,
                    is_heal: false,
                    y_offset: 0.0,
                    alpha: 1.0,
                });
            }
            CombatEvent::EnemyAttack { damage, .. } => {
                // Enemy attacks - show enemy attack animation
                if self.action_target == ActiveAnim::None {
                    self.set_animation(ActiveAnim::Enemy, AnimType::Attack);
                }
                self.damage_popups.push(DamagePopup {
                    damage,
                    is_player_damage: false,
                    is_heal: false,
                    y_offset: 0.0,
                    alpha: 1.0,
                });
            }
            CombatEvent::PlayerSkill { damage, .. } => {
                if self.action_target == ActiveAnim::None {
                    self.set_animation(ActiveAnim::Player, AnimType::Attack);
                }
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
                    is_player_damage: false,
                    is_heal: true,
                    y_offset: 0.0,
                    alpha: 1.0,
                });
            }
            CombatEvent::MonsterSwap { .. } => {
                // Update player species and reload metadata
                if let Some(monster) = self.combat_state.active_monster() {
                    if monster.species_id != self.player_species {
                        self.player_species = monster.species_id.clone();
                        self.player_gif = load_gif_meta(&self.player_species, AnimType::Idle);
                        self.player_anim_type = AnimType::Idle;
                        self.player_frame = 0;
                    }
                }
            }
            CombatEvent::Victory { .. } => {
                if self.action_target == ActiveAnim::None {
                    self.set_animation(ActiveAnim::Enemy, AnimType::Death);
                }
            }
            CombatEvent::Defeat => {
                if self.action_target == ActiveAnim::None {
                    self.set_animation(ActiveAnim::Player, AnimType::Death);
                }
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

    pub fn combat_state(&self) -> &CombatState {
        &self.combat_state
    }
}

impl Page for DungeonCombatPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));

        if full_redraw {
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        // ===== TOP ROW: Header =====
        let header_rect = Rectangle::new(Point::new(5, 2), Size::new(230, 18));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(4, 4)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(180, 140, 140))
            .build())
            .draw(display)?;

        let dungeon_name = if self.dungeon_name.len() > 10 { &self.dungeon_name[..10] } else { &self.dungeon_name };
        let header_text = format!("{} F{} W{}/{}", dungeon_name, self.combat_state.current_floor,
            self.combat_state.current_wave, self.combat_state.total_waves);
        Text::new(&header_text, Point::new(10, 14), text_style).draw(display)?;

        let reward_text = format!("+{}", self.combat_state.crystals_earned);
        Text::new(&reward_text, Point::new(200, 14), text_style).draw(display)?;

        // ===== STATS ROW =====
        let stats_y = 24;
        let card_height = 42u32;
        let card_width = 114u32;

        // Enemy stats card
        let enemy = &self.combat_state.enemy;
        let enemy_card = Rectangle::new(Point::new(4, stats_y), Size::new(card_width, card_height));
        let enemy_rounded = RoundedRectangle::new(enemy_card, CornerRadii::new(Size::new(5, 5)));
        enemy_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(255, 235, 235))
            .build())
            .draw(display)?;
        enemy_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(200, 150, 150))
            .stroke_width(1)
            .build())
            .draw(display)?;

        let elem_color = Self::element_color(enemy.element);
        let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
        let enemy_name = if enemy.name.len() > 8 { &enemy.name[..8] } else { &enemy.name };
        Text::new(&format!("{}{}", Self::element_char(enemy.element), enemy_name),
            Point::new(8, stats_y + 11), elem_style).draw(display)?;
        Text::new(&format!("L{}", enemy.level), Point::new(85, stats_y + 11), dim_style).draw(display)?;

        // Enemy HP bar
        let bar_x = 8;
        let bar_y = stats_y + 16;
        let bar_w = 100u32;
        let bar_h = 8u32;
        let hp_bg = Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_w, bar_h));
        display.fill_solid(&hp_bg, Rgb888::new(200, 180, 180))?;
        let hp_pct = enemy.hp_current as f32 / enemy.hp_max as f32;
        let hp_fill_w = ((bar_w as f32) * hp_pct) as u32;
        if hp_fill_w > 0 {
            let hp_fill = Rectangle::new(Point::new(bar_x, bar_y), Size::new(hp_fill_w, bar_h));
            display.fill_solid(&hp_fill, Rgb888::new(220, 80, 80))?;
        }

        // Enemy SKL bar
        let skl_y = bar_y + 10;
        let skl_bg = Rectangle::new(Point::new(bar_x, skl_y), Size::new(bar_w, 5));
        display.fill_solid(&skl_bg, Rgb888::new(200, 200, 220))?;
        let skl_fill_w = (bar_w as f32 * self.combat_state.enemy_skl_bar) as u32;
        if skl_fill_w > 0 {
            let skl_fill = Rectangle::new(Point::new(bar_x, skl_y), Size::new(skl_fill_w, 5));
            display.fill_solid(&skl_fill, Rgb888::new(150, 100, 200))?;
        }

        // Player stats card
        if let Some(monster) = self.combat_state.active_monster() {
            let player_card = Rectangle::new(Point::new(122, stats_y), Size::new(card_width, card_height));
            let player_rounded = RoundedRectangle::new(player_card, CornerRadii::new(Size::new(5, 5)));
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
            let monster_name = if monster.name.len() > 8 { &monster.name[..8] } else { &monster.name };
            Text::new(&format!("{}{}", Self::element_char(monster.element), monster_name),
                Point::new(126, stats_y + 11), elem_style).draw(display)?;
            Text::new(&format!("L{}", monster.level), Point::new(203, stats_y + 11), dim_style).draw(display)?;

            let p_bar_x = 126;
            let hp_bg = Rectangle::new(Point::new(p_bar_x, bar_y), Size::new(bar_w, bar_h));
            display.fill_solid(&hp_bg, Rgb888::new(180, 200, 180))?;
            let hp_pct = monster.hp_current as f32 / monster.hp_max as f32;
            let hp_fill_w = ((bar_w as f32) * hp_pct) as u32;
            if hp_fill_w > 0 {
                let hp_fill = Rectangle::new(Point::new(p_bar_x, bar_y), Size::new(hp_fill_w, bar_h));
                display.fill_solid(&hp_fill, Rgb888::new(80, 200, 80))?;
            }

            let atk_y = bar_y + 10;
            let atk_bg = Rectangle::new(Point::new(p_bar_x, atk_y), Size::new(bar_w, 5));
            display.fill_solid(&atk_bg, Rgb888::new(200, 200, 200))?;
            let atk_fill_w = (bar_w as f32 * self.combat_state.player_atk_bar) as u32;
            if atk_fill_w > 0 {
                let atk_fill = Rectangle::new(Point::new(p_bar_x, atk_y), Size::new(atk_fill_w, 5));
                display.fill_solid(&atk_fill, Rgb888::new(255, 180, 80))?;
            }
        }

        // ===== MIDDLE: Animation area =====
        let anim_y = 70;
        let anim_h = 130;

        let anim_bg = Rectangle::new(Point::new(0, anim_y), Size::new(240, anim_h as u32));
        display.fill_solid(&anim_bg, Rgb888::new(240, 240, 245))?;

        let enemy_x = 60;
        let player_x = 180;
        let center_y = anim_y + anim_h / 2;

        // Render enemy GIF using shared canvas
        if let Some(ref gif) = self.enemy_gif {
            let frame = self.enemy_frame.min(gif.frame_count().saturating_sub(1));
            gif.render_frame(display, frame, &mut self.shared_canvas, Some((enemy_x, center_y)), false, true)?;
        }

        // Render player GIF using shared canvas (flipped)
        if let Some(ref gif) = self.player_gif {
            let frame = self.player_frame.min(gif.frame_count().saturating_sub(1));
            gif.render_frame(display, frame, &mut self.shared_canvas, Some((player_x, center_y)), true, true)?;
        }

        // Damage popups
        for popup in &self.damage_popups {
            let popup_color = if popup.is_heal {
                Rgb888::new(50, 200, 50)
            } else if popup.is_player_damage {
                Rgb888::new(50, 50, 50)
            } else {
                Rgb888::new(220, 80, 80)
            };
            let popup_style = MonoTextStyle::new(&FONT_7X13, popup_color);
            let popup_x = if popup.is_player_damage { enemy_x - 15 } else { player_x - 15 };
            let popup_y = (center_y as f32 - 20.0 - popup.y_offset) as i32;
            let popup_text = if popup.is_heal { format!("+{}", popup.damage) } else { format!("-{}", popup.damage) };
            Text::new(&popup_text, Point::new(popup_x, popup_y), popup_style).draw(display)?;
        }

        // ===== BOTTOM: Swap buttons + Skill =====
        let bottom_y = 204;
        let swap_y = bottom_y;
        let swap_btn_w = 74u32;
        let swap_btn_h = 32u32;

        for (i, monster) in self.combat_state.player_monsters.iter().take(3).enumerate() {
            let x = 4 + (i as i32 * 78);
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

            let rect = Rectangle::new(Point::new(x, swap_y), Size::new(swap_btn_w, swap_btn_h));
            let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(6, 6)));
            rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(bg_color).build()).draw(display)?;
            rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(border_color).stroke_width(1).build()).draw(display)?;

            let elem_color = Self::element_color(monster.element);
            let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
            let name = if monster.name.len() > 6 { &monster.name[..6] } else { &monster.name };
            Text::new(&format!("{}{}", Self::element_char(monster.element), name),
                Point::new(x + 4, swap_y + 12), elem_style).draw(display)?;

            let status = if is_active { "ACTIVE" } else if is_dead { "KO" }
                else if on_cooldown { &format!("{:.0}s", self.combat_state.swap_cooldowns[i]) }
                else { "SWAP" };
            Text::new(status, Point::new(x + 4, swap_y + 26), dim_style).draw(display)?;

            if !is_active && !is_dead {
                self.swap_button_areas[i] = Some(rect);
            } else {
                self.swap_button_areas[i] = None;
            }
        }

        // Skill button
        let skill_y = swap_y + swap_btn_h as i32 + 4;
        let skill_ready = self.combat_state.player_skl_bar >= 1.0;

        let (skill_bg, skill_border) = if skill_ready {
            (Rgb888::new(220, 200, 240), Rgb888::new(150, 100, 200))
        } else {
            (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
        };

        let skill_rect = Rectangle::new(Point::new(4, skill_y), Size::new(232, 36));
        let skill_rounded = RoundedRectangle::new(skill_rect, CornerRadii::new(Size::new(6, 6)));
        skill_rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(skill_bg).build()).draw(display)?;
        skill_rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(skill_border).stroke_width(2).build()).draw(display)?;

        if let Some(monster) = self.combat_state.active_monster() {
            let skill_name = if monster.skill.name.len() > 16 { &monster.skill.name[..16] } else { &monster.skill.name };
            let skill_text = if skill_ready {
                format!("SKILL: {}", skill_name)
            } else {
                format!("{}", skill_name)
            };
            Text::new(&skill_text, Point::new(12, skill_y + 14), text_style).draw(display)?;

            let skl_bar_y = skill_y + 20;
            let skl_bar_w = 210u32;
            let skl_bg = Rectangle::new(Point::new(12, skl_bar_y), Size::new(skl_bar_w, 8));
            display.fill_solid(&skl_bg, Rgb888::new(200, 200, 220))?;

            let skl_fill_w = (skl_bar_w as f32 * self.combat_state.player_skl_bar.min(1.0)) as u32;
            if skl_fill_w > 0 {
                let skl_color = if skill_ready { Rgb888::new(220, 150, 255) } else { Rgb888::new(150, 100, 200) };
                let skl_fill = Rectangle::new(Point::new(12, skl_bar_y), Size::new(skl_fill_w, 8));
                display.fill_solid(&skl_fill, skl_color)?;
            }
        }

        self.skill_button_area = Some(skill_rect);

        // Wave transition
        if self.combat_state.is_wave_transitioning {
            let wave_style = MonoTextStyle::new(&FONT_7X13, Rgb888::new(200, 150, 50));
            Text::new(&format!("Wave {} cleared!", self.combat_state.current_wave),
                Point::new(60, anim_y + anim_h / 2), wave_style).draw(display)?;
        }

        // Combat ended
        if self.combat_state.combat_ended {
            let (msg, msg_color) = if self.combat_state.player_won {
                ("VICTORY!", Rgb888::new(50, 180, 50))
            } else {
                ("DEFEAT", Rgb888::new(200, 80, 80))
            };
            let msg_style = MonoTextStyle::new(&FONT_7X13, msg_color);
            Text::new(msg, Point::new(90, anim_y + anim_h / 2), msg_style).draw(display)?;
        }

        display.flush()?;
        self.dirty = false;
        Ok(())
    }

    fn update(&mut self) -> bool {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        let delta = delta.min(0.1);

        // Combat ended delay
        if self.combat_state.combat_ended {
            self.end_delay += delta;
            self.dirty = true;
            return self.end_delay < 2.0;
        }

        // Update combat state
        let events = self.combat_state.update(delta);
        for event in events {
            self.handle_combat_event(event);
        }

        // Update action animation timer
        if self.action_target != ActiveAnim::None {
            self.action_timer += delta;

            // Get frame count for the active animation
            let (frame_count, is_death) = match self.action_target {
                ActiveAnim::Enemy => (
                    self.enemy_gif.as_ref().map(|g| g.frame_count()).unwrap_or(1),
                    self.enemy_anim_type == AnimType::Death
                ),
                ActiveAnim::Player => (
                    self.player_gif.as_ref().map(|g| g.frame_count()).unwrap_or(1),
                    self.player_anim_type == AnimType::Death
                ),
                _ => (1, false),
            };

            // Advance action frame at ~15 FPS
            if self.action_timer >= 0.067 {
                self.action_timer = 0.0;

                match self.action_target {
                    ActiveAnim::Enemy => self.enemy_frame += 1,
                    ActiveAnim::Player => self.player_frame += 1,
                    _ => {}
                }

                // Check if animation complete
                let current_frame = match self.action_target {
                    ActiveAnim::Enemy => self.enemy_frame,
                    ActiveAnim::Player => self.player_frame,
                    _ => 0,
                };

                if current_frame >= frame_count {
                    if is_death {
                        // Stay on last frame for death
                        match self.action_target {
                            ActiveAnim::Enemy => self.enemy_frame = frame_count.saturating_sub(1),
                            ActiveAnim::Player => self.player_frame = frame_count.saturating_sub(1),
                            _ => {}
                        }
                    } else {
                        self.end_action();
                    }
                }
            }
        }

        // Update idle frame animations at ~10 FPS
        self.frame_timer += delta;
        if self.frame_timer >= 0.1 {
            self.frame_timer = 0.0;

            // Only advance idle animations (not action animations)
            if self.action_target != ActiveAnim::Enemy {
                if let Some(ref gif) = self.enemy_gif {
                    if self.enemy_anim_type == AnimType::Idle {
                        self.enemy_frame = (self.enemy_frame + 1) % gif.frame_count().max(1);
                    }
                }
            }
            if self.action_target != ActiveAnim::Player {
                if let Some(ref gif) = self.player_gif {
                    if self.player_anim_type == AnimType::Idle {
                        self.player_frame = (self.player_frame + 1) % gif.frame_count().max(1);
                    }
                }
            }
        }

        // Update damage popups
        self.damage_popups.retain_mut(|popup| {
            popup.y_offset += delta * 50.0;
            popup.alpha -= delta * 2.0;
            popup.alpha > 0.0
        });

        self.dirty = true;
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
