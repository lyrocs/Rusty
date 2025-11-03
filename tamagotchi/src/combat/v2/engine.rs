/// Combat engine v2 - Main update loop and combat logic
///
/// Implements the fixed 100ms update loop and state machine transitions

use crate::core::GameState;
use crate::combat::{Enemy, IdleFarmState};
use crate::tamagotchi::models::GamePage;
use super::state::{CombatPhase, CombatState};
use super::animation::AnimationController;
use super::calculator::CombatCalculator;
use super::frame_tracker::{FrameTracker, AnimationType};

/// Combat engine that manages all combat state
#[derive(Debug, Clone)]
pub struct CombatEngine {
    pub combat_state: CombatState,
    pub animation_controller: AnimationController,
    pub calculator: CombatCalculator,
    pub accumulated_time_ms: u32,
    pub hero_frame_tracker: FrameTracker,
    pub monster_frame_tracker: FrameTracker,
}

impl CombatEngine {
    /// Create a new combat engine
    pub fn new(start_ms: u32) -> Self {
        Self {
            combat_state: CombatState::new(start_ms),
            animation_controller: AnimationController::new(start_ms),
            calculator: CombatCalculator::new(),
            accumulated_time_ms: 0,
            hero_frame_tracker: FrameTracker::new(start_ms),
            monster_frame_tracker: FrameTracker::new(start_ms),
        }
    }

    /// Update combat with fixed 100ms time steps
    pub fn update(
        &mut self,
        game_state: &mut GameState,
        delta_ms: u32,
    ) {
        // Accumulate time for fixed update
        self.accumulated_time_ms += delta_ms;

        // Fixed 100ms combat updates
        while self.accumulated_time_ms >= 100 {
            self.update_combat_logic(game_state, 100);
            self.accumulated_time_ms -= 100;
        }

        // Always update animations (frame-rate independent)
        self.update_animations(game_state);
    }

