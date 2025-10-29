/// Enemy data management
///
/// Loads and provides access to enemy data from JSON.
use heapless::Vec as HeaplessVec;
use serde::Deserialize;

use super::common::LazyData;
use super::drops::DropEntry;

// Embed JSON file at compile time
const ENEMIES_JSON: &str = include_str!("../../assets/data/enemies.json");

/// Enemy data structure (matches enemies.json)
#[derive(Debug, Deserialize, Clone)]
pub struct EnemyData {
    pub id: u32,
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub base_exp: u32,
    #[serde(default)]
    pub drops: HeaplessVec<DropEntry, 8>,
}

// Static storage for parsed enemy data
static ENEMIES: LazyData<HeaplessVec<EnemyData, 16>> = LazyData::new();

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

/// Get enemy data by ID
pub fn get_enemy_data(id: u32) -> Option<crate::combat::Enemy> {
    let enemies = ENEMIES.get_or_init(parse_enemies);

    enemies
        .iter()
        .find(|e| e.id == id)
        .map(|e| crate::combat::Enemy {
            id: e.id,
            name: e.name,
            level: e.level,
            hp: e.hp,
            max_hp: e.hp,
            attack: e.attack,
            defense: e.defense,
            base_exp: e.base_exp,
            zeny_reward: e.base_exp / 10, // Calculate zeny from base_exp
        })
}

/// Get all enemies
pub fn get_all_enemies() -> HeaplessVec<crate::combat::Enemy, 16> {
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
