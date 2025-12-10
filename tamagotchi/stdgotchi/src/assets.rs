//! Asset Loading System
//!
//! Loads game assets (GIFs, images) from SD card.
//! Path format: images/{monster_name}/{action}.gif

use crate::ecs::resources::SdCardWrapper;
use std::collections::HashMap;

/// Sprite action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpriteAction {
    Idle,      // 6.gif
    Attack,    // 22.gif
    Attacked,  // 30.gif
    Death,     // 38.gif
    Icon,      // icon.gif
}

impl SpriteAction {
    /// Get the filename for this action (GIF format)
    pub fn filename(&self) -> &'static str {
        match self {
            SpriteAction::Idle => "6.gif",
            SpriteAction::Attack => "22.gif",
            SpriteAction::Attacked => "30.gif",
            SpriteAction::Death => "38.gif",
            SpriteAction::Icon => "icon.gif",
        }
    }

    /// Get the filename for this action (RAW format for fast playback)
    pub fn raw_filename(&self) -> &'static str {
        match self {
            SpriteAction::Idle => "6.raw",
            SpriteAction::Attack => "22.raw",
            SpriteAction::Attacked => "30.raw",
            SpriteAction::Death => "38.raw",
            SpriteAction::Icon => "icon.raw",
        }
    }
}

/// Get monster folder name from enemy ID
pub fn enemy_id_to_folder(enemy_id: u32) -> &'static str {
    match enemy_id {
        1002 => "poring",
        1004 => "hornet",
        1007 => "fabre",
        1008 => "lunatic",
        1051 => "thief_bug",
        _ => "poring", // fallback
    }
}

/// Get SD card path for an enemy sprite
/// Format: IMAGES/{monster}/{action}.GIF (8.3 format uppercase)
pub fn get_enemy_sprite_path(enemy_id: u32, action: SpriteAction) -> String {
    let folder = enemy_id_to_folder(enemy_id).to_uppercase();
    let filename = action.filename().to_uppercase();
    format!("IMAGES/{}/{}", folder, filename)
}

/// Get SD card path for a monster sprite by species name
/// Format: IMAGES/{species}/{action}.GIF
pub fn get_monster_sprite_path(species: &str, action: SpriteAction) -> String {
    let folder = species.to_uppercase();
    let filename = action.filename().to_uppercase();
    format!("IMAGES/{}/{}", folder, filename)
}

/// Get SD card path for a monster raw animation by species name
/// Format: IMAGES/{species}/{action}.RAW
pub fn get_monster_raw_path(species: &str, action: SpriteAction) -> String {
    let folder = species.to_uppercase();
    let filename = action.raw_filename().to_uppercase();
    format!("IMAGES/{}/{}", folder, filename)
}

/// Get SD card path for a hero sprite
/// Format: IMAGES/{class}/{size}.GIF
pub fn get_hero_sprite_path(class: &str, size: u8) -> String {
    let folder = class.to_uppercase();
    format!("IMAGES/{}/{}.GIF", folder, size)
}

/// Get SD card path for a map background
/// Format: IMAGES/MAP/{id}.GIF
pub fn get_map_sprite_path(map_id: u32) -> String {
    format!("IMAGES/MAP/{}.GIF", map_id)
}

/// Get SD card path for UI element
/// Format: IMAGES/UI/{name}.GIF
pub fn get_ui_sprite_path(name: &str) -> String {
    format!("IMAGES/UI/{}.GIF", name.to_uppercase())
}

/// Sprite cache for loaded sprites
pub struct SpriteCache {
    cache: HashMap<String, Vec<u8>>,
}