    /// Update combat logic with fixed time step
    fn update_combat_logic(&mut self, game_state: &mut GameState, _fixed_delta_ms: u32) {
        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        if session.state != IdleFarmState::Active {
            return;
        }

        let current_time = game_state.last_update_ms;

        // Update stats if needed
        self.calculator.update_hero_stats(&game_state.hero);

        if let Some(enemy) = Enemy::from_id(session.enemy_id) {
            self.calculator.update_enemy_stats(&enemy);
        }

        // Update frame trackers - this drives animation completion
        let hero_just_completed = self.hero_frame_tracker.update(current_time);
        let monster_just_completed = self.monster_frame_tracker.update(current_time);

        // Check for animation-driven phase transitions
        if hero_just_completed {
            match self.combat_state.phase {
                CombatPhase::HeroAttacking => {
                    // Hero attack animation finished, calculate damage
                    esp_println::println!("[ENGINE] Hero attack animation complete, calculating damage");
                    self.process_hero_attack_complete(game_state, current_time);
                    return; // Skip other processing this frame
                }
                _ => {}
            }
        }

        if monster_just_completed {
            match self.combat_state.phase {
                CombatPhase::EnemyAttacking => {
                    // Enemy attack animation finished, calculate damage
                    esp_println::println!("[ENGINE] Enemy attack animation complete, calculating damage");
                    self.process_enemy_attack_complete(game_state, current_time);
                    return; // Skip other processing this frame
                }
                CombatPhase::EnemyReacting => {
                    // Enemy reaction animation finished, return to idle
                    esp_println::println!("[ENGINE] Enemy reaction complete, returning to idle");
                    self.combat_state.transition_to(CombatPhase::Idle, 0, current_time);

                    // Reset to idle with actual frame counts
                    let hero_idle_frames = self.get_hero_idle_frame_count(game_state);
                    self.hero_frame_tracker.reset_to_idle_with_frames(hero_idle_frames, current_time);

                    let monster_name = game_state.idle_farm_session.as_ref()
                        .and_then(|s| Enemy::from_id(s.enemy_id).map(|e| e.name))
                        .unwrap_or("Poring");
                    let monster_idle_frames = self.get_monster_idle_frame_count(monster_name);
                    self.monster_frame_tracker.reset_to_idle_with_frames(monster_idle_frames, current_time);
                }
                CombatPhase::HeroReacting => {
                    // Hero reaction animation finished, return to idle
                    esp_println::println!("[ENGINE] Hero reaction complete, returning to idle");
                    self.combat_state.transition_to(CombatPhase::Idle, 0, current_time);

                    // Reset to idle with actual frame counts
                    let hero_idle_frames = self.get_hero_idle_frame_count(game_state);
                    self.hero_frame_tracker.reset_to_idle_with_frames(hero_idle_frames, current_time);

                    let monster_name = game_state.idle_farm_session.as_ref()
                        .and_then(|s| Enemy::from_id(s.enemy_id).map(|e| e.name))
                        .unwrap_or("Poring");
                    let monster_idle_frames = self.get_monster_idle_frame_count(monster_name);
                    self.monster_frame_tracker.reset_to_idle_with_frames(monster_idle_frames, current_time);
                }
                CombatPhase::EnemyDying => {
                    // Death animation complete, spawn new enemy
                    esp_println::println!("[ENGINE] Death animation complete, spawning new enemy");
                    self.select_new_enemy(game_state);

                    // Reset animations to idle for spawn phase with actual frame counts
                    let hero_idle_frames = self.get_hero_idle_frame_count(game_state);
                    self.hero_frame_tracker.reset_to_idle_with_frames(hero_idle_frames, current_time);

                    let monster_name = game_state.idle_farm_session.as_ref()
                        .and_then(|s| Enemy::from_id(s.enemy_id).map(|e| e.name))
                        .unwrap_or("Poring");
                    let monster_idle_frames = self.get_monster_idle_frame_count(monster_name);
                    self.monster_frame_tracker.reset_to_idle_with_frames(monster_idle_frames, current_time);

                    self.combat_state.transition_to(CombatPhase::EnemySpawning, 2000, current_time);
                    self.animation_controller.start_spawn_animation();
                }
                _ => {}
            }
        }

        // Check for time-based phase completion (for non-animation phases)
        if self.combat_state.is_phase_complete(current_time) {
            match self.combat_state.phase {
                CombatPhase::EnemySpawning => {
                    // Spawn complete - reset position and return to idle
                    self.animation_controller.reset_spawn_position();

                    let session = game_state.idle_farm_session.as_mut().unwrap();
                    // Add delay before first attacks
                    session.next_hero_attack_ms = current_time + 1000;
                    session.next_enemy_attack_ms = current_time + self.calculator.get_enemy_attack_cycle_ms() + 1000;

                    esp_println::println!("[ENGINE] Enemy reached battle position - combat starting!");
                    self.combat_state.transition_to(CombatPhase::Idle, 0, current_time);

                    // Reset to idle with actual frame counts
                    let hero_idle_frames = self.get_hero_idle_frame_count(game_state);
                    self.hero_frame_tracker.reset_to_idle_with_frames(hero_idle_frames, current_time);

                    let monster_name = game_state.idle_farm_session.as_ref()
                        .and_then(|s| Enemy::from_id(s.enemy_id).map(|e| e.name))
                        .unwrap_or("Poring");
                    let monster_idle_frames = self.get_monster_idle_frame_count(monster_name);
                    self.monster_frame_tracker.reset_to_idle_with_frames(monster_idle_frames, current_time);
                }
                _ => {}
            }
        }

        // Process current phase
        match self.combat_state.phase {
            CombatPhase::Idle => {
                self.process_idle_phase(game_state, current_time);
            }
            CombatPhase::EnemySpawning => {
                // Update spawn animation
                let elapsed = self.combat_state.phase_elapsed_ms(current_time);
                self.animation_controller.update_spawn_animation(elapsed, 2000);
                game_state.needs_redraw = true;
            }
            _ => {
                // Animation phases are handled by frame tracker completion above
            }
        }
    }

