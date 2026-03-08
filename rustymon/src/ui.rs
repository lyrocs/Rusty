use bevy_ecs::world::World;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_10X20},
        MonoFont, MonoTextStyle,
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::Text,
};

use crate::game::{
    ActiveSlot, BattleData, CurrentScreen, Exp, Health, Level, MenuCursor, MenuCursorRes,
    MonName, RosterEntities, RosterSlot, Screen, Stats,
};

// ─── Render snapshot ─────────────────────────────────────────────────────────
// Plain data extracted from the World each frame so render functions are
// completely pure (no ECS borrows inside drawing code).

pub struct MonData {
    pub name: &'static str,
    pub level: u8,
    pub atk: u16,
    pub def: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub exp: u32,
    pub exp_next: u32,
}

impl MonData {
    fn hp_pct(&self) -> u8 {
        if self.max_hp == 0 { return 0; }
        ((self.hp as u32 * 100) / self.max_hp as u32) as u8
    }
    fn exp_pct(&self) -> u8 {
        if self.exp_next == 0 { return 100; }
        ((self.exp as u64 * 100) / self.exp_next as u64) as u8
    }
    fn is_fainted(&self) -> bool { self.hp == 0 }
}

/// All battle data the render layer needs, cloned out of the ECS world.
pub struct BattleRenderData {
    pub log: Vec<String>,
    pub player_won: bool,
    pub player_name: &'static str,
    pub player_level: u8,
    pub player_max_hp: u16,
    pub enemy_name: &'static str,
    pub enemy_level: u8,
    pub enemy_max_hp: u16,
    /// `(player_hp, enemy_hp)` recorded after every log entry.
    pub hp_at: Vec<(u16, u16)>,
}

impl BattleRenderData {
    /// HP pair current at `lines_shown` animation frames.
    fn hp_now(&self, lines_shown: usize) -> (u16, u16) {
        if lines_shown == 0 || self.hp_at.is_empty() {
            return (self.player_max_hp, self.enemy_max_hp);
        }
        self.hp_at[(lines_shown - 1).min(self.hp_at.len() - 1)]
    }
}

pub struct RenderData {
    pub screen: Screen,
    pub cursor: MenuCursor,
    pub active_slot: usize,
    /// Roster sorted ascending by slot index.
    pub roster: Vec<MonData>,
    pub battle_lines_shown: usize,
    pub battle: Option<BattleRenderData>,
}

impl RenderData {
    pub fn battle_is_done(&self) -> bool {
        self.battle
            .as_ref()
            .map_or(true, |b| self.battle_lines_shown >= b.log.len())
    }
}

/// Extract a cheap snapshot from the ECS world. Called once per frame from main
/// **after** the schedule has run so all state is up-to-date.
pub fn extract_render_data(world: &World) -> RenderData {
    // ── Resources ──────────────────────────────────────────────────────────
    let screen = world.resource::<CurrentScreen>().0.clone();
    let cursor = world.resource::<MenuCursorRes>().0.clone();
    let active_slot = world.resource::<ActiveSlot>().0;
    let battle_lines_shown = world.resource::<BattleData>().lines_shown;
    let battle = world.resource::<BattleData>().result.as_ref().map(|r| BattleRenderData {
        log:            r.log.clone(),
        player_won:     r.player_won,
        player_name:    r.player_name,
        player_level:   r.player_level,
        player_max_hp:  r.player_max_hp,
        enemy_name:     r.enemy_name,
        enemy_level:    r.enemy_level,
        enemy_max_hp:   r.enemy_max_hp,
        hp_at:          r.hp_at.clone(),
    });

    // ── Roster entities (cloned Vec so no lingering world borrow) ──────────
    let entity_ids: Vec<_> = world.resource::<RosterEntities>().0.clone();

    let mut pairs: Vec<(usize, MonData)> = entity_ids
        .iter()
        .map(|&entity| {
            let e = world.entity(entity);
            let slot = e.get::<RosterSlot>().unwrap().0;
            let data = MonData {
                name:     e.get::<MonName>().unwrap().0,
                level:    e.get::<Level>().unwrap().0,
                atk:      e.get::<Stats>().unwrap().atk,
                def:      e.get::<Stats>().unwrap().def,
                hp:       e.get::<Health>().unwrap().hp,
                max_hp:   e.get::<Health>().unwrap().max_hp,
                exp:      e.get::<Exp>().unwrap().current,
                exp_next: e.get::<Exp>().unwrap().next,
            };
            (slot, data)
        })
        .collect();

    pairs.sort_by_key(|(slot, _)| *slot);

    RenderData {
        screen,
        cursor,
        active_slot,
        roster: pairs.into_iter().map(|(_, d)| d).collect(),
        battle_lines_shown,
        battle,
    }
}

