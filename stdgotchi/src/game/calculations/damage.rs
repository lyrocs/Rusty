//! Damage Calculations
//!
//! All damage-related calculations: base damage, element modifiers, critical hits.
//! This is the SINGLE SOURCE OF TRUTH for damage formulas.

use crate::game::core::Element;

/// Minimum damage multiplier (ensures attacks always deal some damage)
pub const MIN_DAMAGE_MULTIPLIER: f32 = 0.1;

/// DEF reduction factor
pub const DEF_FACTOR: f32 = 0.5;

/// Element advantage multiplier
pub const ELEMENT_ADVANTAGE: f32 = 1.5;

/// Element disadvantage multiplier
pub const ELEMENT_DISADVANTAGE: f32 = 0.5;

/// Critical hit damage multiplier (1.5x damage)
pub const CRITICAL_MULTIPLIER: f32 = 1.5;

/// Result of a damage calculation including crit info
#[derive(Debug, Clone, Copy)]
pub struct DamageResult {
    pub damage: u16,
    pub is_critical: bool,
    pub is_miss: bool,
}

impl DamageResult {
    pub fn miss() -> Self {
        Self { damage: 0, is_critical: false, is_miss: true }
    }

    pub fn hit(damage: u16, is_critical: bool) -> Self {
        Self { damage, is_critical, is_miss: false }
    }
}

/// Roll for accuracy (returns true if hit, false if miss)
pub fn roll_accuracy(accuracy: u8) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let roll = ((seed * 1103515245 + 12345) % 100) as u8;
    roll < accuracy
}

/// Roll for critical hit (returns true if crit)
pub fn roll_critical(crit_chance: u8) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| (d.as_nanos() / 7) as u64)  // Different seed offset for variety
        .unwrap_or(0);
    let roll = ((seed * 1103515245 + 12345) % 100) as u8;
    roll < crit_chance
}

/// Calculate base damage
/// Formula: base = max(ATK - DEF*0.5, ATK*0.1)
pub fn calculate_base_damage(atk: u16, def: u16) -> f32 {
    let raw_damage = atk as f32 - (def as f32 * DEF_FACTOR);
    let min_damage = atk as f32 * MIN_DAMAGE_MULTIPLIER;
    raw_damage.max(min_damage)
}

/// Get element multiplier based on attacker vs defender element
/// Returns multiplier: 1.5 (advantage), 1.0 (neutral), 0.5 (disadvantage)
pub fn get_element_multiplier(attacker: Element, defender: Element) -> f32 {
    // Based on GDD section 2.1.2
    match (attacker, defender) {
        // Fire is strong against Earth and Wind
        (Element::Fire, Element::Earth) => ELEMENT_ADVANTAGE,
        (Element::Fire, Element::Wind) => ELEMENT_ADVANTAGE,
        (Element::Fire, Element::Water) => ELEMENT_DISADVANTAGE,

        // Water is strong against Fire
        (Element::Water, Element::Fire) => ELEMENT_ADVANTAGE,
        (Element::Water, Element::Earth) => ELEMENT_DISADVANTAGE,
        (Element::Water, Element::Wind) => ELEMENT_DISADVANTAGE,

        // Earth is strong against Water
        (Element::Earth, Element::Water) => ELEMENT_ADVANTAGE,
        (Element::Earth, Element::Fire) => ELEMENT_DISADVANTAGE,

        // Wind is strong against Earth
        (Element::Wind, Element::Earth) => ELEMENT_ADVANTAGE,
        (Element::Wind, Element::Fire) => ELEMENT_DISADVANTAGE,

        // Thunder is strong against Water
        (Element::Thunder, Element::Water) => ELEMENT_ADVANTAGE,
        (Element::Thunder, Element::Earth) => ELEMENT_DISADVANTAGE,

        // Shadow is strong against Holy and Ghost
        (Element::Shadow, Element::Holy) => ELEMENT_ADVANTAGE,
        (Element::Shadow, Element::Ghost) => ELEMENT_ADVANTAGE,

        // Holy is strong against Shadow
        (Element::Holy, Element::Shadow) => ELEMENT_ADVANTAGE,
        (Element::Holy, Element::Ghost) => ELEMENT_DISADVANTAGE,

        // Ghost is special - neutral vs most, weak to Holy
        (Element::Ghost, Element::Holy) => ELEMENT_DISADVANTAGE,

        // Default: neutral
        _ => 1.0,
    }
}