    /// Process idle phase - check if anyone should attack
    fn process_idle_phase(&mut self, game_state: &mut GameState, current_time: u32) {
        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        // Check if hero should attack
        if current_time >= session.next_hero_attack_ms
            && !self.combat_state.phase.blocks_hero_attack() {
            self.start_hero_attack(game_state, current_time);
        }
        // Check if enemy should attack
        else if current_time >= session.next_enemy_attack_ms
            && !self.combat_state.phase.blocks_enemy_attack() {
            self.start_enemy_attack(game_state, current_time);
        }
    }

    /// Start hero attack sequence (animation only, damage calculated when complete)
    fn start_hero_attack(&mut self, game_state: &mut GameState, current_time: u32) {
        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        // Check for skill usage
        let use_skill = current_time >= session.next_skill_use_ms;

        // Update skill tracking if using skill
        if use_skill {
            session.last_skill_use_ms = current_time;
            session.next_skill_use_ms = current_time + session.skill_cooldown_ms;
            esp_println::println!("[ENGINE] Hero starts SKILL attack animation!");
        } else {
            esp_println::println!("[ENGINE] Hero starts attack animation!");
        }

        // Start frame tracking for hero attack
        self.hero_frame_tracker.start_animation(AnimationType::HeroAttacking, current_time);

        // Monster stays idle during hero attack - use actual frame count
        let monster_name = Enemy::from_id(session.enemy_id)
            .map(|e| e.name)
            .unwrap_or("Poring");
        let monster_idle_frames = self.get_monster_idle_frame_count(monster_name);
        self.monster_frame_tracker.start_animation_with_frames(AnimationType::Idle, monster_idle_frames, current_time);

        // Start hero attack phase (no fixed duration - animation driven)
        self.combat_state.start_hero_attack(
            current_time,
            0, // Duration will be driven by animation completion
            use_skill,
        );

        // Update next attack time
        session.next_hero_attack_ms = current_time + self.calculator.get_hero_attack_cycle_ms();
    }

    /// Start enemy attack sequence (animation only, damage calculated when complete)
    fn start_enemy_attack(&mut self, game_state: &mut GameState, current_time: u32) {
        esp_println::println!("[ENGINE] Enemy starts attack animation!");

        // Get hero idle frame count before borrowing session mutably
        let hero_idle_frames = self.get_hero_idle_frame_count(game_state);

        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        // Start frame tracking for enemy attack
        self.monster_frame_tracker.start_animation(AnimationType::MonsterAttacking, current_time);

        // Hero stays idle during enemy attack - use actual frame count
        self.hero_frame_tracker.start_animation_with_frames(AnimationType::Idle, hero_idle_frames, current_time);

        // Start enemy attack phase (no fixed duration - animation driven)
        self.combat_state.start_enemy_attack(
            current_time,
            0, // Duration will be driven by animation completion
        );

        // Update next attack time
        session.next_enemy_attack_ms = current_time + self.calculator.get_enemy_attack_cycle_ms();
    }

