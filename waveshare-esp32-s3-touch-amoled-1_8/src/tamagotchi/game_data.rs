// Game data loaded from JSON files at compile time
// This module provides access to enemy and map data without generating Rust code

use heapless::{String, Vec as HeaplessVec};
use serde::Deserialize;

// Embed JSON files as strings at compile time
const ENEMIES_JSON: &str = include_str!("data/enemies.json");
const MAPS_JSON: &str = include_str!("data/maps.json");

/// Drop table entry (simplified structure for runtime use)
#[derive(Debug, Clone, Copy)]
pub struct DropEntry {
    pub item_id: u32,
    pub item_name: &'static str,
    pub quantity: u16,
    pub chance: f32, // 0.0 to 100.0
}

/// Enemy data structure (matches enemies.json, but we skip drops in deserialization)
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
    // Note: drops are parsed separately due to no_std limitations
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

// Lazy-loaded drop tables parsed from JSON
struct DropTableCache {
    initialized: AtomicBool,
    // Store drop tables for each enemy (enemy_id -> Vec of drops)
    tables: UnsafeCell<Option<HeaplessVec<(u32, HeaplessVec<DropEntry, 8>), 16>>>,
}

unsafe impl Sync for DropTableCache {}

impl DropTableCache {
    const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            tables: UnsafeCell::new(None),
        }
    }

    fn get_or_init(&self) -> &HeaplessVec<(u32, HeaplessVec<DropEntry, 8>), 16> {
        if !self.initialized.load(Ordering::Acquire) {
            unsafe {
                *self.tables.get() = Some(parse_drop_tables());
            }
            self.initialized.store(true, Ordering::Release);
        }
        unsafe { (*self.tables.get()).as_ref().unwrap() }
    }
}

static DROP_TABLES: DropTableCache = DropTableCache::new();

/// Parse drop tables from enemies.json
/// This manually extracts drops from the JSON since serde untagged enums don't work in no_std
fn parse_drop_tables() -> HeaplessVec<(u32, HeaplessVec<DropEntry, 8>), 16> {
    esp_println::println!("[DROP PARSER] Starting JSON drop table parsing...");
    let mut all_tables = HeaplessVec::new();

    let json = ENEMIES_JSON;

    // Split by enemy objects - each starts with opening brace after a comma or array start
    // Strategy: find each "id": value, then look for "drops": [...] that follows it
    let mut search_pos = 0;

    while search_pos < json.len() {
        // Find next enemy ID
        let Some(id_pos) = json[search_pos..].find("\"id\":") else {
            break;
        };
        let id_abs_pos = search_pos + id_pos;

        // Extract the ID value
        let after_id = &json[id_abs_pos + 5..];
        let trimmed = after_id.trim_start();
        let mut enemy_id_str = String::<16>::new();
        for ch in trimmed.chars() {
            if ch.is_ascii_digit() {
                enemy_id_str.push(ch).ok();
            } else {
                break;
            }
        }
        let enemy_id = enemy_id_str.parse::<u32>().unwrap_or(0);

        if enemy_id == 0 {
            search_pos = id_abs_pos + 5;
            continue;
        }

        // Now find the "drops": array that belongs to this enemy
        // It should be after this ID and before the next enemy object
        let after_id_section = &json[id_abs_pos..];

        // Find the next enemy object (starts with opening brace)
        let next_enemy_pos = if let Some(pos) = after_id_section[10..].find("\n    {") {
            pos + 10
        } else {
            after_id_section.len()
        };

        // Search for "drops": within this enemy's section
        let enemy_section = &after_id_section[..next_enemy_pos];

        if let Some(drops_pos) = enemy_section.find("\"drops\":") {
            // Find the drops array
            let after_drops = &enemy_section[drops_pos + 8..];

            if let Some(arr_start) = after_drops.find('[') {
                if let Some(arr_end) = after_drops[arr_start..].find(']') {
                    let drops_json = &after_drops[arr_start + 1..arr_start + arr_end];

                    // Parse each drop object
                    let mut drops = HeaplessVec::new();

                    // Split by "}, {" to separate drop objects
                    let drop_parts: heapless::Vec<&str, 16> = drops_json
                        .split("        },")
                        .filter(|s| !s.trim().is_empty())
                        .collect();

                    for drop_obj in drop_parts.iter() {
                        if let Some(drop_entry) = parse_drop_entry(drop_obj) {
                            drops.push(drop_entry).ok();
                            if drops.is_full() {
                                break;
                            }
                        }
                    }

                    if !drops.is_empty() {
                        let enemy_name = extract_enemy_name(enemy_section);
                        esp_println::println!(
                            "[DROP PARSER] Enemy {} ({}) has {} drop entries:",
                            enemy_id,
                            enemy_name,
                            drops.len()
                        );
                        // Log each drop for verification
                        for drop in drops.iter() {
                            esp_println::println!(
                                "  - {} (ID: {}) x{} @ {}%",
                                drop.item_name,
                                drop.item_id,
                                drop.quantity,
                                drop.chance
                            );
                        }
                        all_tables.push((enemy_id, drops)).ok();
                    }
                }
            }
        }

        // Move to next enemy
        search_pos = id_abs_pos + next_enemy_pos;

        if all_tables.is_full() {
            break;
        }
    }

    esp_println::println!(
        "[DROP PARSER] Parsed {} enemy drop tables from JSON",
        all_tables.len()
    );
    all_tables
}

