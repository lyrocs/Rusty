/// Combat animations
///
/// Manages monster and hero animation states and GIF data.

/// Monster GIF animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterAnimation {
    Idle,      // 6.gif - loops
    Attacking, // 22.gif - plays once
    Attacked,  // 30.gif - plays once (when monster takes damage)
    Dying,     // 38.gif - plays once
}

impl MonsterAnimation {
    /// Get GIF data for a specific monster and animation state
    pub fn gif_data(&self, monster_name: &str) -> &'static [u8] {
        // Convert monster name to lowercase for folder matching
        let monster_lower = monster_name.to_lowercase();

        match (monster_lower.as_str(), self) {
            // Poring animations
            ("poring", MonsterAnimation::Idle) => include_bytes!("../../assets/images/poring/6.gif"),
            ("poring", MonsterAnimation::Attacking) => include_bytes!("../../assets/images/poring/22.gif"),
            ("poring", MonsterAnimation::Attacked) => include_bytes!("../../assets/images/poring/30.gif"),
            ("poring", MonsterAnimation::Dying) => include_bytes!("../../assets/images/poring/38.gif"),

            // Fabre animations
            ("fabre", MonsterAnimation::Idle) => include_bytes!("../../assets/images/fabre/6.gif"),
            ("fabre", MonsterAnimation::Attacking) => include_bytes!("../../assets/images/fabre/22.gif"),
            ("fabre", MonsterAnimation::Attacked) => include_bytes!("../../assets/images/fabre/30.gif"),
            ("fabre", MonsterAnimation::Dying) => include_bytes!("../../assets/images/fabre/38.gif"),

            // Hornet animations
            ("hornet", MonsterAnimation::Idle) => include_bytes!("../../assets/images/hornet/6.gif"),
            ("hornet", MonsterAnimation::Attacking) => include_bytes!("../../assets/images/hornet/22.gif"),
            ("hornet", MonsterAnimation::Attacked) => include_bytes!("../../assets/images/hornet/30.gif"),
            ("hornet", MonsterAnimation::Dying) => include_bytes!("../../assets/images/hornet/38.gif"),

            // Thief Bug animations
            ("thief bug", MonsterAnimation::Idle) => include_bytes!("../../assets/images/thief_bug/6.gif"),
            ("thief bug", MonsterAnimation::Attacking) => include_bytes!("../../assets/images/thief_bug/22.gif"),
            ("thief bug", MonsterAnimation::Attacked) => include_bytes!("../../assets/images/thief_bug/30.gif"),
            ("thief bug", MonsterAnimation::Dying) => include_bytes!("../../assets/images/thief_bug/38.gif"),

            // Default fallback to Poring if monster not found
            _ => {
                esp_println::println!(
                    "[WARNING] No GIF found for monster '{}', using Poring",
                    monster_name
                );
                match self {
                    MonsterAnimation::Idle => include_bytes!("../../assets/images/poring/6.gif"),
                    MonsterAnimation::Attacking => include_bytes!("../../assets/images/poring/22.gif"),
                    MonsterAnimation::Attacked => include_bytes!("../../assets/images/poring/30.gif"),
                    MonsterAnimation::Dying => include_bytes!("../../assets/images/poring/38.gif"),
                }
            }
        }
    }

    pub fn should_loop(&self) -> bool {
        matches!(self, MonsterAnimation::Idle)
    }
}

/// Hero GIF animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroAnimation {
    Resting,   // 16.gif - loops (shown on rest page)
    Idle,      // 32.gif - loops (main loop on battle/farm)
    Attacking, // 80.gif - plays once (hero attacks)
    Attacked,  // 48.gif - plays once (hero takes damage)
}

impl HeroAnimation {
    /// Get GIF data for hero based on job class
    pub fn gif_data(&self, job: &str) -> &'static [u8] {
        // Convert job name to lowercase for folder matching
        let job_lower = job.to_lowercase();

        match (job_lower.as_str(), self) {
            // Novice animations
            ("novice", HeroAnimation::Resting) => include_bytes!("../../assets/images/novice/16.gif"),
            ("novice", HeroAnimation::Idle) => include_bytes!("../../assets/images/novice/32.gif"),
            ("novice", HeroAnimation::Attacking) => include_bytes!("../../assets/images/novice/80.gif"),
            ("novice", HeroAnimation::Attacked) => include_bytes!("../../assets/images/novice/48.gif"),

            // Swordman animations
            ("swordman", HeroAnimation::Resting) => include_bytes!("../../assets/images/swordman/16.gif"),
            ("swordman", HeroAnimation::Idle) => include_bytes!("../../assets/images/swordman/32.gif"),
            ("swordman", HeroAnimation::Attacking) => include_bytes!("../../assets/images/swordman/80.gif"),
            ("swordman", HeroAnimation::Attacked) => include_bytes!("../../assets/images/swordman/48.gif"),

            // Knight animations
            ("knight", HeroAnimation::Resting) => include_bytes!("../../assets/images/knight/16.gif"),
            ("knight", HeroAnimation::Idle) => include_bytes!("../../assets/images/knight/32.gif"),
            ("knight", HeroAnimation::Attacking) => include_bytes!("../../assets/images/knight/80.gif"),
            ("knight", HeroAnimation::Attacked) => include_bytes!("../../assets/images/knight/48.gif"),

            // Default fallback to Swordman if job not found
            _ => {
                esp_println::println!(
                    "[WARNING] No GIF found for job '{}', using Swordman",
                    job
                );
                match self {
                    HeroAnimation::Resting => include_bytes!("../../assets/images/swordman/16.gif"),
                    HeroAnimation::Idle => include_bytes!("../../assets/images/swordman/32.gif"),
                    HeroAnimation::Attacking => include_bytes!("../../assets/images/swordman/80.gif"),
                    HeroAnimation::Attacked => include_bytes!("../../assets/images/swordman/48.gif"),
                }
            }
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
