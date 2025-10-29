/// Map data management
///
/// Loads and provides access to map data from JSON.

use heapless::Vec as HeaplessVec;
use serde::Deserialize;

use super::common::LazyData;
use super::enemies::get_enemy_data;

// Embed JSON file at compile time
const MAPS_JSON: &str = include_str!("../tamagotchi/data/maps.json");

/// Map data structure (matches maps.json)
#[derive(Debug, Deserialize, Clone)]
pub struct MapData {
    pub id: u32,
    pub name: &'static str,
    #[serde(default)]
    pub north: Option<u32>,
    #[serde(default)]
    pub south: Option<u32>,
    #[serde(default)]
    pub east: Option<u32>,
    #[serde(default)]
    pub west: Option<u32>,
    #[serde(default)]
    pub enemies: HeaplessVec<u32, 8>,  // Just enemy IDs
    #[serde(default)]
    pub npcs: HeaplessVec<u32, 8>,     // Just NPC IDs
}

// Static storage for parsed map data
static MAPS: LazyData<HeaplessVec<MapData, 16>> = LazyData::new();

// Constants
pub const MAP_PRONTERA_ID: u32 = 1;

/// Parse maps from JSON (done once, cached)
fn parse_maps() -> HeaplessVec<MapData, 16> {
    esp_println::println!("[GAME_DATA] Parsing maps.json...");

    match serde_json_core::from_str::<HeaplessVec<MapData, 16>>(MAPS_JSON) {
        Ok((maps, _)) => {
            esp_println::println!("[GAME_DATA] Successfully parsed {} maps", maps.len());
            for map in &maps {
                esp_println::println!(
                    "  - {} (ID: {}) with {} enemies",
                    map.name,
                    map.id,
                    map.enemies.len()
                );
            }
            maps
        }
        Err(e) => {
            esp_println::println!("[ERROR] Failed to parse maps.json: {:?}", e);
            HeaplessVec::new()
        }
    }
}

/// Get map data by ID
pub fn get_map_data(id: u32) -> Option<&'static MapData> {
    let maps = MAPS.get_or_init(parse_maps);
    maps.iter().find(|m| m.id == id)
}

/// Get all maps
pub fn get_all_maps() -> &'static [MapData] {
    let maps = MAPS.get_or_init(parse_maps);
    // SAFETY: Same as above - data lives in static storage
    unsafe { core::mem::transmute(maps.as_slice()) }
}

/// Get map name by ID
pub fn get_map_name(map_id: u32) -> &'static str {
    get_map_data(map_id).map(|m| m.name).unwrap_or("Unknown")
}

/// Get map connections (N, S, E, W)
pub fn get_map_connections(map_id: u32) -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    get_map_data(map_id)
        .map(|m| (m.north, m.south, m.east, m.west))
        .unwrap_or((None, None, None, None))
}

/// Get enemy IDs from a specific map
pub fn get_map_enemies(map_id: u32) -> HeaplessVec<u32, 8> {
    get_map_data(map_id)
        .map(|m| m.enemies.clone())
        .unwrap_or_else(|| HeaplessVec::new())
}

/// Get all Enemy objects from a specific map
pub fn get_map_enemy_data(map_id: u32) -> HeaplessVec<crate::combat::Enemy, 8> {
    let mut result = HeaplessVec::new();

    if let Some(map) = get_map_data(map_id) {
        for enemy_id in &map.enemies {
            if let Some(enemy) = get_enemy_data(*enemy_id) {
                result.push(enemy).ok();
                if result.is_full() {
                    break;
                }
            }
        }
    }

    result
}

/// Check if a map is a city (has NPCs)
pub fn is_city(map_id: u32) -> bool {
    get_map_data(map_id)
        .map(|m| !m.npcs.is_empty())
        .unwrap_or(false)
}
