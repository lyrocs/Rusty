use bevy_ecs::world::World;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_10X20},
        MonoFont, MonoTextStyle,
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};

use crate::game::{
    ActiveSlot, CircleKind, Count, CurrentScreen, EncounterState, Exp, Health, Level, MenuCursor,
    MenuCursorRes, MonName, RosterEntities, RosterHover, RosterScroll, RosterSlot, Screen, Stats,
    TapBattleState,
};
use crate::sprite::Sprite;

// ─── Render snapshot ─────────────────────────────────────────────────────────

pub struct MonData {
    pub name: &'static str,
    pub level: u8,
    pub atk: u16,
    pub def: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub exp: u32,
    pub exp_next: u32,
    pub count: u8,
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

/// Snapshot of a single shrinking circle, radius pre-computed at extract time.
pub struct CircleSnapshot {
    pub cx: u16,
    pub cy: u16,
    pub radius: u16,
    pub kind: CircleKind,
}

/// All tap-battle data the render layer needs.
pub struct TapBattleRenderData {
    pub player_name: &'static str,
    pub player_hp: u16,
    pub player_max_hp: u16,
    pub enemy_name: &'static str,
    pub enemy_hp: u16,
    pub enemy_max_hp: u16,
    pub circles: Vec<CircleSnapshot>,
    pub outcome: Option<bool>,          // Some(true)=won, Some(false)=lost
    pub result_cooldown_ms_left: u32,   // ms remaining before tap is accepted (0 = ready)
    pub captured: bool,                 // enemy joined roster on victory
    pub captured_name: &'static str,
    pub capture_is_upgrade: bool,       // true = duplicate (upgraded existing)
    pub capture_new_count: u8,          // +N after upgrade
}

pub struct EncounterRenderData {
    pub enemy_name:  &'static str,
    pub enemy_level: u8,
    pub enemy_atk:   u16,
    pub enemy_def:   u16,
    pub enemy_hp:    u16,
    pub seconds_left: u8,
}

pub struct RenderData {
    pub screen: Screen,
    pub cursor: MenuCursor,
    pub active_slot: usize,
    pub roster: Vec<MonData>,
    pub roster_scroll: usize,
    pub roster_hover: Option<usize>,
    pub battle: Option<TapBattleRenderData>,
    pub encounter: Option<EncounterRenderData>,
}

/// Extract a cheap snapshot from the ECS world each frame.
pub fn extract_render_data(world: &World) -> RenderData {
    let screen        = world.resource::<CurrentScreen>().0.clone();
    let cursor        = world.resource::<MenuCursorRes>().0.clone();
    let active_slot   = world.resource::<ActiveSlot>().0;
    let roster_scroll = world.resource::<RosterScroll>().0;
    let roster_hover  = world.resource::<RosterHover>().0;

    // ── Battle data ────────────────────────────────────────────────────────
    let bs = world.resource::<TapBattleState>();
    let battle = if bs.active || bs.outcome.is_some() {
        let circles = bs.circles.iter().map(|c| CircleSnapshot {
            cx: c.cx,
            cy: c.cy,
            radius: c.current_radius(),
            kind: c.kind,
        }).collect();
        const RESULT_COOLDOWN_MS: u128 = 2_000;
        let result_cooldown_ms_left = bs.outcome_time
            .map(|t| {
                let elapsed = t.elapsed().as_millis();
                RESULT_COOLDOWN_MS.saturating_sub(elapsed) as u32
            })
            .unwrap_or(0);
        Some(TapBattleRenderData {
            player_name:  bs.player_name,
            player_hp:    bs.player_hp,
            player_max_hp: bs.player_max_hp,
            enemy_name:   bs.enemy_name,
            enemy_hp:     bs.enemy_hp,
            enemy_max_hp: bs.enemy_max_hp,
            circles,
            outcome: bs.outcome,
            result_cooldown_ms_left,
            captured: bs.captured,
            captured_name: bs.enemy_name,
            capture_is_upgrade: bs.capture_is_upgrade,
            capture_new_count:  bs.capture_new_count,
        })
    } else {
        None
    };

    // ── Encounter data ─────────────────────────────────────────────────────
    let enc_state = world.resource::<EncounterState>();
    let encounter = enc_state.0.as_ref().map(|e| {
        const TIMEOUT_MS: u128 = 10_000;
        let elapsed = e.shown_at.elapsed().as_millis();
        let ms_left = TIMEOUT_MS.saturating_sub(elapsed);
        let seconds_left = ((ms_left + 999) / 1000).min(10) as u8;
        EncounterRenderData {
            enemy_name:  e.enemy_name,
            enemy_level: e.enemy_level,
            enemy_atk:   e.enemy_atk,
            enemy_def:   e.enemy_def,
            enemy_hp:    e.enemy_hp,
            seconds_left,
        }
    });

    // ── Roster entities ────────────────────────────────────────────────────
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
                count:    e.get::<Count>().map_or(0, |c| c.0),
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
        roster_scroll,
        roster_hover,
        battle,
        encounter,
    }
}