/// Extract enemy name from JSON section
fn extract_enemy_name(section: &str) -> &str {
    if let Some(name_pos) = section.find("\"name\":") {
        let after_name = &section[name_pos + 7..].trim_start();
        if let Some(quote_start) = after_name.find('"') {
            let after_quote = &after_name[quote_start + 1..];
            if let Some(quote_end) = after_quote.find('"') {
                return &after_quote[..quote_end];
            }
        }
    }
    "Unknown"
}

/// Parse a single drop entry from JSON
fn parse_drop_entry(drop_json: &str) -> Option<DropEntry> {
    use heapless::String;

    // Extract chance
    let chance = if let Some(chance_pos) = drop_json.find("\"chance\":") {
        let after_chance = &drop_json[chance_pos + 9..].trim_start();
        let mut num_str = String::<16>::new();
        for ch in after_chance.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(ch).ok();
            } else {
                break;
            }
        }
        num_str.parse::<f32>().unwrap_or(0.0)
    } else {
        return None;
    };

    // Check if it's Item or Equipment
    let is_item = drop_json.contains("\"Item\"");

    // Extract id
    let id_pattern = if is_item { "\"Item\"" } else { "\"Equipment\"" };
    let search_start = drop_json.find(id_pattern)?;
    let id_search = &drop_json[search_start..];

    let item_id = if let Some(id_pos) = id_search.find("\"id\":") {
        let after_id = &id_search[id_pos + 5..].trim_start();
        let mut num_str = String::<16>::new();
        for ch in after_id.chars() {
            if ch.is_ascii_digit() {
                num_str.push(ch).ok();
            } else {
                break;
            }
        }
        num_str.parse().unwrap_or(0)
    } else {
        return None;
    };

    // Extract name
    let item_name = if let Some(name_pos) = id_search.find("\"name\":") {
        let after_name = &id_search[name_pos + 7..].trim_start();
        if let Some(quote_start) = after_name.find('"') {
            let after_quote = &after_name[quote_start + 1..];
            if let Some(quote_end) = after_quote.find('"') {
                // Leak the string to get 'static lifetime
                let name_slice = &after_quote[..quote_end];
                Some(leak_str(name_slice))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }?;

    // Extract quantity (default 1 for equipment)
    let quantity = if is_item {
        if let Some(qty_pos) = id_search.find("\"quantity\":") {
            let after_qty = &id_search[qty_pos + 11..].trim_start();
            let mut num_str = String::<16>::new();
            for ch in after_qty.chars() {
                if ch.is_ascii_digit() {
                    num_str.push(ch).ok();
                } else {
                    break;
                }
            }
            num_str.parse().unwrap_or(1)
        } else {
            1
        }
    } else {
        1 // Equipment always quantity 1
    };

    Some(DropEntry {
        item_id,
        item_name,
        quantity,
        chance,
    })
}

/// Leak a string slice to get 'static lifetime (safe because JSON is embedded at compile time)
fn leak_str(s: &str) -> &'static str {
    use core::mem;
    unsafe { mem::transmute::<&str, &'static str>(s) }
}

/// Get drop table for an enemy ID (loaded from JSON)
/// Returns a list of possible drops with their chances
pub fn get_enemy_drops(enemy_id: u32) -> &'static [DropEntry] {
    let tables = DROP_TABLES.get_or_init();

    // Find the drop table for this enemy
    for (id, drops) in tables.iter() {
        if *id == enemy_id {
            // Safe to transmute because the data is stored in a static
            return unsafe {
                core::mem::transmute::<&[DropEntry], &'static [DropEntry]>(drops.as_slice())
            };
        }
    }

    // No drops for this enemy
    &[]
}

/// Roll for item drops when an enemy is defeated
/// Returns a list of items that dropped based on RNG
pub fn roll_drops(enemy_id: u32, rng_value: u8) -> HeaplessVec<(u32, &'static str, u16), 4> {
    let mut drops = HeaplessVec::new();
    let drop_table = get_enemy_drops(enemy_id);

    // Use different bits of the RNG value for each potential drop
    // This gives us pseudo-random chances for multiple drops
    let mut shift = 0;
    for drop in drop_table {
        // Calculate random value 0-100 using bits from rng_value
        let random_val = ((rng_value.wrapping_mul(37).wrapping_add(shift * 17)) % 100) as f32;

        if random_val < drop.chance {
            // Item dropped!
            drops
                .push((drop.item_id, drop.item_name, drop.quantity))
                .ok();
            if drops.is_full() {
                break;
            }
        }
        shift += 1;
    }

    drops
}

/// Get item name by ID (searches all drop tables loaded from JSON)
pub fn get_item_name(item_id: u32) -> Option<&'static str> {
    let tables = DROP_TABLES.get_or_init();

    // Search through all enemy drop tables to find the item name
    for (_, drops) in tables.iter() {
        for drop in drops.iter() {
            if drop.item_id == item_id {
                return Some(drop.item_name);
            }
        }
    }
    None
}
