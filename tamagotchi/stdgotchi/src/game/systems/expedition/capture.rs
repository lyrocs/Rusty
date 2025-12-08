//! Monster Capture System
//!
//! Handles capture rolls during expeditions.

use rand::Rng;

/// Roll for capture during expedition
/// Returns true if capture succeeds
pub fn roll_capture(capture_chance: f32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>() < capture_chance
}

/// Select a random species from the capturable list
pub fn select_capture_species<'a>(capturable_species: &'a [String]) -> Option<&'a String> {
    if capturable_species.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..capturable_species.len());
    Some(&capturable_species[index])
}