// ─── Palette ──────────────────────────────────────────────────────────────────
const BG: Rgb888          = Rgb888::new(8, 8, 18);
const TITLE_BG: Rgb888    = Rgb888::new(15, 35, 110);
const CARD_ACTIVE: Rgb888 = Rgb888::new(15, 45, 18);
const CARD_IDLE: Rgb888   = Rgb888::new(18, 18, 40);
const WHITE: Rgb888        = Rgb888::WHITE;
const YELLOW: Rgb888       = Rgb888::new(255, 220, 0);
const ORANGE: Rgb888       = Rgb888::new(255, 140, 0);
const GREEN: Rgb888        = Rgb888::new(50, 200, 80);
const RED: Rgb888          = Rgb888::new(220, 60, 60);
const BLUE_BTN: Rgb888     = Rgb888::new(40, 80, 200);
const GRAY: Rgb888         = Rgb888::new(90, 90, 100);
const DARK_GRAY: Rgb888    = Rgb888::new(40, 40, 50);

// Circle colours
const HERO_FILL:   Rgb888 = Rgb888::new(20, 110, 50);
const HERO_RING:   Rgb888 = Rgb888::new(80, 255, 120);
const ENEMY_FILL:  Rgb888 = Rgb888::new(120, 20, 20);
const ENEMY_RING:  Rgb888 = Rgb888::new(255, 80, 80);

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn render_screen<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    data: &RenderData,
    encounter_sprite: Option<&Sprite>,
) {
    match data.screen {
        Screen::Overview  => render_overview(display, data),
        Screen::Encounter => render_encounter(display, data, encounter_sprite),
        Screen::Roster    => render_roster(display, data),
        Screen::Battle    => render_battle(display, data),
    }
}

// ─── Overview ────────────────────────────────────────────────────────────────

fn render_overview<D: DrawTarget<Color = Rgb888>>(display: &mut D, data: &RenderData) {
    fill_rect(display, 0, 0, 240, 284, BG);

    fill_rect(display, 0, 0, 240, 28, TITLE_BG);
    draw_text(display, "OVERVIEW", 62, 20, &FONT_10X20, YELLOW);

    let mon = &data.roster[data.active_slot];

    if mon.count > 0 {
        let name_plus = format!("{} +{}", mon.name, mon.count);
        draw_text(display, &name_plus, 10, 52, &FONT_10X20, WHITE);
    } else {
        draw_text(display, mon.name, 10, 52, &FONT_10X20, WHITE);
    }
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

// ─── Encounter ───────────────────────────────────────────────────────────────

fn render_encounter<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    data: &RenderData,
    sprite: Option<&Sprite>,
) {
    fill_rect(display, 0, 0, 240, 284, BG);

    // Title bar
    fill_rect(display, 0, 0, 240, 28, Rgb888::new(80, 20, 20));
    draw_text(display, "WILD ENCOUNTER!", 20, 20, &FONT_10X20, YELLOW);

    let Some(enc) = &data.encounter else {
        draw_text(display, "Looking...", 70, 140, &FONT_10X20, GRAY);
        return;
    };

    // Monster name + level
    draw_text(display, enc.enemy_name, 20, 70, &FONT_10X20, WHITE);
    let lv = format!("Lv.{}", enc.enemy_level);
    draw_text(display, &lv, 178, 70, &FONT_10X20, YELLOW);

    // Stats row
    let atk_s = format!("ATK: {}", enc.enemy_atk);
    let def_s = format!("DEF: {}", enc.enemy_def);
    let hp_s  = format!("HP:  {}", enc.enemy_hp);
    draw_text(display, &atk_s, 20,  94, &FONT_6X10, WHITE);
    draw_text(display, &def_s, 120, 94, &FONT_6X10, WHITE);
    draw_text(display, &hp_s,  20, 108, &FONT_6X10, WHITE);

    // Monster sprite or fallback silhouette
    match sprite {
        Some(spr) => {
            let sx = (240 - spr.width as i32) / 2;
            let sy = 118;
            spr.draw_with_bg(display, sx, sy, BG);
        }
        None => draw_monster_icon(display, 90, 130, ENEMY_RING),
    }

    // "Tap to battle!" prompt
    draw_text(display, "TAP TO BATTLE!", 30, 210, &FONT_10X20, GREEN);

    // Countdown bar + text
    let secs = enc.seconds_left.max(1);
    let bar_pct = (secs as u32 * 10) as u8; // 10s = 100%, 1s ≈ 10%
    let bar_color = if secs > 5 { GREEN } else if secs > 2 { YELLOW } else { RED };
    draw_bar(display, 20, 228, 200, 10, bar_pct, bar_color);
    let countdown = format!("{}s", secs);
    let cx = (240 - countdown.len() as i32 * 10) / 2;
    draw_text(display, &countdown, cx, 252, &FONT_10X20, bar_color);

    draw_text(display, "Tap=fight  Swipe down=flee", 8, 282, &FONT_6X10, DARK_GRAY);
}

