# Implementation Plan: Mini MVP/MVP Monsters, Semi-Active Battle, and Card Skills System

## Overview

This document outlines the implementation plan for three major features:
1. **Mini MVP and MVP Monster Types** with spawn timers
2. **Semi-Active Battle Mode** with animations and skill cooldowns
3. **Skills from Cards** with card selection system

---

## Feature 1: Mini MVP and MVP Monster System

### 1.1 Data Structure Updates

#### 1.1.1 Modify `assets/data/enemies.json`
Add new fields to enemy definitions:
```json
{
  "id": 1096,
  "name": "Angeling",
  "level": 77,
  "hp": 23000,
  "attack": 250,
  "defense": 72,
  "hit": 200,
  "flee": 200,
  "base_exp": 50000,
  "element": "holy",
  "monster_type": "mini_mvp",
  "spawn_timer_minutes": 20,
  "spawn_map_id": 10,
  "drop_rate": 0.01,
  "card": {
    "name": "Angeling Card",
    "rarity": 5,
    "atk_bonus": 100,
    "def_bonus": 100
  }
}
```

```json
{
  "id": 1112,
  "name": "Drake",
  "level": 91,
  "hp": 150000,
  "attack": 500,
  "defense": 100,
  "hit": 250,
  "flee": 150,
  "base_exp": 250000,
  "element": "wind",
  "monster_type": "mvp",
  "spawn_timer_minutes": 180,
  "spawn_map_id": 15,
  "drop_rate": 0.02,
  "card": {
    "name": "Drake Card",
    "rarity": 5,
    "atk_bonus": 200,
    "def_bonus": 150
  }
}
```

Add regular monster types:
```json
{
  "monster_type": "normal"
}
```