/// Calculate final damage with all modifiers
/// Formula: final = base * element_mult * reaction_mult
pub fn calculate_final_damage(
    atk: u16,
    def: u16,
    attacker_element: Element,
    defender_element: Element,
    reaction_multiplier: f32,
) -> u16 {
    let base = calculate_base_damage(atk, def);
    let element_mult = get_element_multiplier(attacker_element, defender_element);
    let final_damage = base * element_mult * reaction_multiplier;
    final_damage.round().max(1.0) as u16 // Always deal at least 1 damage
}

/// Calculate skill damage (skills typically have a multiplier)
pub fn calculate_skill_damage(
    atk: u16,
    def: u16,
    attacker_element: Element,
    defender_element: Element,
    skill_multiplier: f32,
    reaction_multiplier: f32,
) -> u16 {
    let base = calculate_base_damage(atk, def);
    let element_mult = get_element_multiplier(attacker_element, defender_element);
    let final_damage = base * element_mult * skill_multiplier * reaction_multiplier;
    final_damage.round().max(1.0) as u16
}

/// Calculate damage ignoring a percentage of DEF (for Soul Strike-like skills)
pub fn calculate_damage_ignore_def(
    atk: u16,
    def: u16,
    def_ignore_percent: f32, // 0.0 to 1.0
    attacker_element: Element,
    defender_element: Element,
) -> u16 {
    let effective_def = (def as f32 * (1.0 - def_ignore_percent)).round() as u16;
    calculate_final_damage(atk, effective_def, attacker_element, defender_element, 1.0)
}

/// Calculate full skill damage with accuracy check, crit roll, and all modifiers
/// Returns DamageResult with damage amount and flags for miss/crit
pub fn calculate_skill_damage_full(
    atk: u16,
    def: u16,
    skill_power: u16,       // Skill power (100 = base ATK)
    accuracy: u8,           // 0-100 hit chance
    crit_chance: u8,        // 0-100 crit chance
    skill_element: Element,
    defender_element: Element,
    reaction_multiplier: f32,
) -> DamageResult {
    // First check accuracy
    if !roll_accuracy(accuracy) {
        log::info!("Attack missed! (acc: {}%)", accuracy);
        return DamageResult::miss();
    }

    // Calculate base damage with skill power
    let effective_atk = (atk as f32 * skill_power as f32 / 100.0) as u16;
    let base = calculate_base_damage(effective_atk, def);
    let element_mult = get_element_multiplier(skill_element, defender_element);

    // Roll for critical hit
    let is_critical = roll_critical(crit_chance);
    let crit_mult = if is_critical { CRITICAL_MULTIPLIER } else { 1.0 };

    // Calculate final damage
    let final_damage = base * element_mult * reaction_multiplier * crit_mult;
    let damage = final_damage.round().max(1.0) as u16;

    if is_critical {
        log::info!("CRITICAL HIT! Damage: {} (crit: {}%)", damage, crit_chance);
    }

    DamageResult::hit(damage, is_critical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_damage() {
        // Normal case: ATK 50, DEF 20
        // base = 50 - 20*0.5 = 50 - 10 = 40
        assert_eq!(calculate_base_damage(50, 20), 40.0);

        // Min damage case: ATK 10, DEF 100
        // raw = 10 - 100*0.5 = 10 - 50 = -40 → capped to 10*0.1 = 1
        assert_eq!(calculate_base_damage(10, 100), 1.0);
    }

    #[test]
    fn test_element_multiplier() {
        assert_eq!(get_element_multiplier(Element::Fire, Element::Earth), 1.5);
        assert_eq!(get_element_multiplier(Element::Fire, Element::Water), 0.5);
        assert_eq!(get_element_multiplier(Element::Fire, Element::Fire), 1.0);
    }

    #[test]
    fn test_final_damage() {
        // ATK 50, DEF 20, Fire vs Earth (1.5x), no reaction (1.0x)
        // base = 40, final = 40 * 1.5 * 1.0 = 60
        let damage = calculate_final_damage(50, 20, Element::Fire, Element::Earth, 1.0);
        assert_eq!(damage, 60);

        // With VAPORIZE reaction (2.0x)
        let damage = calculate_final_damage(50, 20, Element::Fire, Element::Water, 2.0);
        // base = 40, element = 0.5, reaction = 2.0
        // final = 40 * 0.5 * 2.0 = 40
        assert_eq!(damage, 40);
    }
}
