/// Combat state machine for idle farm v2
///
/// Provides a clean state-based approach to combat timing and transitions

/// Combat phase representing the current state of battle
#[derive(Debug, Clone, PartialEq)]
pub enum CombatPhase {
    /// Both entities idle, waiting for next action
    Idle,
    /// Hero playing full attack animation (damage calculated at end)
    HeroAttacking,
    /// Enemy reacting to hero's attack (hit/dodge animation)
    EnemyReacting,
    /// Enemy playing full attack animation (damage calculated at end)
    EnemyAttacking,
    /// Hero reacting to enemy's attack (hit/dodge animation)
    HeroReacting,
    /// Enemy playing death animation
    EnemyDying,
    /// New enemy entering the battlefield
    EnemySpawning,
}

impl CombatPhase {
    /// Returns true if this phase blocks hero attacks
    pub fn blocks_hero_attack(&self) -> bool {
        matches!(
            self,
            CombatPhase::HeroAttacking
                | CombatPhase::EnemyReacting
                | CombatPhase::EnemyDying
                | CombatPhase::EnemySpawning
        )
    }

    /// Returns true if this phase blocks enemy attacks
    pub fn blocks_enemy_attack(&self) -> bool {
        matches!(
            self,
            CombatPhase::EnemyAttacking
                | CombatPhase::HeroReacting
                | CombatPhase::EnemyDying
                | CombatPhase::EnemySpawning
        )
    }

    /// Returns true if combat actions are paused during this phase
    pub fn is_paused(&self) -> bool {
        matches!(self, CombatPhase::EnemyDying | CombatPhase::EnemySpawning)
    }
}

/// Combat state tracking current phase and timing
#[derive(Debug, Clone)]
pub struct CombatState {
    /// Current combat phase
    pub phase: CombatPhase,
    /// When the current phase started (ms)
    pub phase_start_ms: u32,
    /// How long the current phase lasts (ms)
    pub phase_duration_ms: u32,
    /// Next phase to transition to (if predetermined)
    pub next_phase: Option<CombatPhase>,
    /// Damage calculated during windup, applied during strike
    pub pending_damage: u16,
    /// Whether the pending action will miss
    pub pending_miss: bool,
    /// Whether the pending action uses a skill
    pub pending_skill: bool,
}

impl CombatState {
    /// Create a new combat state in Idle phase
    pub fn new(start_ms: u32) -> Self {
        Self {
            phase: CombatPhase::Idle,
            phase_start_ms: start_ms,
            phase_duration_ms: 0,
            next_phase: None,
            pending_damage: 0,
            pending_miss: false,
            pending_skill: false,
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
    pub fn start_hero_attack(
        &mut self,
        current_ms: u32,
        attack_duration_ms: u32,
        use_skill: bool,
    ) {
        self.phase = CombatPhase::HeroAttacking;
        self.phase_start_ms = current_ms;
        self.phase_duration_ms = attack_duration_ms;
        self.next_phase = None; // Will be set when animation completes
        self.pending_damage = 0;
        self.pending_miss = false;
        self.pending_skill = use_skill;
    }

    /// Start enemy attack sequence (animation plays first, damage calculated at end)
    pub fn start_enemy_attack(
        &mut self,
        current_ms: u32,
        attack_duration_ms: u32,
    ) {
        self.phase = CombatPhase::EnemyAttacking;
        self.phase_start_ms = current_ms;
        self.phase_duration_ms = attack_duration_ms;
        self.next_phase = None; // Will be set when animation completes
        self.pending_damage = 0;
        self.pending_miss = false;
        self.pending_skill = false;
    }

    /// Start enemy death sequence
    pub fn start_enemy_death(&mut self, current_ms: u32, death_duration_ms: u32) {
        self.phase = CombatPhase::EnemyDying;
        self.phase_start_ms = current_ms;
        self.phase_duration_ms = death_duration_ms;
        self.next_phase = Some(CombatPhase::EnemySpawning);
        self.pending_damage = 0;
        self.pending_miss = false;
        self.pending_skill = false;
    }

    /// Start enemy spawn sequence
    pub fn start_enemy_spawn(&mut self, current_ms: u32, spawn_duration_ms: u32) {
        self.phase = CombatPhase::EnemySpawning;
        self.phase_start_ms = current_ms;
        self.phase_duration_ms = spawn_duration_ms;
        self.next_phase = Some(CombatPhase::Idle);
        self.pending_damage = 0;
        self.pending_miss = false;
        self.pending_skill = false;
    }

    /// Consume pending damage value (returns and clears it)
    pub fn consume_pending_damage(&mut self) -> (u16, bool, bool) {
        let damage = self.pending_damage;
        let miss = self.pending_miss;
        let skill = self.pending_skill;
        self.pending_damage = 0;
        self.pending_miss = false;
        self.pending_skill = false;
        (damage, miss, skill)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_state_creation() {
        let state = CombatState::new(1000);
        assert_eq!(state.phase, CombatPhase::Idle);
        assert_eq!(state.phase_start_ms, 1000);
    }

    #[test]
    fn test_phase_timing() {
        let mut state = CombatState::new(1000);
        state.transition_to(CombatPhase::HeroAttacking, 500, 1000);

        assert_eq!(state.phase_elapsed_ms(1250), 250);
        assert_eq!(state.phase_remaining_ms(1250), 250);
        assert!(!state.is_phase_complete(1250));
        assert!(state.is_phase_complete(1500));
    }

    #[test]
    fn test_phase_blocking() {
        assert!(CombatPhase::HeroAttacking.blocks_hero_attack());
        assert!(!CombatPhase::HeroAttacking.blocks_enemy_attack());
        assert!(CombatPhase::EnemyAttacking.blocks_enemy_attack());
        assert!(!CombatPhase::EnemyAttacking.blocks_hero_attack());
    }
}
