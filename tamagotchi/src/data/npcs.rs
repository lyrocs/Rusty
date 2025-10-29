/// NPC data management
///
/// Provides NPC information for cities.

use heapless::Vec as HeaplessVec;

use super::maps::get_map_data;

/// Get NPC name by ID
pub fn get_npc_name(npc_id: u32) -> &'static str {
    match npc_id {
        1001 => "Items Trader",
        1002 => "Equipment Trader",
        1003 => "Skill Trader",
        1004 => "Refinery",
        _ => "Unknown NPC",
    }
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
