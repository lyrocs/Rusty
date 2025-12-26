//! Dungeon Combat Page
//!
//! Real-time combat UI for dungeon battles with GIF animations.
//! Uses shared canvas for memory efficiency - only ONE canvas buffer in RAM.
//! Loads sprites from SD card at runtime.
//!
//! Layout:
//! - Top: Enemy stats (left) | Player monster stats (right)
//! - Middle: Enemy GIF (left) | Player GIF (right)
//! - Bottom: Swap buttons | Skill button

use crate::assets::{get_monster_raw_path, SpriteAction, SpriteCache};
use crate::display::{St7789pDriver, RawAnimPlayer};
use crate::ecs::resources::SdCardWrapper;
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
use std::collections::HashMap;
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

/// Turn-based combat phase
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurnPhase {
    /// Determining who goes first based on SPD
    DeterminingTurnOrder,
    /// Player's turn - waiting for action selection
    PlayerSelectAction,
    /// Player action is executing (animation playing)
    PlayerActionExecuting { action_type: TurnAction, timer: f32 },
    /// Brief pause after player action to show damage (0.5s)
    PlayerActionComplete { timer: f32 },
    /// Enemy's turn - executing attack
    EnemyActionExecuting { timer: f32 },
    /// Brief pause after enemy action to show damage (0.5s)
    EnemyActionComplete { timer: f32 },
    /// Showing battle message (Pokemon-style text)
    ShowingBattleMessage { timer: f32 },
    /// Combat ended (victory or defeat)
    CombatEnded { victory: bool },
}

/// Player action for turn-based combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnAction {
    /// Use skill from slot (0, 1, or 2)
    UseSkill { slot: u8 },
    /// Swap to another monster
    Swap { target_index: u8 },
}

impl AnimType {
    fn to_sprite_action(self) -> SpriteAction {
        match self {
            AnimType::Idle => SpriteAction::Idle,
            AnimType::Attack => SpriteAction::Attack,
            AnimType::Hurt => SpriteAction::Attacked,
            AnimType::Death => SpriteAction::Death,
        }
    }

    /// Get array index for this animation type
    fn index(self) -> usize {
        match self {
            AnimType::Idle => 0,
            AnimType::Attack => 1,
            AnimType::Hurt => 2,
            AnimType::Death => 3,
        }
    }
}

/// Load raw animation from SD card as RawAnimPlayer
/// Uses different strategies based on animation type:
/// - Idle: Load full file (small, fast playback)
/// - Action animations: Use streaming (large files)
fn load_raw_from_sd(sd_card: &mut SdCardWrapper, species_id: &str, anim_type: AnimType) -> Option<RawAnimPlayer> {
    use crate::assets::{get_monster_raw_path, SpriteAction};

    let path = get_monster_raw_path(species_id, anim_type.to_sprite_action());

    match anim_type {
        AnimType::Idle => {
            // For idle animations, load the entire file (small, ~13KB)
            // This gives smooth playback without per-frame SD card reads
            log::info!("Loading full raw animation: {} {:?}", species_id, anim_type);
            let data = sd_card.load_binary_file(&path).ok()?;
            log::info!("Loaded {} bytes for {} idle animation", data.len(), species_id);
            RawAnimPlayer::from_file_data(data)
        }
        _ => {
            // For action animations (Attack/Hurt/Death), use streaming
            // These can be large (>100KB) and would fail to allocate
            log::info!("Loading streaming raw animation: {} {:?}", species_id, anim_type);
            let mut player = crate::assets::load_streaming_raw_animation(sd_card, species_id, anim_type.to_sprite_action())?;

            // Load initial frame (frame 0) for immediate rendering
            if let Err(e) = player.load_frame(0, sd_card) {
                log::warn!("Failed to load initial frame for {} {:?}: {}", species_id, anim_type, e);
                return None;
            }

            Some(player)
        }
    }
}

/// Action from combat page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonCombatAction {
    None,
    UseSkill,
    SwapMonster(u8),
    CombatEnded { victory: bool },
}

/// Animation timing constants
const ACTION_ANIM_DURATION: f32 = 0.4;  // Time for attack/action animation
const POST_ACTION_DELAY: f32 = 0.5;     // Time to show damage before next turn

/// Dungeon combat page with fast raw RGB565 animations
///
/// Memory layout (optimized for ESP32-C6 with no PSRAM):
/// - Animation cache: HashMap<species_id, RawAnimPlayer> (~13KB per unique species)
/// - Lazy loading: Only enemy + active player loaded at combat start
/// - Other team members loaded on-demand when swapped in
/// - Cache prevents duplicate loading of same species
pub struct DungeonCombatPage {
    combat_state: CombatState,
    last_update: Instant,
    dirty: bool,

    // Touch areas for turn-based actions
    skill_button_areas: [Option<Rectangle>; 3],  // 3 skill buttons
    swap_button_area: Option<Rectangle>,

    // Swap popup (when swap button tapped)
    show_swap_popup: bool,
    swap_popup_buttons: [Option<Rectangle>; 3],

    // Damage feedback
    damage_popups: Vec<DamagePopup>,

    // Dungeon info
    dungeon_name: String,

    // End delay timer (seconds)
    end_delay: f32,

    // Animation cache: species_id -> RawAnimPlayer
    // Lazy loaded: enemy + active player at start, others on-demand during swaps
    anim_cache: HashMap<String, RawAnimPlayer>,

    // Current animation state
    enemy_anim_type: AnimType,
    player_anim_type: AnimType,

    // Pending animation loads (currently unused with preloading)
    pending_enemy_anim: Option<AnimType>,

    // Action animation state
    action_target: ActiveAnim,
    action_timer: f32,

    // Loading state - shows "Loading..." before animations are loaded
    // Two-phase: is_loading=true, loading_drawn=false → draw "Loading..."
    //            is_loading=true, loading_drawn=true → ready to load animations
    is_loading: bool,
    loading_drawn: bool,

    // Player reload flag - set when active player changes and animation not cached
    needs_player_anim_reload: bool,

    // Death animation state
    // When enemy dies: spin and fly off screen to the left
    death_anim_active: bool,
    death_anim_timer: f32,
    death_anim_species: String,  // Species of the dying enemy (for rendering)

    // Reaction popup (displayed when elemental reaction triggers)
    reaction_popup: Option<ReactionPopup>,

    // Turn-based combat state
    turn_phase: TurnPhase,
    player_turn_counter: f32,  // SPD accumulator for turn order
    enemy_turn_counter: f32,   // SPD accumulator for turn order
    last_actor_was_player: bool,  // For tie-breaking equal SPD
    last_player_action: Option<TurnAction>,  // Track last action for swap penalty

    // Turn indicator message
    action_message: Option<String>,
    message_timer: f32,

    // Battle message queue (Pokemon-style)
    battle_messages: Vec<String>,
    current_battle_message: Option<String>,
    pending_victory: bool,
    pending_defeat: bool,
    last_was_enemy_turn: bool,  // Track if we came from enemy turn

    // Pending skill action - delays animation until damage message is shown
    // Stores the skill slot to execute after "X uses Y!" message
    pending_skill_action: Option<u8>,
    // Pending enemy action - delays animation until damage message
    pending_enemy_action: bool,
}

struct DamagePopup {
    damage: u16,
    is_player_damage: bool,
    is_heal: bool,
    y_offset: f32,
    alpha: f32,
}

struct ReactionPopup {
    name: String,
    timer: f32,  // Seconds remaining to display
}

