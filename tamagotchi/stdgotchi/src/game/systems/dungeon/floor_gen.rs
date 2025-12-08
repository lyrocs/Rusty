//! Floor Generation
//!
//! Generates enemies for dungeon floors.

use rand::Rng;

/// Generate enemy species for a floor
pub fn generate_floor_enemies(
    floor: u16,
    enemy_pool: &[String],
    enemies_per_floor: u8,
) -> Vec<String> {
    if enemy_pool.is_empty() {
        return vec![];
    }

    let mut rng = rand::thread_rng();
    let mut enemies = Vec::new();

    for _ in 0..enemies_per_floor {
        let index = rng.gen_range(0..enemy_pool.len());
        enemies.push(enemy_pool[index].clone());
    }

    enemies
}

/// Calculate enemy stat scaling based on floor
/// Early floors are easier, scaling up gradually
pub fn floor_stat_multiplier(floor: u16) -> f32 {
    // Early floors (1-10): enemies start weak at 40% stats, scaling to 80%
    // Mid floors (11-20): 80% to 100%
    // Late floors (21+): 100% and increasing by 3% per floor
    match floor {
        0..=5 => 0.4 + (floor as f32 * 0.06),   // 0.4 to 0.7
        6..=10 => 0.7 + ((floor - 5) as f32 * 0.04), // 0.74 to 0.9
        11..=20 => 0.9 + ((floor - 10) as f32 * 0.01), // 0.91 to 1.0
        _ => 1.0 + ((floor - 20) as f32 * 0.03), // 1.03+ (scaling for late game)
    }
}

/// Check if floor is a boss floor (every 10 floors)
pub fn is_boss_floor(floor: u16) -> bool {
    floor > 0 && floor % 10 == 0
}
