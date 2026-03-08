//! Data Loader
//!
//! Loads game data from JSON files embedded in the binary.

use serde::{Deserialize, de::DeserializeOwned};
use std::collections::HashMap;
use crate::game::core::{Species, Skill, SkillEffectType, StatType, Element, Zone, TamerMap, UnlockCondition, EssenceReward, MapBaseRewards, Dungeon, EnemyPool, LearnableSkill};

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

/// Learnable skill data from JSON
#[derive(Debug, Deserialize)]
struct LearnableSkillData {
    skill_id: String,
    level_required: u8,
}

/// Species data from JSON (slightly different from core Species)
#[derive(Debug, Deserialize)]
struct SpeciesData {
    id: String,
    name: String,
    element: String,
    #[serde(default = "default_level")]
    level: u8,
    base_hp: u16,
    base_atk: u16,
    base_def: u16,
    base_spd: u16,
    #[serde(default)]
    base_exp: u32,
    /// Skills this species can learn at specific levels
    learnable_skills: Vec<LearnableSkillData>,
    zones: Vec<String>,
    #[serde(default)]
    swap_talent: Option<String>,
    #[serde(default)]
    is_boss: bool,
}

fn default_level() -> u8 {
    1
}

impl SpeciesData {
    fn to_species(&self) -> Species {
        let learnable_skills: Vec<LearnableSkill> = self.learnable_skills
            .iter()
            .map(|ls| LearnableSkill {
                skill_id: ls.skill_id.clone(),
                level_required: ls.level_required,
            })
            .collect();

        Species {
            id: self.id.clone(),
            name: self.name.clone(),
            element: parse_element(&self.element),
            base_level: self.level,
            base_hp: self.base_hp,
            base_atk: self.base_atk,
            base_def: self.base_def,
            base_spd: self.base_spd,
            base_exp: self.base_exp,
            learnable_skills,
            zones: self.zones.clone(),
        }
    }
}

/// Skills file wrapper
#[derive(Debug, Deserialize)]
struct SkillsFile {
    skills: Vec<SkillData>,
}

/// Skill data from JSON (Pokemon-style with power, accuracy, cooldown)
#[derive(Debug, Deserialize)]
struct SkillData {
    id: String,
    name: String,
    element: String,
    description: String,
    effect_type: String,
    /// Base power for damage calculation (0 for non-damage skills)
    #[serde(default)]
    power: u16,
    /// Accuracy percentage (0-100, 100 = always hits)
    #[serde(default = "default_accuracy")]
    accuracy: u8,
    /// Critical hit chance percentage (0-100, default 10%)
    #[serde(default = "default_crit_chance")]
    crit_chance: u8,
    /// Cooldown in turns after use (0 = no cooldown)
    #[serde(default)]
    cooldown: u8,
    /// Effect value (heal percent, buff percent, etc.)
    #[serde(default = "default_effect_value")]
    effect_value: f32,
    /// Stat affected by buff/debuff
    #[serde(default)]
    buff_stat: Option<String>,
    /// Buff/debuff duration in turns
    #[serde(default)]
    buff_duration: u8,
    /// DoT damage per turn
    #[serde(default)]
    dot_damage: u16,
    /// DoT duration in turns
    #[serde(default)]
    dot_duration: u8,
}

fn default_accuracy() -> u8 {
    100
}

fn default_crit_chance() -> u8 {
    10  // 10% base crit chance
}

fn default_effect_value() -> f32 {
    1.0
}

impl SkillData {
    fn to_skill(&self) -> Skill {
        let effect_type = match self.effect_type.as_str() {
            "damage" => SkillEffectType::Damage,
            "damage_dot" => SkillEffectType::DamageDot,
            "damage_pierce" | "damage_ignore_def" => SkillEffectType::DamageIgnoreDef,
            "heal" => SkillEffectType::Heal,
            "buff" => SkillEffectType::Buff,
            "debuff" => SkillEffectType::Debuff,
            _ => SkillEffectType::Damage,
        };

        // Parse buff stat if present
        let buff_stat = self.buff_stat.as_ref().map(|s| match s.to_lowercase().as_str() {
            "atk" => StatType::Atk,
            "def" => StatType::Def,
            "spd" => StatType::Spd,
            _ => StatType::Atk,
        });

        Skill {
            id: self.id.clone(),
            name: self.name.clone(),
            element: parse_element(&self.element),
            description: self.description.clone(),
            effect_type,
            power: self.power,
            accuracy: self.accuracy,
            crit_chance: self.crit_chance,
            cooldown: self.cooldown,
            effect_value: self.effect_value,
            buff_stat,
            buff_duration: self.buff_duration,
            dot_damage: self.dot_damage,
            dot_duration: self.dot_duration,
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
        "neutral" => Element::Neutral,
        _ => Element::Neutral, // Default to Neutral for unknown
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
