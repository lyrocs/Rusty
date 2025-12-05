//! Save/Load System
//!
//! Handles serialization and persistence of game state to SD card.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;

use super::KillTracker;
use super::quest::QuestManager;
use super::hero::Hero;
use super::mvp_spawn::MvpSpawnManager;

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

    /// The player's hero
    pub hero: Hero,

    /// Quest progress and state
    #[serde(default)]
    pub quest_manager: QuestManager,

    /// MVP spawn timer tracking
    #[serde(default)]
    pub mvp_spawn_manager: MvpSpawnManager,
}

impl SaveData {
    /// Current save data version
    pub const CURRENT_VERSION: u32 = 6; // Bumped version for skill system and MVP spawns

    /// Default save file name
    pub const SAVE_FILE_NAME: &'static str = "stdgotchi_save.json";

    /// Create new save data from game state
    pub fn new(
        kill_tracker: KillTracker,
        current_location_id: u32,
        play_time_seconds: u64,
        hero: Hero,
        quest_manager: QuestManager,
        mvp_spawn_manager: MvpSpawnManager,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            kill_tracker,
            current_location_id,
            play_time_seconds,
            save_timestamp: Self::current_timestamp(),
            hero,
            quest_manager,
            mvp_spawn_manager,
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
        let rustymon_collection = Vec::new();
        let rustymon_team = RustymonTeam::new();
        let fragment_collection = FragmentCollection::new();
        let quest_manager = QuestManager::new();
        let save_data = SaveData::new(
            kill_tracker,
            1,
            3600,
            rustymon_collection,
            rustymon_team,
            fragment_collection,
            quest_manager,
        );

        // Serialize
        let json = save_data.to_json().unwrap();
        assert!(json.contains("version"));
        assert!(json.contains("rustymon_collection"));
        assert!(json.contains("quest_manager"));

        // Deserialize
        let loaded = SaveData::from_json(&json).unwrap();
        assert_eq!(loaded.version, SaveData::CURRENT_VERSION);
        assert_eq!(loaded.current_location_id, 1);
    }
}