// ─── Roster ──────────────────────────────────────────────────────────────────

fn render_roster<D: DrawTarget<Color = Rgb888>>(display: &mut D, data: &RenderData) {
    fill_rect(display, 0, 0, 240, 284, BG);

    fill_rect(display, 0, 0, 240, 28, TITLE_BG);
    // Show monster count in title
    let title = format!("ROSTER  {}/{}", data.active_slot + 1, data.roster.len());
    draw_text(display, &title, 10, 20, &FONT_10X20, YELLOW);

    const VISIBLE: usize = 3;
    let start = data.roster_scroll;
    let end = (start + VISIBLE).min(data.roster.len());

    for (visible_i, mon) in data.roster[start..end].iter().enumerate() {
        let slot = start + visible_i;
        let y = 32 + visible_i as i32 * 80;
        let is_active = slot == data.active_slot;

        let is_hovered = data.roster_hover == Some(slot);
        let (card_col, border_col, indicator) = if is_hovered {
            (Rgb888::new(40, 38, 10), YELLOW, ">")   // hover: yellow
        } else if is_active {
            (CARD_ACTIVE, GREEN, ">")                 // active: green
        } else {
            (CARD_IDLE, Rgb888::new(50, 50, 90), " ") // idle: dim
        };

        fill_rect(display, 8, y, 224, 72, card_col);
        stroke_rect(display, 8, y, 224, 72, border_col);

        let ind_color = if is_hovered { YELLOW } else { GREEN };
        draw_text(display, indicator, 12, y + 18, &FONT_10X20, ind_color);

        if mon.count > 0 {
            let name_plus = format!("{} +{}", mon.name, mon.count);
            draw_text(display, &name_plus, 26, y + 18, &FONT_10X20, WHITE);
        } else {
            draw_text(display, mon.name, 26, y + 18, &FONT_10X20, WHITE);
        }
        let lv = format!("Lv.{}", mon.level);
        draw_text(display, &lv, 180, y + 18, &FONT_10X20, YELLOW);

        let atk_s = format!("ATK:{}", mon.atk);
        let def_s = format!("DEF:{}", mon.def);
        draw_text(display, &atk_s, 26,  y + 36, &FONT_6X10, ORANGE);
        draw_text(display, &def_s, 100, y + 36, &FONT_6X10, BLUE_BTN);

        let hp_s = format!("HP {}/{}", mon.hp, mon.max_hp);
        draw_text(display, &hp_s, 26, y + 50, &FONT_6X10, WHITE);
        draw_bar(display, 26, y + 54, 192, 6, mon.hp_pct(), hp_bar_color(mon.hp_pct()));

        if mon.is_fainted() {
            draw_text(display, "FAINTED", 150, y + 50, &FONT_6X10, RED);
        }
    }

    // Scroll arrows
    if data.roster_scroll > 0 {
        draw_text(display, "^ more", 94, 32, &FONT_6X10, GRAY);
    }
    if end < data.roster.len() {
        draw_text(display, "v more", 94, 272, &FONT_6X10, GRAY);
    }

    draw_text(display, "Tap=hover  Tap again=select", 14, 282, &FONT_6X10, DARK_GRAY);
}

// ─── Battle (tap game) ───────────────────────────────────────────────────────
//
// Layout (240 × 284 px):
//   y=  0..10   Player name (left) │ Enemy name (right)
//   y= 11..19   HP bars side by side
//   y= 20..21   Separator line
//   y= 21..263  Circle tap-game area
//   y=264..284  Status bar (hint / outcome)

