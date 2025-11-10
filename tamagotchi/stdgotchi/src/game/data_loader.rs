//! Game Data Loader
//!
//! Centralized JSON data loading for maps, enemies, items, etc.

use super::item::{ItemData, ItemDrop, Recipe, UpgradeRecipe};
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
    pub gold_min: u32,
    pub gold_max: u32,
    pub drops: Vec<ItemDrop>,
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

/// Items JSON structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ItemsJson {
    materials: Vec<ItemData>,
    equipment: Vec<ItemData>,
}

/// Recipes JSON structure (by city)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecipesJson {
    prontera: Vec<Recipe>,
    #[serde(default)]
    payon: Vec<Recipe>,
    #[serde(default)]
    geffen: Vec<Recipe>,
}

/// Upgrade recipes JSON structure (by equipment type)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpgradeRecipesJson {
    weapon_upgrades: Vec<UpgradeRecipe>,
    armor_upgrades: Vec<UpgradeRecipe>,
    shoes_upgrades: Vec<UpgradeRecipe>,
    garment_upgrades: Vec<UpgradeRecipe>,
    accessory_upgrades: Vec<UpgradeRecipe>,
    headgear_upgrades: Vec<UpgradeRecipe>,
}

/// Centralized game data
#[derive(Debug, Clone)]
pub struct GameData {
    pub maps: HashMap<u32, MapData>,
    pub enemies: HashMap<u32, EnemyData>,
    pub items: HashMap<u32, ItemData>,
    pub recipes_by_city: HashMap<String, Vec<Recipe>>,
    pub upgrade_recipes: HashMap<String, Vec<UpgradeRecipe>>,
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

        // Load items
        let items_json = include_str!("../../assets/data/items.json");
        let items_data: ItemsJson = serde_json::from_str(items_json)?;
        let mut items = HashMap::new();
        for item in items_data.materials.into_iter().chain(items_data.equipment.into_iter()) {
            items.insert(item.id, item);
        }
        log::info!("Loaded {} items", items.len());

        // Load recipes
        let recipes_json = include_str!("../../assets/data/recipes.json");
        let recipes_data: RecipesJson = serde_json::from_str(recipes_json)?;
        let mut recipes_by_city = HashMap::new();
        recipes_by_city.insert("prontera".to_string(), recipes_data.prontera);
        recipes_by_city.insert("payon".to_string(), recipes_data.payon);
        recipes_by_city.insert("geffen".to_string(), recipes_data.geffen);
        let total_recipes: usize = recipes_by_city.values().map(|r| r.len()).sum();
        log::info!("Loaded {} recipes across {} cities", total_recipes, recipes_by_city.len());

        // Load upgrade recipes
        let upgrade_json = include_str!("../../assets/data/upgrade_recipes.json");
        let upgrade_data: UpgradeRecipesJson = serde_json::from_str(upgrade_json)?;
        let mut upgrade_recipes = HashMap::new();
        upgrade_recipes.insert("weapon".to_string(), upgrade_data.weapon_upgrades);
        upgrade_recipes.insert("armor".to_string(), upgrade_data.armor_upgrades);
        upgrade_recipes.insert("shoes".to_string(), upgrade_data.shoes_upgrades);
        upgrade_recipes.insert("garment".to_string(), upgrade_data.garment_upgrades);
        upgrade_recipes.insert("accessory".to_string(), upgrade_data.accessory_upgrades);
        upgrade_recipes.insert("headgear".to_string(), upgrade_data.headgear_upgrades);
        log::info!("Loaded upgrade recipes for {} equipment types", upgrade_recipes.len());

        Ok(Self {
            maps,
            enemies,
            items,
            recipes_by_city,
            upgrade_recipes
        })
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

    /// Get item data by ID
    pub fn get_item(&self, id: u32) -> Option<&ItemData> {
        self.items.get(&id)
    }

    /// Get recipes for a specific city
    pub fn get_recipes_for_city(&self, city: &str) -> Option<&Vec<Recipe>> {
        self.recipes_by_city.get(city)
    }

    /// Get upgrade recipe for equipment type and level
    pub fn get_upgrade_recipe(&self, equipment_type: &str, from_level: u32) -> Option<&UpgradeRecipe> {
        if let Some(recipes) = self.upgrade_recipes.get(equipment_type) {
            recipes.iter().find(|r| r.from_level == from_level)
        } else {
            None
        }
    }

    /// Get all item data (for inventory display)
    pub fn get_all_items(&self) -> &HashMap<u32, ItemData> {
        &self.items
    }

    /// Get all upgrade recipes
    pub fn get_upgrade_recipes(&self) -> &HashMap<String, Vec<UpgradeRecipe>> {
        &self.upgrade_recipes
    }
}
