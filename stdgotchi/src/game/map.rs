//! Map and Navigation System
//!
//! Manages world map navigation using the data from data_loader

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::data_loader::{Direction, GameData, MapData};

/// World map state
#[derive(Debug, Clone)]
pub struct WorldMap {
    pub current_location_id: u32,
    game_data: GameData,
}

impl WorldMap {
    /// Create a new world map with game data
    pub fn new(game_data: GameData, starting_location_id: u32) -> Self {
        Self {
            current_location_id: starting_location_id,
            game_data,
        }
    }

    /// Get current location data
    pub fn current_location(&self) -> Option<&MapData> {
        self.game_data.get_map(self.current_location_id)
    }

    /// Get location by ID
    pub fn get_location(&self, id: u32) -> Option<&MapData> {
        self.game_data.get_map(id)
    }

    /// Get all connected location IDs from current position
    pub fn connected_location_ids(&self) -> Vec<u32> {
        if let Some(current) = self.current_location() {
            current.connections()
        } else {
            Vec::new()
        }
    }

    /// Get all connected locations with their directions
    pub fn connected_locations_with_directions(&self) -> Vec<(Direction, &MapData)> {
        let mut result = Vec::new();
        if let Some(current) = self.current_location() {
            if let Some(north_id) = current.north {
                if let Some(map) = self.game_data.get_map(north_id) {
                    result.push((Direction::North, map));
                }
            }
            if let Some(south_id) = current.south {
                if let Some(map) = self.game_data.get_map(south_id) {
                    result.push((Direction::South, map));
                }
            }
            if let Some(east_id) = current.east {
                if let Some(map) = self.game_data.get_map(east_id) {
                    result.push((Direction::East, map));
                }
            }
            if let Some(west_id) = current.west {
                if let Some(map) = self.game_data.get_map(west_id) {
                    result.push((Direction::West, map));
                }
            }
        }
        result
    }

    /// Move to a new location (if connected)
    pub fn travel_to(&mut self, destination_id: u32) -> Result<(), String> {
        // Check if destination exists
        if self.game_data.get_map(destination_id).is_none() {
            return Err(format!("Location '{}' does not exist", destination_id));
        }

        // Check if destination is connected to current location
        if let Some(current) = self.current_location() {
            if !current.is_connected(destination_id) {
                return Err(format!(
                    "Cannot travel to '{}' from '{}'",
                    destination_id, current.name
                ));
            }
        }

        // Move to new location
        self.current_location_id = destination_id;
        log::info!("Traveled to: {}", destination_id);
        Ok(())
    }

    /// Travel in a specific direction
    pub fn travel_direction(&mut self, direction: Direction) -> Result<(), String> {
        if let Some(current) = self.current_location() {
            let destination_id = match direction {
                Direction::North => current.north,
                Direction::South => current.south,
                Direction::East => current.east,
                Direction::West => current.west,
            };

            if let Some(dest_id) = destination_id {
                self.travel_to(dest_id)
            } else {
                Err(format!("No location to the {:?}", direction))
            }
        } else {
            Err("Current location not found".to_string())
        }
    }

    /// Check if can travel to destination from current location
    pub fn can_travel_to(&self, destination_id: u32) -> bool {
        if let Some(current) = self.current_location() {
            current.is_connected(destination_id)
        } else {
            false
        }
    }

    /// Get current location ID
    pub fn current_location_id(&self) -> u32 {
        self.current_location_id
    }

    /// Set current location (used when loading from save)
    pub fn set_current_location(&mut self, location_id: u32) {
        self.current_location_id = location_id;
    }

    /// Get available enemies for current map
    pub fn current_map_enemies(&self) -> Vec<u32> {
        if let Some(current) = self.current_location() {
            current.enemies.clone()
        } else {
            Vec::new()
        }
    }

    /// Get a reference to the game data
    pub fn game_data(&self) -> &GameData {
        &self.game_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_map() {
        // Would need to load actual game data for this test
        // Placeholder for now
    }
}
