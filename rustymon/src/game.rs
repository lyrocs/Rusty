use bevy_ecs::prelude::*;
use serde::Deserialize;
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
    let variance = 80 + (random_u32() % 41); // 80–120 %
    ((base * variance) / 100).max(1) as u16
}

// ─── ECS Components ───────────────────────────────────────────────────────────

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

#[derive(Component, Clone, Copy)]
pub struct RosterSlot(pub usize);

/// How many times this monster has been caught as a duplicate (0 = first catch).
/// Capped at 10; each increment gives +5% atk/def bonus.
#[derive(Component, Clone, Copy)]
pub struct Count(pub u8);

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

#[derive(Resource, Default)]
pub struct ActiveSlot(pub usize);

/// First visible card index on the Roster screen.
#[derive(Resource, Default)]
pub struct RosterScroll(pub usize);

/// Slot index of the card the user has tapped once (highlighted, not yet confirmed).
#[derive(Resource, Default)]
pub struct RosterHover(pub Option<usize>);

#[derive(Resource)]
pub struct RosterEntities(pub Vec<Entity>);

// ─── Capture queue ────────────────────────────────────────────────────────────
// Set by tap_battle_update_system on victory; consumed by main.rs to spawn the
// new entity (direct World access is easier there than Commands in a system).

pub struct CapturedMonster {
    pub name: &'static str,
    pub level: u8,
    pub atk: u16,
    pub def: u16,
    pub hp: u16,
}

#[derive(Resource, Default)]
pub struct PendingCapture(pub Option<CapturedMonster>);

// ─── Tap Battle ───────────────────────────────────────────────────────────────

/// Circle kind: green means "tap to attack", red means "block or take damage".
#[derive(Clone, Copy, PartialEq)]
pub enum CircleKind {
    HeroAttack,  // player taps it → damage to enemy
    EnemyAttack, // expires without tap → damage to player
}

/// A shrinking circle on-screen. Radius goes from max_radius → 0 over lifetime_ms.
#[derive(Clone, Copy)]
pub struct TapCircle {
    pub cx: u16,
    pub cy: u16,
    pub max_radius: u16,
    pub kind: CircleKind,
    pub spawned_at: Instant,
    pub lifetime_ms: u32,
}

impl TapCircle {
    pub fn current_radius(&self) -> u16 {
        let elapsed = self.spawned_at.elapsed().as_millis() as u32;
        if elapsed >= self.lifetime_ms {
            return 0;
        }
        let remaining = self.lifetime_ms - elapsed;
        ((self.max_radius as u32 * remaining) / self.lifetime_ms) as u16
    }

    pub fn is_expired(&self) -> bool {
        self.spawned_at.elapsed().as_millis() as u32 >= self.lifetime_ms
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        // Add a generous touch tolerance so finger imprecision doesn't cause
        // misses, and enforce a minimum tappable radius so late-stage circles
        // (which have nearly vanished) can still be hit.
        const HIT_TOLERANCE: i32 = 12;
        const MIN_HIT_RADIUS: i32 = 16;
        let r = (self.current_radius() as i32 + HIT_TOLERANCE).max(MIN_HIT_RADIUS);
        let dx = x as i32 - self.cx as i32;
        let dy = y as i32 - self.cy as i32;
        dx * dx + dy * dy <= r * r
    }
}

/// Live tap-battle state. Replaces the old pre-computed BattleResult.
#[derive(Resource)]
pub struct TapBattleState {
    pub active: bool,
    pub entity_slot: usize,
    // Player snapshot
    pub player_name: &'static str,
    pub player_level: u8,
    pub player_atk: u16,
    pub player_def: u16,
    pub player_hp: u16,
    pub player_max_hp: u16,
    // Enemy
    pub enemy_name: &'static str,
    pub enemy_level: u8,
    pub enemy_atk: u16,
    pub enemy_def: u16,
    pub enemy_hp: u16,
    pub enemy_max_hp: u16,
    // Game state
    pub circles: Vec<TapCircle>,
    pub last_spawn: Option<Instant>,
    pub outcome: Option<bool>,         // Some(true)=won, Some(false)=lost
    pub outcome_time: Option<Instant>, // when outcome was set (for cooldown)
    pub captured: bool,                // did the defeated enemy join the roster?
    pub exp_gained: u32,
}

impl Default for TapBattleState {
    fn default() -> Self {
        Self {
            active: false,
            entity_slot: 0,
            player_name: "",
            player_level: 1,
            player_atk: 10,
            player_def: 10,
            player_hp: 50,
            player_max_hp: 50,
            enemy_name: "",
            enemy_level: 1,
            enemy_atk: 10,
            enemy_def: 10,
            enemy_hp: 50,
            enemy_max_hp: 50,
            circles: Vec::new(),
            last_spawn: None,
            outcome: None,
            outcome_time: None,
            captured: false,
            exp_gained: 0,
        }
    }
}

