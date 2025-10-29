#![no_std]

extern crate alloc;

pub mod core;       // Core game state and types
pub mod hero;       // Hero domain (character, stats, inventory, equipment)
pub mod combat;     // Combat domain (enemies, battles, skills, animations)
pub mod quest;      // Quest domain (quests, objectives, rewards)
pub mod world;      // World domain (maps, navigation, locations)
pub mod systems;    // ECS systems (organized by responsibility)
pub mod data;       // Game data (enemies, maps, items, NPCs, drops)
pub mod drivers;
pub mod display;
pub mod ui;
pub mod ecs;
pub mod utils;
pub mod tamagotchi;
