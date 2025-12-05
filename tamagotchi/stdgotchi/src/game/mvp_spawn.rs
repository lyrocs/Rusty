//! MVP Spawn Manager
//!
//! Manages spawn timers for Mini MVP and MVP monsters.
//! Each MVP has a specific map and respawn timer after being defeated.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Spawn state for a single MVP monster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvpSpawnState {
    /// The monster ID
    pub monster_id: u32,
    /// When the monster was last killed (Unix timestamp)
    pub last_killed_timestamp: Option<u64>,
    /// When the monster will respawn (Unix timestamp)
    pub spawn_time: u64,
}

impl MvpSpawnState {
    pub fn new(monster_id: u32) -> Self {
        Self {
            monster_id,
            last_killed_timestamp: None,
            spawn_time: current_timestamp(), // Available immediately
        }
    }
}

/// Manager for all MVP spawn timers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MvpSpawnManager {
    /// Map ID -> spawn state for MVPs on that map
    pub spawn_states: HashMap<u32, MvpSpawnState>,
}

impl MvpSpawnManager {
    pub fn new() -> Self {
        Self {
            spawn_states: HashMap::new(),
        }
    }

    /// Register an MVP with its spawn map
    pub fn register_mvp(&mut self, map_id: u32, monster_id: u32) {
        self.spawn_states.insert(map_id, MvpSpawnState::new(monster_id));
    }

    /// Check if an MVP is available to fight on a specific map
    pub fn is_available(&self, map_id: u32) -> bool {
        if let Some(state) = self.spawn_states.get(&map_id) {
            current_timestamp() >= state.spawn_time
        } else {
            false
        }
    }

    /// Get time remaining until spawn (in seconds), or 0 if available
    pub fn time_until_spawn(&self, map_id: u32) -> Option<u64> {
        if let Some(state) = self.spawn_states.get(&map_id) {
            let now = current_timestamp();
            if now < state.spawn_time {
                Some(state.spawn_time - now)
            } else {
                Some(0)
            }
        } else {
            None
        }
    }

    /// Get formatted time string (MM:SS or HH:MM:SS)
    pub fn formatted_time_until_spawn(&self, map_id: u32) -> Option<String> {
        self.time_until_spawn(map_id).map(|seconds| {
            if seconds == 0 {
                "Available!".to_string()
            } else {
                let hours = seconds / 3600;
                let minutes = (seconds % 3600) / 60;
                let secs = seconds % 60;
                if hours > 0 {
                    format!("{}:{:02}:{:02}", hours, minutes, secs)
                } else {
                    format!("{}:{:02}", minutes, secs)
                }
            }
        })
    }

    /// Record that an MVP was killed, starting the respawn timer
    pub fn record_kill(&mut self, map_id: u32, respawn_minutes: u32) {
        let now = current_timestamp();
        let spawn_time = now + (respawn_minutes as u64 * 60);

        if let Some(state) = self.spawn_states.get_mut(&map_id) {
            state.last_killed_timestamp = Some(now);
            state.spawn_time = spawn_time;
        }
    }

    /// Get monster ID for a specific map (if an MVP exists there)
    pub fn get_monster_id(&self, map_id: u32) -> Option<u32> {
        self.spawn_states.get(&map_id).map(|s| s.monster_id)
    }

    /// Check if a map has an MVP
    pub fn has_mvp(&self, map_id: u32) -> bool {
        self.spawn_states.contains_key(&map_id)
    }

    /// Get all map IDs that have MVPs
    pub fn get_all_mvp_maps(&self) -> Vec<u32> {
        self.spawn_states.keys().copied().collect()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvp_spawn_manager() {
        let mut manager = MvpSpawnManager::new();

        // Register an MVP on map 10
        manager.register_mvp(10, 1096); // Angeling

        // Should be available immediately
        assert!(manager.is_available(10));
        assert_eq!(manager.get_monster_id(10), Some(1096));

        // Record kill with 20 minute respawn
        manager.record_kill(10, 20);

        // Should no longer be available
        assert!(!manager.is_available(10));

        // Time until spawn should be around 20 minutes
        let time = manager.time_until_spawn(10).unwrap();
        assert!(time > 0 && time <= 1200);
    }

    #[test]
    fn test_formatted_time() {
        let mut manager = MvpSpawnManager::new();
        manager.register_mvp(10, 1096);

        // Available
        let formatted = manager.formatted_time_until_spawn(10).unwrap();
        assert_eq!(formatted, "Available!");

        // After kill
        manager.record_kill(10, 180); // 3 hours
        let formatted = manager.formatted_time_until_spawn(10).unwrap();
        assert!(formatted.contains(":")); // Should have HH:MM:SS format
    }
}