// ─── Input abstraction ────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum InputEvent {
    ToggleCursor,
    CursorToBattle,
    CursorToRoster,
    Confirm,
    SelectBattle,
    SelectRoster,
    Back,
    /// Touch tap at (x, y) – used during battle to hit circles.
    TapAt { x: u16, y: u16 },
}

#[derive(Resource, Default)]
pub struct InputQueue(pub Vec<InputEvent>);

// ─── JSON deserialization ─────────────────────────────────────────────────────

#[derive(Deserialize)]
struct JsonEnemy {
    name: String,
    level: u8,
    atk: u16,
    def: u16,
    hp: u16,
}


/// Leak a heap-allocated String into a `'static` str.
/// Acceptable here because monster names live for the entire program lifetime.
fn leak_name(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

// ─── Enemy pool ───────────────────────────────────────────────────────────────

pub struct EnemyDef {
    pub name: &'static str,
    pub level: u8,
    pub atk: u16,
    pub def: u16,
    pub hp: u16,
}

/// All possible enemies, loaded from `monsters.json` at startup.
#[derive(Resource)]
pub struct EnemyPool(pub Vec<EnemyDef>);

fn pick_random_enemy(pool: &EnemyPool) -> &EnemyDef {
    let idx = (random_u32() % pool.0.len() as u32) as usize;
    &pool.0[idx]
}

// ─── Systems ──────────────────────────────────────────────────────────────────

pub fn navigation_system(
    mut input_queue: ResMut<InputQueue>,
    mut screen: ResMut<CurrentScreen>,
    mut cursor: ResMut<MenuCursorRes>,
    mut battle: ResMut<TapBattleState>,
    mut active_slot: ResMut<ActiveSlot>,
    mut roster_scroll: ResMut<RosterScroll>,
    mut roster_hover: ResMut<RosterHover>,
    enemy_pool: Res<EnemyPool>,
    mut monsters: Query<(&MonName, &RosterSlot, &mut Level, &mut Stats, &mut Health, &mut Exp)>,
) {
    for event in input_queue.0.drain(..) {
        match screen.0 {
            Screen::Overview => handle_overview_event(
                event,
                &mut screen,
                &mut cursor,
                &mut battle,
                &mut active_slot,
                &mut roster_scroll,
                &mut roster_hover,
                &enemy_pool,
                &mut monsters,
            ),
            Screen::Roster => {
                let monster_count = monsters.iter().count();
                match event {
                    // Scroll down (SwipeUp → Confirm) – clears hover
                    InputEvent::Confirm => {
                        if roster_scroll.0 + 1 < monster_count {
                            roster_scroll.0 += 1;
                            roster_hover.0 = None;
                        }
                    }
                    // Scroll up (SwipeDown → SelectRoster) – clears hover
                    InputEvent::SelectRoster | InputEvent::ToggleCursor => {
                        if roster_scroll.0 > 0 {
                            roster_scroll.0 -= 1;
                            roster_hover.0 = None;
                        }
                    }
                    // Tap on a card: first tap = hover, second tap on same = confirm + leave
                    InputEvent::TapAt { x, y } => {
                        let in_card = x >= 8 && x <= 232 && y >= 32 && y < 264;
                        if in_card {
                            let visible_idx = (y as usize - 32) / 80;
                            let slot = roster_scroll.0 + visible_idx;
                            if slot < monster_count {
                                if roster_hover.0 == Some(slot) {
                                    // Second tap on same card → confirm
                                    active_slot.0 = slot;
                                    roster_hover.0 = None;
                                    screen.0 = Screen::Overview;
                                } else {
                                    // First tap → highlight only
                                    roster_hover.0 = Some(slot);
                                }
                            }
                        } else {
                            roster_hover.0 = None;
                            screen.0 = Screen::Overview;
                        }
                    }
                    // Back / other – exit roster
                    _ => {
                        roster_hover.0 = None;
                        screen.0 = Screen::Overview;
                    }
                }
            }
            Screen::Battle => {
                if battle.active {
                    // During active battle: process circle taps.
                    if let InputEvent::TapAt { x, y } = event {
                        process_battle_tap(x, y, &mut battle);
                    }
                } else if battle.outcome.is_some() {
                    // Battle done: wait for the 2-second result screen cooldown
                    // before accepting any input (prevents the finishing tap
                    // from immediately closing the screen).
                    const RESULT_COOLDOWN_MS: u128 = 2_000;
                    let cooldown_elapsed = battle.outcome_time
                        .map_or(true, |t| t.elapsed().as_millis() >= RESULT_COOLDOWN_MS);

                    if cooldown_elapsed {
                        match event {
                            InputEvent::TapAt { .. } | InputEvent::Back => {
                                apply_battle_to_monsters(&battle, &mut monsters);
                                screen.0 = Screen::Overview;
                                battle.outcome = None;
                                battle.outcome_time = None;
                                battle.circles.clear();
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Shouldn't happen – safety valve.
                    screen.0 = Screen::Overview;
                }
            }
        }
    }
}

fn process_battle_tap(x: u16, y: u16, battle: &mut TapBattleState) {
    // Hit the first circle whose area contains the tap point.
    if let Some(idx) = battle.circles.iter().position(|c| c.contains(x, y)) {
        let circle = battle.circles.remove(idx);
        match circle.kind {
            CircleKind::HeroAttack => {
                let dmg = calc_damage(battle.player_atk, battle.enemy_def);
                battle.enemy_hp = battle.enemy_hp.saturating_sub(dmg);
            }
            CircleKind::EnemyAttack => {
                // Blocked – no damage to player.
            }
        }
    }
}

fn apply_battle_to_monsters(
    battle: &TapBattleState,
    monsters: &mut Query<(&MonName, &RosterSlot, &mut Level, &mut Stats, &mut Health, &mut Exp)>,
) {
    for (_, slot, mut level, mut stats, mut health, mut exp) in monsters.iter_mut() {
        if slot.0 != battle.entity_slot {
            continue;
        }
        health.hp = battle.player_hp;
        exp.current += battle.exp_gained;

        while exp.current >= exp.next {
            exp.current -= exp.next;
            level.0 += 1;
            exp.next = (level.0 as u32 + 1) * 100;
            stats.atk += 2;
            stats.def += 1;
            health.max_hp += 5;
            if battle.outcome == Some(true) {
                health.hp = health.max_hp; // full heal on level-up after win
            }
        }
        return;
    }
}

fn handle_overview_event(
    event: InputEvent,
    screen: &mut ResMut<CurrentScreen>,
    cursor: &mut ResMut<MenuCursorRes>,
    battle: &mut ResMut<TapBattleState>,
    active_slot: &mut ResMut<ActiveSlot>,
    roster_scroll: &mut ResMut<RosterScroll>,
    roster_hover: &mut ResMut<RosterHover>,
    enemy_pool: &EnemyPool,
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
            MenuCursor::Battle => try_start_battle(screen, battle, active_slot.0, enemy_pool, monsters),
            MenuCursor::Roster => {
                roster_scroll.0 = 0;
                roster_hover.0 = None;
                screen.0 = Screen::Roster;
            }
        },
        InputEvent::SelectBattle => {
            cursor.0 = MenuCursor::Battle;
            try_start_battle(screen, battle, active_slot.0, enemy_pool, monsters);
        }
        InputEvent::SelectRoster => {
            cursor.0 = MenuCursor::Roster;
            roster_scroll.0 = 0;
            roster_hover.0 = None;
            screen.0 = Screen::Roster;
        }
        InputEvent::Back | InputEvent::TapAt { .. } => {}
    }
}

fn try_start_battle(
    screen: &mut ResMut<CurrentScreen>,
    battle: &mut ResMut<TapBattleState>,
    active_slot: usize,
    enemy_pool: &EnemyPool,
    monsters: &mut Query<(&MonName, &RosterSlot, &mut Level, &mut Stats, &mut Health, &mut Exp)>,
) {
    if enemy_pool.0.is_empty() {
        return;
    }
    for (name, slot, level, stats, health, _exp) in monsters.iter() {
        if slot.0 != active_slot {
            continue;
        }
        if health.is_fainted() {
            return;
        }
        let enemy = pick_random_enemy(enemy_pool);
        **battle = TapBattleState {
            active: true,
            entity_slot: active_slot,
            player_name: name.0,
            player_level: level.0,
            player_atk: stats.atk,
            player_def: stats.def,
            player_hp: health.hp,
            player_max_hp: health.max_hp,
            enemy_name: enemy.name,
            enemy_level: enemy.level,
            enemy_atk: enemy.atk,
            enemy_def: enemy.def,
            enemy_hp: enemy.hp,
            enemy_max_hp: enemy.hp,
            circles: Vec::new(),
            last_spawn: None,
            outcome: None,
            outcome_time: None,
            captured: false,
            exp_gained: 0,
        };
        screen.0 = Screen::Battle;
        return;
    }
}

/// Time-based battle update: spawns circles, expires them, checks win/lose.
/// Runs after navigation_system so tap damage is already applied before the check.
pub fn tap_battle_update_system(
    screen: Res<CurrentScreen>,
    mut battle: ResMut<TapBattleState>,
    mut pending_capture: ResMut<PendingCapture>,
) {
    if screen.0 != Screen::Battle || !battle.active {
        return;
    }

    // Win/lose check first (navigation may have dealt the killing blow).
    if battle.enemy_hp == 0 {
        // 50% chance to capture the defeated enemy.
        let captured = random_u32() % 2 == 0;
        battle.captured = captured;
        if captured {
            pending_capture.0 = Some(CapturedMonster {
                name: battle.enemy_name,
                level: battle.enemy_level,
                atk: battle.enemy_atk,
                def: battle.enemy_def,
                hp: battle.enemy_max_hp,
            });
        }
        battle.outcome = Some(true);
        battle.outcome_time = Some(Instant::now());
        battle.exp_gained = 40 + battle.enemy_level as u32 * 10;
        battle.active = false;
        battle.circles.clear();
        return;
    }
    if battle.player_hp == 0 {
        battle.outcome = Some(false);
        battle.outcome_time = Some(Instant::now());
        battle.exp_gained = 10;
        battle.active = false;
        battle.circles.clear();
        return;
    }

    // Expire circles; enemy circles that vanish deal damage to the player.
    let mut i = 0;
    while i < battle.circles.len() {
        if battle.circles[i].is_expired() {
            let circle = battle.circles.remove(i);
            if circle.kind == CircleKind::EnemyAttack {
                let dmg = calc_damage(battle.enemy_atk, battle.player_def);
                battle.player_hp = battle.player_hp.saturating_sub(dmg);
            }
        } else {
            i += 1;
        }
    }

    // Check again after expiry damage.
    if battle.player_hp == 0 {
        battle.outcome = Some(false);
        battle.outcome_time = Some(Instant::now());
        battle.exp_gained = 10;
        battle.active = false;
        battle.circles.clear();
        return;
    }

    // Spawn a new circle periodically.
    const SPAWN_INTERVAL_MS: u128 = 300;
    const MAX_CIRCLES: usize = 15;

    let should_spawn = match battle.last_spawn {
        None => true,
        Some(t) => t.elapsed().as_millis() >= SPAWN_INTERVAL_MS,
    };

    if should_spawn && battle.circles.len() < MAX_CIRCLES {
        // Circle center safe zone: x=[35,204], y=[55,224], max_radius up to 32.
        let cx = 35 + (random_u32() % 170) as u16;
        let cy = 55 + (random_u32() % 170) as u16;
        let max_radius = 20 + (random_u32() % 13) as u16;
        let kind = if random_u32() % 5 == 0 {
            CircleKind::EnemyAttack
        } else {
            CircleKind::HeroAttack
        };
        battle.circles.push(TapCircle {
            cx,
            cy,
            max_radius,
            kind,
            spawned_at: Instant::now(),
            lifetime_ms: 2_500,
        });
        battle.last_spawn = Some(Instant::now());
    }
}

// ─── World bootstrap ──────────────────────────────────────────────────────────

pub fn setup_world() -> World {
    let mut world = World::new();

    // ── Parse monsters.json (embedded at compile time) ────────────────────
    let json = include_str!("../monsters.json");
    let enemies: Vec<JsonEnemy> = serde_json::from_str(json)
        .expect("monsters.json is invalid – fix the JSON before flashing");

    // ── Spawn starter monster (roster will be loaded from SD save data later) ──
    let entities = vec![
        world.spawn((MonName("Ferrobit"), Level(5), Stats { atk: 25, def: 20 }, Health { hp: 50, max_hp: 50 }, Exp { current: 0, next: 600 }, RosterSlot(0), Count(0))).id(),
    ];

    // ── Build enemy pool ──────────────────────────────────────────────────
    let enemy_pool = EnemyPool(
        enemies.into_iter().map(|e| EnemyDef {
            name: leak_name(e.name),
            level: e.level,
            atk: e.atk,
            def: e.def,
            hp: e.hp,
        }).collect(),
    );

    world.insert_resource(CurrentScreen::default());
    world.insert_resource(MenuCursorRes::default());
    world.insert_resource(ActiveSlot::default());
    world.insert_resource(RosterScroll::default());
    world.insert_resource(RosterHover::default());
    world.insert_resource(TapBattleState::default());
    world.insert_resource(PendingCapture::default());
    world.insert_resource(InputQueue::default());
    world.insert_resource(RosterEntities(entities));
    world.insert_resource(enemy_pool);

    world
}

pub fn build_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems((
        navigation_system,
        tap_battle_update_system.after(navigation_system),
    ));
    schedule
}