impl DungeonCombatPage {
    /// Create combat page in loading state (animations loaded later via load_initial_animations)
    /// This allows showing "Loading..." screen before the slow SD card reads
    pub fn new(combat_state: CombatState, dungeon_name: String) -> Self {
        // Don't load animations yet - will be done in load_initial_animations()
        Self {
            combat_state,
            last_update: Instant::now(),
            dirty: true,
            skill_button_areas: [None; 3],
            swap_button_area: None,
            show_swap_popup: false,
            swap_popup_buttons: [None; 3],
            damage_popups: Vec::new(),
            dungeon_name,
            end_delay: 0.0,
            anim_cache: HashMap::new(),
            enemy_anim_type: AnimType::Idle,
            player_anim_type: AnimType::Idle,
            pending_enemy_anim: None,
            action_target: ActiveAnim::None,
            action_timer: 0.0,
            is_loading: true,  // Start in loading state
            loading_drawn: false,  // Not drawn yet - wait for first render
            needs_player_anim_reload: false,  // No reload needed initially
            death_anim_active: false,
            death_anim_timer: 0.0,
            death_anim_species: String::new(),
            reaction_popup: None,
            // Turn-based state
            turn_phase: TurnPhase::DeterminingTurnOrder,
            player_turn_counter: 0.0,
            enemy_turn_counter: 0.0,
            last_actor_was_player: false,
            last_player_action: None,
            action_message: None,
            message_timer: 0.0,
            // Battle messages
            battle_messages: Vec::new(),
            current_battle_message: None,
            pending_victory: false,
            pending_defeat: false,
            last_was_enemy_turn: false,
            // Pending actions for delayed animation
            pending_skill_action: None,
            pending_enemy_action: false,
        }
    }

    /// Check if the page needs initial animation loading
    /// Only returns true AFTER the loading screen has been drawn at least once
    pub fn needs_initial_load(&self) -> bool {
        self.is_loading && self.loading_drawn
    }

    /// Load only immediately needed animations (enemy + active player) - lazy loading approach
    /// Other team monsters are loaded on-demand when swapped in
    /// Call this after showing the loading screen
    pub fn load_initial_animations(&mut self, sd_card: &mut SdCardWrapper) {
        if !self.is_loading {
            return;
        }

        // Log combat stats for debugging
        self.log_combat_stats();

        // Load only what's immediately visible:
        // 1. Current enemy
        // 2. Active player monster (index 0)

        // Load enemy animation
        let enemy_species = &self.combat_state.enemy.species_id;
        if let Some(anim) = load_raw_from_sd(sd_card, enemy_species, AnimType::Idle) {
            log::info!("Loaded enemy animation: {}", enemy_species);
            self.anim_cache.insert(enemy_species.clone(), anim);
        } else {
            log::warn!("Failed to load enemy animation: {}", enemy_species);
        }

        // Load active player monster animation
        if let Some(active_monster) = self.combat_state.active_monster() {
            let player_species = &active_monster.species_id;
            if !self.anim_cache.contains_key(player_species) {
                if let Some(anim) = load_raw_from_sd(sd_card, player_species, AnimType::Idle) {
                    log::info!("Loaded active player animation: {}", player_species);
                    self.anim_cache.insert(player_species.clone(), anim);
                } else {
                    log::warn!("Failed to load active player animation: {}", player_species);
                }
            }
        }

        self.is_loading = false;
        self.dirty = true;
        log::info!("Initial combat animations loaded ({} cached), ready to fight!", self.anim_cache.len());
    }

    /// Check if player animation needs reload (after swap to uncached monster)
    pub fn needs_player_reload(&self) -> bool {
        self.needs_player_anim_reload
    }

    /// Reload player species animation (after swap)
    /// With lazy loading, this loads the animation on-demand if not already cached
    pub fn reload_player_species(&mut self, sd_card: &mut SdCardWrapper) {
        // Check if the new active player's animation is already cached
        if let Some(active_monster) = self.combat_state.active_monster() {
            let species_id = &active_monster.species_id;

            if self.anim_cache.contains_key(species_id) {
                log::info!("Player swap: using cached animation for {}", species_id);
            } else {
                // Not cached - load it now (lazy loading)
                log::info!("Player swap: loading animation on-demand for {}", species_id);
                if let Some(anim) = load_raw_from_sd(sd_card, species_id, AnimType::Idle) {
                    self.anim_cache.insert(species_id.clone(), anim);
                    log::info!("Loaded on-demand animation for {}", species_id);
                } else {
                    log::warn!("Failed to load on-demand animation for {}", species_id);
                }
            }
        }

        // Clear reload flag
        self.needs_player_anim_reload = false;
    }

    /// Reload enemy species animation (for new wave)
    /// Note: With cache, this is now a no-op since all wave enemies are preloaded
    pub fn reload_enemy_species(&mut self, _sd_card: &mut SdCardWrapper) {
        // With single enemy per floor, this is now a no-op
        // Kept for API compatibility
        log::info!("Enemy species reload: {}", self.combat_state.enemy.species_id);
        self.enemy_anim_type = AnimType::Idle;
    }

    /// Check if enemy needs animation reload (new wave)
    /// Note: With cache, this always returns false since all enemies are preloaded
    pub fn needs_enemy_reload(&self) -> bool {
        // All wave enemies are preloaded, so no reload needed
        false
    }

    /// Check if any animation needs its current frame loaded from SD card
    pub fn needs_frame_reload(&self) -> bool {
        // Check enemy animation
        let enemy_species = &self.combat_state.enemy.species_id;
        let enemy_needs = self.anim_cache.get(enemy_species)
            .map(|a| a.needs_frame_load()).unwrap_or(false);

        // Check active player animation
        if let Some(monster) = self.combat_state.active_monster() {
            let player_needs = self.anim_cache.get(&monster.species_id)
                .map(|a| a.needs_frame_load()).unwrap_or(false);
            return enemy_needs || player_needs;
        }

        enemy_needs
    }

    /// Reload current frames for animations that need it (streaming playback)
    pub fn reload_needed_frames(&mut self, sd_card: &mut SdCardWrapper) {
        // Reload enemy animation current frame if needed
        let enemy_species = self.combat_state.enemy.species_id.clone();
        if let Some(anim) = self.anim_cache.get_mut(&enemy_species) {
            if anim.needs_frame_load() {
                let current_frame = anim.current_frame();
                if let Err(e) = anim.load_frame(current_frame, sd_card) {
                    log::warn!("Failed to reload enemy frame {}: {}", current_frame, e);
                }
            }
        }

        // Reload active player animation current frame if needed
        if let Some(monster) = self.combat_state.active_monster() {
            let player_species = monster.species_id.clone();
            if let Some(anim) = self.anim_cache.get_mut(&player_species) {
                if anim.needs_frame_load() {
                    let current_frame = anim.current_frame();
                    if let Err(e) = anim.load_frame(current_frame, sd_card) {
                        log::warn!("Failed to reload player frame {}: {}", current_frame, e);
                    }
                }
            }
        }
    }

    /// Check if there are pending animation loads
    /// Note: With cache-based preloading, this is always false
    pub fn has_pending_animations(&self) -> bool {
        // All animations are preloaded at combat start
        false
    }

    /// Load pending animations from SD card
    /// Note: With cache-based preloading, this is a no-op
    pub fn load_pending_animations(&mut self, _sd_card: &mut SdCardWrapper) {
        // All animations are preloaded at combat start, nothing to do
    }

    /// Start the death animation for the current enemy
    /// Call this when enemy HP reaches 0
    pub fn start_death_animation(&mut self) {
        if !self.death_anim_active {
            self.death_anim_active = true;
            self.death_anim_timer = 0.0;
            self.death_anim_species = self.combat_state.enemy.species_id.clone();
            log::info!("Starting death animation for {}", self.death_anim_species);
        }
    }

    /// Check if death animation is currently playing
    pub fn is_death_anim_playing(&self) -> bool {
        self.death_anim_active
    }

    /// Death animation duration in seconds
    const DEATH_ANIM_DURATION: f32 = 0.8;

    /// End action animation and return to idle
    /// Note: We no longer free idle animations, so no reload needed
    fn end_action_to_idle(&mut self) {
        self.action_target = ActiveAnim::None;
    }

    /// Queue an animation action (uses idle animation with visual effects)
    /// Note: We don't actually load action animations due to SD card speed constraints.
    /// Instead, we use the idle animation with position offsets (lunge effect).
    fn queue_animation(&mut self, target: ActiveAnim, _anim_type: AnimType) {
        // Set action state for visual effect (lunge)
        self.action_target = target;
        self.action_timer = 0.0;
    }

    // ===== TURN-BASED COMBAT METHODS =====

