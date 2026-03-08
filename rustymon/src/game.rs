use bevy_ecs::prelude::*;
use std::time::Instant;

// ─── Hardware RNG ─────────────────────────────────────────────────────────────

fn random_u32() -> u32 {
    extern "C" {
        fn esp_random() -> u32;
    }
    unsafe { esp_random() }
}

fn calc_damage(atk: u16, def: u16) -> u16 {
    let base = (atk as i32 - def as i32 / 2).max(1) as u32;
    let variance = 80 + (random_u32() % 41); // 80 – 120 %
    ((base * variance) / 100).max(1) as u16
}

// ─── ECS Components ───────────────────────────────────────────────────────────
// Each component is a thin newtype or plain struct so it stays Copy / Clone and
// can be queried individually with minimal overhead.

#[derive(Component, Clone, Copy)]
pub struct MonName(pub &'static str);

#[derive(Component, Clone, Copy)]
pub struct Level(pub u8);

#[derive(Component, Clone, Copy)]
pub struct Stats {
    pub atk: u16,
    pub def: u16,
}

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub hp: u16,
    pub max_hp: u16,
}

impl Health {
    pub fn is_fainted(self) -> bool {
        self.hp == 0
    }
}

#[derive(Component, Clone, Copy)]
pub struct Exp {
    pub current: u32,
    pub next: u32,
}

/// The position of this entity inside the player's roster (0-based).
#[derive(Component, Clone, Copy)]
pub struct RosterSlot(pub usize);

// ─── ECS Resources ────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Default)]
pub enum Screen {
    #[default]
    Overview,
    Roster,
    Battle,
}

#[derive(PartialEq, Clone, Default)]
pub enum MenuCursor {
    #[default]
    Battle,
    Roster,
}

#[derive(Resource, Default)]
pub struct CurrentScreen(pub Screen);

#[derive(Resource, Default)]
pub struct MenuCursorRes(pub MenuCursor);

/// Index into the sorted roster that is currently the "active" (lead) monster.
#[derive(Resource, Default)]
pub struct ActiveSlot(pub usize);

/// Pre-computed battle result and animation state.
#[derive(Resource, Default)]
pub struct BattleData {
    pub result: Option<BattleResult>,
    pub lines_shown: usize,
    pub last_tick: Option<Instant>,
}

impl BattleData {
    pub fn is_done(&self) -> bool {
        self.result.as_ref().map_or(true, |r| self.lines_shown >= r.log.len())
    }
}

/// Stable entity handles for the roster, sorted by slot index.
/// Main inserts this after spawning so that render can query by entity ID.
#[derive(Resource)]
pub struct RosterEntities(pub Vec<Entity>);

// ─── Input abstraction ────────────────────────────────────────────────────────
// Raw hardware events (touch / button) are translated to these semantic events
// in main.rs before being pushed into the queue.

#[derive(Clone)]
pub enum InputEvent {
    /// Cycle the highlighted button left/right.
    ToggleCursor,
    /// Explicitly move the cursor to the Battle button.
    CursorToBattle,
    /// Explicitly move the cursor to the Roster button.
    CursorToRoster,
    /// Confirm the currently highlighted button (swipe-up / long-press).
    Confirm,
    /// Tap directly on the Battle button area.
    SelectBattle,
    /// Tap directly on the Roster button area.
    SelectRoster,
    /// Go back to Overview (any input on Roster / finished Battle).
    Back,
}

/// Per-frame input queue filled by main before each schedule tick.
#[derive(Resource, Default)]
pub struct InputQueue(pub Vec<InputEvent>);

// ─── Pure battle data ─────────────────────────────────────────────────────────

pub struct BattleResult {
    pub log: Vec<String>,
    pub player_won: bool,
    pub player_final_hp: u16,
    pub exp_gained: u32,
}

/// A transient snapshot used only inside `simulate_battle`.
struct RustymonSnapshot {
    name: &'static str,
    level: u8,
    atk: u16,
    def: u16,
    hp: u16,
    max_hp: u16,
}

