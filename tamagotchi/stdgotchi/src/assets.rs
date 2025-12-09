//! Asset Loading System
//!
//! Provides unified interface for loading game assets (GIFs, images, etc.)
//! with support for SD card storage and fallback to embedded assets.

use crate::sdcard::SdCardOps;
use std::error::Error;

/// Asset type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    /// Hero sprites (novice, knight, swordman)
    Hero,
    /// Enemy sprites (poring, fabre, hornet, etc.)
    Enemy,
    /// Map backgrounds
    Map,
    /// UI elements (menu, battle background, etc.)
    UI,
}

/// Sprite action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteAction {
    Idle,      // 6.gif
    Attack,    // 22.gif
    Attacked,  // 30.gif
    Death,     // 38.gif
}

impl SpriteAction {
    /// Get the filename number for this action
    pub fn filename_number(&self) -> u8 {
        match self {
            SpriteAction::Idle => 6,
            SpriteAction::Attack => 22,
            SpriteAction::Attacked => 30,
            SpriteAction::Death => 38,
        }
    }
}

/// Sprite size variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSize {
    Tiny = 16,
    Small = 32,
    Medium = 48,
    Large = 80,
}

/// Asset identifier for organized loading
#[derive(Debug)]
pub enum AssetId {
    /// Hero sprite: (class_name, size, action)
    HeroSprite(&'static str, SpriteSize, u8),
    /// Enemy sprite: (enemy_id, action)
    EnemySprite(u32, SpriteAction),
    /// Map background by map ID
    MapBackground(u32),
    /// UI element
    UIElement(&'static str),
    /// Monster icon by species_id
    MonsterIcon(&'static str),
}

impl AssetId {
    /// Get the SD card path for this asset (8.3 format)
    pub fn sd_path(&self) -> String {
        match self {
            AssetId::HeroSprite(class, _size, action) => {
                format!("/SPRITES/HERO/{}{}.GIF", class.to_uppercase(), action)
            }
            AssetId::EnemySprite(enemy_id, action) => {
                // Map enemy IDs to names for file paths
                let enemy_name = match enemy_id {
                    1002 => "PORING",
                    1004 => "HORNET",
                    1007 => "FABRE",
                    1051 => "THIEF_BUG",
                    _ => "UNKNOWN",
                };
                format!(
                    "/SPRITES/ENEMY/{}{}.GIF",
                    enemy_name,
                    action.filename_number()
                )
            }
            AssetId::MapBackground(map_id) => {
                format!("/SPRITES/MAP/{}.GIF", map_id)
            }
            AssetId::UIElement(name) => {
                format!("/SPRITES/UI/{}.GIF", name.to_uppercase())
            }
            AssetId::MonsterIcon(species_id) => {
                format!("/SPRITES/ICON/{}.GIF", species_id.to_uppercase())
            }
        }
    }

    /// Get embedded asset bytes (fallback)
    pub fn embedded_bytes(&self) -> Option<&'static [u8]> {
        match self {
            AssetId::HeroSprite("novice", _size, 16) => {
                Some(include_bytes!("../assets/images/novice/16.gif"))
            }
            AssetId::HeroSprite("novice", _size, 32) => {
                Some(include_bytes!("../assets/images/novice/32.gif"))
            }
            AssetId::HeroSprite("novice", _size, 48) => {
                Some(include_bytes!("../assets/images/novice/48.gif"))
            }
            AssetId::HeroSprite("novice", _size, 80) => {
                Some(include_bytes!("../assets/images/novice/80.gif"))
            }
            // Poring (1002)
            AssetId::EnemySprite(1002, SpriteAction::Idle) => {
                Some(include_bytes!("../assets/images/poring/6.gif"))
            }
            AssetId::EnemySprite(1002, SpriteAction::Attack) => {
                Some(include_bytes!("../assets/images/poring/22.gif"))
            }
            AssetId::EnemySprite(1002, SpriteAction::Attacked) => {
                Some(include_bytes!("../assets/images/poring/30.gif"))
            }
            AssetId::EnemySprite(1002, SpriteAction::Death) => {
                Some(include_bytes!("../assets/images/poring/38.gif"))
            }
            // Hornet (1004)
            AssetId::EnemySprite(1004, SpriteAction::Idle) => {
                Some(include_bytes!("../assets/images/hornet/6.gif"))
            }
            AssetId::EnemySprite(1004, SpriteAction::Attack) => {
                Some(include_bytes!("../assets/images/hornet/22.gif"))
            }
            AssetId::EnemySprite(1004, SpriteAction::Attacked) => {
                Some(include_bytes!("../assets/images/hornet/30.gif"))
            }
            AssetId::EnemySprite(1004, SpriteAction::Death) => {
                Some(include_bytes!("../assets/images/hornet/38.gif"))
            }
            // Fabre (1007)
            AssetId::EnemySprite(1007, SpriteAction::Idle) => {
                Some(include_bytes!("../assets/images/fabre/6.gif"))
            }
            AssetId::EnemySprite(1007, SpriteAction::Attack) => {
                Some(include_bytes!("../assets/images/fabre/22.gif"))
            }
            AssetId::EnemySprite(1007, SpriteAction::Attacked) => {
                Some(include_bytes!("../assets/images/fabre/30.gif"))
            }
            AssetId::EnemySprite(1007, SpriteAction::Death) => {
                Some(include_bytes!("../assets/images/fabre/38.gif"))
            }
            // Thief Bug (1051)
            AssetId::EnemySprite(1051, SpriteAction::Idle) => {
                Some(include_bytes!("../assets/images/thief_bug/6.gif"))
            }
            AssetId::EnemySprite(1051, SpriteAction::Attack) => {
                Some(include_bytes!("../assets/images/thief_bug/22.gif"))
            }
            AssetId::EnemySprite(1051, SpriteAction::Attacked) => {
                Some(include_bytes!("../assets/images/thief_bug/30.gif"))
            }
            AssetId::EnemySprite(1051, SpriteAction::Death) => {
                Some(include_bytes!("../assets/images/thief_bug/38.gif"))
            }
            // Map backgrounds
            AssetId::MapBackground(1) => {
                Some(include_bytes!("../assets/images/map/1.gif"))
            }
            AssetId::MapBackground(2) => {
                Some(include_bytes!("../assets/images/map/2.gif"))
            }
            AssetId::MapBackground(3) => {
                Some(include_bytes!("../assets/images/map/3.gif"))
            }
            AssetId::MapBackground(5) => {
                Some(include_bytes!("../assets/images/map/5.gif"))
            }
            AssetId::UIElement("battle") => {
                Some(include_bytes!("../assets/images/ui/battle.gif"))
            }
            AssetId::UIElement("menu") => {
                Some(include_bytes!("../assets/images/ui/menu.gif"))
            }
            AssetId::UIElement("background") => {
                Some(include_bytes!("../assets/images/ui/background.gif"))
            }
            // Monster icons
            AssetId::MonsterIcon("poring") => {
                Some(include_bytes!("../assets/images/poring/icon.gif"))
            }
            AssetId::MonsterIcon("lunatic") => {
                Some(include_bytes!("../assets/images/lunatic/icon.gif"))
            }
            AssetId::MonsterIcon("familiar") => {
                Some(include_bytes!("../assets/images/familiar/icon.gif"))
            }
            // Add more mappings as needed
            _ => None,
        }
    }
}

/// Asset source - where the asset was loaded from
#[derive(Debug)]
pub enum AssetSource {
    /// Loaded from SD card
    SdCard(Vec<u8>),
    /// Loaded from embedded bytes
    Embedded(&'static [u8]),
}

impl AssetSource {
    /// Get the asset bytes
    pub fn bytes(&self) -> &[u8] {
        match self {
            AssetSource::SdCard(data) => data.as_slice(),
            AssetSource::Embedded(data) => data,
        }
    }
}

/// Asset loader with SD card support and embedded fallback
#[derive(Clone)]
pub struct AssetLoader<SD>
where
    SD: SdCardOps + Clone,
{
    sd_card: Option<SD>,
    prefer_sd: bool,
}

impl<SD> AssetLoader<SD>
where
    SD: SdCardOps + Clone,
{
    /// Create a new asset loader
    ///
    /// # Arguments
    /// * `sd_card` - Optional SD card resource
    /// * `prefer_sd` - If true, always try SD card first before fallback
    pub fn new(sd_card: Option<SD>, prefer_sd: bool) -> Self {
        Self { sd_card, prefer_sd }
    }

    /// Load an asset by ID
    ///
    /// Tries to load from SD card first (if available and prefer_sd is true),
    /// then falls back to embedded assets.
    pub fn load(&mut self, asset_id: &AssetId) -> Result<AssetSource, Box<dyn Error>> {
        // Try SD card first if available and preferred
        if self.prefer_sd {
            if let Some(ref mut sd) = self.sd_card {
                if sd.is_mounted() {
                    let sd_path = asset_id.sd_path();
                    log::info!("Attempting to load asset from SD: {}", sd_path);

                    match sd.load_binary_file(&sd_path) {
                        Ok(data) => {
                            log::info!("Successfully loaded {} bytes from SD card: {}", data.len(), sd_path);
                            return Ok(AssetSource::SdCard(data));
                        }
                        Err(e) => {
                            log::warn!("Failed to load from SD card ({}), trying embedded fallback", e);
                        }
                    }
                }
            }
        }

        // Fallback to embedded
        if let Some(embedded) = asset_id.embedded_bytes() {
            log::info!("Loading embedded asset ({:?})", asset_id);
            return Ok(AssetSource::Embedded(embedded));
        }

        Err(format!("Asset not found: {:?}", asset_id).into())
    }

    /// Load asset bytes directly (convenience method)
    pub fn load_bytes(&mut self, asset_id: &AssetId) -> Result<Vec<u8>, Box<dyn Error>> {
        let source = self.load(asset_id)?;
        Ok(source.bytes().to_vec())
    }

    /// Check if SD card is available
    pub fn has_sd_card(&self) -> bool {
        self.sd_card.as_ref().map(|sd| sd.is_mounted()).unwrap_or(false)
    }

    /// Set SD card preference
    pub fn set_prefer_sd(&mut self, prefer: bool) {
        self.prefer_sd = prefer;
    }
}

/// Helper function to get monster icon bytes by species ID
pub fn get_monster_icon(species_id: &str) -> Option<&'static [u8]> {
    match species_id {
        "poring" => Some(include_bytes!("../assets/images/poring/icon.gif")),
        "lunatic" => Some(include_bytes!("../assets/images/lunatic/icon.gif")),
        "familiar" => Some(include_bytes!("../assets/images/familiar/icon.gif")),
        _ => None,
    }
}

/// Helper functions for loading battle assets
pub mod battle {
    use super::*;

    /// Load enemy sprites from AssetLoader or embedded fallback
    ///
    /// Returns (idle, attack, attacked, death) as Vec<u8>
    ///
    /// # Example
    /// ```no_run
    /// use crate::assets::battle::load_enemy_sprites;
    ///
    /// let mut loader = AssetLoader::new(Some(sd_card), true);
    /// let (idle, attack, attacked, death) = load_enemy_sprites(&mut loader, 1002)?; // Poring
    /// ```
    pub fn load_enemy_sprites<SD: SdCardOps + Clone>(
        loader: &mut AssetLoader<SD>,
        enemy_id: u32,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>, Option<Vec<u8>>), Box<dyn Error>> {
        let idle = loader.load_bytes(&AssetId::EnemySprite(enemy_id, SpriteAction::Idle))?;
        let attack = loader.load_bytes(&AssetId::EnemySprite(enemy_id, SpriteAction::Attack))?;
        let attacked =
            loader.load_bytes(&AssetId::EnemySprite(enemy_id, SpriteAction::Attacked))?;
        let death = loader
            .load_bytes(&AssetId::EnemySprite(enemy_id, SpriteAction::Death))
            .ok();

        Ok((idle, attack, attacked, death))
    }

    /// Load embedded enemy sprites (backward compatibility)
    ///
    /// This is the same as the old get_enemy_data() but uses enemy IDs
    pub fn load_enemy_sprites_embedded(
        enemy_id: u32,
    ) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>, Option<Vec<u8>>)> {
        match enemy_id {
            1002 => Some((
                // Poring
                include_bytes!("../assets/images/poring/6.gif").to_vec(),
                include_bytes!("../assets/images/poring/22.gif").to_vec(),
                include_bytes!("../assets/images/poring/30.gif").to_vec(),
                Some(include_bytes!("../assets/images/poring/38.gif").to_vec()),
            )),
            1004 => Some((
                // Hornet
                include_bytes!("../assets/images/hornet/6.gif").to_vec(),
                include_bytes!("../assets/images/hornet/22.gif").to_vec(),
                include_bytes!("../assets/images/hornet/30.gif").to_vec(),
                Some(include_bytes!("../assets/images/hornet/38.gif").to_vec()),
            )),
            1007 => Some((
                // Fabre
                include_bytes!("../assets/images/fabre/6.gif").to_vec(),
                include_bytes!("../assets/images/fabre/22.gif").to_vec(),
                include_bytes!("../assets/images/fabre/30.gif").to_vec(),
                Some(include_bytes!("../assets/images/fabre/38.gif").to_vec()),
            )),
            1051 => Some((
                // Thief Bug
                include_bytes!("../assets/images/thief_bug/6.gif").to_vec(),
                include_bytes!("../assets/images/thief_bug/22.gif").to_vec(),
                include_bytes!("../assets/images/thief_bug/30.gif").to_vec(),
                Some(include_bytes!("../assets/images/thief_bug/38.gif").to_vec()),
            )),
            _ => None,
        }
    }
}
