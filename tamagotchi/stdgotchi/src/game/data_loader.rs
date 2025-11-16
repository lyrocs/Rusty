//! Game Data Loader
//!
//! Centralized JSON data loading for maps, enemies, etc.

use super::rustymon::Element;
use super::skill::{Skill, LearnableSkill};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

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
    // Base stats
    pub str: u32,
    pub int: u32,
    pub dex: u32,
    pub vit: u32,
    pub agi: u32,
    pub luk: u32,
    // Combat stats
    pub hit: u32,
    pub flee: u32,
    pub element: String, // Will be parsed to Element enum
    pub fragment_drop_rate: f32,
    pub fragments_required: u32,
    #[serde(default)]
    pub learnable_skills: Vec<LearnableSkill>,
}

impl EnemyData {
    /// Get the element type as Element enum
    pub fn get_element(&self) -> Element {
        Element::from_str(&self.element).unwrap_or(Element::Neutral)
    }
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
    pub skills: HashMap<u32, Skill>,
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

        // Load skills
        let skills_json = include_str!("../../assets/data/skills.json");
        let skills_vec: Vec<Skill> = serde_json::from_str(skills_json)?;
        let mut skills = HashMap::new();
        for skill in skills_vec {
            skills.insert(skill.id, skill);
        }
        log::info!("Loaded {} skills", skills.len());

        Ok(Self {
            maps,
            enemies,
            skills,
        })
    }

    /// Get map by ID
    pub fn get_map(&self, id: u32) -> Option<&MapData> {
        self.maps.get(&id)
    }

    /// Get all map IDs
    pub fn get_all_map_ids(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.maps.keys().copied().collect();
        ids.sort();
        ids
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

    /// Get skill by ID
    pub fn get_skill(&self, id: u32) -> Option<&Skill> {
        self.skills.get(&id)
    }

    /// Get all skills
    pub fn get_all_skills(&self) -> &HashMap<u32, Skill> {
        &self.skills
    }

    /// Get learnable skills for an enemy/Rustymon species
    pub fn get_learnable_skills(&self, species_id: u32) -> Option<&Vec<LearnableSkill>> {
        self.get_enemy(species_id).map(|e| &e.learnable_skills)
    }
}