#### 1.1.2 Update `src/game/enemy.rs`
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MonsterType {
    Normal,
    MiniMvp,
    Mvp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyData {
    pub id: u32,
    pub name: String,
    pub level: u32,
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub hit: u32,
    pub flee: u32,
    pub base_exp: u64,
    pub element: Element,
    pub str_stat: u16,
    pub agi: u16,
    pub int: u16,
    pub dex: u16,
    pub vit: u16,
    pub luk: u16,
    pub drop_rate: f32,
    pub card: Option<CardData>,

    // NEW FIELDS
    pub monster_type: MonsterType,
    pub spawn_timer_minutes: Option<u32>,  // Only for Mini MVP/MVP
    pub spawn_map_id: Option<u32>,         // Only for Mini MVP/MVP
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enemy {
    // ... existing fields ...
    pub monster_type: MonsterType,
}
```

### 1.2 Spawn Timer System

#### 1.2.1 Create `src/game/mvp_spawn_manager.rs`
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MvpSpawnState {
    pub monster_id: u32,
    pub last_killed_timestamp: Option<u64>,  // Unix timestamp
    pub spawn_time: u64,                      // Unix timestamp when available
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MvpSpawnManager {
    pub spawn_states: HashMap<u32, MvpSpawnState>,  // map_id -> spawn state
}

impl MvpSpawnManager {
    pub fn new() -> Self {
        Self {
            spawn_states: HashMap::new(),
        }
    }

    /// Initialize spawn states for all Mini MVP/MVP monsters
    pub fn initialize_from_game_data(&mut self, game_data: &GameData) {
        for (enemy_id, enemy_data) in &game_data.enemies {
            if matches!(enemy_data.monster_type, MonsterType::MiniMvp | MonsterType::Mvp) {
                if let Some(map_id) = enemy_data.spawn_map_id {
                    self.spawn_states.insert(map_id, MvpSpawnState {
                        monster_id: *enemy_id,
                        last_killed_timestamp: None,
                        spawn_time: Self::current_timestamp(),  // Available immediately on first load
                    });
                }
            }
        }
    }

    /// Check if a monster is available to fight on a specific map
    pub fn is_available(&self, map_id: u32) -> bool {
        if let Some(state) = self.spawn_states.get(&map_id) {
            Self::current_timestamp() >= state.spawn_time
        } else {
            false
        }
    }

    /// Get time remaining until spawn (in seconds)
    pub fn time_until_spawn(&self, map_id: u32) -> Option<u64> {
        if let Some(state) = self.spawn_states.get(&map_id) {
            let current = Self::current_timestamp();
            if current < state.spawn_time {
                Some(state.spawn_time - current)
            } else {
                Some(0)  // Available now
            }
        } else {
            None
        }
    }

    /// Called when a Mini MVP/MVP is defeated
    pub fn record_kill(&mut self, map_id: u32, respawn_minutes: u32) {
        let current = Self::current_timestamp();
        let spawn_time = current + (respawn_minutes as u64 * 60);

        if let Some(state) = self.spawn_states.get_mut(&map_id) {
            state.last_killed_timestamp = Some(current);
            state.spawn_time = spawn_time;
        }
    }

    /// Get monster ID for a specific map
    pub fn get_monster_id(&self, map_id: u32) -> Option<u32> {
        self.spawn_states.get(&map_id).map(|s| s.monster_id)
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
```

#### 1.2.2 Add to `src/game/save.rs`
```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct SaveData {
    pub version: u32,
    pub kill_tracker: KillTracker,
    pub current_location_id: u32,
    pub play_time_seconds: u64,
    pub save_timestamp: u64,
    pub hero: Hero,
    pub quest_manager: QuestManager,

    // NEW FIELD
    pub mvp_spawn_manager: MvpSpawnManager,
}

// Update VERSION to 6
const VERSION: u32 = 6;
```

### 1.3 Map Detail UI Updates

#### 1.3.1 Modify `src/ui/pages/map.rs`
Add button to fight Mini MVP/MVP when available:

```rust
// In MapPage struct
pub struct MapPage {
    // ... existing fields ...
    show_mvp_button: bool,
    mvp_monster_name: Option<String>,
}

// In draw() method
fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
    // ... existing drawing code ...

    // Check if current map has a Mini MVP/MVP
    let current_map_id = game_manager.world_map.current_location_id;
    if let Some(mvp_spawn_manager) = &game_manager.save_data.mvp_spawn_manager {
        if let Some(monster_id) = mvp_spawn_manager.get_monster_id(current_map_id) {
            let is_available = mvp_spawn_manager.is_available(current_map_id);

            if let Some(enemy_data) = game_data.enemies.get(&monster_id) {
                self.show_mvp_button = true;
                self.mvp_monster_name = Some(enemy_data.name.clone());

                if is_available {
                    // Draw button: "Fight [Monster Name]" (green/active)
                    // Position: Below regular action buttons
                    draw_button(display, "Fight Angeling", 120, ACTIVE_COLOR)?;
                } else {
                    // Draw button with timer: "Angeling spawns in 15:30" (gray/disabled)
                    if let Some(seconds) = mvp_spawn_manager.time_until_spawn(current_map_id) {
                        let minutes = seconds / 60;
                        let secs = seconds % 60;
                        let text = format!("{} spawns in {}:{:02}", enemy_data.name, minutes, secs);
                        draw_button(display, &text, 120, DISABLED_COLOR)?;
                    }
                }
            }
        }
    }

    // ... rest of drawing code ...
}

// In update() method
fn update(&mut self) -> bool {
    // ... existing input handling ...

    // Handle MVP button press
    if self.show_mvp_button && button_pressed(MVP_BUTTON_ID) {
        if let Some(monster_id) = mvp_spawn_manager.get_monster_id(current_map_id) {
            if mvp_spawn_manager.is_available(current_map_id) {
                // Transition to battle with this specific monster
                // Use new semi-active battle mode
                self.pending_battle = Some(BattleLoadingData {
                    enemy_id: monster_id,
                    battle_mode: BattleMode::SemiActive,  // NEW battle mode
                });
                return false;  // Close page, transition to battle
            }
        }
    }

    true
}
```

### 1.4 Battle Result Updates

#### 1.4.1 Modify `src/ui/pages/battle_result.rs`
When battle ends, check if defeated enemy was Mini MVP/MVP:

```rust
fn on_enter(&mut self) {
    // ... existing code ...

    // Check if this was a Mini MVP/MVP battle
    if let Some(enemy) = &self.defeated_enemy {
        match enemy.monster_type {
            MonsterType::MiniMvp => {
                if let Some(spawn_timer) = enemy.spawn_timer_minutes {
                    game_manager.save_data.mvp_spawn_manager
                        .record_kill(current_map_id, spawn_timer);
                }
            }
            MonsterType::Mvp => {
                if let Some(spawn_timer) = enemy.spawn_timer_minutes {
                    game_manager.save_data.mvp_spawn_manager
                        .record_kill(current_map_id, spawn_timer);
                }
            }
            MonsterType::Normal => {
                // No special handling
            }
        }
    }
}
```

---

## Feature 2: Semi-Active Battle Mode

### 2.1 Battle Mode Enum

#### 2.1.1 Create `src/game/battle_mode.rs`
```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BattleMode {
    Auto,          // Existing auto-battle (AFK farm)
    SemiActive,    // NEW: Turn-based with active skill usage
}
```

### 2.2 Skill System Foundation

#### 2.2.1 Create `src/game/skill.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SkillType {
    Attack,      // Damage-dealing skill
    Heal,        // Restore HP
    Buff,        // Temporary stat boost
    Debuff,      // Temporary stat reduction on enemy
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SkillTarget {
    Self_,       // Targets the caster
    Enemy,       // Targets the enemy
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillData {
    pub id: u32,
    pub name: String,
    pub skill_type: SkillType,
    pub target: SkillTarget,
    pub cooldown_seconds: f32,
    pub power: u32,              // Damage multiplier or heal amount
    pub description: String,
    pub animation_name: String,  // For future skill animations
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveSkill {
    pub skill_id: u32,
    pub remaining_cooldown: f32,  // Seconds until can use again
}

impl ActiveSkill {
    pub fn new(skill_id: u32) -> Self {
        Self {
            skill_id,
            remaining_cooldown: 0.0,  // Available immediately
        }
    }

    pub fn is_ready(&self) -> bool {
        self.remaining_cooldown <= 0.0
    }

    pub fn use_skill(&mut self, cooldown_seconds: f32) {
        self.remaining_cooldown = cooldown_seconds;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.remaining_cooldown > 0.0 {
            self.remaining_cooldown -= delta_time;
            if self.remaining_cooldown < 0.0 {
                self.remaining_cooldown = 0.0;
            }
        }
    }
}
```

#### 2.2.2 Create `assets/data/skills.json`
```json
[
  {
    "id": 1,
    "name": "Bash",
    "skill_type": "Attack",
    "target": "Enemy",
    "cooldown_seconds": 5.0,
    "power": 150,
    "description": "Deals 150% ATK damage",
    "animation_name": "bash"
  },
  {
    "id": 2,
    "name": "Heal Lvl 1",
    "skill_type": "Heal",
    "target": "Self",
    "cooldown_seconds": 10.0,
    "power": 100,
    "description": "Restores 100 HP",
    "animation_name": "heal"
  },
  {
    "id": 3,
    "name": "Heal Lvl 2",
    "skill_type": "Heal",
    "target": "Self",
    "cooldown_seconds": 10.0,
    "power": 200,
    "description": "Restores 200 HP",
    "animation_name": "heal"
  },
  {
    "id": 4,
    "name": "Fire Bolt",
    "skill_type": "Attack",
    "target": "Enemy",
    "cooldown_seconds": 3.0,
    "power": 120,
    "description": "Deals 120% MATK damage",
    "animation_name": "fire_bolt"
  },
  {
    "id": 5,
    "name": "Power Strike",
    "skill_type": "Attack",
    "target": "Enemy",
    "cooldown_seconds": 7.0,
    "power": 200,
    "description": "Powerful attack dealing 200% ATK damage",
    "animation_name": "power_strike"
  }
]
```

### 2.3 Hero Skill Slots

#### 2.3.1 Update `src/game/hero.rs`
```rust
pub struct Hero {
    // ... existing fields ...

    // NEW FIELDS
    pub equipped_skill_slots: [Option<u32>; 3],  // Up to 3 skill IDs equipped
    pub active_skills: Vec<ActiveSkill>,          // Tracks cooldowns during battle
}

impl Hero {
    /// Equip a skill to a specific slot (0, 1, or 2)
    pub fn equip_skill(&mut self, slot_index: usize, skill_id: u32) -> Result<(), String> {
        if slot_index >= 3 {
            return Err("Invalid slot index".to_string());
        }
        self.equipped_skill_slots[slot_index] = Some(skill_id);
        Ok(())
    }

    /// Unequip a skill from a slot
    pub fn unequip_skill(&mut self, slot_index: usize) {
        if slot_index < 3 {
            self.equipped_skill_slots[slot_index] = None;
        }
    }

    /// Initialize active skills at battle start
    pub fn initialize_battle_skills(&mut self) {
        self.active_skills.clear();
        for slot in &self.equipped_skill_slots {
            if let Some(skill_id) = slot {
                self.active_skills.push(ActiveSkill::new(*skill_id));
            }
        }
    }
}
```

### 2.4 Semi-Active Battle Page

#### 2.4.1 Create `src/ui/pages/semi_active_battle.rs`
```rust
use crate::game::battle::{BattleState, hero_attack_enemy, enemy_attack_hero};
use crate::game::skill::{SkillData, ActiveSkill, SkillType, SkillTarget};
use crate::ui::sprite::{AnimatedSprite, BattleEntity};
use std::time::Instant;

pub struct SemiActiveBattlePage {
    // Battle entities
    hero_entity: BattleEntity,
    enemy_entity: BattleEntity,

    // Battle state
    battle_state: BattleState,
    turn: BattleTurn,
    turn_timer: Instant,

    // Skill UI
    skill_buttons: [SkillButton; 3],
    selected_skill: Option<usize>,  // Which skill button was pressed

    // Animation state
    current_animation: Option<AnimationState>,
    animation_timer: Instant,

    // Damage numbers
    damage_numbers: Vec<DamageNumber>,

    // Battle result
    battle_ended: bool,
    victory: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum BattleTurn {
    HeroTurn,       // Waiting for player to select action
    HeroAttack,     // Playing hero attack animation
    EnemyHit,       // Playing enemy hit animation
    EnemyTurn,      // Enemy AI deciding
    EnemyAttack,    // Playing enemy attack animation
    HeroHit,        // Playing hero hit animation
}

#[derive(Clone, Debug)]
enum AnimationState {
    HeroAttacking,
    EnemyTakingDamage,
    EnemyAttacking,
    HeroTakingDamage,
    SkillCasting { skill_name: String },
}

#[derive(Clone, Debug)]
struct SkillButton {
    skill_id: Option<u32>,
    skill_name: String,
    cooldown_remaining: f32,
    position: (i32, i32),
    is_ready: bool,
}

impl SemiActiveBattlePage {
    pub fn new(
        hero: &Hero,
        enemy: &Enemy,
        game_data: &GameData,
    ) -> Result<Self, Box<dyn Error>> {
        // Load hero sprite
        let hero_entity = BattleEntity::new(
            &format!("assets/sprites/hero/{}_idle.gif", hero.job.sprite_name()),
            &format!("assets/sprites/hero/{}_attack.gif", hero.job.sprite_name()),
            &format!("assets/sprites/hero/{}_hit.gif", hero.job.sprite_name()),
            &format!("assets/sprites/hero/{}_death.gif", hero.job.sprite_name()),
            (60, 140),
            false,  // Don't flip hero
        )?;

        // Load enemy sprite
        let enemy_entity = BattleEntity::new(
            &format!("assets/sprites/enemies/{}_idle.gif", enemy.id),
            &format!("assets/sprites/enemies/{}_attack.gif", enemy.id),
            &format!("assets/sprites/enemies/{}_hit.gif", enemy.id),
            &format!("assets/sprites/enemies/{}_death.gif", enemy.id),
            (220, 140),
            true,  // Flip enemy to face left
        )?;

        // Initialize skill buttons
        let mut skill_buttons = [
            SkillButton::empty(),
            SkillButton::empty(),
            SkillButton::empty(),
        ];

        for (i, slot) in hero.equipped_skill_slots.iter().enumerate() {
            if let Some(skill_id) = slot {
                if let Some(skill_data) = game_data.skills.get(skill_id) {
                    skill_buttons[i] = SkillButton {
                        skill_id: Some(*skill_id),
                        skill_name: skill_data.name.clone(),
                        cooldown_remaining: 0.0,
                        position: ((20 + i * 90) as i32, 220),
                        is_ready: true,
                    };
                }
            }
        }

        Ok(Self {
            hero_entity,
            enemy_entity,
            battle_state: BattleState::new(),
            turn: BattleTurn::HeroTurn,
            turn_timer: Instant::now(),
            skill_buttons,
            selected_skill: None,
            current_animation: None,
            animation_timer: Instant::now(),
            damage_numbers: Vec::new(),
            battle_ended: false,
            victory: false,
        })
    }

    fn handle_hero_turn(&mut self, input: &InputEvent, hero: &mut Hero, enemy: &mut Enemy, game_data: &GameData) {
        // Check for basic attack button press
        if input.button == Button::A {
            self.execute_hero_attack(hero, enemy);
        }

        // Check for skill button presses
        for (i, button) in self.skill_buttons.iter().enumerate() {
            if button.is_ready && input.position_in_bounds(button.position, (80, 30)) {
                self.execute_hero_skill(i, hero, enemy, game_data);
                break;
            }
        }
    }

    fn execute_hero_attack(&mut self, hero: &mut Hero, enemy: &mut Enemy) {
        // Change state
        self.turn = BattleTurn::HeroAttack;
        self.animation_timer = Instant::now();
        self.current_animation = Some(AnimationState::HeroAttacking);

        // Play hero attack animation
        self.hero_entity.set_animation(AnimationType::Attack);

        // Calculate damage (will be applied after animation)
        let (damage, hit, critical) = hero_attack_enemy(hero, enemy, &mut self.battle_state);

        // Store damage for after animation
        self.pending_damage = Some(PendingDamage {
            amount: damage,
            hit,
            critical,
            target: DamageTarget::Enemy,
        });
    }

    fn execute_hero_skill(&mut self, skill_index: usize, hero: &mut Hero, enemy: &mut Enemy, game_data: &GameData) {
        if let Some(skill_id) = self.skill_buttons[skill_index].skill_id {
            if let Some(skill_data) = game_data.skills.get(&skill_id) {
                // Start skill use
                self.turn = BattleTurn::HeroAttack;
                self.animation_timer = Instant::now();
                self.current_animation = Some(AnimationState::SkillCasting {
                    skill_name: skill_data.name.clone(),
                });

                // Play animation based on skill type
                self.hero_entity.set_animation(AnimationType::Attack);

                // Apply skill effect
                match skill_data.skill_type {
                    SkillType::Attack => {
                        let base_damage = hero.attack * skill_data.power / 100;
                        let damage = base_damage.saturating_sub(enemy.def / 2);
                        enemy.current_hp = enemy.current_hp.saturating_sub(damage);

                        self.damage_numbers.push(DamageNumber::new(
                            damage,
                            enemy.position(),
                            false,  // Not a miss
                            false,  // Regular skill damage
                        ));
                    }
                    SkillType::Heal => {
                        let heal_amount = skill_data.power as i32;
                        hero.current_health = (hero.current_health + heal_amount).min(hero.max_health);

                        self.damage_numbers.push(DamageNumber::new_heal(
                            heal_amount as u32,
                            hero.position(),
                        ));
                    }
                    _ => {
                        // TODO: Implement Buff/Debuff skills
                    }
                }

                // Put skill on cooldown
                if let Some(active_skill) = hero.active_skills.get_mut(skill_index) {
                    active_skill.use_skill(skill_data.cooldown_seconds);
                    self.skill_buttons[skill_index].cooldown_remaining = skill_data.cooldown_seconds;
                    self.skill_buttons[skill_index].is_ready = false;
                }
            }
        }
    }

    fn handle_enemy_turn(&mut self, hero: &mut Hero, enemy: &mut Enemy) {
        // Simple AI: Always attack
        self.turn = BattleTurn::EnemyAttack;
        self.animation_timer = Instant::now();
        self.current_animation = Some(AnimationState::EnemyAttacking);

        // Play enemy attack animation
        self.enemy_entity.set_animation(AnimationType::Attack);

        // Calculate damage
        let (damage, hit, critical) = enemy_attack_hero(enemy, hero, &mut self.battle_state);

        // Store damage for after animation
        self.pending_damage = Some(PendingDamage {
            amount: damage,
            hit,
            critical,
            target: DamageTarget::Hero,
        });
    }

    fn update_animations(&mut self, delta_time: f32) {
        // Update sprite animations
        self.hero_entity.update();
        self.enemy_entity.update();

        // Update damage numbers
        self.damage_numbers.retain_mut(|dn| {
            dn.update(delta_time);
            !dn.is_finished()
        });

        // Update skill cooldowns
        for (i, button) in self.skill_buttons.iter_mut().enumerate() {
            if !button.is_ready {
                button.cooldown_remaining -= delta_time;
                if button.cooldown_remaining <= 0.0 {
                    button.cooldown_remaining = 0.0;
                    button.is_ready = true;
                }
            }
        }

        // Handle animation state transitions
        match self.turn {
            BattleTurn::HeroAttack => {
                if self.hero_entity.animation_finished() {
                    // Apply damage and transition to enemy hit state
                    if let Some(pending) = self.pending_damage.take() {
                        if pending.hit {
                            self.enemy_entity.set_animation(AnimationType::Attacked);
                            self.turn = BattleTurn::EnemyHit;
                            self.animation_timer = Instant::now();
                        } else {
                            // Miss - skip to enemy turn
                            self.turn = BattleTurn::EnemyTurn;
                        }
                    }
                }
            }
            BattleTurn::EnemyHit => {
                if self.enemy_entity.animation_finished() {
                    // Check if enemy died
                    if enemy.current_hp == 0 {
                        self.enemy_entity.set_animation(AnimationType::Death);
                        self.battle_ended = true;
                        self.victory = true;
                    } else {
                        self.turn = BattleTurn::EnemyTurn;
                    }
                }
            }
            BattleTurn::EnemyAttack => {
                if self.enemy_entity.animation_finished() {
                    // Apply damage and transition to hero hit state
                    if let Some(pending) = self.pending_damage.take() {
                        if pending.hit {
                            self.hero_entity.set_animation(AnimationType::Attacked);
                            self.turn = BattleTurn::HeroHit;
                            self.animation_timer = Instant::now();
                        } else {
                            // Miss - skip to hero turn
                            self.turn = BattleTurn::HeroTurn;
                        }
                    }
                }
            }
            BattleTurn::HeroHit => {
                if self.hero_entity.animation_finished() {
                    // Check if hero died
                    if hero.current_health <= 0 {
                        self.hero_entity.set_animation(AnimationType::Death);
                        self.battle_ended = true;
                        self.victory = false;
                    } else {
                        self.turn = BattleTurn::HeroTurn;
                    }
                }
            }
            BattleTurn::EnemyTurn => {
                // Small delay before enemy attacks
                if self.animation_timer.elapsed().as_millis() > 500 {
                    self.handle_enemy_turn(hero, enemy);
                }
            }
            _ => {}
        }
    }
}

impl Page for SemiActiveBattlePage {
    fn update(&mut self) -> bool {
        let delta_time = self.turn_timer.elapsed().as_secs_f32();
        self.turn_timer = Instant::now();

        self.update_animations(delta_time);

        // Handle input only during hero turn
        if self.turn == BattleTurn::HeroTurn {
            if let Some(input) = get_input() {
                self.handle_hero_turn(&input, hero, enemy, game_data);
            }
        }

        // Check if battle ended
        if self.battle_ended {
            // Transition to battle result page
            return false;
        }

        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw {
            display.clear()?;
        }

        // Draw background
        // ... background drawing ...

        // Draw battle entities
        self.hero_entity.draw(display)?;
        self.enemy_entity.draw(display)?;

        // Draw HP bars
        draw_hp_bar(display, hero, (10, 10))?;
        draw_hp_bar(display, enemy, (180, 10))?;

        // Draw damage numbers
        for damage_number in &mut self.damage_numbers {
            damage_number.draw(display)?;
        }

        // Draw skill buttons at bottom
        for button in &self.skill_buttons {
            if button.skill_id.is_some() {
                let color = if button.is_ready { GREEN } else { GRAY };
                draw_skill_button(display, button, color)?;

                // Draw cooldown overlay if not ready
                if !button.is_ready {
                    let cooldown_text = format!("{:.1}s", button.cooldown_remaining);
                    draw_text(display, &cooldown_text, button.position.0 + 10, button.position.1 + 10, RED)?;
                }
            }
        }

        // Draw turn indicator
        match self.turn {
            BattleTurn::HeroTurn => {
                draw_text(display, "YOUR TURN", 100, 200, GREEN)?;
            }
            BattleTurn::EnemyTurn | BattleTurn::EnemyAttack => {
                draw_text(display, "ENEMY TURN", 100, 200, RED)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn on_enter(&mut self) {
        // Initialize hero battle skills
        // hero.initialize_battle_skills();
    }

    // ... other Page trait methods ...
}
```

### 2.5 Data Loading Updates

#### 2.5.1 Update `src/game/data_loader.rs`
```rust
pub struct GameData {
    pub maps: HashMap<u32, MapData>,
    pub enemies: HashMap<u32, EnemyData>,
    pub exp_table: HashMap<u32, u32>,
    pub quests: HashMap<u32, QuestData>,

    // NEW FIELD
    pub skills: HashMap<u32, SkillData>,
}

impl GameData {
    pub fn load_from_assets() -> Result<Self, Box<dyn Error>> {
        // ... existing loading code ...

        // Load skills
        let skills_json = include_str!("../../assets/data/skills.json");
        let skills_vec: Vec<SkillData> = serde_json::from_str(skills_json)?;
        let skills = skills_vec.into_iter()
            .map(|s| (s.id, s))
            .collect();

        Ok(Self {
            maps,
            enemies,
            exp_table,
            quests,
            skills,
        })
    }
}
```

---

## Feature 3: Skills from Cards

### 3.1 Card Data Structure Updates

#### 3.1.1 Modify `assets/data/enemies.json`
Add skill unlock to card definitions:
```json
{
  "id": 1002,
  "name": "Poring",
  "level": 1,
  "hp": 50,
  "attack": 7,
  "defense": 0,
  "hit": 22,
  "flee": 82,
  "base_exp": 150,
  "element": "water",
  "monster_type": "normal",
  "drop_rate": 0.005,
  "card": {
    "name": "Poring Card",
    "rarity": 1,
    "atk_bonus": 5,
    "def_bonus": 0,
    "unlocks_skill": 3
  }
}
```

```json
{
  "id": 1051,
  "name": "Thief Bug",
  "level": 10,
  "hp": 200,
  "attack": 30,
  "defense": 5,
  "hit": 50,
  "flee": 100,
  "base_exp": 500,
  "element": "earth",
  "monster_type": "normal",
  "drop_rate": 0.008,
  "card": {
    "name": "Thief Bug Card",
    "rarity": 2,
    "atk_bonus": 15,
    "def_bonus": 5,
    "unlocks_skill": 1
  }
}
```

#### 3.1.2 Update `src/game/expedition.rs`
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub monster_id: u32,
    pub name: String,
    pub rarity: u8,
    pub atk_bonus: u32,
    pub def_bonus: u32,

    // NEW FIELD
    pub unlocks_skill: Option<u32>,  // Skill ID if this card unlocks a skill
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardData {
    pub name: String,
    pub rarity: u8,
    pub atk_bonus: u32,
    pub def_bonus: u32,
    pub unlocks_skill: Option<u32>,
}
```

### 3.2 Card Skill Selection System

#### 3.2.1 Create `src/ui/pages/skill_selection.rs`
```rust
use crate::game::hero::Hero;
use crate::game::skill::SkillData;
use crate::game::expedition::Card;

pub struct SkillSelectionPage {
    // Current equipment state
    equipped_card_slots: [Option<CardSlot>; 3],

    // Available cards with skills
    available_skill_cards: Vec<CardWithSkill>,

    // UI state
    selected_slot: Option<usize>,  // Which slot is being edited
    scroll_offset: usize,
    cursor_position: usize,

    needs_redraw: bool,
}

#[derive(Clone, Debug)]
struct CardSlot {
    card: Card,
    skill_data: SkillData,
}

#[derive(Clone, Debug)]
struct CardWithSkill {
    card: Card,
    skill_data: SkillData,
    is_equipped: bool,
}

impl SkillSelectionPage {
    pub fn new(hero: &Hero, game_data: &GameData) -> Result<Self, Box<dyn Error>> {
        // Load currently equipped skills
        let mut equipped_card_slots = [None, None, None];

        // Populate available skill cards from hero's collection
        let mut available_skill_cards = Vec::new();

        for card in &hero.cards {
            if let Some(skill_id) = card.unlocks_skill {
                if let Some(skill_data) = game_data.skills.get(&skill_id) {
                    available_skill_cards.push(CardWithSkill {
                        card: card.clone(),
                        skill_data: skill_data.clone(),
                        is_equipped: false,
                    });
                }
            }
        }

        Ok(Self {
            equipped_card_slots,
            available_skill_cards,
            selected_slot: None,
            scroll_offset: 0,
            cursor_position: 0,
            needs_redraw: true,
        })
    }

    fn handle_input(&mut self, input: &InputEvent) {
        match self.selected_slot {
            None => {
                // Navigating equipment slots
                match input.button {
                    Button::Left => {
                        if self.cursor_position > 0 {
                            self.cursor_position -= 1;
                            self.needs_redraw = true;
                        }
                    }
                    Button::Right => {
                        if self.cursor_position < 2 {
                            self.cursor_position += 1;
                            self.needs_redraw = true;
                        }
                    }
                    Button::A => {
                        // Select this slot to change
                        self.selected_slot = Some(self.cursor_position);
                        self.cursor_position = 0;
                        self.needs_redraw = true;
                    }
                    Button::B => {
                        // Exit page
                        // Save changes to hero
                        // ... transition back to menu ...
                    }
                    _ => {}
                }
            }
            Some(slot_index) => {
                // Selecting a card for the slot
                match input.button {
                    Button::Up => {
                        if self.cursor_position > 0 {
                            self.cursor_position -= 1;
                            if self.cursor_position < self.scroll_offset {
                                self.scroll_offset = self.cursor_position;
                            }
                            self.needs_redraw = true;
                        }
                    }
                    Button::Down => {
                        if self.cursor_position < self.available_skill_cards.len() - 1 {
                            self.cursor_position += 1;
                            if self.cursor_position >= self.scroll_offset + 5 {
                                self.scroll_offset = self.cursor_position - 4;
                            }
                            self.needs_redraw = true;
                        }
                    }
                    Button::A => {
                        // Equip selected card to slot
                        if let Some(card_with_skill) = self.available_skill_cards.get(self.cursor_position) {
                            // Check if already equipped in another slot
                            let already_equipped = self.equipped_card_slots.iter()
                                .any(|slot| {
                                    if let Some(equipped) = slot {
                                        equipped.card.monster_id == card_with_skill.card.monster_id
                                    } else {
                                        false
                                    }
                                });

                            if !already_equipped {
                                self.equipped_card_slots[slot_index] = Some(CardSlot {
                                    card: card_with_skill.card.clone(),
                                    skill_data: card_with_skill.skill_data.clone(),
                                });
                            }
                        }

                        // Return to slot selection
                        self.selected_slot = None;
                        self.cursor_position = slot_index;
                        self.needs_redraw = true;
                    }
                    Button::B => {
                        // Cancel selection, return to slot view
                        self.selected_slot = None;
                        self.cursor_position = slot_index;
                        self.needs_redraw = true;
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Page for SkillSelectionPage {
    fn update(&mut self) -> bool {
        if let Some(input) = get_input() {
            self.handle_input(&input);
        }

        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_redraw {
            display.clear()?;

            // Title
            draw_text(display, "SKILL CARD EQUIPMENT", 10, 10, WHITE)?;

            match self.selected_slot {
                None => {
                    // Draw equipment slots (3 boxes side by side)
                    for i in 0..3 {
                        let x = 20 + (i * 90);
                        let y = 40;
                        let is_selected = i == self.cursor_position;

                        // Draw slot box
                        let color = if is_selected { YELLOW } else { WHITE };
                        draw_rectangle(display, x, y, 80, 100, color)?;

                        // Draw slot number
                        draw_text(display, &format!("Slot {}", i + 1), x + 5, y + 5, WHITE)?;

                        // Draw equipped card if any
                        if let Some(card_slot) = &self.equipped_card_slots[i] {
                            draw_text(display, &card_slot.card.name, x + 5, y + 25, WHITE)?;
                            draw_text(display, &card_slot.skill_data.name, x + 5, y + 45, GREEN)?;

                            // Draw card icon
                            draw_card_icon(display, &card_slot.card, x + 30, y + 65)?;
                        } else {
                            draw_text(display, "Empty", x + 20, y + 50, GRAY)?;
                        }
                    }

                    // Instructions
                    draw_text(display, "Left/Right: Select slot", 10, 160, WHITE)?;
                    draw_text(display, "A: Change card  B: Exit", 10, 180, WHITE)?;
                }
                Some(slot_index) => {
                    // Draw card selection list
                    draw_text(display, &format!("Select card for Slot {}", slot_index + 1), 10, 30, WHITE)?;

                    let visible_count = 5;
                    let start = self.scroll_offset;
                    let end = (start + visible_count).min(self.available_skill_cards.len());

                    for (i, card_with_skill) in self.available_skill_cards[start..end].iter().enumerate() {
                        let y = 50 + (i * 30);
                        let is_selected = (start + i) == self.cursor_position;
                        let is_equipped = self.equipped_card_slots.iter().any(|slot| {
                            if let Some(equipped) = slot {
                                equipped.card.monster_id == card_with_skill.card.monster_id
                            } else {
                                false
                            }
                        });

                        let bg_color = if is_selected {
                            YELLOW
                        } else if is_equipped {
                            DARK_GRAY
                        } else {
                            BLACK
                        };

                        // Draw selection background
                        draw_filled_rectangle(display, 5, y - 2, 310, 28, bg_color)?;

                        // Draw card name and skill
                        draw_text(display, &card_with_skill.card.name, 10, y, WHITE)?;
                        draw_text(display, &format!("-> {}", card_with_skill.skill_data.name), 10, y + 12, GREEN)?;

                        // Draw "EQUIPPED" badge if applicable
                        if is_equipped {
                            draw_text(display, "[EQUIPPED]", 200, y + 6, RED)?;
                        }
                    }

                    // Scroll indicator
                    if self.available_skill_cards.len() > visible_count {
                        let scroll_text = format!("{}/{}", start + 1, self.available_skill_cards.len());
                        draw_text(display, &scroll_text, 270, 30, GRAY)?;
                    }

                    // Instructions
                    draw_text(display, "Up/Down: Navigate", 10, 200, WHITE)?;
                    draw_text(display, "A: Equip  B: Cancel", 10, 220, WHITE)?;
                }
            }

            self.needs_redraw = false;
        }

        Ok(())
    }

    fn on_exit(&mut self) {
        // Save equipped skills back to hero
        // for (i, slot) in self.equipped_card_slots.iter().enumerate() {
        //     if let Some(card_slot) = slot {
        //         hero.equip_skill(i, card_slot.skill_data.id);
        //     } else {
        //         hero.unequip_skill(i);
        //     }
        // }
    }

    // ... other Page trait methods ...
}
```

### 3.3 Menu Integration

#### 3.3.1 Update `src/ui/pages/menu.rs`
Add new menu option "Skill Setup" that opens the skill selection page:
```rust
pub enum MenuOption {
    Continue,
    HeroInfo,
    Cards,
    SkillSetup,  // NEW
    Quests,
    Expeditions,
    Save,
    Exit,
}

// In draw method, add new option
fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
    // ... existing menu drawing ...

    draw_menu_option(display, "Skill Setup", y_offset, is_selected)?;

    // ... rest of options ...
}

// In update method, handle selection
fn update(&mut self) -> bool {
    match selected_option {
        MenuOption::SkillSetup => {
            // Transition to skill selection page
            let skill_page = SkillSelectionPage::new(&hero, &game_data)?;
            game_manager.set_page(Box::new(skill_page));
        }
        // ... other options ...
    }
}
```

---

## Implementation Order

### Phase 1: Foundation (Skills System)
1. Create `src/game/skill.rs` with skill data structures
2. Create `assets/data/skills.json` with initial skill definitions
3. Update `src/game/data_loader.rs` to load skills
4. Update `src/game/hero.rs` with skill slot system
5. Update save system version to 6

**Testing:** Verify skills load correctly, hero can equip/unequip skills

### Phase 2: Semi-Active Battle Mode
1. Create `src/game/battle_mode.rs` enum
2. Create `src/ui/pages/semi_active_battle.rs` with turn-based logic
3. Implement skill cooldown system
4. Create skill button UI components
5. Implement battle animations with turn transitions
6. Add skill damage calculation
7. Wire battle mode selection in map page

**Testing:** Complete a battle using all 3 equipped skills, verify cooldowns work, confirm animations play correctly

### Phase 3: Card Skill Unlocks
1. Update `src/game/expedition.rs` Card structure with `unlocks_skill` field
2. Update `assets/data/enemies.json` to add skill unlocks to cards
3. Create `src/ui/pages/skill_selection.rs`
4. Add skill selection page to menu navigation
5. Implement card equipping logic
6. Integrate equipped skills into semi-active battle

**Testing:** Unlock cards, verify skills appear in selection page, equip skills, use in battle

### Phase 4: Mini MVP/MVP System
1. Create `src/game/mvp_spawn_manager.rs`
2. Update `src/game/enemy.rs` with MonsterType enum
3. Add MVP monster definitions to `assets/data/enemies.json`
4. Update save system to include MvpSpawnManager
5. Update `src/ui/pages/map.rs` to show MVP button
6. Implement spawn timer logic
7. Connect MVP battles to semi-active battle mode
8. Update battle result page to record MVP kills

**Testing:** Defeat MVP, verify respawn timer, wait for respawn, fight again

### Phase 5: Polish & Integration
1. Add visual indicators for Mini MVP vs MVP (crown icons, special colors)
2. Create sprite animations for skills
3. Add sound effects (if audio system exists)
4. Balance skill power values
5. Balance MVP health/damage values
6. Add achievement tracking for MVP kills
7. Add quest support for MVP hunting
8. Update card collection page to show skill unlocks

**Testing:** Full playthrough, test all features together

---

## Asset Requirements

### Sprite Animations Needed

#### Skills (optional, can use generic attack animation initially)
- `assets/sprites/skills/bash.gif`
- `assets/sprites/skills/heal.gif`
- `assets/sprites/skills/fire_bolt.gif`
- `assets/sprites/skills/power_strike.gif`

#### Mini MVP Sprites
- `assets/sprites/enemies/1096_idle.gif` (Angeling)
- `assets/sprites/enemies/1096_attack.gif`
- `assets/sprites/enemies/1096_hit.gif`
- `assets/sprites/enemies/1096_death.gif`

#### MVP Sprites
- `assets/sprites/enemies/1112_idle.gif` (Drake)
- `assets/sprites/enemies/1112_attack.gif`
- `assets/sprites/enemies/1112_hit.gif`
- `assets/sprites/enemies/1112_death.gif`

### UI Assets
- Crown icon for MVP designation (can use text initially)
- Skill cooldown overlay effect
- Card slot background images
- Skill button backgrounds

---

## Technical Considerations

### Performance
- Sprite animations are already optimized with caching
- Skill cooldown updates are lightweight
- MVP spawn checks happen only on map page render

### Memory
- Skill data loaded once at startup
- Active battle state only during combat
- Spawn manager state is minimal (HashMap of timestamps)

### Save Compatibility
- Increment save version to 6
- Add migration path from version 5 (initialize empty skill slots and spawn manager)
- Existing saves will work with default values

### Future Extensions
- Add more skills (20+ total)
- Implement buff/debuff system
- Add skill leveling system
- Add mini-boss category between normal and mini MVP
- Implement MVP death announcements
- Add MVP leaderboards
- Create skill combo system
- Add passive skill tree

---

## Testing Checklist

### Skills System
- [ ] Skills load from JSON
- [ ] Hero can equip 3 skills
- [ ] Hero can unequip skills
- [ ] Skills persist in save data
- [ ] Skill data accessible in battle

### Semi-Active Battle
- [ ] Battle starts in hero turn
- [ ] Hero can basic attack
- [ ] Hero can use skill 1
- [ ] Hero can use skill 2
- [ ] Hero can use skill 3
- [ ] Cooldowns prevent spamming
- [ ] Cooldowns count down correctly
- [ ] Attack animations play
- [ ] Hit animations play
- [ ] Damage numbers display
- [ ] Turn alternates correctly
- [ ] Battle ends on victory
- [ ] Battle ends on defeat
- [ ] Experience awarded
- [ ] Card drops work

### Card Skills
- [ ] Cards with skills show unlock indicator
- [ ] Skill selection page opens from menu
- [ ] Available skill cards display
- [ ] Can equip card to slot 1
- [ ] Can equip card to slot 2
- [ ] Can equip card to slot 3
- [ ] Cannot equip same card twice
- [ ] Equipped skills show in battle
- [ ] Unequipping works
- [ ] Changes persist

### MVP System
- [ ] MVP data loads from JSON
- [ ] Spawn manager initializes
- [ ] MVP button shows on correct map
- [ ] MVP button hidden on wrong map
- [ ] Timer displays correctly
- [ ] Cannot fight when on cooldown
- [ ] Can fight when available
- [ ] MVP battle uses semi-active mode
- [ ] Defeating MVP triggers respawn timer
- [ ] Timer persists across game sessions
- [ ] Mini MVP 20-minute timer works
- [ ] MVP 3-hour timer works
- [ ] Multiple MVPs tracked independently

---

## Data Examples

### Complete Enemy Entry (Mini MVP)
```json
{
  "id": 1096,
  "name": "Angeling",
  "level": 77,
  "hp": 23000,
  "attack": 250,
  "defense": 72,
  "hit": 200,
  "flee": 200,
  "base_exp": 50000,
  "element": "holy",
  "str_stat": 1,
  "agi": 50,
  "int": 77,
  "dex": 100,
  "vit": 50,
  "luk": 200,
  "monster_type": "mini_mvp",
  "spawn_timer_minutes": 20,
  "spawn_map_id": 10,
  "drop_rate": 0.01,
  "card": {
    "name": "Angeling Card",
    "rarity": 5,
    "atk_bonus": 100,
    "def_bonus": 100,
    "unlocks_skill": null
  }
}
```

### Complete Enemy Entry (MVP)
```json
{
  "id": 1112,
  "name": "Drake",
  "level": 91,
  "hp": 150000,
  "attack": 500,
  "defense": 100,
  "hit": 250,
  "flee": 150,
  "base_exp": 250000,
  "element": "wind",
  "str_stat": 80,
  "agi": 50,
  "int": 25,
  "dex": 100,
  "vit": 100,
  "luk": 50,
  "monster_type": "mvp",
  "spawn_timer_minutes": 180,
  "spawn_map_id": 15,
  "drop_rate": 0.02,
  "card": {
    "name": "Drake Card",
    "rarity": 5,
    "atk_bonus": 200,
    "def_bonus": 150,
    "unlocks_skill": 5
  }
}
```

### Complete Enemy Entry (Normal with Skill Card)
```json
{
  "id": 1002,
  "name": "Poring",
  "level": 1,
  "hp": 50,
  "attack": 7,
  "defense": 0,
  "hit": 22,
  "flee": 82,
  "base_exp": 150,
  "element": "water",
  "str_stat": 1,
  "agi": 1,
  "int": 0,
  "dex": 6,
  "vit": 1,
  "luk": 30,
  "monster_type": "normal",
  "spawn_timer_minutes": null,
  "spawn_map_id": null,
  "drop_rate": 0.005,
  "card": {
    "name": "Poring Card",
    "rarity": 1,
    "atk_bonus": 5,
    "def_bonus": 0,
    "unlocks_skill": 3
  }
}
```

---

## Estimated Complexity

### Low Complexity
- Skill data structures
- Card skill unlock field
- MVP spawn manager
- Skill selection page UI

### Medium Complexity
- Semi-active battle page
- Turn-based animation system
- Skill cooldown tracking
- Map page MVP button integration

### High Complexity
- Battle animation state machine
- Skill effect application
- Integration of all systems
- Balancing MVP difficulty

---

## Success Criteria

### Minimum Viable Product
1. Hero can equip 3 skills from cards
2. Semi-active battle mode functional with skills
3. At least 1 Mini MVP and 1 MVP implemented
4. Spawn timers work correctly
5. Skills have cooldowns in battle

### Full Feature Set
1. All planned skills implemented (5+)
2. Multiple Mini MVPs and MVPs (3+ each)
3. Skill animations play correctly
4. All UI pages fully functional
5. Save/load preserves all state
6. Battle mode selection works
7. MVP battles feel epic and rewarding

### Polish
1. Visual feedback for all actions
2. Damage numbers with colors
3. Skill effects have unique visuals
4. MVP monsters have distinct appearance
5. Smooth transitions between states
6. Helpful UI tooltips
7. Achievement tracking

---

## End of Implementation Plan
