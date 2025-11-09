//! Game Data Loader
//!
//! Centralized JSON data loading for maps, enemies, items, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

/// Drop data from enemies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropData {
    pub item_id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub quantity: u32,
    pub chance: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_slots: Option<Vec<String>>,
}

/// Enemy data loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyData {
    pub id: u32,
    pub name: String,
    pub level: u32,
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub base_exp: u64,
    pub drops: Vec<DropData>,
}

/// Map data loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub north: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub south: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub east: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub west: Option<u32>,
    #[serde(default)]
    pub enemies: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npcs: Option<Vec<u32>>,
}

impl MapData {
    /// Get all connected map IDs
    pub fn connections(&self) -> Vec<u32> {
        let mut connections = Vec::new();
        if let Some(north) = self.north {
            connections.push(north);
        }
        if let Some(south) = self.south {
            connections.push(south);
        }
        if let Some(east) = self.east {
            connections.push(east);
        }
        if let Some(west) = self.west {
            connections.push(west);
        }
        connections
    }

    /// Check if a map ID is connected to this map
    pub fn is_connected(&self, map_id: u32) -> bool {
        self.north == Some(map_id)
            || self.south == Some(map_id)
            || self.east == Some(map_id)
            || self.west == Some(map_id)
    }

    /// Get direction to a connected map
    pub fn direction_to(&self, map_id: u32) -> Option<Direction> {
        if self.north == Some(map_id) {
            Some(Direction::North)
        } else if self.south == Some(map_id) {
            Some(Direction::South)
        } else if self.east == Some(map_id) {
            Some(Direction::East)
        } else if self.west == Some(map_id) {
            Some(Direction::West)
        } else {
            None
        }
    }
}

/// Direction for map navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::North => "North",
            Direction::South => "South",
            Direction::East => "East",
            Direction::West => "West",
        }
    }
}

/// Centralized game data
#[derive(Debug, Clone)]
pub struct GameData {
    pub maps: HashMap<u32, MapData>,
    pub enemies: HashMap<u32, EnemyData>,
}

impl GameData {
    /// Load game data from embedded JSON assets
    pub fn load_from_assets() -> Result<Self, Box<dyn Error>> {
        // Load maps
        let maps_json = include_str!("../../assets/data/maps.json");
        let maps_vec: Vec<MapData> = serde_json::from_str(maps_json)?;
        let mut maps = HashMap::new();
        for map in maps_vec {
            maps.insert(map.id, map);
        }
        log::info!("Loaded {} maps", maps.len());

        // Load enemies
        let enemies_json = include_str!("../../assets/data/enemies.json");
        let enemies_vec: Vec<EnemyData> = serde_json::from_str(enemies_json)?;
        let mut enemies = HashMap::new();
        for enemy in enemies_vec {
            enemies.insert(enemy.id, enemy);
        }
        log::info!("Loaded {} enemies", enemies.len());

        Ok(Self { maps, enemies })
    }

    /// Get map by ID
    pub fn get_map(&self, id: u32) -> Option<&MapData> {
        self.maps.get(&id)
    }

    /// Get enemy by ID
    pub fn get_enemy(&self, id: u32) -> Option<&EnemyData> {
        self.enemies.get(&id)
    }

    /// Get random enemy ID from a map
    pub fn get_random_enemy_for_map(&self, map_id: u32) -> Option<u32> {
        if let Some(map) = self.get_map(map_id) {
            if map.enemies.is_empty() {
                return None;
            }
            // Simple random selection (in real implementation, use proper RNG)
            // For now, just return the first enemy
            Some(map.enemies[0])
        } else {
            None
        }
    }
}
