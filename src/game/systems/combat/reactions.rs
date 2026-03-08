//! Elemental Reactions
//!
//! Handles elemental reaction triggers and effects.

use crate::game::core::Element;

/// Types of elemental reactions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReactionType {
    /// VAPORIZE: 2x damage (Water+Fire or Fire+Water)
    Vaporize,
    /// ELECTROCUTE: Damage + 1s stun (Water+Thunder)
    Electrocute,
    /// BLOOM: Heal team 15% (Water+Earth)
    Bloom,
    /// MELT: 1.5x damage (Fire+Earth)
    Melt,
    /// BURNING: DoT for 5 seconds (Earth+Fire)
    Burning,
    /// SUPERCONDUCT: DEF -30% for 5s (Thunder+Water)
    Superconduct,
    /// SWIRL: Propagate aura to all enemies (Any+Wind)
    Swirl,
    /// PURIFY: 2x damage (Shadow+Holy)
    Purify,
    /// CORRUPT: 2x damage (Holy+Shadow)
    Corrupt,
}

/// Reaction result
#[derive(Debug, Clone)]
pub struct ReactionResult {
    pub reaction_type: ReactionType,
    pub name: &'static str,
    pub damage_multiplier: f32,
    pub stun_duration: f32,
    pub heal_percent: f32,
    pub def_reduction: f32,
    pub def_reduction_duration: f32,
    pub dot_duration: f32,
}

/// Check if a reaction occurs and return its effect
pub fn check_reaction(aura_element: Element, attack_element: Element) -> Option<ReactionResult> {
    use Element::*;
    use ReactionType::*;

    match (aura_element, attack_element) {
        // VAPORIZE: Water aura + Fire attack or Fire aura + Water attack
        (Water, Fire) | (Fire, Water) => Some(ReactionResult {
            reaction_type: Vaporize,
            name: "VAPORIZE",
            damage_multiplier: 2.0,
            stun_duration: 0.0,
            heal_percent: 0.0,
            def_reduction: 0.0,
            def_reduction_duration: 0.0,
            dot_duration: 0.0,
        }),

        // ELECTROCUTE: Water aura + Thunder attack
        (Water, Thunder) => Some(ReactionResult {
            reaction_type: Electrocute,
            name: "ELECTROCUTE",
            damage_multiplier: 1.0,
            stun_duration: 1.0,
            heal_percent: 0.0,
            def_reduction: 0.0,
            def_reduction_duration: 0.0,
            dot_duration: 0.0,
        }),

        // BLOOM: Water aura + Earth attack
        (Water, Earth) => Some(ReactionResult {
            reaction_type: Bloom,
            name: "BLOOM",
            damage_multiplier: 1.0,
            stun_duration: 0.0,
            heal_percent: 0.15,
            def_reduction: 0.0,
            def_reduction_duration: 0.0,
            dot_duration: 0.0,
        }),

        // MELT: Fire aura + Earth attack
        (Fire, Earth) => Some(ReactionResult {
            reaction_type: Melt,
            name: "MELT",
            damage_multiplier: 1.5,
            stun_duration: 0.0,
            heal_percent: 0.0,
            def_reduction: 0.0,
            def_reduction_duration: 0.0,
            dot_duration: 0.0,
        }),

        // BURNING: Earth aura + Fire attack
        (Earth, Fire) => Some(ReactionResult {
            reaction_type: Burning,
            name: "BURNING",
            damage_multiplier: 1.0,
            stun_duration: 0.0,
            heal_percent: 0.0,
            def_reduction: 0.0,
            def_reduction_duration: 0.0,
            dot_duration: 5.0,
        }),

        // SUPERCONDUCT: Thunder aura + Water attack
        (Thunder, Water) => Some(ReactionResult {
            reaction_type: Superconduct,
            name: "SUPERCONDUCT",
            damage_multiplier: 1.0,
            stun_duration: 0.0,
            heal_percent: 0.0,
            def_reduction: 0.30,
            def_reduction_duration: 5.0,
            dot_duration: 0.0,
        }),

        // SWIRL: Any aura + Wind attack (propagates aura)
        (_, Wind) if aura_element != Wind => Some(ReactionResult {
            reaction_type: Swirl,
            name: "SWIRL",
            damage_multiplier: 1.0,
            stun_duration: 0.0,
            heal_percent: 0.0,
            def_reduction: 0.0,
            def_reduction_duration: 0.0,
            dot_duration: 0.0,
        }),

        // PURIFY: Shadow aura + Holy attack
        (Shadow, Holy) => Some(ReactionResult {
            reaction_type: Purify,
            name: "PURIFY",
            damage_multiplier: 2.0,
            stun_duration: 0.0,
            heal_percent: 0.0,
            def_reduction: 0.0,
            def_reduction_duration: 0.0,
            dot_duration: 0.0,
        }),

        // CORRUPT: Holy aura + Shadow attack
        (Holy, Shadow) => Some(ReactionResult {
            reaction_type: Corrupt,
            name: "CORRUPT",
            damage_multiplier: 2.0,
            stun_duration: 0.0,
            heal_percent: 0.0,
            def_reduction: 0.0,
            def_reduction_duration: 0.0,
            dot_duration: 0.0,
        }),

        // No reaction
        _ => None,
    }
}