fn simulate_battle(
    p_name: &'static str, p_level: u8, p_atk: u16, p_def: u16, p_hp: u16, p_max_hp: u16,
    enemy: RustymonSnapshot,
) -> BattleResult {
    let mut p = RustymonSnapshot { name: p_name, level: p_level, atk: p_atk, def: p_def, hp: p_hp, max_hp: p_max_hp };
    let mut e = enemy;
    let mut log: Vec<String> = Vec::new();

    log.push(format!("VS {} begins!", e.name));
    log.push(format!("You: {} HP:{} ATK:{} DEF:{}", p.name, p.hp, p.atk, p.def));
    log.push(format!("Foe: {} HP:{} ATK:{} DEF:{}", e.name, e.hp, e.atk, e.def));

    for turn in 1u32.. {
        log.push(format!("--- Turn {} ---", turn));

        let p_dmg = calc_damage(p.atk, e.def);
        e.hp = e.hp.saturating_sub(p_dmg);
        log.push(format!("{} -> {} dmg", p.name, p_dmg));
        log.push(format!("Foe HP: {}/{}", e.hp, e.max_hp));

        if e.hp == 0 {
            log.push(format!("{} fainted!", e.name));
            log.push("*** YOU WIN! ***".to_string());
            return BattleResult {
                log,
                player_won: true,
                player_final_hp: p.hp,
                exp_gained: 40 + (e.level as u32) * 10,
            };
        }

        let e_dmg = calc_damage(e.atk, p.def);
        p.hp = p.hp.saturating_sub(e_dmg);
        log.push(format!("{} -> {} dmg", e.name, e_dmg));
        log.push(format!("Your HP: {}/{}", p.hp, p.max_hp));

        if p.hp == 0 {
            log.push(format!("{} fainted!", p.name));
            log.push("*** YOU LOSE... ***".to_string());
            return BattleResult {
                log,
                player_won: false,
                player_final_hp: 0,
                exp_gained: 10,
            };
        }

        if turn >= 50 {
            log.push("Time ran out! Draw.".to_string());
            return BattleResult {
                log,
                player_won: false,
                player_final_hp: p.hp,
                exp_gained: 5,
            };
        }
    }
    unreachable!()
}

fn random_enemy() -> RustymonSnapshot {
    let table: [RustymonSnapshot; 4] = [
        RustymonSnapshot { name: "Toxibolt",  level: 4, atk: 22, def: 14, hp: 40, max_hp: 40 },
        RustymonSnapshot { name: "Glitchrat", level: 3, atk: 18, def: 12, hp: 32, max_hp: 32 },
        RustymonSnapshot { name: "Ironclad",  level: 5, atk: 20, def: 22, hp: 48, max_hp: 48 },
        RustymonSnapshot { name: "Virebug",   level: 4, atk: 24, def: 16, hp: 38, max_hp: 38 },
    ];
    let idx = (random_u32() % 4) as usize;
    // Safe: copy each field individually (no Clone on the array)
    let e = &table[idx];
    RustymonSnapshot { name: e.name, level: e.level, atk: e.atk, def: e.def, hp: e.hp, max_hp: e.max_hp }
}

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Drains the `InputQueue` and updates navigation / battle-start state.
pub fn navigation_system(
    mut input_queue: ResMut<InputQueue>,
    mut screen: ResMut<CurrentScreen>,
    mut cursor: ResMut<MenuCursorRes>,
    mut battle_data: ResMut<BattleData>,
    active_slot: Res<ActiveSlot>,
    mut monsters: Query<(&MonName, &RosterSlot, &mut Level, &mut Stats, &mut Health, &mut Exp)>,
) {
    for event in input_queue.0.drain(..) {
        match screen.0 {
            Screen::Overview => handle_overview_event(
                event,
                &mut screen,
                &mut cursor,
                &mut battle_data,
                active_slot.0,
                &mut monsters,
            ),
            Screen::Roster => {
                screen.0 = Screen::Overview;
            }
            Screen::Battle => {
                if battle_data.is_done() {
                    screen.0 = Screen::Overview;
                }
            }
        }
    }
}

