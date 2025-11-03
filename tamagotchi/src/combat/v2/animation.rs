/// Animation controller for combat v2
///
/// Manages hero and monster animations with preloading support

use crate::combat::animations::{HeroAnimation, MonsterAnimation};
use super::state::CombatPhase;

/// Controls animation state and timing for both hero and monster
#[derive(Debug, Clone)]
pub struct AnimationController {
    // Hero animation state
    pub hero_animation: HeroAnimation,
    pub hero_start_ms: u32,
    pub hero_duration_ms: u32,
    pub hero_preloaded: bool,

    // Monster animation state
    pub monster_animation: MonsterAnimation,
    pub monster_start_ms: u32,
    pub monster_duration_ms: u32,
    pub monster_preloaded: bool,

    // Enemy spawn state
    pub spawn_position_x: i32,
    pub spawn_target_x: i32,
}

impl AnimationController {
    /// Create a new animation controller with idle animations
    pub fn new(start_ms: u32) -> Self {
        Self {
            hero_animation: HeroAnimation::Idle,
            hero_start_ms: start_ms,
            hero_duration_ms: 0,
            hero_preloaded: false,

            monster_animation: MonsterAnimation::Idle,
            monster_start_ms: start_ms,
            monster_duration_ms: 0,
            monster_preloaded: false,

            spawn_position_x: 90, // Final position
            spawn_target_x: 90,
        }
    }

    /// Get animations for a specific combat phase
    pub fn animations_for_phase(phase: &CombatPhase) -> (HeroAnimation, MonsterAnimation) {
        match phase {
            CombatPhase::Idle => (HeroAnimation::Idle, MonsterAnimation::Idle),
            CombatPhase::HeroAttacking => (HeroAnimation::Attacking, MonsterAnimation::Idle),
            CombatPhase::EnemyReacting => (HeroAnimation::Idle, MonsterAnimation::Attacked),
            CombatPhase::EnemyAttacking => (HeroAnimation::Idle, MonsterAnimation::Attacking),
            CombatPhase::HeroReacting => (HeroAnimation::Attacked, MonsterAnimation::Idle),
            CombatPhase::EnemyDying => (HeroAnimation::Idle, MonsterAnimation::Dying),
            CombatPhase::EnemySpawning => (HeroAnimation::Idle, MonsterAnimation::Idle),
        }
    }

    /// Check if animation preload should occur
    pub fn should_preload(phase_remaining_ms: u32, preload_time_ms: u32) -> bool {
        phase_remaining_ms <= preload_time_ms && phase_remaining_ms > 0
    }

    /// Start preloading animation for next phase
    pub fn preload_for_phase(
        &mut self,
        next_phase: &CombatPhase,
        current_ms: u32,
        preload_time_ms: u32,
    ) {
        let (hero_anim, monster_anim) = Self::animations_for_phase(next_phase);

        // Preload hero animation if changed
        if self.hero_animation != hero_anim && !self.hero_preloaded {
            self.hero_animation = hero_anim;
            self.hero_start_ms = current_ms;
            self.hero_preloaded = true;
        }

        // Preload monster animation if changed
        if self.monster_animation != monster_anim && !self.monster_preloaded {
            self.monster_animation = monster_anim;
            self.monster_start_ms = current_ms;
            self.monster_preloaded = true;
        }
    }

    /// Set animations for current phase (called on phase transition)
    pub fn set_for_phase(&mut self, phase: &CombatPhase, current_ms: u32) {
        let (hero_anim, monster_anim) = Self::animations_for_phase(phase);

        // Only change if not already preloaded
        if !self.hero_preloaded {
            self.hero_animation = hero_anim;
            self.hero_start_ms = current_ms;
        } else {
            // Clear preload flag
            self.hero_preloaded = false;
        }

        if !self.monster_preloaded {
            self.monster_animation = monster_anim;
            self.monster_start_ms = current_ms;
        } else {
            // Clear preload flag
            self.monster_preloaded = false;
        }
    }

    /// Update spawn animation (for enemy walk-in)
    pub fn update_spawn_animation(&mut self, elapsed_ms: u32, spawn_duration_ms: u32) {
        const START_X: i32 = -64;
        const TARGET_X: i32 = 90;

        let progress = (elapsed_ms as f32 / spawn_duration_ms as f32).min(1.0);
        let eased_progress = ease_in_out_cubic(progress);

        self.spawn_position_x =
            START_X + ((TARGET_X - START_X) as f32 * eased_progress) as i32;
        self.spawn_target_x = TARGET_X;
    }

    /// Reset spawn position to final position
    pub fn reset_spawn_position(&mut self) {
        self.spawn_position_x = 90;
        self.spawn_target_x = 90;
    }

    /// Initialize spawn animation from left side
    pub fn start_spawn_animation(&mut self) {
        self.spawn_position_x = -64;
        self.spawn_target_x = 90;
    }
}

/// Cubic ease-in-out function for smooth animations
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let val = -2.0 * t + 2.0;
        1.0 - (val * val * val) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_controller_creation() {
        let controller = AnimationController::new(1000);
        assert_eq!(controller.hero_animation, HeroAnimation::Idle);
        assert_eq!(controller.monster_animation, MonsterAnimation::Idle);
        assert_eq!(controller.spawn_position_x, 90);
    }

    #[test]
    fn test_animations_for_phase() {
        let (hero, monster) = AnimationController::animations_for_phase(&CombatPhase::HeroAttacking);
        assert_eq!(hero, HeroAnimation::Attacking);
        assert_eq!(monster, MonsterAnimation::Idle);

        let (hero, monster) = AnimationController::animations_for_phase(&CombatPhase::EnemyReacting);
        assert_eq!(hero, HeroAnimation::Idle);
        assert_eq!(monster, MonsterAnimation::Attacked);
    }

    #[test]
    fn test_should_preload() {
        assert!(AnimationController::should_preload(150, 200));
        assert!(AnimationController::should_preload(200, 200));
        assert!(!AnimationController::should_preload(250, 200));
        assert!(!AnimationController::should_preload(0, 200));
    }

    #[test]
    fn test_spawn_animation() {
        let mut controller = AnimationController::new(1000);
        controller.start_spawn_animation();
        assert_eq!(controller.spawn_position_x, -64);

        controller.update_spawn_animation(0, 2000);
        assert_eq!(controller.spawn_position_x, -64);

        controller.update_spawn_animation(1000, 2000);
        assert!(controller.spawn_position_x > -64 && controller.spawn_position_x < 90);

        controller.update_spawn_animation(2000, 2000);
        assert_eq!(controller.spawn_position_x, 90);
    }

    #[test]
    fn test_easing_function() {
        assert_eq!(ease_in_out_cubic(0.0), 0.0);
        assert_eq!(ease_in_out_cubic(1.0), 1.0);
        assert!(ease_in_out_cubic(0.5) > 0.4 && ease_in_out_cubic(0.5) < 0.6);
    }
}
