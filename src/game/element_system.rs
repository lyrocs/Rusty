//! Element System (Legacy)
//!
//! Handles element advantages/disadvantages and UI display.
//! NOTE: This is the legacy element system. New Monster Tamer code should use game::core::Element.

use embedded_graphics::pixelcolor::Rgb888;
use serde::{Deserialize, Serialize};

/// Legacy Element type for existing battle system
/// The new Monster Tamer system uses game::core::Element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    #[default]
    Neutral,
    Water,
    Earth,
    Fire,
    Wind,
    Poison,
    Holy,
    Shadow,
    Ghost,
    Undead,
}

impl Element {
    /// Parse element from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "neutral" | "" => Some(Element::Neutral),
            "water" => Some(Element::Water),
            "earth" => Some(Element::Earth),
            "fire" => Some(Element::Fire),
            "wind" => Some(Element::Wind),
            "poison" => Some(Element::Poison),
            "holy" => Some(Element::Holy),
            "shadow" => Some(Element::Shadow),
            "ghost" => Some(Element::Ghost),
            "undead" => Some(Element::Undead),
            _ => None,
        }
    }

    /// Get element name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Element::Neutral => "Neutral",
            Element::Water => "Water",
            Element::Earth => "Earth",
            Element::Fire => "Fire",
            Element::Wind => "Wind",
            Element::Poison => "Poison",
            Element::Holy => "Holy",
            Element::Shadow => "Shadow",
            Element::Ghost => "Ghost",
            Element::Undead => "Undead",
        }
    }
}

/// Get damage multiplier based on element matchup
///
/// Returns:
/// - 1.5 for super effective (advantage)
/// - 0.5 for not very effective (disadvantage)
/// - 1.0 for neutral
/// - Special cases for certain matchups
pub fn get_element_advantage(attacker: Element, defender: Element) -> f32 {
    use Element::*;

    match (attacker, defender) {
        // Elemental cycle: Fire > Wind > Earth > Water > Fire
        (Fire, Wind) => 1.5,
        (Wind, Earth) => 1.5,
        (Earth, Water) => 1.5,
        (Water, Fire) => 1.5,

        // Reverse (disadvantages)
        (Wind, Fire) => 0.5,
        (Earth, Wind) => 0.5,
        (Water, Earth) => 0.5,
        (Fire, Water) => 0.5,

        // Holy vs Shadow (mutual advantage)
        (Holy, Shadow) => 1.5,
        (Shadow, Holy) => 1.5,

        // Ghost advantages
        (Ghost, Neutral) => 1.5,
        (Neutral, Ghost) => 0.5,

        // Poison advantages
        (Poison, Holy) => 1.5,
        (Holy, Poison) => 0.5,

        // Undead immunity to poison (heavy resistance)
        (Poison, Undead) => 0.1,
        (Undead, Poison) => 1.2,

        // Shadow and Ghost resistance
        (Shadow, Shadow) => 0.75,
        (Ghost, Ghost) => 0.75,

        // Undead vs Holy (extra effective)
        (Holy, Undead) => 2.0,
        (Undead, Holy) => 0.5,

        // Neutral matchups (no advantage)
        _ => 1.0,
    }
}

/// Get a descriptive text for the element advantage
pub fn get_advantage_text(multiplier: f32) -> &'static str {
    if multiplier >= 2.0 {
        "SUPER EFFECTIVE!"
    } else if multiplier >= 1.5 {
        "Effective"
    } else if multiplier > 1.0 {
        "Slightly Effective"
    } else if multiplier == 1.0 {
        ""
    } else if multiplier >= 0.5 {
        "Not Very Effective"
    } else {
        "Resisted!"
    }
}

/// Get element color for UI display
pub fn get_element_color(element: Element) -> Rgb888 {
    use Element::*;

    match element {
        Neutral => Rgb888::new(200, 200, 200), // Gray
        Water => Rgb888::new(100, 150, 255),   // Blue
        Earth => Rgb888::new(139, 90, 43),     // Brown
        Fire => Rgb888::new(255, 100, 100),    // Red
        Wind => Rgb888::new(150, 255, 150),    // Light Green
        Poison => Rgb888::new(150, 50, 200),   // Purple
        Holy => Rgb888::new(255, 255, 150),    // Light Yellow
        Shadow => Rgb888::new(100, 50, 150),   // Dark Purple
        Ghost => Rgb888::new(200, 150, 255),   // Lavender
        Undead => Rgb888::new(100, 100, 100),  // Dark Gray
    }
}

/// Get element icon text (emoji or letter)
pub fn get_element_icon(element: Element) -> &'static str {
    use Element::*;

    match element {
        Neutral => "○",
        Water => "≈",
        Earth => "▲",
        Fire => "※",
        Wind => "~",
        Poison => "☠",
        Holy => "☼",
        Shadow => "◆",
        Ghost => "♦",
        Undead => "†",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_cycle() {
        // Fire > Wind
        assert_eq!(get_element_advantage(Element::Fire, Element::Wind), 1.5);
        assert_eq!(get_element_advantage(Element::Wind, Element::Fire), 0.5);

        // Wind > Earth
        assert_eq!(get_element_advantage(Element::Wind, Element::Earth), 1.5);
        assert_eq!(get_element_advantage(Element::Earth, Element::Wind), 0.5);

        // Earth > Water
        assert_eq!(get_element_advantage(Element::Earth, Element::Water), 1.5);
        assert_eq!(get_element_advantage(Element::Water, Element::Earth), 0.5);

        // Water > Fire
        assert_eq!(get_element_advantage(Element::Water, Element::Fire), 1.5);
        assert_eq!(get_element_advantage(Element::Fire, Element::Water), 0.5);
    }

    #[test]
    fn test_holy_vs_shadow() {
        assert_eq!(get_element_advantage(Element::Holy, Element::Shadow), 1.5);
        assert_eq!(get_element_advantage(Element::Shadow, Element::Holy), 1.5);
    }

    #[test]
    fn test_neutral_matchups() {
        assert_eq!(get_element_advantage(Element::Neutral, Element::Neutral), 1.0);
        assert_eq!(get_element_advantage(Element::Fire, Element::Fire), 1.0);
        assert_eq!(get_element_advantage(Element::Water, Element::Poison), 1.0);
    }

    #[test]
    fn test_undead_vs_poison() {
        assert_eq!(get_element_advantage(Element::Poison, Element::Undead), 0.1);
        assert_eq!(get_element_advantage(Element::Undead, Element::Poison), 1.2);
    }

    #[test]
    fn test_holy_vs_undead() {
        assert_eq!(get_element_advantage(Element::Holy, Element::Undead), 2.0);
        assert_eq!(get_element_advantage(Element::Undead, Element::Holy), 0.5);
    }

    #[test]
    fn test_advantage_text() {
        assert_eq!(get_advantage_text(2.0), "SUPER EFFECTIVE!");
        assert_eq!(get_advantage_text(1.5), "Effective");
        assert_eq!(get_advantage_text(1.0), "");
        assert_eq!(get_advantage_text(0.5), "Not Very Effective");
        assert_eq!(get_advantage_text(0.1), "Resisted!");
    }

    #[test]
    fn test_element_colors() {
        // Just ensure all elements have colors defined
        for element in [
            Element::Neutral,
            Element::Water,
            Element::Earth,
            Element::Fire,
            Element::Wind,
            Element::Poison,
            Element::Holy,
            Element::Shadow,
            Element::Ghost,
            Element::Undead,
        ] {
            let color = get_element_color(element);
            assert!(color.r() > 0 || color.g() > 0 || color.b() > 0);
        }
    }
}