fn render_battle<D: DrawTarget<Color = Rgb888>>(display: &mut D, data: &RenderData) {
    fill_rect(display, 0, 0, 240, 284, BG);

    let Some(battle) = &data.battle else {
        draw_text(display, "Starting...", 70, 140, &FONT_10X20, GRAY);
        return;
    };

    // ── HP bar row ────────────────────────────────────────────────────────
    // Player (left half, x=0..117)
    let p_pct = pct(battle.player_hp, battle.player_max_hp);
    draw_text(display, battle.player_name, 2, 9, &FONT_6X10, HERO_RING);
    draw_bar(display, 2, 11, 111, 8, p_pct, hp_bar_color(p_pct));

    // Enemy (right half, x=127..238)
    let e_pct = pct(battle.enemy_hp, battle.enemy_max_hp);
    // Right-align enemy name
    let e_x = (238i32 - battle.enemy_name.len() as i32 * 6).max(127);
    draw_text(display, battle.enemy_name, e_x, 9, &FONT_6X10, ENEMY_RING);
    draw_bar(display, 127, 11, 111, 8, e_pct, hp_bar_color(e_pct));

    // VS divider in the centre gap
    draw_text(display, "VS", 113, 9, &FONT_6X10, GRAY);
    draw_hline(display, 0, 240, 21, GRAY);

    // ── Circle game area ──────────────────────────────────────────────────
    for circle in &battle.circles {
        let r  = circle.radius as i32;
        let cx = circle.cx as i32;
        let cy = circle.cy as i32;
        let (fill, ring) = match circle.kind {
            CircleKind::HeroAttack  => (HERO_FILL,  HERO_RING),
            CircleKind::EnemyAttack => (ENEMY_FILL, ENEMY_RING),
        };
        draw_circle(display, cx, cy, r, fill, ring);
    }

    // ── Status bar ────────────────────────────────────────────────────────
    fill_rect(display, 0, 264, 240, 20, Rgb888::new(15, 15, 25));
    draw_hline(display, 0, 240, 264, GRAY);

    match battle.outcome {
        Some(won) => {
            let (label, color) = if won {
                ("VICTORY!", GREEN)
            } else {
                ("DEFEAT...", RED)
            };
            // Large result text centred in the game area
            let box_h = if won && battle.captured { 80 } else { 64 };
            let lx = (240 - label.len() as i32 * 10) / 2;
            fill_rect(display, 0, 100, 240, box_h, Rgb888::new(10, 10, 20));
            stroke_rect(display, 4, 104, 232, box_h - 8, color);
            draw_text(display, label, lx, 130, &FONT_10X20, color);

            // Capture notification
            if won && battle.captured {
                let msg = if battle.capture_is_upgrade {
                    format!("{} upgraded to +{}!", battle.captured_name, battle.capture_new_count)
                } else {
                    format!("{} joined!", battle.captured_name)
                };
                let mx = (240 - msg.len() as i32 * 6) / 2;
                draw_text(display, &msg, mx, 158, &FONT_6X10, YELLOW);
            }

            if battle.result_cooldown_ms_left > 0 {
                // Show seconds remaining before the screen becomes dismissable
                let secs = (battle.result_cooldown_ms_left + 999) / 1000; // ceil
                let countdown = format!("Please wait {}s...", secs);
                let cx = (240 - countdown.len() as i32 * 6) / 2;
                draw_text(display, &countdown, cx, 278, &FONT_6X10, GRAY);
            } else {
                draw_text(display, "   Tap to continue   ", 20, 278, &FONT_6X10, color);
            }
        }
        None => {
            // Hint: green = tap to attack, red = tap to block
            draw_text(display, "GREEN=atk  RED=block", 18, 278, &FONT_6X10, GRAY);
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn pct(current: u16, max: u16) -> u8 {
    if max == 0 { return 0; }
    ((current as u32 * 100) / max as u32) as u8
}

fn hp_bar_color(pct: u8) -> Rgb888 {
    if pct > 50 { GREEN } else if pct > 25 { YELLOW } else { RED }
}

/// Draw a filled circle with a contrasting stroke ring.
fn draw_circle<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    cx: i32,
    cy: i32,
    r: i32,
    fill: Rgb888,
    ring: Rgb888,
) {
    if r <= 0 { return; }
    let diam     = (r * 2) as u32;
    let top_left = Point::new(cx - r, cy - r);
    let _ = Circle::new(top_left, diam)
        .into_styled(PrimitiveStyle::with_fill(fill))
        .draw(display);
    let _ = Circle::new(top_left, diam)
        .into_styled(PrimitiveStyle::with_stroke(ring, 2))
        .draw(display);
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
    fill_rect(display, x + 12, y,      26, 20, color);
    fill_rect(display, x + 16, y + 5,   6,  6, YELLOW);
    fill_rect(display, x + 28, y + 5,   6,  6, YELLOW);
    fill_rect(display, x +  8, y + 22, 34, 22, color);
    fill_rect(display, x,      y + 24,  8, 14, color);
    fill_rect(display, x + 42, y + 24,  8, 14, color);
    fill_rect(display, x + 12, y + 46, 10, 14, color);
    fill_rect(display, x + 28, y + 46, 10, 14, color);
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
