//! RPG Game System Module
//!
//! Implements core RPG mechanics including stats, battle system,
//! hero progression, kill tracking, and map navigation.

pub mod stats;
pub mod hero;
pub mod enemy;
pub mod battle;
pub mod kill_tracker;
pub mod map;
pub mod save;
pub mod data_loader;

pub use stats::*;
pub use hero::*;
pub use enemy::*;
pub use battle::*;
pub use kill_tracker::*;
pub use map::*;
pub use save::*;
pub use data_loader::*;

use bevy_ecs::prelude::*;

/// Game state resource holding all RPG data
#[derive(Resource)]
pub struct GameState {
    pub hero: Hero,
    pub current_enemy: Option<Enemy>,
    pub battle_state: BattleState,
    pub kill_tracker: KillTracker,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            hero: Hero::new(),
            current_enemy: None,
            battle_state: BattleState::default(),
            kill_tracker: KillTracker::default(),
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new enemy
    pub fn spawn_enemy(&mut self, enemy: Enemy) {
        log::info!(
            "Spawned {} (Level {}, HP: {})",
            enemy.name,
            enemy.level,
            enemy.max_hp
        );
        self.current_enemy = Some(enemy);
    }

    /// Handle enemy death
    pub fn on_enemy_death(&mut self) {
        if let Some(enemy) = &self.current_enemy {
            // Award EXP to hero
            let exp_reward = enemy.exp_reward;
            log::info!("{} defeated! Gained {} EXP", enemy.name, exp_reward);

            // Record kill
            self.kill_tracker.record_kill(enemy.id, &enemy.name);

            // Give EXP to hero
            self.hero.gain_exp(exp_reward);
        }
    }
}
