//! Central Game Data Store
//!
//! Holds all loaded game data and provides lookup methods.

use std::collections::HashMap;
use crate::game::core::{Species, Skill, Monster, create_monster, create_monster_at_level, Zone, TamerMap, Dungeon};
use super::loader;

/// Central store for all game data loaded from JSON files
#[derive(Debug, Clone, Default)]
pub struct TamerGameData {
    /// All species indexed by ID
    pub species: HashMap<String, Species>,
    /// All skills indexed by ID
    pub skills: HashMap<String, Skill>,
    /// All zones indexed by ID
    pub zones: HashMap<String, Zone>,
    /// All tamer maps indexed by ID
    pub tamer_maps: HashMap<String, TamerMap>,
    /// All dungeons indexed by ID
    pub dungeons: HashMap<String, Dungeon>,
}

impl TamerGameData {
    /// Create a new empty game data store
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all game data from embedded JSON files
    pub fn load() -> Result<Self, String> {
        let mut data = Self::new();

        // Load skills first (species reference skills)
        match loader::load_skills() {
            Ok(skills) => {
                for skill in skills {
                    data.add_skill(skill);
                }
            }
            Err(e) => log::warn!("Failed to load skills: {}", e),
        }

        // Load species
        match loader::load_species() {
            Ok(species) => {
                for sp in species {
                    data.add_species(sp);
                }
            }
            Err(e) => log::warn!("Failed to load species: {}", e),
        }

        // Load zones
        match loader::load_zones() {
            Ok(zones) => {
                for zone in zones {
                    data.add_zone(zone);
                }
            }
            Err(e) => log::warn!("Failed to load zones: {}", e),
        }

        // Load tamer maps
        match loader::load_tamer_maps() {
            Ok(maps) => {
                for map in maps {
                    data.add_tamer_map(map);
                }
            }
            Err(e) => log::warn!("Failed to load tamer maps: {}", e),
        }

        // Load dungeons
        match loader::load_dungeons() {
            Ok(dungeons) => {
                for dungeon in dungeons {
                    data.add_dungeon(dungeon);
                }
            }
            Err(e) => log::warn!("Failed to load dungeons: {}", e),
        }

        log::info!("TamerGameData loaded: {} species, {} skills, {} zones, {} maps, {} dungeons",
            data.species.len(), data.skills.len(), data.zones.len(), data.tamer_maps.len(), data.dungeons.len());

        Ok(data)
    }

    /// Get species by ID
    pub fn get_species(&self, id: &str) -> Option<&Species> {
        self.species.get(id)
    }

    /// Get skill by ID
    pub fn get_skill(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }

    /// Add a species to the store
    pub fn add_species(&mut self, species: Species) {
        self.species.insert(species.id.clone(), species);
    }

    /// Add a skill to the store
    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.id.clone(), skill);
    }

    /// Get all species
    pub fn all_species(&self) -> impl Iterator<Item = &Species> {
        self.species.values()
    }

    /// Get all skills
    pub fn all_skills(&self) -> impl Iterator<Item = &Skill> {
        self.skills.values()
    }

    /// Create a new monster from a species ID
    /// Monster will have all skills it can learn at its base level
    pub fn create_monster(&self, species_id: &str) -> Option<Monster> {
        let species = self.get_species(species_id)?;
        let initial_skills = self.get_skills_for_level(species, species.base_level);
        Some(create_monster(species, initial_skills))
    }

    /// Create a monster at a specific level
    /// Monster will have all skills it can learn up to the specified level
    pub fn create_monster_at_level(&self, species_id: &str, level: u8) -> Option<Monster> {
        let species = self.get_species(species_id)?;
        let initial_skills = self.get_skills_for_level(species, level);
        Some(create_monster_at_level(species, initial_skills, level))
    }

    /// Get all skills a species can learn at a given level
    fn get_skills_for_level(&self, species: &Species, level: u8) -> Vec<Skill> {
        species.learnable_skills
            .iter()
            .filter(|ls| ls.level_required <= level)
            .filter_map(|ls| self.get_skill(&ls.skill_id).cloned())
            .collect()
    }

    /// Get capturable species for a zone
    pub fn get_species_for_zone(&self, zone: &str) -> Vec<&Species> {
        self.species.values()
            .filter(|s| s.zones.iter().any(|z| z == zone))
            .collect()
    }

    /// Add a zone to the store
    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.insert(zone.id.clone(), zone);
    }

    /// Add a tamer map to the store
    pub fn add_tamer_map(&mut self, map: TamerMap) {
        self.tamer_maps.insert(map.id.clone(), map);
    }

    /// Get zone by ID
    pub fn get_zone(&self, id: &str) -> Option<&Zone> {
        self.zones.get(id)
    }

    /// Get tamer map by ID
    pub fn get_tamer_map(&self, id: &str) -> Option<&TamerMap> {
        self.tamer_maps.get(id)
    }

    /// Get all zones
    pub fn all_zones(&self) -> impl Iterator<Item = &Zone> {
        self.zones.values()
    }

    /// Get all tamer maps
    pub fn all_tamer_maps(&self) -> impl Iterator<Item = &TamerMap> {
        self.tamer_maps.values()
    }

    /// Get maps for a zone
    pub fn get_maps_for_zone(&self, zone_id: &str) -> Vec<&TamerMap> {
        self.tamer_maps.values()
            .filter(|m| m.zone_id == zone_id)
            .collect()
    }

    /// Get unlocked zones based on dungeon progress
    pub fn get_unlocked_zones(&self, dungeon_progress: &std::collections::HashMap<String, u16>) -> Vec<&Zone> {
        self.zones.values()
            .filter(|z| z.is_unlocked(dungeon_progress))
            .collect()
    }

    /// Add a dungeon to the store
    pub fn add_dungeon(&mut self, dungeon: Dungeon) {
        self.dungeons.insert(dungeon.id.clone(), dungeon);
    }

    /// Get dungeon by ID
    pub fn get_dungeon(&self, id: &str) -> Option<&Dungeon> {
        self.dungeons.get(id)
    }

    /// Get all dungeons
    pub fn all_dungeons(&self) -> impl Iterator<Item = &Dungeon> {
        self.dungeons.values()
    }

    /// Get dungeon for a zone
    pub fn get_dungeon_for_zone(&self, zone_id: &str) -> Option<&Dungeon> {
        self.zones.get(zone_id)
            .and_then(|zone| self.dungeons.get(&zone.dungeon_id))
    }
}