// ─── Palette ──────────────────────────────────────────────────────────────────
const BG: Rgb888 = Rgb888::new(8, 8, 18);
const TITLE_BG: Rgb888 = Rgb888::new(15, 35, 110);
const BATTLE_BG: Rgb888 = Rgb888::new(80, 12, 12);
const CARD_ACTIVE: Rgb888 = Rgb888::new(15, 45, 18);
const CARD_IDLE: Rgb888 = Rgb888::new(18, 18, 40);
const WHITE: Rgb888 = Rgb888::WHITE;
const YELLOW: Rgb888 = Rgb888::new(255, 220, 0);
const ORANGE: Rgb888 = Rgb888::new(255, 140, 0);
const GREEN: Rgb888 = Rgb888::new(50, 200, 80);
const RED: Rgb888 = Rgb888::new(220, 60, 60);
const BLUE_BTN: Rgb888 = Rgb888::new(40, 80, 200);
const GRAY: Rgb888 = Rgb888::new(90, 90, 100);
const DARK_GRAY: Rgb888 = Rgb888::new(40, 40, 50);

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn render_screen<D: DrawTarget<Color = Rgb888>>(display: &mut D, data: &RenderData) {
    match data.screen {
        Screen::Overview => render_overview(display, data),
        Screen::Roster   => render_roster(display, data),
        Screen::Battle   => render_battle(display, data),
    }
}

// ─── Overview ────────────────────────────────────────────────────────────────

fn render_overview<D: DrawTarget<Color = Rgb888>>(display: &mut D, data: &RenderData) {
    fill_rect(display, 0, 0, 240, 284, BG);

    fill_rect(display, 0, 0, 240, 28, TITLE_BG);
    draw_text(display, "OVERVIEW", 62, 20, &FONT_10X20, YELLOW);

    let mon = &data.roster[data.active_slot];

    draw_text(display, mon.name, 10, 52, &FONT_10X20, WHITE);
    let lv = format!("Lv.{}", mon.level);
    draw_text(display, &lv, 178, 52, &FONT_10X20, YELLOW);

    draw_hline(display, 10, 230, 59, GRAY);

    let hp_label = format!("HP  {}/{}", mon.hp, mon.max_hp);
    draw_text(display, &hp_label, 10, 76, &FONT_6X10, WHITE);
    draw_bar(display, 10, 80, 220, 10, mon.hp_pct(), hp_bar_color(mon.hp_pct()));

    let atk_s = format!("ATK: {}", mon.atk);
    let def_s = format!("DEF: {}", mon.def);
    draw_text(display, &atk_s, 10, 106, &FONT_6X10, WHITE);
    draw_text(display, &def_s, 128, 106, &FONT_6X10, WHITE);

    let exp_s = format!("EXP {}/{}", mon.exp, mon.exp_next);
    draw_text(display, &exp_s, 10, 124, &FONT_6X10, GRAY);
    draw_bar(display, 10, 128, 220, 6, mon.exp_pct(), ORANGE);

    draw_hline(display, 10, 230, 146, GRAY);

    draw_monster_icon(display, 96, 158, GRAY);

    let b_sel = data.cursor == MenuCursor::Battle;
    let r_sel = data.cursor == MenuCursor::Roster;
    draw_button(display, 14, 244, 96, 30, "BATTLE", b_sel);
    draw_button(display, 130, 244, 96, 30, "ROSTER", r_sel);

    draw_text(display, "Tap btn | Swipe< > | SwipeUp", 6, 282, &FONT_6X10, DARK_GRAY);
}

// ─── Roster ──────────────────────────────────────────────────────────────────

fn render_roster<D: DrawTarget<Color = Rgb888>>(display: &mut D, data: &RenderData) {
    fill_rect(display, 0, 0, 240, 284, BG);

    fill_rect(display, 0, 0, 240, 28, TITLE_BG);
    draw_text(display, "ROSTER", 76, 20, &FONT_10X20, YELLOW);

    for (i, mon) in data.roster.iter().enumerate() {
        let y = 32 + i as i32 * 80;
        let is_active = i == data.active_slot;

        let card_col   = if is_active { CARD_ACTIVE } else { CARD_IDLE };
        let border_col = if is_active { GREEN } else { Rgb888::new(50, 50, 90) };

        fill_rect(display, 8, y, 224, 72, card_col);
        stroke_rect(display, 8, y, 224, 72, border_col);

        if is_active {
            draw_text(display, ">", 12, y + 18, &FONT_10X20, GREEN);
        }

        draw_text(display, mon.name, 26, y + 18, &FONT_10X20, WHITE);
        let lv = format!("Lv.{}", mon.level);
        draw_text(display, &lv, 180, y + 18, &FONT_10X20, YELLOW);

        let hp_s = format!("HP {}/{}", mon.hp, mon.max_hp);
        draw_text(display, &hp_s, 26, y + 38, &FONT_6X10, WHITE);
        draw_bar(display, 26, y + 42, 192, 8, mon.hp_pct(), hp_bar_color(mon.hp_pct()));

        if mon.is_fainted() {
            draw_text(display, "FAINTED", 150, y + 38, &FONT_6X10, RED);
        }
    }

    draw_text(display, "Tap or swipe to go back", 6, 282, &FONT_6X10, DARK_GRAY);
}

