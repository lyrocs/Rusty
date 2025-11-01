/// Item data management
///
/// Provides item information and lookups.

use super::enemies::get_enemy_drops;

/// Get item name by ID
pub fn get_item_name(item_id: u32) -> &'static str {
    // First check if it's in enemy drops
    let enemies = super::enemies::get_all_enemies();
    for enemy in enemies.iter() {
        let drops = get_enemy_drops(enemy.id);
        for drop in drops {
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

/// Get item icon GIF data by ID (returns compiled-in bytes)
pub fn get_item_icon(item_id: u32) -> Option<&'static [u8]> {
    match item_id {
        909 => Some(include_bytes!("../../assets/images/items/909.gif")),  // Jellopy
        914 => Some(include_bytes!("../../assets/images/items/914.gif")),  // Fluff
        938 => Some(include_bytes!("../../assets/images/items/938.gif")),  // Sticky Mucus
        939 => Some(include_bytes!("../../assets/images/items/939.gif")),  // Bee Sting
        // Add more item icons as they become available
        _ => None,
    }
}
