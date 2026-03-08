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

pub struct RenderData {
    pub screen: Screen,
    pub cursor: MenuCursor,
    pub active_slot: usize,
    /// Roster sorted ascending by slot index.
    pub roster: Vec<MonData>,
    pub battle_lines_shown: usize,
    /// `Some((log_lines, player_won))` when a battle result is available.
    pub battle_log: Option<(Vec<String>, bool)>,
}

impl RenderData {
    pub fn battle_is_done(&self) -> bool {
        self.battle_log
            .as_ref()
            .map_or(true, |(log, _)| self.battle_lines_shown >= log.len())
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
    let battle_log = world
        .resource::<BattleData>()
        .result
        .as_ref()
        .map(|r| (r.log.clone(), r.player_won));

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
        battle_log,
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

fn render_battle<D: DrawTarget<Color = Rgb888>>(display: &mut D, data: &RenderData) {
    fill_rect(display, 0, 0, 240, 284, BG);

    fill_rect(display, 0, 0, 240, 28, BATTLE_BG);
    draw_text(display, "BATTLE", 76, 20, &FONT_10X20, RED);

    let Some((log, player_won)) = &data.battle_log else {
        draw_text(display, "No battle data", 30, 150, &FONT_10X20, GRAY);
        return;
    };

    let visible = data.battle_lines_shown.min(log.len());
    const MAX_VISIBLE: usize = 18;
    let start = visible.saturating_sub(MAX_VISIBLE);

    for (slot, idx) in (start..visible).enumerate() {
        let y = 42 + slot as i32 * 13;
        let line = &log[idx];
        draw_text(display, line, 4, y, &FONT_6X10, log_line_color(line));
    }

    if data.battle_is_done() {
        fill_rect(display, 0, 268, 240, 16, Rgb888::new(30, 30, 30));
        let (msg, col) = if *player_won {
            ("VICTORY!  Tap to return", GREEN)
        } else {
            ("DEFEAT...  Tap to return", RED)
        };
        draw_text(display, msg, 6, 280, &FONT_6X10, col);
    }
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
