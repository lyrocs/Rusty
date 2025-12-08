//! Aura System
//!
//! Manages elemental auras applied to targets during combat.

use crate::game::core::Element;
use crate::game::calculations::combat::{AURA_DURATION_AUTO, AURA_DURATION_SKILL};

/// Aura state on a target
#[derive(Debug, Clone)]
pub struct Aura {
    pub element: Element,
    pub remaining_time: f32,
}

impl Aura {
    /// Create a new aura from an auto-attack
    pub fn from_auto_attack(element: Element) -> Self {
        Self {
            element,
            remaining_time: AURA_DURATION_AUTO,
        }
    }

    /// Create a new aura from a skill
    pub fn from_skill(element: Element) -> Self {
        Self {
            element,
            remaining_time: AURA_DURATION_SKILL,
        }
    }

    /// Update aura (decrease time), returns true if still active
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.remaining_time -= delta_time;
        self.remaining_time > 0.0
    }

    /// Check if aura is still active
    pub fn is_active(&self) -> bool {
        self.remaining_time > 0.0
    }
}

/// Apply or refresh an aura on a target
pub fn apply_aura(
    current_aura: &mut Option<(Element, f32)>,
    new_element: Element,
    duration: f32,
) {
    *current_aura = Some((new_element, duration));
}

/// Update aura timer, returns remaining element or None if expired
pub fn update_aura(
    current_aura: &mut Option<(Element, f32)>,
    delta_time: f32,
) -> Option<Element> {
    if let Some((element, ref mut time)) = current_aura {
        *time -= delta_time;
        if *time <= 0.0 {
            *current_aura = None;
            None
        } else {
            Some(*element)
        }
    } else {
        None
    }
}
