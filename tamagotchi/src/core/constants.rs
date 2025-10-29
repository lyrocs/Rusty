/// Core game constants

/// Default map ID (Prontera)
/// Note: This constant is also defined in game_data.rs for backward compatibility
/// TODO: Consolidate all map IDs once data extraction is complete
pub const MAP_PRONTERA_ID: u32 = 1;

/// Default farm duration in milliseconds (1 minute)
pub const DEFAULT_FARM_DURATION_MS: u32 = 60000;

/// Default SP regeneration rate (5 SP per second while resting)
pub const DEFAULT_SP_REGEN_RATE: u16 = 5;

/// Default battle duration in milliseconds (30 seconds)
pub const DEFAULT_BATTLE_DURATION_MS: u32 = 30000;

/// Default battle spawn interval in milliseconds (800ms)
pub const DEFAULT_BATTLE_SPAWN_INTERVAL_MS: u32 = 800;

/// Default screen brightness (80% = 204/255)
pub const DEFAULT_BRIGHTNESS: u8 = 204;