    /// Determine who goes next based on SPD
    /// Returns true if player goes first
    fn determine_next_turn(&mut self) -> bool {
        let player_spd = self.combat_state.active_monster()
            .map(|m| m.spd as f32)
            .unwrap_or(0.0);
        let enemy_spd = self.combat_state.enemy.spd as f32;

        // Both accumulate SPD
        self.player_turn_counter += player_spd;
        self.enemy_turn_counter += enemy_spd;

        const TURN_THRESHOLD: f32 = 100.0;

        let player_ready = self.player_turn_counter >= TURN_THRESHOLD;
        let enemy_ready = self.enemy_turn_counter >= TURN_THRESHOLD;

        match (player_ready, enemy_ready) {
            (true, true) => {
                // Both ready - higher counter goes first
                let player_goes = self.player_turn_counter >= self.enemy_turn_counter;
                if player_goes {
                    self.player_turn_counter -= TURN_THRESHOLD;
                } else {
                    self.enemy_turn_counter -= TURN_THRESHOLD;
                }
                player_goes
            }
            (true, false) => {
                self.player_turn_counter -= TURN_THRESHOLD;
                true
            }
            (false, true) => {
                self.enemy_turn_counter -= TURN_THRESHOLD;
                false
            }
            (false, false) => {
                // Neither ready - keep accumulating (recursive)
                self.determine_next_turn()
            }
        }
    }

    /// Check if player can use a skill at the given slot
    /// Returns true if:
    /// - Monster is alive
    /// - Skill exists at that slot
    /// - Skill is not on cooldown
    fn can_use_skill_slot(&self, slot: u8) -> bool {
        let monster = match self.combat_state.active_monster() {
            Some(m) if m.is_alive() => m,
            _ => return false,
        };

        // Check if skill exists at this slot
        if slot as usize >= monster.equipped_skills.len() {
            return false;
        }

        // Check if skill is on cooldown
        !monster.is_skill_on_cooldown(slot as usize)
    }

    /// Check if player can swap (any other alive teammate)
    fn can_swap(&self) -> bool {
        self.combat_state.player_monsters.iter()
            .enumerate()
            .any(|(i, m)| i != self.combat_state.active_index as usize && m.is_alive())
    }

    /// Execute player skill action from a specific slot
    /// Shows "X uses Y!" message first, then plays animation during damage message
    fn execute_player_skill_slot(&mut self, slot: u8) {
        if !self.can_use_skill_slot(slot) { return; }

        // Queue the "uses skill" message first
        if let Some(monster) = self.combat_state.active_monster() {
            let skill_name = monster.equipped_skills.get(slot as usize)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "Skill".to_string());
            self.queue_battle_message(format!("{} uses {}!", monster.name, skill_name));
        }

        // Store pending action - animation will play when damage message shows
        self.pending_skill_action = Some(slot);
        self.last_player_action = Some(TurnAction::UseSkill { slot });

