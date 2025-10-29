// Game data loaded from JSON files at compile time

use heapless::Vec as HeaplessVec;
use serde::Deserialize;

// Embed JSON files as strings at compile time
const ENEMIES_JSON: &str = include_str!("data/enemies.json");
const MAPS_JSON: &str = include_str!("data/maps.json");

/// Drop entry structure (flat, simple structure)
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct DropEntry {
    pub item_id: u32,
    pub name: &'static str,
    pub quantity: u16,
    pub chance: f32, // 0.0 to 100.0
}

/// Enemy data structure (matches simplified enemies.json)
#[derive(Debug, Deserialize, Clone)]
pub struct EnemyData {
    pub id: u32,
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub base_exp: u32,
    pub job_exp: u32,
    #[serde(default)]
    pub drops: HeaplessVec<DropEntry, 8>,
}

/// Map data structure (matches simplified maps.json)
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
    esp_println::println!("[GAME_DATA] Parsing enemies.json...");

    match serde_json_core::from_str::<HeaplessVec<EnemyData, 16>>(ENEMIES_JSON) {
        Ok((enemies, _)) => {
            esp_println::println!("[GAME_DATA] Successfully parsed {} enemies", enemies.len());
            for enemy in &enemies {
                esp_println::println!(
                    "  - {} (ID: {}, Lvl: {}) with {} drops",
                    enemy.name,
                    enemy.id,
                    enemy.level,
                    enemy.drops.len()
                );
            }
            enemies
        }
        Err(e) => {
            esp_println::println!("[ERROR] Failed to parse enemies.json: {:?}", e);
            HeaplessVec::new()
        }
    }
}

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
            max_hp: e.hp,
            attack: e.attack,
            defense: e.defense,
            base_exp: e.base_exp,
            job_exp: e.job_exp,
            zeny_reward: e.base_exp / 10, // Calculate zeny from base_exp
        })
}

/// Get map data by ID
pub fn get_map_data(id: u32) -> Option<&'static MapData> {
    let maps = MAPS.get_or_init(parse_maps);
    maps.iter().find(|m| m.id == id)
}

/// Get enemy IDs from a specific map
pub fn get_map_enemies(map_id: u32) -> HeaplessVec<u32, 8> {
    get_map_data(map_id)
        .map(|m| m.enemies.clone())
        .unwrap_or_else(|| HeaplessVec::new())
}

/// Get all Enemy objects from a specific map
pub fn get_map_enemy_data(map_id: u32) -> HeaplessVec<crate::tamagotchi::models::Enemy, 8> {
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

/// Get drop table for an enemy ID
pub fn get_enemy_drops(enemy_id: u32) -> &'static [DropEntry] {
    let enemies = ENEMIES.get_or_init(parse_enemies);

    for enemy in enemies.iter() {
        if enemy.id == enemy_id {
            // SAFETY: We transmute the lifetime to 'static because the data is stored
            // in a static lazy-initialized structure that lives for the program lifetime
            return unsafe { core::mem::transmute(enemy.drops.as_slice()) };
        }
    }

    &[]
}

/// Get all maps
pub fn get_all_maps() -> &'static [MapData] {
    let maps = MAPS.get_or_init(parse_maps);
    // SAFETY: Same as above - data lives in static storage
    unsafe { core::mem::transmute(maps.as_slice()) }
}

/// Get all enemies
pub fn get_all_enemies() -> HeaplessVec<crate::tamagotchi::models::Enemy, 16> {
    let enemies = ENEMIES.get_or_init(parse_enemies);
    let mut result = HeaplessVec::new();

    for enemy_data in enemies.iter() {
        if let Some(enemy) = get_enemy_data(enemy_data.id) {
            result.push(enemy).ok();
            if result.is_full() {
                break;
            }
        }
    }

    result
}

// Constants
pub const MAP_PRONTERA_ID: u32 = 1;

/// Get map name by ID
pub fn get_map_name(map_id: u32) -> &'static str {
    get_map_data(map_id).map(|m| m.name).unwrap_or("Unknown")
}

/// Check if a map is a city (has NPCs)
pub fn is_city(map_id: u32) -> bool {
    get_map_data(map_id)
        .map(|m| !m.npcs.is_empty())
        .unwrap_or(false)
}

/// Get NPC names in a city
pub fn get_city_npcs(map_id: u32) -> HeaplessVec<&'static str, 8> {
    let mut result = HeaplessVec::new();

    if let Some(map) = get_map_data(map_id) {
        for npc_id in &map.npcs {
            let npc_name = get_npc_name(*npc_id);
            result.push(npc_name).ok();
            if result.is_full() {
                break;
            }
        }
    }

    result
}

/// Get NPC name by ID
fn get_npc_name(npc_id: u32) -> &'static str {
    match npc_id {
        1001 => "Items Trader",
        1002 => "Equipment Trader",
        1003 => "Skill Trader",
        1004 => "Refinery",
        _ => "Unknown NPC",
    }
}

/// Get map connections (N, S, E, W)
pub fn get_map_connections(map_id: u32) -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    get_map_data(map_id)
        .map(|m| (m.north, m.south, m.east, m.west))
        .unwrap_or((None, None, None, None))
}

/// Get item name by ID (simplified - returns static strings)
pub fn get_item_name(item_id: u32) -> &'static str {
    // First check if it's in enemy drops
    let enemies = ENEMIES.get_or_init(parse_enemies);
    for enemy in enemies.iter() {
        for drop in &enemy.drops {
            if drop.item_id == item_id {
                return drop.name;
            }
        }
    }

    // Fallback to generic name
    match item_id {
        909 => "Jellopy",
        512 => "Apple",
        1208 => "Main Gauche",
        4001 => "Poring Card",
        914 => "Fluff",
        511 => "Green Herb",
        4002 => "Fabre Card",
        939 => "Bee Sting",
        4003 => "Hornet Card",
        955 => "Worm Peeling",
        507 => "Red Herb",
        4004 => "Thief Bug Card",
        _ => "Unknown Item",
    }
}

/// Roll for drops when an enemy is defeated
pub fn roll_drops(enemy_id: u32, rng_value: u8) -> HeaplessVec<(u32, &'static str, u16), 4> {
    let mut result = HeaplessVec::new();
    let drops = get_enemy_drops(enemy_id);

    // Use rng_value (0-255) to determine what drops
    // Convert to 0-100 range for percentage comparison
    let roll = (rng_value as f32 / 255.0) * 100.0;

    for drop in drops {
        // Simple drop chance check
        if roll < drop.chance {
            result.push((drop.item_id, drop.name, drop.quantity)).ok();
            if result.is_full() {
                break;
            }
        }
    }

    result
}