// ─── Battle ──────────────────────────────────────────────────────────────────
//
// Layout (240 × 284 px):
//   y=  0..28   Title bar
//   y= 28..82   Enemy stat card  (h=54)
//   y= 82..94   "─── VS ───" separator
//   y= 94..148  Player stat card (h=54)
//   y=150..264  Turn log text box (h=114, up to 8 lines)
//   y=264..284  Result / hint bar (h=20)

const CARD_ENEMY:  Rgb888 = Rgb888::new(50, 12, 12);
const CARD_PLAYER: Rgb888 = Rgb888::new(10, 40, 14);
const LOG_BG:      Rgb888 = Rgb888::new(14, 14, 28);

fn render_battle<D: DrawTarget<Color = Rgb888>>(display: &mut D, data: &RenderData) {
    fill_rect(display, 0, 0, 240, 284, BG);

    // ── Title bar ────────────────────────────────────────────────────────
    fill_rect(display, 0, 0, 240, 28, BATTLE_BG);
    draw_text(display, "BATTLE", 76, 20, &FONT_10X20, RED);

    let Some(battle) = &data.battle else {
        draw_text(display, "No battle data", 30, 150, &FONT_10X20, GRAY);
        return;
    };

    let (player_hp, enemy_hp) = battle.hp_now(data.battle_lines_shown);

    // ── Enemy card – top-RIGHT (y=28..80, x=76..236) ─────────────────────
    // Mirrored like Pokémon: name/level on the right, HP bar flush-right,
    // no HP numbers (the bar hides the exact value until it drains).
    const EX: i32 = 76; const EW: u32 = 160; // card left-edge and width
    fill_rect(display, EX, 28, EW, 52, CARD_ENEMY);
    stroke_rect(display, EX, 28, EW, 52, RED);

    // Level right-aligned inside card ("Lv.X" = 4 chars × 10 px = 40 px)
    let lv_e = format!("Lv.{}", battle.enemy_level);
    let lv_e_x = EX + EW as i32 - 4 - lv_e.len() as i32 * 10;
    draw_text(display, battle.enemy_name, EX + 4, 44, &FONT_10X20, WHITE);
    draw_text(display, &lv_e, lv_e_x, 44, &FONT_10X20, YELLOW);

    // HP bar flush with the card's right edge
    let e_pct = pct(enemy_hp, battle.enemy_max_hp);
    draw_bar(display, EX + 4, 52, EW - 8, 9, e_pct, hp_bar_color(e_pct));

    // ── VS separator (y=82..94) ──────────────────────────────────────────
    draw_hline(display, 4, 236, 84, GRAY);
    draw_text(display, "--- VS ---", 78, 93, &FONT_6X10, GRAY);

    // ── Player card – bottom-LEFT (y=94..152, x=4..164) ──────────────────
    const PX: i32 = 4; const PW: u32 = 160;
    fill_rect(display, PX, 94, PW, 58, CARD_PLAYER);
    stroke_rect(display, PX, 94, PW, 58, GREEN);

    // Level right-aligned inside card
    let lv_p = format!("Lv.{}", battle.player_level);
    let lv_p_x = PX + PW as i32 - 4 - lv_p.len() as i32 * 10;
    draw_text(display, battle.player_name, PX + 4, 110, &FONT_10X20, WHITE);
    draw_text(display, &lv_p, lv_p_x, 110, &FONT_10X20, YELLOW);

    let p_pct = pct(player_hp, battle.player_max_hp);
    draw_bar(display, PX + 4, 118, PW - 8, 9, p_pct, hp_bar_color(p_pct));
    // Player shows exact HP numbers (classic Pokémon bottom box)
    let hp_p = format!("HP {}/{}", player_hp, battle.player_max_hp);
    draw_text(display, &hp_p, PX + 4, 141, &FONT_6X10, WHITE);

    // ── Turn log text box (y=156..264) ───────────────────────────────────
    fill_rect(display, 4, 156, 232, 108, LOG_BG);
    stroke_rect(display, 4, 156, 232, 108, GRAY);

    let visible = data.battle_lines_shown.min(battle.log.len());
    const MAX_LOG_LINES: usize = 7;
    let start = visible.saturating_sub(MAX_LOG_LINES);

    for (slot, idx) in (start..visible).enumerate() {
        let y = 169 + slot as i32 * 14;
        let line = &battle.log[idx];
        draw_text(display, line, 8, y, &FONT_6X10, log_line_color(line));
    }

    // ── Result / hint bar (y=264..284) ───────────────────────────────────
    if data.battle_is_done() {
        let (msg, col) = if battle.player_won {
            ("  VICTORY!  Tap to return  ", GREEN)
        } else {
            ("  DEFEAT...  Tap to return  ", RED)
        };
        fill_rect(display, 0, 264, 240, 20, Rgb888::new(20, 20, 20));
        draw_hline(display, 0, 240, 264, if battle.player_won { GREEN } else { RED });
        draw_text(display, msg, 4, 278, &FONT_6X10, col);
    }
}

