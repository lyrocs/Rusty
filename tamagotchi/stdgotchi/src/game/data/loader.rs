//! Data Loader
//!
//! Loads game data from JSON files embedded in the binary.

use serde::{Deserialize, de::DeserializeOwned};
use std::collections::HashMap;
use crate::game::core::{Species, Skill, SkillEffectType, Element, Zone, TamerMap, UnlockCondition, EssenceReward, MapBaseRewards, Dungeon, EnemyPool};

/// Load JSON data from embedded asset
pub fn load_json<T: DeserializeOwned>(json_str: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json_str)
}

/// Species file wrapper
#[derive(Debug, Deserialize)]
struct SpeciesFile {
    species: Vec<SpeciesData>,
    #[serde(default)]
    swap_talents: HashMap<String, serde_json::Value>,
}

/// Species data from JSON (slightly different from core Species)
#[derive(Debug, Deserialize)]
struct SpeciesData {
    id: String,
    name: String,
    element: String,
    base_hp: u16,
    base_atk: u16,
    base_def: u16,
    base_spd: u16,
    skill_id: String,
    zones: Vec<String>,
    #[serde(default)]
    swap_talent: Option<String>,
    #[serde(default)]
    is_boss: bool,
}

impl SpeciesData {
    fn to_species(&self) -> Species {
        Species {
            id: self.id.clone(),
            name: self.name.clone(),
            element: parse_element(&self.element),
            base_hp: self.base_hp,
            base_atk: self.base_atk,
            base_def: self.base_def,
            base_spd: self.base_spd,
            skill_id: self.skill_id.clone(),
            zones: self.zones.clone(),
        }
    }
}

/// Skills file wrapper
#[derive(Debug, Deserialize)]
struct SkillsFile {
    skills: Vec<SkillData>,
}

/// Skill data from JSON
#[derive(Debug, Deserialize)]
struct SkillData {
    id: String,
    name: String,
    element: String,
    description: String,
    effect_type: String,
    #[serde(default)]
    damage_multiplier: Option<f32>,
    #[serde(default)]
    heal_percent: Option<f32>,
    #[serde(default)]
    dot_damage: Option<u16>,
    #[serde(default)]
    dot_duration: Option<f32>,
    #[serde(default)]
    dot_tick_interval: Option<f32>,
    #[serde(default)]
    buff_stat: Option<String>,
    #[serde(default)]
    buff_percent: Option<f32>,
    #[serde(default)]
    buff_duration: Option<f32>,
    #[serde(default)]
    debuff_stat: Option<String>,
    #[serde(default)]
    debuff_percent: Option<f32>,
    #[serde(default)]
    debuff_duration: Option<f32>,
    #[serde(default)]
    def_ignore_percent: Option<f32>,
    #[serde(default)]
    applies_aura: Option<String>,
    #[serde(default)]
    aura_duration: Option<f32>,
}

impl SkillData {
    fn to_skill(&self) -> Skill {
        let effect_type = match self.effect_type.as_str() {
            "damage" => SkillEffectType::Damage,
            "damage_dot" => SkillEffectType::DamageDot,
            "damage_aura" => SkillEffectType::Damage, // Simplified
            "damage_pierce" => SkillEffectType::DamageIgnoreDef,
            "damage_swirl" => SkillEffectType::Damage,
            "damage_steal" => SkillEffectType::Damage,
            "heal" => SkillEffectType::Heal,
            "buff" | "debuff" => SkillEffectType::Buff,
            _ => SkillEffectType::Damage,
        };

        // Get the primary effect value
        let effect_value = self.damage_multiplier
            .or(self.heal_percent)
            .or(self.buff_percent)
            .or(self.debuff_percent)
            .unwrap_or(1.0);

        Skill {
            id: self.id.clone(),
            name: self.name.clone(),
            element: parse_element(&self.element),
            description: self.description.clone(),
            effect_type,
            effect_value,
            dot_duration: self.dot_duration.unwrap_or(0.0),
            buff_duration: self.buff_duration.or(self.debuff_duration).unwrap_or(0.0),
        }
    }
}

/// Parse element string to Element enum
fn parse_element(s: &str) -> Element {
    match s.to_lowercase().as_str() {
        "fire" => Element::Fire,
        "water" => Element::Water,
        "earth" => Element::Earth,
        "wind" => Element::Wind,
        "thunder" => Element::Thunder,
        "shadow" => Element::Shadow,
        "holy" => Element::Holy,
        "ghost" => Element::Ghost,
        _ => Element::Water, // Default
    }
}

/// Load species data from embedded JSON
pub fn load_species() -> Result<Vec<Species>, String> {
    let json_str = include_str!("../../../assets/data/species.json");
    let file: SpeciesFile = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse species.json: {}", e))?;

    let species: Vec<Species> = file.species.iter().map(|s| s.to_species()).collect();
    log::info!("Loaded {} species from species.json", species.len());
    Ok(species)
}

/// Load skills data from embedded JSON
pub fn load_skills() -> Result<Vec<Skill>, String> {
    let json_str = include_str!("../../../assets/data/tamer_skills.json");
    let file: SkillsFile = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse tamer_skills.json: {}", e))?;

    let skills: Vec<Skill> = file.skills.iter().map(|s| s.to_skill()).collect();
    log::info!("Loaded {} skills from tamer_skills.json", skills.len());
    Ok(skills)
}

// ============================================
// Zone and Map Loading
// ============================================

/// Zones file wrapper
#[derive(Debug, Deserialize)]
struct ZonesFile {
    zones: Vec<ZoneData>,
}

