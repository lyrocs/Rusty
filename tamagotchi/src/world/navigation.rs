/// Map navigation and helper functions
///
/// Provides utilities for navigating between maps and accessing map data.

use heapless::Vec as HeaplessVec;

use crate::core::MapId;
use crate::data::{get_city_npcs, get_map_connections, get_map_enemies, get_map_name, is_city};

use super::location::LocationType;

/// Exit from a location
#[derive(Debug, Clone, Copy)]
pub struct MapExit {
    pub direction: &'static str,
    pub destination: MapId,
}

/// Map helper functions (uses generated data from maps.json)
pub struct MapHelper;

impl MapHelper {
    pub fn name(map_id: MapId) -> &'static str {
        get_map_name(map_id)
    }

    pub fn location_type(map_id: MapId) -> LocationType {
        if is_city(map_id) {
            LocationType::City
        } else {
            LocationType::Field
        }
    }

    /// Get available exits from this location (from maps.json)
    pub fn exits(map_id: MapId) -> HeaplessVec<MapExit, 4> {
        let (north, south, east, west) = get_map_connections(map_id);
        let mut exits = HeaplessVec::new();

        if let Some(dest) = north {
            exits
                .push(MapExit {
                    direction: "North",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = south {
            exits
                .push(MapExit {
                    direction: "South",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = east {
            exits
                .push(MapExit {
                    direction: "East",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = west {
            exits
                .push(MapExit {
                    direction: "West",
                    destination: dest,
                })
                .ok();
        }

        exits
    }

    /// Get enemy IDs for a map (from maps.json)
    pub fn enemies(map_id: MapId) -> HeaplessVec<u32, 8> {
        get_map_enemies(map_id)
    }

    /// Get NPCs for city locations (from maps.json)
    pub fn npcs(map_id: MapId) -> HeaplessVec<&'static str, 8> {
        get_city_npcs(map_id)
    }
}