        // Start showing the "uses skill" message (no animation yet)
        self.start_battle_messages();
    }

    /// Execute player swap action
    fn execute_player_swap(&mut self, target_index: u8) {
        // Verify swap is valid
        if target_index >= self.combat_state.player_monsters.len() as u8 { return; }
        if !self.combat_state.player_monsters[target_index as usize].is_alive() { return; }
        if target_index == self.combat_state.active_index { return; }

        self.action_message = Some("Swapping...".to_string());
        self.message_timer = 0.5;

        self.turn_phase = TurnPhase::PlayerActionExecuting {
            action_type: TurnAction::Swap { target_index },
            timer: 0.0,
        };
        // No lunge animation for swap
        self.action_target = ActiveAnim::None;
        self.last_player_action = Some(TurnAction::Swap { target_index });
        self.show_swap_popup = false;
    }

    /// Execute the actual action effect (damage, heal, etc.)
    /// Note: Skill effects are now applied in next_battle_message() for delayed animation
    fn apply_action_effect(&mut self, action_type: TurnAction) {
        match action_type {
            TurnAction::UseSkill { .. } => {
                // Skill effects are now handled in next_battle_message()
                // This is called after animation completes - nothing more to do
            }
            TurnAction::Swap { target_index } => {
                // Perform the swap
                let old_index = self.combat_state.active_index;
                self.combat_state.active_index = target_index;
                let event = CombatEvent::MonsterSwap {
                    from_index: old_index,
                    to_index: target_index,
                };
                self.handle_combat_event(event);
            }
        }
    }

    /// Execute turn-based attack (no bars, direct damage)
    fn execute_turn_attack(&mut self) -> Option<CombatEvent> {
        use crate::game::calculations::combat::update_skl_bar_after_attack;
        use crate::game::calculations::damage::calculate_final_damage;

        let monster = self.combat_state.active_monster()?;
        if !monster.is_alive() { return None; }

        let atk = monster.atk;
        let element = monster.element;
        let def = self.combat_state.enemy.def;
        let enemy_element = self.combat_state.enemy.element;

        // Check for reaction
        let (reaction_mult, reaction_name, heal_amount) = self.check_reaction(element);
        let damage = calculate_final_damage(atk, def, element, enemy_element, reaction_mult);

        // Apply damage to enemy
        self.combat_state.enemy.take_damage(damage);

        // Apply aura to enemy
        self.combat_state.enemy_aura = Some(element);

        // Update skill bar (gains 20% per attack)
        self.combat_state.player_skl_bar = update_skl_bar_after_attack(self.combat_state.player_skl_bar);

        self.last_actor_was_player = true;

        Some(CombatEvent::PlayerAttack {
            damage,
            element,
            reaction: reaction_name,
            heal_amount,
        })
    }

    /// Execute skill at specified slot
    fn execute_skill_at_slot(&mut self, slot: u8) -> Option<CombatEvent> {
        use crate::game::calculations::damage::{calculate_skill_damage_full, calculate_final_damage};
        use crate::game::core::SkillEffectType;

        let monster = self.combat_state.active_monster()?;
        if !monster.is_alive() { return None; }

        let skill = monster.equipped_skills.get(slot as usize)?.clone();
        let monster_atk = monster.atk;
        let skill_element = skill.element;
        let enemy_def = self.combat_state.enemy.def;
        let enemy_element = self.combat_state.enemy.element;

        // Start cooldown for this skill
        if let Some(monster_mut) = self.combat_state.active_monster_mut() {
            monster_mut.start_skill_cooldown(slot as usize);
        }

        self.last_actor_was_player = true;

        // Handle different skill types
        match skill.effect_type {
            SkillEffectType::Heal => {
                // Heal skill - heal active monster by percentage of max HP
                let monster_mut = self.combat_state.active_monster_mut()?;
                let heal_amount = (monster_mut.hp_max as f32 * skill.effect_value) as u16;
                let old_hp = monster_mut.hp_current;
                monster_mut.hp_current = (monster_mut.hp_current + heal_amount).min(monster_mut.hp_max);
                let actual_heal = monster_mut.hp_current - old_hp;

                log::info!("Heal skill used: {} healed for {} HP", skill.name, actual_heal);

                Some(CombatEvent::PlayerSkillHeal {
                    skill_name: skill.name.clone(),
                    heal_amount: actual_heal,
                })
            }
            SkillEffectType::Buff => {
                // Buff skill - for now just log it
                log::info!("Buff skill used: {}", skill.name);
                Some(CombatEvent::PlayerSkill {
                    skill_name: skill.name.clone(),
                    damage: 0,
                    element: skill_element,
                    is_critical: false,
                    is_miss: false,
                })
            }
            SkillEffectType::Debuff => {
                // Debuff skill - for now just log it
                log::info!("Debuff skill used: {}", skill.name);
                Some(CombatEvent::PlayerSkill {
                    skill_name: skill.name.clone(),
                    damage: 0,
                    element: skill_element,
                    is_critical: false,
                    is_miss: false,
                })
            }
            _ => {
                // Damage skills (Damage, DamageDot, DamageIgnoreDef)
                // Adjust DEF for ignore-def skills
                let def_modifier = match skill.effect_type {
                    SkillEffectType::DamageIgnoreDef => 0.5, // Ignore 50% DEF
                    _ => 1.0,
                };
                let effective_def = (enemy_def as f32 * def_modifier) as u16;

                // Check for reaction
                let (reaction_mult, reaction_name, heal_amount) = self.check_reaction(skill_element);

                // Use the full damage calculation with accuracy, crit, and all modifiers
                let damage_result = calculate_skill_damage_full(
                    monster_atk,
                    effective_def,
                    skill.power,
                    skill.accuracy,
                    skill.crit_chance,
                    skill_element,
                    enemy_element,
                    reaction_mult,
                );

                // Check for miss
                if damage_result.is_miss {
                    log::info!("Skill {} missed! (acc: {}%)", skill.name, skill.accuracy);
                    return Some(CombatEvent::PlayerSkill {
                        skill_name: skill.name.clone(),
                        damage: 0,
                        element: skill_element,
                        is_critical: false,
                        is_miss: true,
                    });
                }

                // Apply damage to enemy
                self.combat_state.enemy.take_damage(damage_result.damage);

                // Apply aura to enemy (for reactions)
                self.combat_state.enemy_aura = Some(skill_element);

                // Log crit/damage info
                if damage_result.is_critical {
                    log::info!("CRITICAL! {} dealt {} damage (crit: {}%)",
                        skill.name, damage_result.damage, skill.crit_chance);
                } else {
                    log::info!("Skill {} dealt {} damage", skill.name, damage_result.damage);
                }

                // Handle DoT if applicable
                if skill.dot_damage > 0 && skill.dot_duration > 0 {
                    log::info!("Skill {} applied DoT: {} dmg for {} turns",
                        skill.name, skill.dot_damage, skill.dot_duration);
                }

                // Handle reaction popup
                if let Some(ref name) = reaction_name {
                    self.reaction_popup = Some(ReactionPopup {
                        name: name.clone(),
                        timer: 1.5,
                    });
                }

                // Handle reaction heal if BLOOM
                if let Some(heal) = heal_amount {
                    self.damage_popups.push(DamagePopup {
                        damage: heal,
                        is_player_damage: true,
                        is_heal: true,
                        y_offset: 0.0,
                        alpha: 1.0,
                    });
                }

                Some(CombatEvent::PlayerSkill {
                    skill_name: skill.name.clone(),
                    damage: damage_result.damage,
                    element: skill_element,
                    is_critical: damage_result.is_critical,
                    is_miss: false,
                })
            }
        }
    }

    /// Execute enemy turn attack
    fn execute_enemy_turn(&mut self) -> Option<CombatEvent> {
        use crate::game::calculations::damage::calculate_final_damage;

        if !self.combat_state.enemy.is_alive() { return None; }

        let monster = self.combat_state.active_monster()?;
        if !monster.is_alive() { return None; }

        let enemy_atk = self.combat_state.enemy.atk;
        let enemy_element = self.combat_state.enemy.element;
        let def = monster.def;
        let monster_element = monster.element;

        let damage = calculate_final_damage(enemy_atk, def, enemy_element, monster_element, 1.0);

        // Apply damage to player monster
        if let Some(monster) = self.combat_state.active_monster_mut() {
            monster.take_damage(damage);
        }

        self.last_actor_was_player = false;

        Some(CombatEvent::EnemyAttack { damage, element: enemy_element })
    }

    /// Check for elemental reaction (copy from CombatState for turn-based use)
    fn check_reaction(&mut self, attack_element: Element) -> (f32, Option<String>, Option<u16>) {
        if let Some(aura_element) = self.combat_state.enemy_aura {
            let (mult, name, heal) = match (aura_element, attack_element) {
                // Water aura + Fire = VAPORIZE (x2 damage)
                (Element::Water, Element::Fire) | (Element::Fire, Element::Water) => (2.0, Some("VAPORIZE"), None),
                // Water aura + Thunder = ELECTROCUTE (stun - no effect in turn-based)
                (Element::Water, Element::Thunder) => (1.5, Some("ELECTROCUTE"), None),
                // Water aura + Earth = BLOOM (heal team 15%)
                (Element::Water, Element::Earth) => {
                    let heal_amount = self.calculate_bloom_heal();
                    (1.0, Some("BLOOM"), Some(heal_amount))
                },
                _ => (1.0, None, None),
            };

            if name.is_some() {
                self.combat_state.enemy_aura = None;
            }

            (mult, name.map(|s| s.to_string()), heal)
        } else {
            (1.0, None, None)
        }
    }

    /// Calculate and apply BLOOM heal
    fn calculate_bloom_heal(&mut self) -> u16 {
        let mut total_healed = 0u16;
        for monster in &mut self.combat_state.player_monsters {
            if monster.is_alive() {
                let heal_amount = (monster.hp_max as f32 * 0.15) as u16;
                let old_hp = monster.hp_current;
                monster.hp_current = (monster.hp_current + heal_amount).min(monster.hp_max);
                total_healed += monster.hp_current - old_hp;
            }
        }
        total_healed
    }

    /// Handle enemy death (victory)
    /// Queue a battle message (Pokemon-style)
    fn queue_battle_message(&mut self, message: String) {
        self.battle_messages.push(message);
    }

    /// Start showing battle messages
    fn start_battle_messages(&mut self) {
        if !self.battle_messages.is_empty() {
            self.current_battle_message = Some(self.battle_messages.remove(0));
            self.turn_phase = TurnPhase::ShowingBattleMessage { timer: 0.0 };
        }
    }

    /// Advance to next battle message or end messages
    fn next_battle_message(&mut self) {
        // Check if there's a pending skill action to execute
        // This happens after showing "X uses Y!" - now we execute and show damage
        if let Some(slot) = self.pending_skill_action.take() {
            // Start player animation
            self.action_target = ActiveAnim::Player;

            // Execute the skill and queue damage messages
            if let Some(event) = self.execute_skill_at_slot(slot) {
                // Queue battle messages based on event (damage, miss, crit, heal)
                match &event {
                    CombatEvent::PlayerSkill { damage, is_critical, is_miss, .. } => {
                        if *is_miss {
                            self.queue_battle_message("But it missed!".to_string());
                        } else if *damage > 0 {
                            if *is_critical {
                                self.queue_battle_message("A critical hit!".to_string());
                            }
                            self.queue_battle_message(format!("Dealt {} damage!", damage));
                        }
                    }
                    CombatEvent::PlayerSkillHeal { heal_amount, .. } => {
                        self.queue_battle_message(format!("Recovered {} HP!", heal_amount));
                    }
                    _ => {}
                }
                self.handle_combat_event(event);
            }

            // Check if enemy died
            if !self.combat_state.enemy.is_alive() {
                self.handle_enemy_death();
                return;
            }

            // Start animation phase - animation plays during damage message
            self.turn_phase = TurnPhase::PlayerActionExecuting {
                action_type: TurnAction::UseSkill { slot },
                timer: 0.0,
            };
            return;
        }

        // Check if there's a pending enemy action to execute
        if self.pending_enemy_action {
            self.pending_enemy_action = false;

            // Start enemy animation
            self.action_target = ActiveAnim::Enemy;

            // Execute enemy attack and queue damage message
            self.execute_enemy_attack_effect();

            // Check if player died
            if self.combat_state.all_players_dead() {
                self.handle_player_defeat();
                return;
            }

            // Start animation phase
            self.turn_phase = TurnPhase::EnemyActionExecuting { timer: 0.0 };
            return;
        }

        if !self.battle_messages.is_empty() {
            self.current_battle_message = Some(self.battle_messages.remove(0));
            self.turn_phase = TurnPhase::ShowingBattleMessage { timer: 0.0 };
        } else {
            self.current_battle_message = None;
            if self.pending_victory {
                // All messages shown, now show victory
                self.turn_phase = TurnPhase::CombatEnded { victory: true };
                self.combat_state.combat_ended = true;
                self.combat_state.player_won = true;
                self.pending_victory = false;
            } else if self.pending_defeat {
                // All messages shown, now show defeat
                self.turn_phase = TurnPhase::CombatEnded { victory: false };
                self.combat_state.combat_ended = true;
                self.combat_state.player_won = false;
                self.pending_defeat = false;
            } else if self.last_was_enemy_turn {
                // Continue combat after enemy turn
                self.last_was_enemy_turn = false;
                self.turn_phase = TurnPhase::EnemyActionComplete { timer: 0.0 };
            } else {
                // Continue combat after player turn
                self.turn_phase = TurnPhase::PlayerActionComplete { timer: 0.0 };
            }
        }
    }

    fn handle_enemy_death(&mut self) {
        // Calculate and award rewards for this floor
        let base_crystals = 10 + (self.combat_state.current_floor as u32 / 3);
        let base_xp = 50 + (self.combat_state.current_floor as u32 * 15);

        if self.combat_state.is_boss_floor {
            self.combat_state.crystals_earned = base_crystals * 3;
            self.combat_state.xp_earned = base_xp * 2;
        } else {
            self.combat_state.crystals_earned = base_crystals;
            self.combat_state.xp_earned = base_xp;
        }

        log::info!("Enemy defeated on floor {}! Rewards: {} crystals, {} XP",
            self.combat_state.current_floor,
            self.combat_state.crystals_earned,
            self.combat_state.xp_earned);

        // Queue victory messages
        let enemy_name = self.combat_state.enemy.name.clone();
        self.queue_battle_message(format!("{} fainted!", enemy_name));
        self.queue_battle_message("You won the battle!".to_string());

        // Mark pending victory and start showing messages
        self.pending_victory = true;
        self.start_death_animation();
        self.start_battle_messages();
    }

    /// Handle player defeat (all monsters fainted)
    fn handle_player_defeat(&mut self) {
        log::info!("Player defeated on floor {}", self.combat_state.current_floor);

        // Queue defeat messages
        if let Some(monster) = self.combat_state.active_monster() {
            self.queue_battle_message(format!("{} fainted!", monster.name));
        }
        self.queue_battle_message("You have no more monsters!".to_string());
        self.queue_battle_message("You lost the battle...".to_string());

        // Mark pending defeat and start showing messages
        self.pending_defeat = true;
        self.start_battle_messages();
    }

    /// Queue enemy attack message (first part - no animation yet)
    fn queue_enemy_attack_message(&mut self) {
        let enemy_name = self.combat_state.enemy.name.clone();
        self.queue_battle_message(format!("{} attacks!", enemy_name));
        self.pending_enemy_action = true;
        self.start_battle_messages();
    }

    /// Execute enemy attack effect (called after "X attacks!" message)
    fn execute_enemy_attack_effect(&mut self) {
        if let Some(event) = self.execute_enemy_turn() {
            // Queue damage message
            match &event {
                CombatEvent::EnemyAttack { damage, .. } => {
                    self.queue_battle_message(format!("Dealt {} damage!", damage));
                }
                _ => {}
            }
            self.handle_combat_event(event);
        }
    }

    /// Handle touch input (turn-based version)
    pub fn handle_touch(&mut self, x: i32, y: i32) -> DungeonCombatAction {
        let point = Point::new(x, y);

        // Only handle touch during PlayerSelectAction phase
        if self.turn_phase != TurnPhase::PlayerSelectAction {
            // Allow closing swap popup even outside player turn
            if self.show_swap_popup {
                // Check if tap is outside popup - close it
                let popup_area = Rectangle::new(Point::new(20, 100), Size::new(200, 100));
                if !popup_area.contains(point) {
                    self.show_swap_popup = false;
                    self.dirty = true;
                }
            }
            return DungeonCombatAction::None;
        }

        // Check swap popup first (if visible)
        if self.show_swap_popup {
            for (i, area) in self.swap_popup_buttons.iter().enumerate() {
                if let Some(rect) = area {
                    if rect.contains(point) {
                        // Execute swap
                        self.execute_player_swap(i as u8);
                        self.dirty = true;
                        return DungeonCombatAction::SwapMonster(i as u8);
                    }
                }
            }
            // Tap outside popup closes it
            self.show_swap_popup = false;
            self.dirty = true;
            return DungeonCombatAction::None;
        }

        // Check skill buttons (3 slots)
        for slot in 0..3 {
            if let Some(rect) = self.skill_button_areas[slot] {
                if rect.contains(point) && self.can_use_skill_slot(slot as u8) {
                    self.execute_player_skill_slot(slot as u8);
                    self.dirty = true;
                    return DungeonCombatAction::UseSkill;
                }
            }
        }

        // Check swap button
        if let Some(rect) = self.swap_button_area {
            if rect.contains(point) && self.can_swap() {
                self.show_swap_popup = true;
                self.dirty = true;
                return DungeonCombatAction::None;
            }
        }

        DungeonCombatAction::None
    }

    fn handle_combat_event(&mut self, event: CombatEvent) {
        match event {
            CombatEvent::PlayerAttack { damage, reaction, heal_amount, .. } => {
                // Player attacks - queue player attack animation
                if self.action_target == ActiveAnim::None {
                    self.queue_animation(ActiveAnim::Player, AnimType::Attack);
                }
                self.damage_popups.push(DamagePopup {
                    damage,
                    is_player_damage: true,
                    is_heal: false,
                    y_offset: 0.0,
                    alpha: 1.0,
                });

                // Show reaction popup if a reaction occurred
                if let Some(reaction_name) = reaction {
                    self.reaction_popup = Some(ReactionPopup {
                        name: reaction_name,
                        timer: 1.5, // Show for 1.5 seconds
                    });
                }

                // Show heal popup if BLOOM healed the team
                if let Some(heal) = heal_amount {
                    if heal > 0 {
                        self.damage_popups.push(DamagePopup {
                            damage: heal,
                            is_player_damage: false,
                            is_heal: true,
                            y_offset: 0.0,
                            alpha: 1.0,
                        });
                    }
                }
            }
            CombatEvent::EnemyAttack { damage, .. } => {
                // Enemy attacks - queue enemy attack animation
                if self.action_target == ActiveAnim::None {
                    self.queue_animation(ActiveAnim::Enemy, AnimType::Attack);
                }
                self.damage_popups.push(DamagePopup {
                    damage,
                    is_player_damage: false,
                    is_heal: false,
                    y_offset: 0.0,
                    alpha: 1.0,
                });
            }
            CombatEvent::PlayerSkill { damage, is_miss, .. } => {
                if self.action_target == ActiveAnim::None {
                    self.queue_animation(ActiveAnim::Player, AnimType::Attack);
                }
                // Only show damage popup if not a miss
                if !is_miss && damage > 0 {
                    self.damage_popups.push(DamagePopup {
                        damage,
                        is_player_damage: true,
                        is_heal: false,
                        y_offset: 0.0,
                        alpha: 1.0,
                    });
                }
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
                // With lazy loading, check if new monster's animation is cached
                if let Some(monster) = self.combat_state.active_monster() {
                    if self.anim_cache.contains_key(&monster.species_id) {
                        log::info!("Monster swap: using cached animation for {} (slot {})",
                            monster.species_id, self.combat_state.active_index);
                    } else {
                        log::info!("Monster swap: need to load animation for {} (slot {})",
                            monster.species_id, self.combat_state.active_index);
                        self.needs_player_anim_reload = true;
                    }
                }
            }
            CombatEvent::Victory { .. } => {
                // Enemy died - start death animation
                log::info!("Victory! Starting death animation");
                self.start_death_animation();
            }
            CombatEvent::Defeat => {
                if self.action_target == ActiveAnim::None {
                    self.queue_animation(ActiveAnim::Player, AnimType::Death);
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
            Element::Neutral => Rgb888::new(180, 180, 180),
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
            Element::Neutral => 'N',
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

    /// Log combat stats for debugging
    fn log_combat_stats(&self) {
        use crate::game::systems::dungeon::floor_gen::floor_stat_multiplier;

        let floor_mult = floor_stat_multiplier(self.combat_state.current_floor);
        log::info!("=== COMBAT STATS ===");
        log::info!("Floor: {} (Boss: {}) - Enemy stat multiplier: {:.0}%",
            self.combat_state.current_floor, self.combat_state.is_boss_floor, floor_mult * 100.0);

        // Enemy stats (already have floor multiplier applied)
        let enemy = &self.combat_state.enemy;
        log::info!("ENEMY: {} (Lv{}) [{:?}]", enemy.name, enemy.level, enemy.element);
        log::info!("  HP: {}/{}", enemy.hp_current, enemy.hp_max);
        log::info!("  ATK: {}, DEF: {}, SPD: {}", enemy.atk, enemy.def, enemy.spd);

        // Player monster stats (NO floor multiplier - full stats)
        log::info!("PLAYER TEAM ({} monsters):", self.combat_state.player_monsters.len());
        for (i, monster) in self.combat_state.player_monsters.iter().enumerate() {
            let active_marker = if i == self.combat_state.active_index as usize { " [ACTIVE]" } else { "" };
            log::info!("  #{}{}: {} (Lv{}) [{:?}]", i, active_marker, monster.name, monster.level, monster.element);
            log::info!("    HP: {}/{}", monster.hp_current, monster.hp_max);
            log::info!("    ATK: {}, DEF: {}, SPD: {}", monster.atk, monster.def, monster.spd);
            log::info!("    Fusion: +{}", monster.fusion_count);
        }
        log::info!("====================");
    }
}

impl Page for DungeonCombatPage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Show loading screen while animations are being loaded
        if self.is_loading {
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(40, 40, 50))?;

            let loading_style = MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE);
            Text::new("Loading...", Point::new(85, 142), loading_style).draw(display)?;

            display.flush()?;

            // Mark that loading screen has been drawn - allows navigation system to load on next frame
            self.loading_drawn = true;
            return Ok(());
        }

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

        let dungeon_name = if self.dungeon_name.len() > 12 { &self.dungeon_name[..12] } else { &self.dungeon_name };
        let floor_type = if self.combat_state.is_boss_floor { "BOSS" } else { "F" };
        let header_text = format!("{} {}{}", dungeon_name, floor_type, self.combat_state.current_floor);
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

        // Enemy aura indicator (small colored box next to HP bar)
        if let Some(aura_element) = self.combat_state.enemy_aura {
            let aura_color = Self::element_color(aura_element);
            let aura_x = bar_x + bar_w as i32 + 4;
            let aura_rect = Rectangle::new(Point::new(aura_x, bar_y), Size::new(8, 8));
            display.fill_solid(&aura_rect, aura_color)?;
            // Draw border
            let border_style = PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::WHITE)
                .stroke_width(1)
                .build();
            aura_rect.into_styled(border_style).draw(display)?;
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

            // SKL bar (skill gauge) for turn-based
            let skl_y = bar_y + 10;
            let skill_ready = self.combat_state.player_skl_bar >= 1.0;
            let skl_bg = Rectangle::new(Point::new(p_bar_x, skl_y), Size::new(bar_w, 5));
            display.fill_solid(&skl_bg, Rgb888::new(200, 200, 220))?;
            let skl_fill_w = (bar_w as f32 * self.combat_state.player_skl_bar.min(1.0)) as u32;
            if skl_fill_w > 0 {
                let skl_color = if skill_ready { Rgb888::new(220, 150, 255) } else { Rgb888::new(150, 100, 200) };
                let skl_fill = Rectangle::new(Point::new(p_bar_x, skl_y), Size::new(skl_fill_w, 5));
                display.fill_solid(&skl_fill, skl_color)?;
            }
        }

        // ===== MIDDLE: Animation area =====
        let anim_y = 70;
        let anim_h = 130;

        let anim_bg = Rectangle::new(Point::new(0, anim_y), Size::new(240, anim_h as u32));
        display.fill_solid(&anim_bg, Rgb888::new(240, 240, 245))?;

        let base_enemy_x = 60;
        let base_player_x = 180;
        let center_y = anim_y + anim_h / 2;

        // Calculate lunge offset based on action state
        // Lunge forward (toward opponent) when attacking
        let lunge_amount = 25; // pixels to lunge
        let (enemy_offset, player_offset) = match self.action_target {
            ActiveAnim::Enemy => (lunge_amount, 0),   // Enemy lunges toward player (right)
            ActiveAnim::Player => (0, -lunge_amount), // Player lunges toward enemy (left)
            ActiveAnim::None => (0, 0),
        };

        let enemy_x = base_enemy_x + enemy_offset;
        let player_x = base_player_x + player_offset;

        // Render enemy animation (or death animation if playing)
        // Don't render enemy at all if they're dead and death animation is complete
        let enemy_is_dead = !self.combat_state.enemy.is_alive();

        if self.death_anim_active {
            // Death animation: spin and fly off to the left
            let progress = (self.death_anim_timer / Self::DEATH_ANIM_DURATION).min(1.0);

            // Position: start at base_enemy_x, fly off screen to the left
            let death_x = base_enemy_x - (progress * 150.0) as i32;

            // Vertical bob: sin wave for a wobbly effect
            let bob_y = (progress * 12.0 * std::f32::consts::PI).sin() * 10.0;
            let death_y = center_y + bob_y as i32;

            // Spin effect: alternate flip_h rapidly (every ~0.1 seconds)
            let spin_flip = ((self.death_anim_timer * 10.0) as i32 % 2) == 0;

            // Render the dying enemy (using stored species from when death started)
            if let Some(anim) = self.anim_cache.get(&self.death_anim_species) {
                // Only render if still on screen
                if death_x > -80 {
                    anim.render(display, death_x, death_y, spin_flip);
                }
            }
        } else if !enemy_is_dead {
            // Normal enemy rendering (only if alive)
            let enemy_species = &self.combat_state.enemy.species_id;
            if let Some(anim) = self.anim_cache.get(enemy_species) {
                anim.render(display, enemy_x, center_y, false);
            }
        }
        // If enemy is dead and death animation is complete, don't render anything

        // Render player animation using raw format (flipped)
        // Look up from cache by active monster's species_id
        if let Some(monster) = self.combat_state.active_monster() {
            if let Some(anim) = self.anim_cache.get(&monster.species_id) {
                anim.render(display, player_x, center_y, true);
            }
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

        // Reaction popup (centered, above animations)
        if let Some(ref reaction) = self.reaction_popup {
            // Get reaction color based on name
            let reaction_color = match reaction.name.as_str() {
                "VAPORIZE" => Rgb888::new(255, 150, 50),   // Orange
                "BLOOM" => Rgb888::new(100, 220, 100),     // Green
                "ELECTROCUTE" => Rgb888::new(255, 255, 100), // Yellow
                _ => Rgb888::new(255, 255, 255),          // White default
            };

            // Draw background box
            let text_len = reaction.name.len() as u32 * 7 + 10;
            let box_x = 120 - (text_len as i32 / 2);
            let box_rect = Rectangle::new(Point::new(box_x, anim_y + 10), Size::new(text_len, 18));
            let box_rounded = RoundedRectangle::new(box_rect, CornerRadii::new(Size::new(4, 4)));
            box_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(40, 40, 50))
                .build())
                .draw(display)?;
            box_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(reaction_color)
                .stroke_width(2)
                .build())
                .draw(display)?;

            // Draw reaction text
            let reaction_style = MonoTextStyle::new(&FONT_7X13, reaction_color);
            let text_x = 120 - (reaction.name.len() as i32 * 7 / 2);
            Text::new(&reaction.name, Point::new(text_x, anim_y + 23), reaction_style).draw(display)?;
        }

        // ===== TURN INDICATOR =====
        let indicator_text = match &self.turn_phase {
            TurnPhase::PlayerSelectAction => Some(("YOUR TURN", Rgb888::new(100, 200, 100))),
            TurnPhase::EnemyActionExecuting { .. } => Some(("ENEMY TURN", Rgb888::new(200, 100, 100))),
            TurnPhase::CombatEnded { victory } => {
                if *victory {
                    Some(("VICTORY!", Rgb888::new(50, 180, 50)))
                } else {
                    Some(("DEFEAT", Rgb888::new(200, 80, 80)))
                }
            }
            _ => None,
        };

        if let Some((text, color)) = indicator_text {
            let indicator_style = MonoTextStyle::new(&FONT_7X13, color);
            let text_x = 120 - (text.len() as i32 * 7 / 2);
            Text::new(text, Point::new(text_x, anim_y + 15), indicator_style).draw(display)?;
        }

        // ===== ACTION MESSAGE =====
        if let Some(ref message) = self.action_message {
            if self.message_timer > 0.0 {
                let msg_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(255, 220, 100));
                let text_x = 120 - (message.len() as i32 * 6 / 2);
                Text::new(message, Point::new(text_x, anim_y + anim_h - 10), msg_style).draw(display)?;
            }
        }

        // ===== BATTLE MESSAGE BOX (Pokemon-style) =====
        if let Some(ref message) = self.current_battle_message {
            // Draw message box background
            let msg_box_y = 200;
            let msg_box_h = 70u32;
            let msg_box = Rectangle::new(Point::new(4, msg_box_y), Size::new(232, msg_box_h));
            let msg_box_rounded = RoundedRectangle::new(msg_box, CornerRadii::new(Size::new(8, 8)));

            // Dark semi-transparent background
            msg_box_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(30, 30, 40))
                .build())
                .draw(display)?;
            msg_box_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(200, 200, 220))
                .stroke_width(2)
                .build())
                .draw(display)?;

            // Draw message text (centered)
            let msg_style = MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE);
            let text_x = 120 - (message.len() as i32 * 7 / 2);
            let text_x = text_x.max(12); // Don't go off screen
            Text::new(message, Point::new(text_x, msg_box_y + 28), msg_style).draw(display)?;

            // Draw "tap to continue" hint
            let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 160));
            Text::new("...", Point::new(210, msg_box_y + 55), hint_style).draw(display)?;

            display.flush()?;
            self.dirty = false;
            return Ok(());
        }

        // ===== BOTTOM: 3 Skill Buttons + Swap =====
        use crate::game::calculations::damage::get_element_multiplier;

        let bottom_y = 200;
        let skill_button_y = bottom_y;
        let skill_button_w = 74u32;
        let skill_button_h = 50u32;  // Taller buttons for more info
        let button_spacing = 4;

        // Only show action buttons during player's turn
        let show_buttons = matches!(self.turn_phase, TurnPhase::PlayerSelectAction);

        // Get enemy element for effectiveness calculation
        let enemy_element = self.combat_state.enemy.element;

        // Get active monster's skills with full details
        // (name, cooldown, usable, power, accuracy, element, effectiveness)
        let monster_skills: Vec<(String, u8, bool, u16, u8, Element, f32)> = self.combat_state.active_monster()
            .map(|m| {
                (0..3).map(|slot| {
                    if let Some(skill) = m.equipped_skills.get(slot) {
                        let skill_name = if skill.name.len() > 8 {
                            skill.name[..8].to_string()
                        } else {
                            skill.name.clone()
                        };
                        let cooldown = m.get_skill_cooldown(slot);
                        let usable = !m.is_skill_on_cooldown(slot) && m.is_alive();
                        let effectiveness = get_element_multiplier(skill.element, enemy_element);
                        (skill_name, cooldown, usable, skill.power, skill.accuracy, skill.element, effectiveness)
                    } else {
                        ("---".to_string(), 0, false, 0, 0, Element::Neutral, 1.0)
                    }
                }).collect()
            })
            .unwrap_or_else(|| vec![("---".to_string(), 0, false, 0, 0, Element::Neutral, 1.0); 3]);

        // Draw 3 skill buttons
        for (slot, (skill_name, cooldown, usable, power, accuracy, element, effectiveness)) in monster_skills.iter().enumerate() {
            let btn_x = 4 + (slot as i32) * (skill_button_w as i32 + button_spacing);
            let enabled = show_buttons && *usable;
            let has_skill = skill_name != "---";

            // Get element color for button tint
            let elem_color = Self::element_color(*element);

            // Button colors based on state and element
            let (bg, border, text_color) = if enabled {
                // Ready to use - tinted by element color
                let r = ((elem_color.r() as u16 + 200) / 2).min(255) as u8;
                let g = ((elem_color.g() as u16 + 200) / 2).min(255) as u8;
                let b = ((elem_color.b() as u16 + 200) / 2).min(255) as u8;
                (Rgb888::new(r, g, b), elem_color, Rgb888::new(40, 40, 50))
            } else if *cooldown > 0 {
                // On cooldown - dim red
                (Rgb888::new(200, 160, 160), Rgb888::new(180, 100, 100), Rgb888::new(120, 70, 70))
            } else {
                // Disabled/no skill
                (Rgb888::new(180, 180, 185), Rgb888::new(140, 140, 145), Rgb888::new(100, 100, 100))
            };

            let btn_rect = Rectangle::new(Point::new(btn_x, skill_button_y), Size::new(skill_button_w, skill_button_h));
            let btn_rounded = RoundedRectangle::new(btn_rect, CornerRadii::new(Size::new(6, 6)));
            btn_rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(bg).build()).draw(display)?;
            btn_rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(border).stroke_width(2).build()).draw(display)?;

            if has_skill {
                // Row 1: Skill name (centered)
                let name_style = MonoTextStyle::new(&FONT_6X10, text_color);
                let name_x = btn_x + (skill_button_w as i32 - skill_name.len() as i32 * 6) / 2;
                Text::new(skill_name, Point::new(name_x, skill_button_y + 12), name_style).draw(display)?;

                // Row 2: Power and Accuracy (if not on cooldown)
                if *cooldown == 0 {
                    // Format: "P120 A95" or "P0" for non-damage skills
                    let stats_text = if *power > 0 {
                        format!("P{} A{}", power, accuracy)
                    } else {
                        format!("A{}", accuracy)
                    };
                    let stats_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(80, 80, 90));
                    let stats_x = btn_x + (skill_button_w as i32 - stats_text.len() as i32 * 6) / 2;
                    Text::new(&stats_text, Point::new(stats_x, skill_button_y + 24), stats_style).draw(display)?;

                    // Row 3: Element char + Effectiveness arrow
                    let elem_char = Self::element_char(*element);
                    let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
                    Text::new(&elem_char.to_string(), Point::new(btn_x + 8, skill_button_y + 36), elem_style).draw(display)?;

                    // Effectiveness indicator (arrow up/down)
                    if *power > 0 {  // Only show for damage skills
                        let (arrow, arrow_color) = if *effectiveness > 1.0 {
                            // Super effective - green up arrow
                            ("^^", Rgb888::new(50, 180, 50))
                        } else if *effectiveness < 1.0 {
                            // Not very effective - red down arrow
                            ("vv", Rgb888::new(200, 80, 80))
                        } else {
                            // Neutral - no indicator
                            ("--", Rgb888::new(120, 120, 120))
                        };
                        let arrow_style = MonoTextStyle::new(&FONT_6X10, arrow_color);
                        Text::new(arrow, Point::new(btn_x + 50, skill_button_y + 36), arrow_style).draw(display)?;
                    }
                } else {
                    // Show cooldown instead
                    let cd_text = format!("CD:{}", cooldown);
                    let cd_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(255, 100, 100));
                    let cd_x = btn_x + (skill_button_w as i32 - cd_text.len() as i32 * 6) / 2;
                    Text::new(&cd_text, Point::new(cd_x, skill_button_y + 30), cd_style).draw(display)?;
                }
            }

            // Store button area for touch
            self.skill_button_areas[slot] = if enabled { Some(btn_rect) } else { None };
        }

        // SWAP button (below skill buttons)
        let swap_y = skill_button_y + skill_button_h as i32 + 2;
        let swap_w = 232u32;
        let swap_h = 24u32;  // Smaller to fit
        let can_swap = self.can_swap();
        let swap_enabled = show_buttons && can_swap;
        let (swap_bg, swap_border) = if swap_enabled {
            (Rgb888::new(180, 220, 240), Rgb888::new(100, 170, 200))
        } else {
            (Rgb888::new(180, 180, 185), Rgb888::new(140, 140, 145))
        };
        let swap_rect = Rectangle::new(Point::new(4, swap_y), Size::new(swap_w, swap_h));
        let swap_rounded = RoundedRectangle::new(swap_rect, CornerRadii::new(Size::new(6, 6)));
        swap_rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(swap_bg).build()).draw(display)?;
        swap_rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(swap_border).stroke_width(2).build()).draw(display)?;
        let swap_text_color = if swap_enabled { Rgb888::new(50, 100, 150) } else { Rgb888::new(100, 100, 100) };
        let swap_style = MonoTextStyle::new(&FONT_7X13, swap_text_color);
        Text::new("SWAP", Point::new(100, swap_y + 16), swap_style).draw(display)?;
        self.swap_button_area = if swap_enabled { Some(swap_rect) } else { None };

        // ===== SWAP POPUP =====
        if self.show_swap_popup {
            // Dim background
            let dim_rect = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&dim_rect, Rgb888::new(30, 30, 40))?;

            // Popup background
            let popup_rect = Rectangle::new(Point::new(20, 90), Size::new(200, 100));
            let popup_rounded = RoundedRectangle::new(popup_rect, CornerRadii::new(Size::new(8, 8)));
            popup_rounded.into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(60, 60, 70))
                .build())
                .draw(display)?;
            popup_rounded.into_styled(PrimitiveStyleBuilder::new()
                .stroke_color(Rgb888::new(100, 170, 200))
                .stroke_width(2)
                .build())
                .draw(display)?;

            // Title
            let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::WHITE);
            Text::new("Select teammate:", Point::new(55, 108), title_style).draw(display)?;

            // Monster buttons
            let popup_btn_w = 58u32;
            let popup_btn_h = 50u32;
            for (i, monster) in self.combat_state.player_monsters.iter().take(3).enumerate() {
                let x = 25 + (i as i32 * 62);
                let y = 118;
                let is_active = i == self.combat_state.active_index as usize;
                let is_dead = !monster.is_alive();

                let (bg_color, border_color) = if is_active {
                    (Rgb888::new(80, 100, 80), Rgb888::new(60, 80, 60))
                } else if is_dead {
                    (Rgb888::new(100, 70, 70), Rgb888::new(80, 50, 50))
                } else {
                    (Rgb888::new(80, 120, 160), Rgb888::new(60, 100, 140))
                };

                let btn_rect = Rectangle::new(Point::new(x, y), Size::new(popup_btn_w, popup_btn_h));
                let btn_rounded = RoundedRectangle::new(btn_rect, CornerRadii::new(Size::new(4, 4)));
                btn_rounded.into_styled(PrimitiveStyleBuilder::new().fill_color(bg_color).build()).draw(display)?;
                btn_rounded.into_styled(PrimitiveStyleBuilder::new().stroke_color(border_color).stroke_width(1).build()).draw(display)?;

                let elem_color = Self::element_color(monster.element);
                let elem_style = MonoTextStyle::new(&FONT_6X10, elem_color);
                let name = if monster.name.len() > 6 { &monster.name[..6] } else { &monster.name };
                Text::new(&format!("{}{}", Self::element_char(monster.element), name),
                    Point::new(x + 2, y + 15), elem_style).draw(display)?;

                // HP bar
                let hp_bar_y = y + 20;
                let hp_bg = Rectangle::new(Point::new(x + 2, hp_bar_y), Size::new(54, 6));
                display.fill_solid(&hp_bg, Rgb888::new(60, 60, 60))?;
                let hp_pct = monster.hp_current as f32 / monster.hp_max as f32;
                let hp_fill_w = (54.0 * hp_pct) as u32;
                if hp_fill_w > 0 {
                    let hp_fill = Rectangle::new(Point::new(x + 2, hp_bar_y), Size::new(hp_fill_w, 6));
                    display.fill_solid(&hp_fill, Rgb888::new(80, 200, 80))?;
                }

                let status = if is_active { "ACTIVE" } else if is_dead { "KO" } else { "SELECT" };
                let status_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 180, 180));
                Text::new(status, Point::new(x + 6, y + 44), status_style).draw(display)?;

                // Only register touch area for selectable monsters
                if !is_active && !is_dead {
                    self.swap_popup_buttons[i] = Some(btn_rect);
                } else {
                    self.swap_popup_buttons[i] = None;
                }
            }
        } else {
            // Clear popup button areas when not showing
            self.swap_popup_buttons = [None; 3];
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
        if let TurnPhase::CombatEnded { .. } = self.turn_phase {
            self.end_delay += delta;
            self.dirty = true;
            return self.end_delay < 2.0;
        }

        // Update death animation timer
        if self.death_anim_active {
            self.death_anim_timer += delta;
            if self.death_anim_timer >= Self::DEATH_ANIM_DURATION {
                self.death_anim_active = false;
                self.death_anim_timer = 0.0;
                log::info!("Death animation complete for {}", self.death_anim_species);
            }
        }

        // Update message timer
        if self.message_timer > 0.0 {
            self.message_timer -= delta;
        }

        // ===== TURN-BASED STATE MACHINE =====
        match self.turn_phase {
            TurnPhase::DeterminingTurnOrder => {
                let player_goes = self.determine_next_turn();
                if player_goes {
                    // Tick cooldowns at start of player's turn
                    if let Some(monster) = self.combat_state.active_monster_mut() {
                        monster.tick_cooldowns();
                    }
                    self.turn_phase = TurnPhase::PlayerSelectAction;
                    log::info!("Player's turn!");
                } else {
                    // Enemy turn: show "X attacks!" message first, then animate
                    self.queue_enemy_attack_message();
                    log::info!("Enemy's turn!");
                }
            }

            TurnPhase::PlayerSelectAction => {
                // Waiting for player touch input - nothing to update
                // Idle animations will play
            }

            TurnPhase::PlayerActionExecuting { action_type, timer } => {
                let new_timer = timer + delta;
                if new_timer >= ACTION_ANIM_DURATION {
                    // Animation complete - execute the action effect
                    self.apply_action_effect(action_type);
                    self.action_target = ActiveAnim::None;

                    // Check if enemy died
                    if !self.combat_state.enemy.is_alive() {
                        self.handle_enemy_death();
                    } else if !self.battle_messages.is_empty() {
                        // Show battle messages first
                        self.start_battle_messages();
                    } else {
                        // Move to post-action delay
                        self.turn_phase = TurnPhase::PlayerActionComplete { timer: 0.0 };
                    }
                } else {
                    self.turn_phase = TurnPhase::PlayerActionExecuting {
                        action_type,
                        timer: new_timer,
                    };
                }
            }

            TurnPhase::PlayerActionComplete { timer } => {
                let new_timer = timer + delta;
                if new_timer >= POST_ACTION_DELAY {
                    // Check if swap - enemy gets free attack after swap
                    if matches!(self.last_player_action, Some(TurnAction::Swap { .. })) {
                        // Queue enemy attack message first, then animate
                        self.queue_enemy_attack_message();
                    } else {
                        self.turn_phase = TurnPhase::DeterminingTurnOrder;
                    }
                } else {
                    self.turn_phase = TurnPhase::PlayerActionComplete { timer: new_timer };
                }
            }

            TurnPhase::ShowingBattleMessage { timer } => {
                let new_timer = timer + delta;
                // Show each message for 1.2 seconds
                if new_timer >= 1.2 {
                    self.next_battle_message();
                } else {
                    self.turn_phase = TurnPhase::ShowingBattleMessage { timer: new_timer };
                }
            }

            TurnPhase::EnemyActionExecuting { timer } => {
                let new_timer = timer + delta;
                if new_timer >= ACTION_ANIM_DURATION {
                    // Animation complete - effect was already applied in next_battle_message()
                    self.action_target = ActiveAnim::None;

                    // Check if player died
                    if self.combat_state.all_players_dead() {
                        self.handle_player_defeat();
                    } else if !self.battle_messages.is_empty() {
                        // Show remaining battle messages (damage message)
                        self.last_was_enemy_turn = true;
                        self.start_battle_messages();
                    } else {
                        self.turn_phase = TurnPhase::EnemyActionComplete { timer: 0.0 };
                    }
                } else {
                    self.turn_phase = TurnPhase::EnemyActionExecuting { timer: new_timer };
                }
            }

            TurnPhase::EnemyActionComplete { timer } => {
                let new_timer = timer + delta;
                if new_timer >= POST_ACTION_DELAY {
                    self.turn_phase = TurnPhase::DeterminingTurnOrder;
                } else {
                    self.turn_phase = TurnPhase::EnemyActionComplete { timer: new_timer };
                }
            }

            TurnPhase::CombatEnded { .. } => {
                // Already handled above
            }
        }

        // Get species IDs for cache lookups
        let enemy_species = self.combat_state.enemy.species_id.clone();
        let player_species = self.combat_state.active_monster()
            .map(|m| m.species_id.clone())
            .unwrap_or_default();

        // Update idle animations
        if let Some(anim) = self.anim_cache.get_mut(&enemy_species) {
            anim.update(delta);
        }
        if let Some(anim) = self.anim_cache.get_mut(&player_species) {
            anim.update(delta);
        }

        // Update damage popups
        self.damage_popups.retain_mut(|popup| {
            popup.y_offset += delta * 50.0;
            popup.alpha -= delta * 2.0;
            popup.alpha > 0.0
        });

        // Update reaction popup timer
        if let Some(ref mut reaction) = self.reaction_popup {
            reaction.timer -= delta;
            if reaction.timer <= 0.0 {
                self.reaction_popup = None;
            }
        }

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