    /// Process hero attack completion - calculate damage, apply, and transition
    /// Called when HeroAttacking animation completes
    fn process_hero_attack_complete(&mut self, game_state: &mut GameState, current_time: u32) {
        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        // NOW calculate damage (after animation played)
        let use_skill = self.combat_state.pending_skill;
        let skill_multiplier = if use_skill { 2 } else { 1 };
        let damage = self.calculator.calculate_hero_damage(skill_multiplier);

        // Roll for hit/miss
        let rng_value = (current_time % 100) as u8;
        let is_hit = self.calculator.roll_hero_hit(rng_value);

        let final_damage = if is_hit { damage } else { 0 };

        // Update display tracking
        session.last_hero_damage = final_damage;
        session.hero_attack_missed = !is_hit;
        session.last_skill_used = use_skill;
        session.hero_damage_apply_ms = current_time; // Set timestamp for damage display animation

        if !is_hit {
            esp_println::println!("[ENGINE] Hero attack MISSED!");
            // Start enemy reaction animation (3 frames for miss)
            self.monster_frame_tracker.start_animation(AnimationType::MonsterAttacked, current_time);

            // Hero returns to idle with actual frame count
            let hero_idle_frames = self.get_hero_idle_frame_count(game_state);
            self.hero_frame_tracker.start_animation_with_frames(AnimationType::Idle, hero_idle_frames, current_time);

            self.combat_state.transition_to(CombatPhase::EnemyReacting, 0, current_time);
            self.animation_controller.set_for_phase(&CombatPhase::EnemyReacting, current_time);
            game_state.needs_redraw = true;
        } else {
            esp_println::println!("[ENGINE] Hero attack lands! Damage: {}", final_damage);

            // Apply damage to enemy
            if session.current_enemy_hp > final_damage {
                session.current_enemy_hp -= final_damage;
                // Start enemy reaction animation (3 frames for hit)
                self.monster_frame_tracker.start_animation(AnimationType::MonsterAttacked, current_time);

                // Hero returns to idle with actual frame count
                let hero_idle_frames = self.get_hero_idle_frame_count(game_state);
                self.hero_frame_tracker.start_animation_with_frames(AnimationType::Idle, hero_idle_frames, current_time);

                self.combat_state.transition_to(CombatPhase::EnemyReacting, 0, current_time);
                self.animation_controller.set_for_phase(&CombatPhase::EnemyReacting, current_time);
                game_state.needs_redraw = true;
            } else {
                // Enemy killed!
                session.current_enemy_hp = 0;
                self.handle_enemy_death(game_state);
            }
        }
    }

    /// Process enemy attack completion - calculate damage, apply, and transition
    /// Called when EnemyAttacking animation completes
    fn process_enemy_attack_complete(&mut self, game_state: &mut GameState, current_time: u32) {
        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        // NOW calculate damage (after animation played)
        let damage = self.calculator.calculate_enemy_damage();

        // Roll for hit/miss
        let rng_value = ((current_time + 50) % 100) as u8;
        let is_hit = self.calculator.roll_enemy_hit(rng_value);

        let final_damage = if is_hit { damage } else { 0 };

        // Track damage for display
        session.last_enemy_damage = final_damage;
        session.enemy_attack_missed = !is_hit;
        session.enemy_damage_apply_ms = current_time; // Set timestamp for damage display animation

        if !is_hit {
            esp_println::println!("[ENGINE] Enemy attack MISSED!");
            // Start hero reaction animation (3 frames for miss)
            self.hero_frame_tracker.start_animation(AnimationType::HeroAttacked, current_time);

            // Monster returns to idle with actual frame count
            let monster_name = session.enemy_id;
            let monster_name = Enemy::from_id(monster_name)
                .map(|e| e.name)
                .unwrap_or("Poring");
            let monster_idle_frames = self.get_monster_idle_frame_count(monster_name);
            self.monster_frame_tracker.start_animation_with_frames(AnimationType::Idle, monster_idle_frames, current_time);

            self.combat_state.transition_to(CombatPhase::HeroReacting, 0, current_time);
            self.animation_controller.set_for_phase(&CombatPhase::HeroReacting, current_time);
            game_state.needs_redraw = true;
        } else {
            esp_println::println!("[ENGINE] Enemy attack lands! Damage: {}", final_damage);

            // Apply damage to hero
            if session.current_hp > final_damage {
                session.current_hp -= final_damage;
                game_state.hero.hp = session.current_hp;
                // Start hero reaction animation (3 frames for hit)
                self.hero_frame_tracker.start_animation(AnimationType::HeroAttacked, current_time);

                // Monster returns to idle with actual frame count
                let monster_name = session.enemy_id;
                let monster_name = Enemy::from_id(monster_name)
                    .map(|e| e.name)
                    .unwrap_or("Poring");
                let monster_idle_frames = self.get_monster_idle_frame_count(monster_name);
                self.monster_frame_tracker.start_animation_with_frames(AnimationType::Idle, monster_idle_frames, current_time);

                self.combat_state.transition_to(CombatPhase::HeroReacting, 0, current_time);
                self.animation_controller.set_for_phase(&CombatPhase::HeroReacting, current_time);
                game_state.needs_redraw = true;
            } else {
                // Hero died!
                self.handle_hero_death(game_state);
            }
        }
    }