fn handle_overview_event(
    event: InputEvent,
    screen: &mut ResMut<CurrentScreen>,
    cursor: &mut ResMut<MenuCursorRes>,
    battle_data: &mut ResMut<BattleData>,
    active_slot: usize,
    monsters: &mut Query<(&MonName, &RosterSlot, &mut Level, &mut Stats, &mut Health, &mut Exp)>,
) {
    match event {
        InputEvent::ToggleCursor => {
            cursor.0 = match cursor.0 {
                MenuCursor::Battle => MenuCursor::Roster,
                MenuCursor::Roster => MenuCursor::Battle,
            };
        }
        InputEvent::CursorToBattle => cursor.0 = MenuCursor::Battle,
        InputEvent::CursorToRoster => cursor.0 = MenuCursor::Roster,
        InputEvent::Confirm => match cursor.0 {
            MenuCursor::Battle => try_start_battle(screen, battle_data, active_slot, monsters),
            MenuCursor::Roster => screen.0 = Screen::Roster,
        },
        InputEvent::SelectBattle => {
            cursor.0 = MenuCursor::Battle;
            try_start_battle(screen, battle_data, active_slot, monsters);
        }
        InputEvent::SelectRoster => {
            cursor.0 = MenuCursor::Roster;
            screen.0 = Screen::Roster;
        }
        InputEvent::Back => {} // no-op on Overview
    }
}

fn try_start_battle(
    screen: &mut ResMut<CurrentScreen>,
    battle_data: &mut ResMut<BattleData>,
    active_slot: usize,
    monsters: &mut Query<(&MonName, &RosterSlot, &mut Level, &mut Stats, &mut Health, &mut Exp)>,
) {
    for (name, slot, mut level, mut stats, mut health, mut exp) in monsters.iter_mut() {
        if slot.0 != active_slot {
            continue;
        }
        if health.is_fainted() {
            return; // cannot battle with a fainted rustymon
        }

        let enemy = random_enemy();
        let result = simulate_battle(
            name.0, level.0, stats.atk, stats.def, health.hp, health.max_hp,
            enemy,
        );

        // Apply result to the monster's components
        health.hp = result.player_final_hp;
        exp.current += result.exp_gained;

        // Level-up loop
        while exp.current >= exp.next {
            exp.current -= exp.next;
            level.0 += 1;
            exp.next = (level.0 as u32 + 1) * 100;
            stats.atk += 2;
            stats.def += 1;
            health.max_hp += 5;
            if result.player_won {
                health.hp = health.max_hp; // full heal on level-up
            }
        }

        battle_data.result = Some(result);
        battle_data.lines_shown = 0;
        battle_data.last_tick = None;
        screen.0 = Screen::Battle;
        return;
    }
}

/// Advances the battle log animation one line at a time.
pub fn battle_advance_system(
    mut battle_data: ResMut<BattleData>,
    screen: Res<CurrentScreen>,
) {
    if screen.0 != Screen::Battle {
        return;
    }
    let total = match &battle_data.result {
        Some(r) => r.log.len(),
        None => return,
    };
    if battle_data.lines_shown >= total {
        return;
    }

    const LINE_DELAY_MS: u128 = 550;

    match battle_data.last_tick {
        None => battle_data.last_tick = Some(Instant::now()),
        Some(t) => {
            if t.elapsed().as_millis() >= LINE_DELAY_MS {
                battle_data.lines_shown += 1;
                battle_data.last_tick = Some(Instant::now());
            }
        }
    }
}

// ─── World bootstrap ──────────────────────────────────────────────────────────

/// Spawn all starter rustymon entities and insert every resource.
/// Returns the world and the sorted list of spawned entity IDs.
pub fn setup_world() -> World {
    let mut world = World::new();

    let e0 = world.spawn((
        MonName("Ferrobit"),
        Level(5),
        Stats { atk: 25, def: 20 },
        Health { hp: 50, max_hp: 50 },
        Exp { current: 0, next: 600 },
        RosterSlot(0),
    )).id();

    let e1 = world.spawn((
        MonName("Blazerust"),
        Level(3),
        Stats { atk: 32, def: 10 },
        Health { hp: 35, max_hp: 35 },
        Exp { current: 0, next: 400 },
        RosterSlot(1),
    )).id();

    let e2 = world.spawn((
        MonName("Aquabyte"),
        Level(4),
        Stats { atk: 20, def: 18 },
        Health { hp: 45, max_hp: 45 },
        Exp { current: 0, next: 500 },
        RosterSlot(2),
    )).id();

    world.insert_resource(CurrentScreen::default());
    world.insert_resource(MenuCursorRes::default());
    world.insert_resource(ActiveSlot::default());
    world.insert_resource(BattleData::default());
    world.insert_resource(InputQueue::default());
    // Sorted by slot so render can iterate in order
    world.insert_resource(RosterEntities(vec![e0, e1, e2]));

    world
}

/// Build the per-frame schedule: navigation first, then battle animation.
pub fn build_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems((
        navigation_system,
        battle_advance_system.after(navigation_system),
    ));
    schedule
}
