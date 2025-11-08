//! Map and Location System
//!
//! Manages world map navigation, locations, and transitions between cities and fields.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::enemy::EnemyType;

/// Type of location in the game world
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LocationType {
    /// Safe zone with services (heal, shop, save)
    City {
        services: Vec<CityService>,
    },
    /// Battle zone with monsters
    Field {
        monsters: Vec<EnemyType>,
        #[serde(default)]
        monster_level_range: (u32, u32), // (min, max)
    },
}

/// Services available in cities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CityService {
    Heal,   // Restore HP/SP
    Shop,   // Buy/sell items
    Save,   // Manual save point
    Storage, // Item storage
}

impl CityService {
    pub fn name(&self) -> &'static str {
        match self {
            CityService::Heal => "Heal",
            CityService::Shop => "Shop",
            CityService::Save => "Save",
            CityService::Storage => "Storage",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            CityService::Heal => "+",
            CityService::Shop => "$",
            CityService::Save => "S",
            CityService::Storage => "#",
        }
    }
}

/// A location in the game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLocation {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub location_type: LocationType,
    pub connections: Vec<String>, // IDs of connected locations
    #[serde(skip)]
    pub background_image: Option<&'static [u8]>, // Optional background GIF (not serialized)
}

impl MapLocation {
    /// Check if this location is a city
    pub fn is_city(&self) -> bool {
        matches!(self.location_type, LocationType::City { .. })
    }

    /// Check if this location is a field
    pub fn is_field(&self) -> bool {
        matches!(self.location_type, LocationType::Field { .. })
    }

    /// Get city services if this is a city
    pub fn services(&self) -> Option<&Vec<CityService>> {
        match &self.location_type {
            LocationType::City { services } => Some(services),
            _ => None,
        }
    }

    /// Get monsters if this is a field
    pub fn monsters(&self) -> Option<&Vec<EnemyType>> {
        match &self.location_type {
            LocationType::Field { monsters, .. } => Some(monsters),
            _ => None,
        }
    }

    /// Get monster level range if this is a field
    pub fn monster_level_range(&self) -> Option<(u32, u32)> {
        match &self.location_type {
            LocationType::Field { monster_level_range, .. } => Some(*monster_level_range),
            _ => None,
        }
    }
}

/// World map containing all locations
#[derive(Debug, Clone)]
pub struct WorldMap {
    pub current_location_id: String,
    locations: HashMap<String, MapLocation>,
}

impl WorldMap {
    /// Create a new world map
    pub fn new(starting_location: String) -> Self {
        Self {
            current_location_id: starting_location,
            locations: HashMap::new(),
        }
    }

    /// Load map from JSON data
    pub fn from_json(json_data: &str, starting_location: String) -> Result<Self, serde_json::Error> {
        #[derive(Deserialize)]
        struct MapData {
            locations: Vec<MapLocation>,
        }

        let map_data: MapData = serde_json::from_str(json_data)?;

        let mut locations = HashMap::new();
        for location in map_data.locations {
            locations.insert(location.id.clone(), location);
        }

        Ok(Self {
            current_location_id: starting_location,
            locations,
        })
    }

    /// Add a location to the map
    pub fn add_location(&mut self, location: MapLocation) {
        self.locations.insert(location.id.clone(), location);
    }

    /// Get current location
    pub fn current_location(&self) -> Option<&MapLocation> {
        self.locations.get(&self.current_location_id)
    }

    /// Get location by ID
    pub fn get_location(&self, id: &str) -> Option<&MapLocation> {
        self.locations.get(id)
    }

    /// Get all connected locations from current position
    pub fn connected_locations(&self) -> Vec<&MapLocation> {
        if let Some(current) = self.current_location() {
            current
                .connections
                .iter()
                .filter_map(|id| self.locations.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Move to a new location (if connected)
    pub fn travel_to(&mut self, destination_id: &str) -> Result<(), String> {
        // Check if destination exists
        if !self.locations.contains_key(destination_id) {
            return Err(format!("Location '{}' does not exist", destination_id));
        }

        // Check if destination is connected to current location
        if let Some(current) = self.current_location() {
            if !current.connections.contains(&destination_id.to_string()) {
                return Err(format!(
                    "Cannot travel to '{}' from '{}'",
                    destination_id, current.name
                ));
            }
        }

        // Move to new location
        self.current_location_id = destination_id.to_string();
        log::info!("Traveled to: {}", destination_id);
        Ok(())
    }

    /// Check if can travel to destination from current location
    pub fn can_travel_to(&self, destination_id: &str) -> bool {
        if let Some(current) = self.current_location() {
            current.connections.contains(&destination_id.to_string())
        } else {
            false
        }
    }

    /// Get all locations
    pub fn all_locations(&self) -> Vec<&MapLocation> {
        self.locations.values().collect()
    }

    /// Get current location ID
    pub fn current_location_id(&self) -> &str {
        &self.current_location_id
    }

    /// Set current location (used when loading from save)
    pub fn set_current_location(&mut self, location_id: String) {
        self.current_location_id = location_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_type() {
        let city = LocationType::City {
            services: vec![CityService::Heal, CityService::Shop],
        };
        assert!(matches!(city, LocationType::City { .. }));

        let field = LocationType::Field {
            monsters: vec![EnemyType::Poring],
            monster_level_range: (1, 5),
        };
        assert!(matches!(field, LocationType::Field { .. }));
    }

    #[test]
    fn test_world_map() {
        let mut map = WorldMap::new("prontera".to_string());

        let prontera = MapLocation {
            id: "prontera".to_string(),
            name: "Prontera".to_string(),
            location_type: LocationType::City {
                services: vec![CityService::Heal],
            },
            connections: vec!["field1".to_string()],
            background_image: None,
        };

        map.add_location(prontera);

        assert_eq!(map.current_location_id, "prontera");
        assert!(map.current_location().is_some());
        assert!(map.current_location().unwrap().is_city());
    }
}