impl SpriteCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Load a sprite from SD card, caching it for future use
    pub fn load(&mut self, sd_card: &mut SdCardWrapper, path: &str) -> Option<Vec<u8>> {
        // Check cache first
        if let Some(data) = self.cache.get(path) {
            return Some(data.clone());
        }

        // Load from SD card
        match sd_card.load_binary_file(path) {
            Ok(data) => {
                log::info!("Loaded sprite: {} ({} bytes)", path, data.len());
                self.cache.insert(path.to_string(), data.clone());
                Some(data)
            }
            Err(e) => {
                log::warn!("Failed to load sprite {}: {:?}", path, e);
                None
            }
        }
    }

    /// Load without caching (for large sprites)
    pub fn load_uncached(sd_card: &mut SdCardWrapper, path: &str) -> Option<Vec<u8>> {
        match sd_card.load_binary_file(path) {
            Ok(data) => {
                log::info!("Loaded sprite: {} ({} bytes)", path, data.len());
                Some(data)
            }
            Err(e) => {
                log::warn!("Failed to load sprite {}: {:?}", path, e);
                None
            }
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for SpriteCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Load enemy sprites from SD card
/// Returns (idle, attack, attacked, death) as Option<Vec<u8>>
pub fn load_enemy_sprites(
    sd_card: &mut SdCardWrapper,
    enemy_id: u32,
) -> (Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>) {
    let idle_path = get_enemy_sprite_path(enemy_id, SpriteAction::Idle);
    let attack_path = get_enemy_sprite_path(enemy_id, SpriteAction::Attack);
    let attacked_path = get_enemy_sprite_path(enemy_id, SpriteAction::Attacked);
    let death_path = get_enemy_sprite_path(enemy_id, SpriteAction::Death);

    let idle = SpriteCache::load_uncached(sd_card, &idle_path);
    let attack = SpriteCache::load_uncached(sd_card, &attack_path);
    let attacked = SpriteCache::load_uncached(sd_card, &attacked_path);
    let death = SpriteCache::load_uncached(sd_card, &death_path);

    (idle, attack, attacked, death)
}

/// Load monster sprites by species name
pub fn load_monster_sprites(
    sd_card: &mut SdCardWrapper,
    species: &str,
) -> (Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>) {
    let idle_path = get_monster_sprite_path(species, SpriteAction::Idle);
    let attack_path = get_monster_sprite_path(species, SpriteAction::Attack);
    let attacked_path = get_monster_sprite_path(species, SpriteAction::Attacked);
    let death_path = get_monster_sprite_path(species, SpriteAction::Death);

    let idle = SpriteCache::load_uncached(sd_card, &idle_path);
    let attack = SpriteCache::load_uncached(sd_card, &attack_path);
    let attacked = SpriteCache::load_uncached(sd_card, &attacked_path);
    let death = SpriteCache::load_uncached(sd_card, &death_path);

    (idle, attack, attacked, death)
}

/// Load monster icon
pub fn load_monster_icon(sd_card: &mut SdCardWrapper, species: &str) -> Option<Vec<u8>> {
    let path = get_monster_sprite_path(species, SpriteAction::Icon);
    SpriteCache::load_uncached(sd_card, &path)
}

/// Load hero sprite
pub fn load_hero_sprite(sd_card: &mut SdCardWrapper, class: &str, size: u8) -> Option<Vec<u8>> {
    let path = get_hero_sprite_path(class, size);
    SpriteCache::load_uncached(sd_card, &path)
}

/// Load map background
pub fn load_map_background(sd_card: &mut SdCardWrapper, map_id: u32) -> Option<Vec<u8>> {
    let path = get_map_sprite_path(map_id);
    SpriteCache::load_uncached(sd_card, &path)
}

/// Load UI element
pub fn load_ui_sprite(sd_card: &mut SdCardWrapper, name: &str) -> Option<Vec<u8>> {
    let path = get_ui_sprite_path(name);
    SpriteCache::load_uncached(sd_card, &path)
}

/// Get monster icon (stub - returns None, use load_monster_icon with SD card instead)
/// This is a compatibility function for pages that don't have SD card access
pub fn get_monster_icon(_species: &str) -> Option<&'static [u8]> {
    // No embedded icons - pages should use fallback element display
    None
}

/// Load a streaming raw animation (loads only metadata + header, not frame data)
/// This is memory-efficient for large animations - frames are loaded on-demand
pub fn load_streaming_raw_animation(
    sd_card: &mut SdCardWrapper,
    species: &str,
    action: SpriteAction,
) -> Option<crate::display::RawAnimPlayer> {
    use crate::display::RawAnimMeta;

    let path = get_monster_raw_path(species, action);

    // Load only the header to get metadata
    // Header size: 8 bytes + (frame_count * 6) bytes
    // We'll load 1KB which should cover headers for most animations
    let header_data = sd_card.load_binary_range(&path, 0, 1024).ok()?;

    // Parse metadata from header
    let meta = RawAnimMeta::from_header(&header_data)?;

    log::info!(
        "Loaded streaming animation metadata: {} ({}x{}, {} frames)",
        path,
        meta.width,
        meta.height,
        meta.frame_count
    );

    // Create streaming player (does not load frame data yet)
    Some(crate::display::RawAnimPlayer::from_metadata(meta, path))
}
