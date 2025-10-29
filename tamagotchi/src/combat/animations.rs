/// Combat animations
///
/// Manages monster and hero animation states and GIF data.

/// Monster GIF animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterAnimation {
    Idle,      // 0.gif - loops
    Attacking, // 16.gif - plays once
    Dying,     // 32.gif - plays once
}

impl MonsterAnimation {
    /// Get GIF data for a specific monster and animation state
    pub fn gif_data(&self, monster_name: &str) -> &'static [u8] {
        // Convert monster name to lowercase for folder matching
        let monster_lower = monster_name.to_lowercase();

        match (monster_lower.as_str(), self) {
            // Poring animations
            ("poring", MonsterAnimation::Idle) => include_bytes!("../../assets/images/poring/0.gif"),
            ("poring", MonsterAnimation::Attacking) => include_bytes!("../../assets/images/poring/16.gif"),
            ("poring", MonsterAnimation::Dying) => include_bytes!("../../assets/images/poring/32.gif"),

            // Fabre animations
            ("fabre", MonsterAnimation::Idle) => include_bytes!("../../assets/images/fabre/0.gif"),
            ("fabre", MonsterAnimation::Attacking) => include_bytes!("../../assets/images/fabre/16.gif"),
            ("fabre", MonsterAnimation::Dying) => include_bytes!("../../assets/images/fabre/32.gif"),

            // Default fallback to Poring if monster not found
            _ => {
                esp_println::println!(
                    "[WARNING] No GIF found for monster '{}', using Poring",
                    monster_name
                );
                match self {
                    MonsterAnimation::Idle => include_bytes!("../../assets/images/poring/0.gif"),
                    MonsterAnimation::Attacking => include_bytes!("../../assets/images/poring/16.gif"),
                    MonsterAnimation::Dying => include_bytes!("../../assets/images/poring/32.gif"),
                }
            }
        }
    }

    pub fn should_loop(&self) -> bool {
        matches!(self, MonsterAnimation::Idle)
    }
}

/// Get monster attacked GIF (24.gif) for a specific monster
pub fn get_monster_attacked_gif(monster_name: &str) -> &'static [u8] {
    let monster_lower = monster_name.to_lowercase();

    match monster_lower.as_str() {
        "poring" => include_bytes!("../../assets/images/poring/24.gif"),
        "fabre" => include_bytes!("../../assets/images/fabre/24.gif"),
        _ => {
            esp_println::println!(
                "[WARNING] No attacked GIF found for monster '{}', using Poring",
                monster_name
            );
            include_bytes!("../../assets/images/poring/24.gif")
        }
    }
}

/// Monster attacked animation (when hero attacks monster)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterAttackedAnimation {
    Normal,   // Not being attacked
    Attacked, // 24.gif - plays once when hero attacks
}

/// Hero GIF animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroAnimation {
    Resting,   // 16.gif - loops (shown on rest page)
    Idle,      // 36.gif - loops (main loop on battle/farm)
    Attacking, // 84.gif - plays once (hero attacks)
    Attacked,  // 52.gif - plays once (hero takes damage)
}

impl HeroAnimation {
    pub fn gif_data(&self) -> &'static [u8] {
        match self {
            HeroAnimation::Resting => include_bytes!("../../assets/images/swordman/16.gif"),
            HeroAnimation::Idle => include_bytes!("../../assets/images/swordman/36.gif"),
            HeroAnimation::Attacking => include_bytes!("../../assets/images/swordman/84.gif"),
            HeroAnimation::Attacked => include_bytes!("../../assets/images/swordman/52.gif"),
        }
    }

    pub fn should_loop(&self) -> bool {
        matches!(self, HeroAnimation::Resting | HeroAnimation::Idle)
    }
}

/// Get map background GIF by map ID
/// Map backgrounds are single-frame GIFs stored in images/map/
///
/// To add a new map:
/// 1. Add map data to maps.json with a unique ID
/// 2. Create a GIF file named with the map ID: images/map/{id}.gif
/// 3. Add a match arm below: id => include_bytes!("images/map/{id}.gif")
pub fn get_map_background(map_id: u32) -> Option<&'static [u8]> {
    match map_id {
        1 => Some(include_bytes!("../../assets/images/map/1.gif")), // Prontera
        2 => Some(include_bytes!("../../assets/images/map/2.gif")), // Prontera South
        3 => Some(include_bytes!("../../assets/images/map/3.gif")), // Prontera West
        5 => Some(include_bytes!("../../assets/images/map/5.gif")), // Prontera East
        _ => {
            esp_println::println!("[WARNING] No background image found for map ID {}", map_id);
            None
        }
    }
}
