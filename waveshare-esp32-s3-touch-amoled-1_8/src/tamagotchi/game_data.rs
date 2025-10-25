// Game data loaded from JSON files at compile time
// This module provides access to enemy and map data without generating Rust code

use heapless::Vec as HeaplessVec;
use serde::Deserialize;

// Embed JSON files as strings at compile time
const ENEMIES_JSON: &str = include_str!("data/enemies.json");
const MAPS_JSON: &str = include_str!("data/maps.json");

/// Enemy data structure (matches enemies.json)
#[derive(Debug, Deserialize, Clone)]
pub struct EnemyData {
    pub name: &'static str,
    pub id: u32,
    pub level: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub base_exp: u32,
    pub job_exp: u32,
}

/// Map enemy reference (from maps.json)
#[derive(Debug, Deserialize, Clone)]
pub struct MapEnemyRef {
    pub name: &'static str,
    pub id: u32,
}

/// NPC reference (from maps.json)
#[derive(Debug, Deserialize, Clone)]
pub struct NpcRef {
    pub name: &'static str,
    pub id: u32,
}

/// Map connections (from maps.json)
#[derive(Debug, Deserialize, Clone)]
pub struct MapConnections {
    pub north: Option<u32>,
    pub south: Option<u32>,
    pub east: Option<u32>,
    pub west: Option<u32>,
}

/// Map data structure (matches maps.json)
#[derive(Debug, Deserialize, Clone)]
pub struct MapData {
    pub name: &'static str,
    pub id: u32,
    pub connections: MapConnections,
    #[serde(default)]
    pub enemies: HeaplessVec<MapEnemyRef, 8>,
    #[serde(default)]
    pub npc: HeaplessVec<NpcRef, 8>,
}

// Static storage for parsed data
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

struct LazyData<T> {
    initialized: AtomicBool,
    data: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for LazyData<T> {}

impl<T> LazyData<T> {
    const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            data: UnsafeCell::new(None),
        }
    }

    fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if !self.initialized.load(Ordering::Acquire) {
            unsafe {
                *self.data.get() = Some(init());
            }
            self.initialized.store(true, Ordering::Release);
        }
        unsafe { (*self.data.get()).as_ref().unwrap() }
    }
}

static ENEMIES: LazyData<HeaplessVec<EnemyData, 16>> = LazyData::new();
static MAPS: LazyData<HeaplessVec<MapData, 16>> = LazyData::new();

/// Parse enemies from JSON (done once, cached)
fn parse_enemies() -> HeaplessVec<EnemyData, 16> {
    let mut enemies = HeaplessVec::new();

    // Parse JSON using serde-json-core
    if let Ok((parsed, _)) = serde_json_core::from_str::<HeaplessVec<EnemyData, 16>>(ENEMIES_JSON) {
        enemies = parsed;
    } else {
        esp_println::println!("[ERROR] Failed to parse enemies.json");
    }

    enemies
}

/// Parse maps from JSON (done once, cached)
fn parse_maps() -> HeaplessVec<MapData, 16> {
    let mut maps = HeaplessVec::new();

    // Parse JSON using serde-json-core
    if let Ok((parsed, _)) = serde_json_core::from_str::<HeaplessVec<MapData, 16>>(MAPS_JSON) {
        maps = parsed;
    } else {
        esp_println::println!("[ERROR] Failed to parse maps.json");
    }

    maps
}

/// Get enemy data by ID
pub fn get_enemy_data(id: u32) -> Option<crate::tamagotchi::models::Enemy> {
    let enemies = ENEMIES.get_or_init(parse_enemies);

    enemies
        .iter()
        .find(|e| e.id == id)
        .map(|e| crate::tamagotchi::models::Enemy {
            id: e.id,
            name: e.name,
            level: e.level,
            hp: e.hp,
            max_hp: e.max_hp,
            attack: e.attack,
            defense: e.defense,
            base_exp: e.base_exp,
            job_exp: e.job_exp,
            zeny_reward: (e.level as u32 * 10),
        })
}

/// Get map name by ID
pub fn get_map_name(id: u32) -> Option<&'static str> {
    let maps = MAPS.get_or_init(parse_maps);
    maps.iter().find(|m| m.id == id).map(|m| m.name)
}

/// Get map connections (North, South, East, West)
pub fn get_map_connections(id: u32) -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    let maps = MAPS.get_or_init(parse_maps);

    if let Some(map) = maps.iter().find(|m| m.id == id) {
        (
            map.connections.north,
            map.connections.south,
            map.connections.east,
            map.connections.west,
        )
    } else {
        (None, None, None, None)
    }
}

/// Get enemy IDs for a map (dynamically from JSON)
/// Returns a heapless Vec since we can't return &'static to dynamically parsed data
pub fn get_map_enemies(map_id: u32) -> HeaplessVec<u32, 8> {
    let maps = MAPS.get_or_init(parse_maps);

    if let Some(map) = maps.iter().find(|m| m.id == map_id) {
        // Convert enemy references to IDs
        let mut enemy_ids = HeaplessVec::new();
        for enemy_ref in &map.enemies {
            enemy_ids.push(enemy_ref.id).ok();
        }
        enemy_ids
    } else {
        HeaplessVec::new()
    }
}

/// Check if map is a city
pub fn is_city(map_id: u32) -> bool {
    let maps = MAPS.get_or_init(parse_maps);

    if let Some(map) = maps.iter().find(|m| m.id == map_id) {
        !map.npc.is_empty() && map.enemies.is_empty()
    } else {
        false
    }
}

/// Get NPC names for a city (dynamically from JSON)
/// Returns a heapless Vec since we can't return &'static to dynamically parsed data
pub fn get_city_npcs(map_id: u32) -> HeaplessVec<&'static str, 8> {
    let maps = MAPS.get_or_init(parse_maps);

    if let Some(map) = maps.iter().find(|m| m.id == map_id) {
        // Convert NPC references to names
        let mut npc_names = HeaplessVec::new();
        for npc_ref in &map.npc {
            npc_names.push(npc_ref.name).ok();
        }
        npc_names
    } else {
        HeaplessVec::new()
    }
}

// Map ID constants
pub const MAP_PRONTERA_ID: u32 = 1;
pub const MAP_PRONTERASOUTH_ID: u32 = 2;
pub const MAP_PRONTERAWEST_ID: u32 = 3;
pub const MAP_PRONTERAEAST_ID: u32 = 5;
pub const MAP_PRONTERANORTH_ID: u32 = 4;
