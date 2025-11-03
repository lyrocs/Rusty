/// Combat state machine for idle farm v2
///
/// Provides a clean state-based approach to combat timing and transitions
/// Supports SIMULTANEOUS actions (both hero and enemy can attack at once)

/// Individual actor action state
#[derive(Debug, Clone, PartialEq)]
pub enum ActorAction {
    /// Actor is idle
    Idle,
    /// Actor is attacking (animation playing, damage will be applied at end)
    Attacking,
    /// Actor was just hit (shows reaction animation if not attacking)
    Attacked,
}

/// Combat phase representing the current state of battle
#[derive(Debug, Clone, PartialEq)]
pub enum CombatPhase {
    /// Normal combat - both can act independently
    Active,
    /// Enemy playing death animation - pauses combat
    EnemyDying,
    /// New enemy entering the battlefield - pauses combat
    EnemySpawning,
}

impl CombatPhase {
    /// Returns true if this phase blocks hero attacks
    pub fn blocks_hero_attack(&self) -> bool {
        matches!(
            self,
            CombatPhase::EnemyDying | CombatPhase::EnemySpawning
        )
    }

    /// Returns true if this phase blocks enemy attacks
    pub fn blocks_enemy_attack(&self) -> bool {
        matches!(
            self,
            CombatPhase::EnemyDying | CombatPhase::EnemySpawning
        )
    }

    /// Returns true if combat actions are paused during this phase
    pub fn is_paused(&self) -> bool {
        matches!(self, CombatPhase::EnemyDying | CombatPhase::EnemySpawning)
    }
}

/// Combat state tracking current phase and individual actor actions
#[derive(Debug, Clone)]
pub struct CombatState {
    /// Current combat phase (global state: Active, EnemyDying, EnemySpawning)
    pub phase: CombatPhase,
    /// When the current phase started (ms)
    pub phase_start_ms: u32,
    /// How long the current phase lasts (ms)
    pub phase_duration_ms: u32,
    /// Next phase to transition to (if predetermined)
    pub next_phase: Option<CombatPhase>,

    // Hero state (tracked independently)
    /// Hero's current action
    pub hero_action: ActorAction,
    /// Whether the hero's pending action uses a skill
    pub hero_pending_skill: bool,
    /// When hero was attacked (for reaction timing)
    pub hero_attacked_ms: u32,

    // Enemy state (tracked independently)
    /// Enemy's current action
    pub enemy_action: ActorAction,
    /// When enemy was attacked (for reaction timing)
    pub enemy_attacked_ms: u32,
}

impl CombatState {
    /// Create a new combat state in Active phase
    pub fn new(start_ms: u32) -> Self {
        Self {
            phase: CombatPhase::Active,
            phase_start_ms: start_ms,
            phase_duration_ms: 0,
            next_phase: None,
            hero_action: ActorAction::Idle,
            hero_pending_skill: false,
            hero_attacked_ms: 0,
            enemy_action: ActorAction::Idle,
            enemy_attacked_ms: 0,
        }
    }

    /// Get elapsed time in current phase
    pub fn phase_elapsed_ms(&self, current_ms: u32) -> u32 {
        current_ms.saturating_sub(self.phase_start_ms)
    }

    /// Get remaining time in current phase
    pub fn phase_remaining_ms(&self, current_ms: u32) -> u32 {
        let elapsed = self.phase_elapsed_ms(current_ms);
        self.phase_duration_ms.saturating_sub(elapsed)
    }

    /// Check if current phase is complete
    pub fn is_phase_complete(&self, current_ms: u32) -> bool {
        self.phase_elapsed_ms(current_ms) >= self.phase_duration_ms
    }

    /// Transition to a new phase
    pub fn transition_to(&mut self, phase: CombatPhase, duration_ms: u32, current_ms: u32) {
        self.phase = phase;
        self.phase_start_ms = current_ms;
        self.phase_duration_ms = duration_ms;
        self.next_phase = None;
    }

    /// Start hero attack sequence (animation plays first, damage calculated at end)
    pub fn start_hero_attack(&mut self, _current_ms: u32, use_skill: bool) {
        self.hero_action = ActorAction::Attacking;
        self.hero_pending_skill = use_skill;
    }