    /// Handle enemy death
    fn handle_enemy_death(&mut self, game_state: &mut GameState) {
        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        let current_time = game_state.last_update_ms;

        session.monsters_killed += 1;

        // Award rewards
        if let Some(enemy) = Enemy::from_id(session.enemy_id) {
            let zeny_gain = enemy.zeny_reward;
            let exp_gain = enemy.base_exp;
            session.zeny_earned += zeny_gain;
            session.exp_gained += exp_gain;
            game_state.hero.zeny += zeny_gain;
            game_state.hero.add_exp(exp_gain);

            // Roll for item drops
            use crate::data::roll_drops;
            let rng_value = ((current_time % 255) + session.monsters_killed as u32) as u8;
            let drops = roll_drops(enemy.id, rng_value);
            for (item_id, item_name, quantity) in drops {
                use crate::hero::inventory::InventoryExt;
                game_state.hero.inventory.add_item(item_id, item_name, quantity);
                session.items_collected += quantity;
            }

            esp_println::println!("[COMBAT V2] Enemy killed! Total: {}", session.monsters_killed);
        }

        // Start death animation tracking (8 frames)
        self.monster_frame_tracker.start_animation(AnimationType::MonsterDying, current_time);

        // Hero returns to idle with actual frame count
        let hero_idle_frames = self.get_hero_idle_frame_count(game_state);
        self.hero_frame_tracker.start_animation_with_frames(AnimationType::Idle, hero_idle_frames, current_time);

        // Start death animation phase (no fixed duration - animation driven)
        self.combat_state.start_enemy_death(current_time, 0);

        // Update animation controller for death phase
        self.animation_controller.set_for_phase(&CombatPhase::EnemyDying, current_time);
    }

    /// Handle hero death
    fn handle_hero_death(&mut self, game_state: &mut GameState) {
        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        let current_time = game_state.last_update_ms;

        session.current_hp = 0;
        game_state.hero.hp = 0;
        session.state = IdleFarmState::Cooldown;
        session.cooldown_end_ms = current_time + 60_000;

        esp_println::println!("[COMBAT V2] Hero DIED!");

        // Show results screen
        game_state.current_page = GamePage::IdleFarmResult;
        game_state.needs_redraw = true;
    }


    /// Select new enemy after death
    fn select_new_enemy(&mut self, game_state: &mut GameState) {
        let session = match &mut game_state.idle_farm_session {
            Some(session) => session,
            None => return,
        };

        let current_time = game_state.last_update_ms;

        // Randomly select new enemy from pool
        if !session.enemy_pool.is_empty() {
            let enemy_index = (current_time % session.enemy_pool.len() as u32) as usize;
            let new_enemy_id = session.enemy_pool[enemy_index];

            if let Some(new_enemy) = Enemy::from_id(new_enemy_id) {
                session.enemy_id = new_enemy_id;
                session.enemy_max_hp = new_enemy.max_hp;
                session.current_enemy_hp = new_enemy.max_hp;
                session.enemy_level = new_enemy.level;

                // Mark enemy stats as dirty for recalculation
                self.calculator.mark_enemy_dirty();

                esp_println::println!("[COMBAT V2] New enemy selected: {} (Level {}) - walking in...",
                    new_enemy.name, new_enemy.level);
            }
        }

        // Start spawn animation
        self.animation_controller.start_spawn_animation();
    }

