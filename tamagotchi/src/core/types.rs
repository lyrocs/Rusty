/// Core game types and enums shared across all domains

/// Type alias for map identifiers
pub type MapId = u32;

/// Game pages that can be displayed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePage {
    Overview,
    Farm,
    Rest,
    Battle,      // Whac-A-Mole mini-game
    JrpgBattle,  // Turn-based JRPG battle
    ZeldaBattle, // Timing-based action battle (Zelda-style)
    Map,         // Navigation and world map
    Menu,
    Inventory,   // Item inventory
    Quests,      // Quest list and management
    Settings,    // Settings page (brightness, etc.)
    Stats,       // Character stats allocation page
    Equipment,   // Equipment management page
    Crafting,    // Blacksmith crafting page
}
