//! Save/Load System
//!
//! Handles serialization and persistence of game state to SD card.
//! Supports full Monster Tamer save data including monsters, team, player, and expeditions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use super::KillTracker;
use super::core::{Monster, Team, Player};
use super::systems::expedition::Expedition;

/// Save data structure containing all persistent game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// Current game version (for migration support)
    pub version: u32,

    /// Monster kill tracking
    pub kill_tracker: KillTracker,

    /// Current map location ID
    pub current_location_id: u32,

    /// Total play time in seconds
    pub play_time_seconds: u64,

    /// Save timestamp (unix timestamp)
    pub save_timestamp: u64,

    /// Player's owned monsters
    #[serde(default)]
    pub monsters: Vec<Monster>,

    /// Player's active team
    #[serde(default)]
    pub team: Team,

    /// Player resources (crystals, essences)
    #[serde(default)]
    pub player: Player,

    /// Active expeditions (max 2)
    #[serde(default = "default_expeditions")]
    pub active_expeditions: [Option<Expedition>; 2],

    /// Dungeon progress (highest floor reached per dungeon)
    #[serde(default)]
    pub dungeon_progress: HashMap<String, u16>,
}

fn default_expeditions() -> [Option<Expedition>; 2] {
    [None, None]
}

impl SaveData {
    /// Current save data version
    pub const CURRENT_VERSION: u32 = 7; // Version 7: Monster Tamer Phase 3 (Expeditions)

    /// Default save file name
    pub const SAVE_FILE_NAME: &'static str = "stdgotchi_save.json";

    /// Create full save data with all game state
    pub fn new(
        kill_tracker: KillTracker,
        current_location_id: u32,
        play_time_seconds: u64,
        monsters: Vec<Monster>,
        team: Team,
        player: Player,
        active_expeditions: [Option<Expedition>; 2],
        dungeon_progress: HashMap<String, u16>,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            kill_tracker,
            current_location_id,
            play_time_seconds,
            save_timestamp: Self::current_timestamp(),
            monsters,
            team,
            player,
            active_expeditions,
            dungeon_progress,
        }
    }

    /// Create minimal save data (backwards compatible, creates empty data)
    pub fn new_minimal(
        kill_tracker: KillTracker,
        current_location_id: u32,
        play_time_seconds: u64,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            kill_tracker,
            current_location_id,
            play_time_seconds,
            save_timestamp: Self::current_timestamp(),
            monsters: Vec::new(),
            team: Team::default(),
            player: Player::default(),
            active_expeditions: [None, None],
            dungeon_progress: HashMap::new(),
        }
    }

    /// Get current unix timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Serialize save data to JSON string
    pub fn to_json(&self) -> Result<String, Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }

    /// Deserialize save data from JSON string
    pub fn from_json(json: &str) -> Result<Self, Box<dyn Error>> {
        let save_data: SaveData = serde_json::from_str(json)?;

        // Version check/migration would go here
        if save_data.version > Self::CURRENT_VERSION {
            return Err("Save file is from a newer version".into());
        }

        Ok(save_data)
    }

    /// Save to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = self.to_json()?;

        // Write directly to file (no directory creation needed for simple filenames)
        fs::write(path, json)?;
        log::info!("Game saved successfully");
        Ok(())
    }

    /// Load from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let json = fs::read_to_string(path)?;
        let save_data = Self::from_json(&json)?;
        log::info!("Game loaded successfully");
        Ok(save_data)
    }

    /// Check if save file exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_data_serialization() {
        let kill_tracker = KillTracker::new();
        let save_data = SaveData::new_minimal(
            kill_tracker,
            1,
            3600,
        );

        // Serialize
        let json = save_data.to_json().unwrap();
        assert!(json.contains("version"));
        assert!(json.contains("kill_tracker"));

        // Deserialize
        let loaded = SaveData::from_json(&json).unwrap();
        assert_eq!(loaded.version, SaveData::CURRENT_VERSION);
        assert_eq!(loaded.current_location_id, 1);
    }
}