    /// Complete hero attack and return to idle
    pub fn complete_hero_attack(&mut self) {
        self.hero_action = ActorAction::Idle;
        self.hero_pending_skill = false;
    }

    /// Start enemy attack sequence (animation plays first, damage calculated at end)
    pub fn start_enemy_attack(&mut self, _current_ms: u32) {
        self.enemy_action = ActorAction::Attacking;
    }

    /// Complete enemy attack and return to idle
    pub fn complete_enemy_attack(&mut self) {
        self.enemy_action = ActorAction::Idle;
    }

    /// Mark hero as attacked (shows reaction if not attacking)
    pub fn mark_hero_attacked(&mut self, current_ms: u32) {
        // Only show attacked state if hero is not currently attacking
        if self.hero_action != ActorAction::Attacking {
            self.hero_action = ActorAction::Attacked;
        }
        self.hero_attacked_ms = current_ms;
    }

    /// Mark enemy as attacked (shows reaction if not attacking)
    pub fn mark_enemy_attacked(&mut self, current_ms: u32) {
        // Only show attacked state if enemy is not currently attacking
        if self.enemy_action != ActorAction::Attacking {
            self.enemy_action = ActorAction::Attacked;
        }
        self.enemy_attacked_ms = current_ms;
    }

    /// Clear hero attacked state (return to idle)
    pub fn clear_hero_attacked(&mut self) {
        if self.hero_action == ActorAction::Attacked {
            self.hero_action = ActorAction::Idle;
        }
    }

    /// Clear enemy attacked state (return to idle)
    pub fn clear_enemy_attacked(&mut self) {
        if self.enemy_action == ActorAction::Attacked {
            self.enemy_action = ActorAction::Idle;
        }
    }

    /// Start enemy death sequence
    pub fn start_enemy_death(&mut self, current_ms: u32, death_duration_ms: u32) {
        self.phase = CombatPhase::EnemyDying;
        self.phase_start_ms = current_ms;
        self.phase_duration_ms = death_duration_ms;
        self.next_phase = Some(CombatPhase::EnemySpawning);
        // Reset actor states
        self.hero_action = ActorAction::Idle;
        self.enemy_action = ActorAction::Idle;
    }

    /// Start enemy spawn sequence
    pub fn start_enemy_spawn(&mut self, current_ms: u32, spawn_duration_ms: u32) {
        self.phase = CombatPhase::EnemySpawning;
        self.phase_start_ms = current_ms;
        self.phase_duration_ms = spawn_duration_ms;
        self.next_phase = Some(CombatPhase::Active);
        // Reset actor states
        self.hero_action = ActorAction::Idle;
        self.enemy_action = ActorAction::Idle;
    }

    /// Get hero's pending skill status
    pub fn is_hero_using_skill(&self) -> bool {
        self.hero_pending_skill
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_state_creation() {
        let state = CombatState::new(1000);
        assert_eq!(state.phase, CombatPhase::Active);
        assert_eq!(state.hero_action, ActorAction::Idle);
        assert_eq!(state.enemy_action, ActorAction::Idle);
    }

    #[test]
    fn test_simultaneous_attacks() {
        let mut state = CombatState::new(1000);

        // Both can attack at the same time
        state.start_hero_attack(1000, false);
        state.start_enemy_attack(1000);

        assert_eq!(state.hero_action, ActorAction::Attacking);
        assert_eq!(state.enemy_action, ActorAction::Attacking);
    }

    #[test]
    fn test_animation_priority() {
        let mut state = CombatState::new(1000);

        // Hero attacks
        state.start_hero_attack(1000, false);
        assert_eq!(state.hero_action, ActorAction::Attacking);

        // Enemy hits hero - hero should still show attacking (priority)
        state.mark_hero_attacked(1000);
        assert_eq!(state.hero_action, ActorAction::Attacking);

        // Hero completes attack
        state.complete_hero_attack();
        assert_eq!(state.hero_action, ActorAction::Idle);
    }

    #[test]
    fn test_phase_blocking() {
        assert!(!CombatPhase::Active.blocks_hero_attack());
        assert!(!CombatPhase::Active.blocks_enemy_attack());
        assert!(CombatPhase::EnemyDying.blocks_hero_attack());
        assert!(CombatPhase::EnemyDying.blocks_enemy_attack());
    }
}