    /// Update animations (called every frame)
    fn update_animations(&mut self, game_state: &mut GameState) {
        // Only update if on battle page and screen is on
        if game_state.current_page != GamePage::BattleOverview || !game_state.screen_on {
            return;
        }

        // Synchronize hero animation with frame tracker
        let hero_anim = self.hero_frame_tracker.animation_type.to_hero_animation();
        if game_state.hero_animation != hero_anim {
            game_state.hero_animation = hero_anim;
            game_state.hero_animation_started_ms = self.hero_frame_tracker.animation_started_ms;
            game_state.needs_redraw = true;
        }

        // Synchronize monster animation with frame tracker
        let monster_anim = self.monster_frame_tracker.animation_type.to_monster_animation();
        if game_state.monster_animation != monster_anim {
            game_state.monster_animation = monster_anim;
            game_state.monster_animation_started_ms = self.monster_frame_tracker.animation_started_ms;
            game_state.needs_redraw = true;
        }

        // Set frame directly from tracker
        let new_hero_frame = self.hero_frame_tracker.get_display_frame() as usize;
        let new_monster_frame = self.monster_frame_tracker.get_display_frame() as usize;

        if game_state.hero_animation_frame != new_hero_frame {
            game_state.hero_animation_frame = new_hero_frame;
            game_state.needs_redraw = true;
        }

        if game_state.monster_animation_frame != new_monster_frame {
            game_state.monster_animation_frame = new_monster_frame;
            game_state.needs_redraw = true;
        }

        // Get enemy_id before borrowing session mutably
        let _enemy_id = match &game_state.idle_farm_session {
            Some(session) => session.enemy_id,
            None => return,
        };

        // Update spawn position for rendering
        if let Some(session) = &mut game_state.idle_farm_session {
            if self.combat_state.phase == CombatPhase::EnemySpawning {
                session.enemy_spawn_position_x = self.animation_controller.spawn_position_x;
                session.enemy_spawning = true;
            } else {
                session.enemy_spawning = false;
                session.enemy_spawn_position_x = 90;
            }
        }

        // Don't call the system animations update functions during combat
        // since we're controlling frames directly through the frame tracker
        // This prevents auto-return to idle and frame conflicts
    }

    /// Get the actual frame count from hero idle GIF
    fn get_hero_idle_frame_count(&self, game_state: &GameState) -> u8 {
        use crate::combat::animations::HeroAnimation;
        use embedded_graphics::pixelcolor::Rgb888;
        use tinygif::Gif;

        let gif_data = HeroAnimation::Idle.gif_data(&game_state.hero.job);
        let gif = Gif::<Rgb888>::from_slice(gif_data).ok();
        gif.map(|g| g.frames().count() as u8).unwrap_or(4)
    }

    /// Get the actual frame count from monster idle GIF
    fn get_monster_idle_frame_count(&self, monster_name: &str) -> u8 {
        use crate::combat::animations::MonsterAnimation;
        use embedded_graphics::pixelcolor::Rgb888;
        use tinygif::Gif;

        let gif_data = MonsterAnimation::Idle.gif_data(monster_name);
        let gif = Gif::<Rgb888>::from_slice(gif_data).ok();
        gif.map(|g| g.frames().count() as u8).unwrap_or(4)
    }
}

/// Main update function for idle farm v2
pub fn update_idle_farm_v2(game_state: &mut GameState, delta_ms: u32) {
    // Create or get combat engine from session
    // For now, we'll create it on-demand
    // TODO: Store in IdleFarmSession

    let session = match &game_state.idle_farm_session {
        Some(session) => session,
        None => return,
    };

    if session.state != IdleFarmState::Active {
        return;
    }

    // For now, create a new engine each time
    // In production, this would be stored in the session
    let mut engine = CombatEngine::new(game_state.last_update_ms);
    engine.update(game_state, delta_ms);
}
