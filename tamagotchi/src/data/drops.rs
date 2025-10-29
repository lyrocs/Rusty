/// Drop system
///
/// Handles item drops from enemies.

use heapless::Vec as HeaplessVec;
use serde::Deserialize;

use super::enemies::get_enemy_drops;

/// Drop entry structure (flat, simple structure)
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct DropEntry {
    pub item_id: u32,
    pub name: &'static str,
    pub quantity: u16,
    pub chance: f32, // 0.0 to 100.0
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