fn pct(current: u16, max: u16) -> u8 {
    if max == 0 { return 0; }
    ((current as u32 * 100) / max as u32) as u8
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn hp_bar_color(pct: u8) -> Rgb888 {
    if pct > 50 { GREEN } else if pct > 25 { YELLOW } else { RED }
}

fn log_line_color(line: &str) -> Rgb888 {
    if line.contains("WIN") { GREEN }
    else if line.contains("LOSE") || line.contains("fainted") { RED }
    else if line.starts_with("---") { YELLOW }
    else if line.starts_with("VS") || line.starts_with("You:") || line.starts_with("Foe:") { ORANGE }
    else { WHITE }
}

fn draw_button<D: DrawTarget<Color = Rgb888>>(
    display: &mut D, x: i32, y: i32, w: u32, h: u32, label: &str, selected: bool,
) {
    let bg     = if selected { BLUE_BTN } else { Rgb888::new(28, 28, 46) };
    let border = if selected { WHITE }    else { GRAY };
    fill_rect(display, x, y, w, h, bg);
    stroke_rect(display, x, y, w, h, border);
    let tx = x + (w as i32 - label.len() as i32 * 6) / 2;
    let ty = y + h as i32 / 2 + 4;
    draw_text(display, label, tx, ty, &FONT_6X10, WHITE);
}

fn draw_bar<D: DrawTarget<Color = Rgb888>>(
    display: &mut D, x: i32, y: i32, w: u32, h: u32, pct: u8, color: Rgb888,
) {
    fill_rect(display, x, y, w, h, DARK_GRAY);
    let filled = (w * pct as u32 / 100).max(if pct > 0 { 1 } else { 0 });
    if filled > 0 {
        fill_rect(display, x, y, filled, h, color);
    }
    stroke_rect(display, x, y, w, h, GRAY);
}

fn draw_monster_icon<D: DrawTarget<Color = Rgb888>>(display: &mut D, x: i32, y: i32, color: Rgb888) {
    fill_rect(display, x + 12, y,      26, 20, color);          // head
    fill_rect(display, x + 16, y + 5,   6,  6, YELLOW);          // left eye
    fill_rect(display, x + 28, y + 5,   6,  6, YELLOW);          // right eye
    fill_rect(display, x +  8, y + 22, 34, 22, color);           // body
    fill_rect(display, x,      y + 24,  8, 14, color);           // left arm
    fill_rect(display, x + 42, y + 24,  8, 14, color);           // right arm
    fill_rect(display, x + 12, y + 46, 10, 14, color);           // left leg
    fill_rect(display, x + 28, y + 46, 10, 14, color);           // right leg
}

// ─── Primitive wrappers ──────────────────────────────────────────────────────

fn fill_rect<D: DrawTarget<Color = Rgb888>>(display: &mut D, x: i32, y: i32, w: u32, h: u32, color: Rgb888) {
    let _ = Rectangle::new(Point::new(x, y), Size::new(w, h))
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display);
}

fn stroke_rect<D: DrawTarget<Color = Rgb888>>(display: &mut D, x: i32, y: i32, w: u32, h: u32, color: Rgb888) {
    let _ = Rectangle::new(Point::new(x, y), Size::new(w, h))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(display);
}

fn draw_hline<D: DrawTarget<Color = Rgb888>>(display: &mut D, x1: i32, x2: i32, y: i32, color: Rgb888) {
    let _ = Line::new(Point::new(x1, y), Point::new(x2, y))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(display);
}

fn draw_text<D: DrawTarget<Color = Rgb888>>(
    display: &mut D, text: &str, x: i32, y: i32, font: &MonoFont<'_>, color: Rgb888,
) {
    let _ = Text::new(text, Point::new(x, y), MonoTextStyle::new(font, color)).draw(display);
}
