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

/// Dungeon combat page with fast raw RGB565 animations
///
/// Memory layout (optimized for ESP32-C6 with no PSRAM):
/// - Animation cache: HashMap<species_id, RawAnimPlayer> (~13KB per unique species)
/// - All unique species (enemy + wave enemies + player team) loaded once at combat start
/// - No duplicate loading - same species shared across team/enemy
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

    // Animation cache: species_id -> RawAnimPlayer
    // All unique species loaded once at combat start (enemy + waves + player team)
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

    // Death animation state
    // When enemy dies: spin and fly off screen to the left
    death_anim_active: bool,
    death_anim_timer: f32,
    death_anim_species: String,  // Species of the dying enemy (for rendering)

    // Hide enemy after death animation until next wave is set up
    // This prevents the dead enemy from briefly reappearing
    hide_enemy_until_next_wave: bool,

    // Reaction popup (displayed when elemental reaction triggers)
    reaction_popup: Option<ReactionPopup>,
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
            skill_button_area: None,
            swap_button_areas: [None; 3],
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
            death_anim_active: false,
            death_anim_timer: 0.0,
            death_anim_species: String::new(),
            hide_enemy_until_next_wave: false,
            reaction_popup: None,
        }
    }

    /// Check if the page needs initial animation loading
    /// Only returns true AFTER the loading screen has been drawn at least once
    pub fn needs_initial_load(&self) -> bool {
        self.is_loading && self.loading_drawn
    }

    /// Load all animations from SD card (all unique species from enemy + waves + player team)
    /// Call this after showing the loading screen
    pub fn load_initial_animations(&mut self, sd_card: &mut SdCardWrapper) {
        if !self.is_loading {
            return;
        }

        // Collect all unique species IDs from:
        // 1. Current enemy
        // 2. All wave enemies
        // 3. All player team monsters
        let mut unique_species: Vec<String> = Vec::new();

        // Current enemy
        let enemy_species = &self.combat_state.enemy.species_id;
        if !unique_species.contains(enemy_species) {
            unique_species.push(enemy_species.clone());
        }

        // All wave enemies (future waves)
        for wave_enemy in &self.combat_state.wave_enemies {
            if !unique_species.contains(&wave_enemy.species_id) {
                unique_species.push(wave_enemy.species_id.clone());
            }
        }

        // All player team monsters
        for monster in &self.combat_state.player_monsters {
            if !unique_species.contains(&monster.species_id) {
                unique_species.push(monster.species_id.clone());
            }
        }

        log::info!("Loading {} unique species animations for battle: {:?}",
            unique_species.len(), unique_species);

        // Load each unique species only once
        for species in &unique_species {
            if let Some(anim) = load_raw_from_sd(sd_card, species, AnimType::Idle) {
                log::info!("Cached animation for species: {}", species);
                self.anim_cache.insert(species.clone(), anim);
            } else {
                log::warn!("Failed to load animation for species: {}", species);
            }
        }

        self.is_loading = false;
        self.dirty = true;
        log::info!("Combat animations loaded ({} cached), ready to fight!", self.anim_cache.len());
    }

    /// Reload player species animation (after swap)
    /// Note: With cache, this is now a no-op since all species are preloaded
    pub fn reload_player_species(&mut self, _sd_card: &mut SdCardWrapper) {
        // All species animations are preloaded at combat start in the cache
        // Swap just changes active_index, no loading needed
        log::info!("Player swap: using cached animation");
    }

    /// Reload enemy species animation (for new wave)
    /// Note: With cache, this is now a no-op since all wave enemies are preloaded
    pub fn reload_enemy_species(&mut self, _sd_card: &mut SdCardWrapper) {
        // All wave enemy species are preloaded at combat start in the cache
        // Wave transition just changes current enemy, no loading needed
        log::info!("New wave: using cached animation for {}", self.combat_state.enemy.species_id);
        self.enemy_anim_type = AnimType::Idle;
        // Clear the hide flag - new enemy is ready to be shown
        self.hide_enemy_until_next_wave = false;
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
    fn queue_animation(&mut self, target: ActiveAnim, anim_type: AnimType) {
        // Only queue Idle animations for actual loading (enemy only)
        // Player team is preloaded, so no loading needed
        // Action animations (Attack/Hurt/Death) use idle + visual effects
        if anim_type == AnimType::Idle && target == ActiveAnim::Enemy {
            self.pending_enemy_anim = Some(anim_type);
        }
        // Set action state for visual effect (lunge)
        self.action_target = target;
        self.action_timer = 0.0;
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
            CombatEvent::PlayerSkill { damage, .. } => {
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
                // With preloading, swap is instant - no loading needed!
                // The render function uses active_index to pick the correct preloaded animation
                log::info!("Monster swap: instant switch to preloaded animation slot {}",
                    self.combat_state.active_index);
            }
            CombatEvent::WaveComplete { wave, total } => {
                // Enemy died, more waves remain - start death animation
                log::info!("Wave {}/{} complete - starting death animation", wave, total);
                self.start_death_animation();
            }
            CombatEvent::Victory { .. } => {
                // Final enemy died - start death animation
                log::info!("Victory! Starting final death animation");
                self.start_death_animation();
            }
            CombatEvent::Defeat => {
                if self.action_target == ActiveAnim::None {
                    self.queue_animation(ActiveAnim::Player, AnimType::Death);
                }
            }
            CombatEvent::WaveStart { wave, .. } => {
                // New wave started - clear the hide flag so new enemy is visible
                log::info!("Wave {} starting - showing new enemy", wave);
                self.hide_enemy_until_next_wave = false;
                self.enemy_anim_type = AnimType::Idle;
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
        } else if !self.hide_enemy_until_next_wave {
            // Normal enemy rendering (only if not hidden after death animation)
            let enemy_species = &self.combat_state.enemy.species_id;
            if let Some(anim) = self.anim_cache.get(enemy_species) {
                anim.render(display, enemy_x, center_y, false);
            }
        }
        // If hide_enemy_until_next_wave is true, don't render any enemy
        // This prevents the dead enemy from briefly reappearing before the next wave

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

        // Update death animation timer
        if self.death_anim_active {
            self.death_anim_timer += delta;
            if self.death_anim_timer >= Self::DEATH_ANIM_DURATION {
                // Death animation complete - hide enemy until next wave is set up
                self.death_anim_active = false;
                self.death_anim_timer = 0.0;
                self.hide_enemy_until_next_wave = true;  // Don't show old enemy at normal position
                log::info!("Death animation complete for {}, hiding until next wave", self.death_anim_species);
            }
        }

        // Get species IDs for cache lookups
        let enemy_species = self.combat_state.enemy.species_id.clone();
        let player_species = self.combat_state.active_monster()
            .map(|m| m.species_id.clone())
            .unwrap_or_default();

        // Update action animation timer
        if self.action_target != ActiveAnim::None {
            self.action_timer += delta;

            // Get frame count for the active animation from cache
            let enemy_frame_count = self.anim_cache.get(&enemy_species)
                .map(|a| a.frame_count()).unwrap_or(1);
            let player_frame_count = self.anim_cache.get(&player_species)
                .map(|a| a.frame_count()).unwrap_or(1);

            let (frame_count, current_frame, is_death) = match self.action_target {
                ActiveAnim::Enemy => (
                    enemy_frame_count,
                    self.anim_cache.get(&enemy_species).map(|a| a.current_frame()).unwrap_or(0),
                    self.enemy_anim_type == AnimType::Death
                ),
                ActiveAnim::Player => (
                    player_frame_count,
                    self.anim_cache.get(&player_species).map(|a| a.current_frame()).unwrap_or(0),
                    self.player_anim_type == AnimType::Death
                ),
                _ => (1, 0, false),
            };

            // Advance action frame using RawAnimPlayer's built-in update
            let frame_changed = match self.action_target {
                ActiveAnim::Enemy => self.anim_cache.get_mut(&enemy_species)
                    .map(|a| a.update(delta)).unwrap_or(false),
                ActiveAnim::Player => self.anim_cache.get_mut(&player_species)
                    .map(|a| a.update(delta)).unwrap_or(false),
                _ => false,
            };

            // Check if animation looped (completed)
            if frame_changed {
                let new_frame = match self.action_target {
                    ActiveAnim::Enemy => self.anim_cache.get(&enemy_species)
                        .map(|a| a.current_frame()).unwrap_or(0),
                    ActiveAnim::Player => self.anim_cache.get(&player_species)
                        .map(|a| a.current_frame()).unwrap_or(0),
                    _ => 0,
                };

                // If frame went back to 0, animation completed
                if new_frame < current_frame || (current_frame >= frame_count.saturating_sub(1)) {
                    if is_death {
                        // Stay on last frame for death
                        match self.action_target {
                            ActiveAnim::Enemy => {
                                if let Some(anim) = self.anim_cache.get_mut(&enemy_species) {
                                    anim.set_frame(frame_count.saturating_sub(1));
                                }
                            }
                            ActiveAnim::Player => {
                                if let Some(anim) = self.anim_cache.get_mut(&player_species) {
                                    anim.set_frame(frame_count.saturating_sub(1));
                                }
                            }
                            _ => {}
                        }
                    } else {
                        self.end_action_to_idle();
                    }
                }
            }
        } else {
            // Update idle animations using RawAnimPlayer's built-in timing
            if self.enemy_anim_type == AnimType::Idle {
                if let Some(anim) = self.anim_cache.get_mut(&enemy_species) {
                    anim.update(delta);
                }
            }
            if self.player_anim_type == AnimType::Idle {
                if let Some(anim) = self.anim_cache.get_mut(&player_species) {
                    anim.update(delta);
                }
            }
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