/// Zone data from JSON
#[derive(Debug, Deserialize)]
struct ZoneData {
    id: String,
    name: String,
    description: String,
    maps: Vec<String>,
    dungeon_id: String,
    unlock_condition: Option<UnlockConditionData>,
    level_range: [u8; 2],
}

/// Unlock condition from JSON
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum UnlockConditionData {
    DungeonFloor {
        dungeon_id: String,
        floor: u16,
    },
}

impl ZoneData {
    fn to_zone(&self) -> Zone {
        let unlock = self.unlock_condition.as_ref().map(|c| match c {
            UnlockConditionData::DungeonFloor { dungeon_id, floor } => {
                UnlockCondition::DungeonFloor {
                    dungeon_id: dungeon_id.clone(),
                    floor: *floor,
                }
            }
        });

        Zone {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            maps: self.maps.clone(),
            dungeon_id: self.dungeon_id.clone(),
            unlock_condition: unlock,
            level_range: (self.level_range[0], self.level_range[1]),
        }
    }
}

/// Maps file wrapper
#[derive(Debug, Deserialize)]
struct TamerMapsFile {
    maps: Vec<TamerMapData>,
}

/// Map data from JSON
#[derive(Debug, Deserialize)]
struct TamerMapData {
    id: String,
    name: String,
    zone_id: String,
    level_range: [u8; 2],
    required_elements: Vec<String>,
    capturable_species: Vec<String>,
    base_rewards: BaseRewardsData,
}

/// Base rewards from JSON
#[derive(Debug, Deserialize)]
struct BaseRewardsData {
    crystals: u16,
    essences: Vec<EssenceData>,
}

/// Essence data from JSON
#[derive(Debug, Deserialize)]
struct EssenceData {
    element: String,
    amount: u8,
}

impl TamerMapData {
    fn to_tamer_map(&self) -> TamerMap {
        let elements: Vec<Element> = self.required_elements
            .iter()
            .map(|e| parse_element(e))
            .collect();

        let essences: Vec<EssenceReward> = self.base_rewards.essences
            .iter()
            .map(|e| EssenceReward {
                element: parse_element(&e.element),
                amount: e.amount,
            })
            .collect();

        TamerMap {
            id: self.id.clone(),
            name: self.name.clone(),
            zone_id: self.zone_id.clone(),
            level_range: (self.level_range[0], self.level_range[1]),
            required_elements: elements,
            capturable_species: self.capturable_species.clone(),
            base_rewards: MapBaseRewards {
                crystals: self.base_rewards.crystals,
                essences,
            },
        }
    }
}

/// Load zones data from embedded JSON
pub fn load_zones() -> Result<Vec<Zone>, String> {
    let json_str = include_str!("../../../assets/data/zones.json");
    let file: ZonesFile = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse zones.json: {}", e))?;

    let zones: Vec<Zone> = file.zones.iter().map(|z| z.to_zone()).collect();
    log::info!("Loaded {} zones from zones.json", zones.len());
    Ok(zones)
}

/// Load tamer maps data from embedded JSON
pub fn load_tamer_maps() -> Result<Vec<TamerMap>, String> {
    let json_str = include_str!("../../../assets/data/tamer_maps.json");
    let file: TamerMapsFile = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse tamer_maps.json: {}", e))?;

    let maps: Vec<TamerMap> = file.maps.iter().map(|m| m.to_tamer_map()).collect();
    log::info!("Loaded {} maps from tamer_maps.json", maps.len());
    Ok(maps)
}

// ============================================
// Dungeon Loading
// ============================================

/// Dungeons file wrapper
#[derive(Debug, Deserialize)]
struct DungeonsFile {
    dungeons: Vec<DungeonData>,
}

/// Dungeon data from JSON
#[derive(Debug, Deserialize)]
struct DungeonData {
    id: String,
    name: String,
    zone_id: String,
    description: String,
    checkpoints: Vec<u16>,
    dominant_elements: Vec<String>,
    enemy_pools: Vec<EnemyPoolData>,
    boss_floors: Vec<u16>,
    bosses: HashMap<String, String>,
    base_crystal_reward: u32,
    base_xp_reward: u32,
}

/// Enemy pool data from JSON
#[derive(Debug, Deserialize)]
struct EnemyPoolData {
    floor_min: u16,
    floor_max: u16,
    species: Vec<String>,
    enemies_per_floor: u8,
}

impl DungeonData {
    fn to_dungeon(&self) -> Dungeon {
        let elements: Vec<Element> = self.dominant_elements
            .iter()
            .map(|e| parse_element(e))
            .collect();

        let pools: Vec<EnemyPool> = self.enemy_pools
            .iter()
            .map(|p| EnemyPool {
                floor_min: p.floor_min,
                floor_max: p.floor_max,
                species: p.species.clone(),
                enemies_per_floor: p.enemies_per_floor,
            })
            .collect();

        Dungeon {
            id: self.id.clone(),
            name: self.name.clone(),
            zone_id: self.zone_id.clone(),
            description: self.description.clone(),
            checkpoints: self.checkpoints.clone(),
            dominant_elements: elements,
            enemy_pools: pools,
            boss_floors: self.boss_floors.clone(),
            bosses: self.bosses.clone(),
            base_crystal_reward: self.base_crystal_reward,
            base_xp_reward: self.base_xp_reward,
        }
    }
}

/// Load dungeons data from embedded JSON
pub fn load_dungeons() -> Result<Vec<Dungeon>, String> {
    let json_str = include_str!("../../../assets/data/dungeons.json");
    let file: DungeonsFile = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse dungeons.json: {}", e))?;

    let dungeons: Vec<Dungeon> = file.dungeons.iter().map(|d| d.to_dungeon()).collect();
    log::info!("Loaded {} dungeons from dungeons.json", dungeons.len());
    Ok(dungeons)
}
