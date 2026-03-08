fn random_u32() -> u32 {
    extern "C" {
        fn esp_random() -> u32;
    }
    unsafe { esp_random() }
}

fn calc_damage(atk: u16, def: u16) -> u16 {
    let base = (atk as i32 - def as i32 / 2).max(1) as u32;
    let variance = 80 + (random_u32() % 41); // 80..=120 percent
    ((base * variance) / 100).max(1) as u16
}

// ─── Rustymon ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Rustymon {
    pub name: &'static str,
    pub level: u8,
    pub exp: u32,
    pub exp_next: u32,
    pub atk: u16,
    pub def: u16,
    pub hp: u16,
    pub max_hp: u16,
}

impl Rustymon {
    pub fn new(name: &'static str, level: u8, atk: u16, def: u16, max_hp: u16) -> Self {
        Rustymon {
            name,
            level,
            exp: 0,
            exp_next: (level as u32 + 1) * 100,
            atk,
            def,
            hp: max_hp,
            max_hp,
        }
    }

    pub fn is_fainted(&self) -> bool {
        self.hp == 0
    }

    pub fn hp_pct(&self) -> u8 {
        if self.max_hp == 0 {
            return 0;
        }
        ((self.hp as u32 * 100) / self.max_hp as u32) as u8
    }

    pub fn exp_pct(&self) -> u8 {
        if self.exp_next == 0 {
            return 100;
        }
        ((self.exp as u64 * 100) / self.exp_next as u64) as u8
    }
}

fn random_enemy() -> Rustymon {
    let enemies: [Rustymon; 4] = [
        Rustymon::new("Toxibolt", 4, 22, 14, 40),
        Rustymon::new("Glitchrat", 3, 18, 12, 32),
        Rustymon::new("Ironclad", 5, 20, 22, 48),
        Rustymon::new("Virebug", 4, 24, 16, 38),
    ];
    enemies[(random_u32() % 4) as usize].clone()
}

// ─── Battle ──────────────────────────────────────────────────────────────────

pub struct BattleResult {
    pub log: Vec<String>,
    pub player_won: bool,
    #[allow(dead_code)]
    pub enemy_name: &'static str,
    pub player_final_hp: u16,
    pub exp_gained: u32,
}

pub fn simulate_battle(player: &Rustymon, enemy_seed: Rustymon) -> BattleResult {
    let mut p = player.clone();
    let mut e = enemy_seed.clone();
    let mut log: Vec<String> = Vec::new();

    log.push(format!("VS {} begins!", e.name));
    log.push(format!(
        "You: {} HP:{} ATK:{} DEF:{}",
        p.name, p.hp, p.atk, p.def
    ));
    log.push(format!(
        "Foe: {} HP:{} ATK:{} DEF:{}",
        e.name, e.hp, e.atk, e.def
    ));

    for turn in 1u32.. {
        log.push(format!("--- Turn {} ---", turn));

        // Player attacks enemy
        let p_dmg = calc_damage(p.atk, e.def);
        e.hp = e.hp.saturating_sub(p_dmg);
        log.push(format!("{} -> {} dmg", p.name, p_dmg));
        log.push(format!("Foe HP: {}/{}", e.hp, e.max_hp));

        if e.is_fainted() {
            log.push(format!("{} fainted!", e.name));
            log.push("*** YOU WIN! ***".to_string());
            return BattleResult {
                log,
                player_won: true,
                enemy_name: enemy_seed.name,
                player_final_hp: p.hp,
                exp_gained: 40 + (e.level as u32) * 10,
            };
        }

        // Enemy attacks player
        let e_dmg = calc_damage(e.atk, p.def);
        p.hp = p.hp.saturating_sub(e_dmg);
        log.push(format!("{} -> {} dmg", e.name, e_dmg));
        log.push(format!("Your HP: {}/{}", p.hp, p.max_hp));

        if p.is_fainted() {
            log.push(format!("{} fainted!", p.name));
            log.push("*** YOU LOSE... ***".to_string());
            return BattleResult {
                log,
                player_won: false,
                enemy_name: enemy_seed.name,
                player_final_hp: 0,
                exp_gained: 10,
            };
        }

        if turn >= 50 {
            log.push("Time ran out! Draw.".to_string());
            return BattleResult {
                log,
                player_won: false,
                enemy_name: enemy_seed.name,
                player_final_hp: p.hp,
                exp_gained: 5,
            };
        }
    }
    unreachable!()
}

// ─── Game state ───────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone)]
pub enum MenuCursor {
    Battle,
    Roster,
}

#[derive(PartialEq, Clone)]
pub enum Screen {
    Overview,
    Roster,
    Battle,
}

pub struct GameState {
    pub roster: Vec<Rustymon>,
    pub active: usize,
    pub screen: Screen,
    pub cursor: MenuCursor,
    pub battle: Option<BattleResult>,
    pub battle_lines_shown: usize,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            roster: vec![
                Rustymon::new("Ferrobit", 5, 25, 20, 50),
                Rustymon::new("Blazerust", 3, 32, 10, 35),
                Rustymon::new("Aquabyte", 4, 20, 18, 45),
            ],
            active: 0,
            screen: Screen::Overview,
            cursor: MenuCursor::Battle,
            battle: None,
            battle_lines_shown: 0,
        }
    }

    pub fn active_rustymon(&self) -> &Rustymon {
        &self.roster[self.active]
    }

    pub fn start_battle(&mut self) {
        if self.active_rustymon().is_fainted() {
            return; // Cannot battle with fainted rustymon
        }
        let enemy = random_enemy();
        let result = simulate_battle(self.active_rustymon(), enemy);

        let mon = &mut self.roster[self.active];
        mon.hp = result.player_final_hp;
        mon.exp += result.exp_gained;

        // Level up check
        while mon.exp >= mon.exp_next {
            mon.exp -= mon.exp_next;
            mon.level += 1;
            mon.exp_next = (mon.level as u32 + 1) * 100;
            mon.atk += 2;
            mon.def += 1;
            mon.max_hp += 5;
            if result.player_won {
                mon.hp = mon.max_hp; // full heal on level-up
            }
        }

        self.battle = Some(result);
        self.battle_lines_shown = 0;
        self.screen = Screen::Battle;
    }

    pub fn go_roster(&mut self) {
        self.screen = Screen::Roster;
    }

    pub fn go_overview(&mut self) {
        self.screen = Screen::Overview;
    }

    pub fn toggle_cursor(&mut self) {
        self.cursor = match self.cursor {
            MenuCursor::Battle => MenuCursor::Roster,
            MenuCursor::Roster => MenuCursor::Battle,
        };
    }

    pub fn confirm_selection(&mut self) {
        match self.cursor {
            MenuCursor::Battle => self.start_battle(),
            MenuCursor::Roster => self.go_roster(),
        }
    }

    pub fn battle_is_done(&self) -> bool {
        match &self.battle {
            Some(b) => self.battle_lines_shown >= b.log.len(),
            None => true,
        }
    }

    pub fn advance_battle_line(&mut self) {
        if let Some(b) = &self.battle {
            if self.battle_lines_shown < b.log.len() {
                self.battle_lines_shown += 1;
            }
        }
    }
}
