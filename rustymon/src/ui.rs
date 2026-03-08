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

use crate::game::{GameState, MenuCursor, Screen};

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

pub fn render_screen<D: DrawTarget<Color = Rgb888>>(display: &mut D, state: &GameState) {
    match &state.screen {
        Screen::Overview => render_overview(display, state),
        Screen::Roster => render_roster(display, state),
        Screen::Battle => render_battle(display, state),
    }
}

// ─── Overview ────────────────────────────────────────────────────────────────

fn render_overview<D: DrawTarget<Color = Rgb888>>(display: &mut D, state: &GameState) {
    fill_rect(display, 0, 0, 240, 284, BG);

    // Title bar
    fill_rect(display, 0, 0, 240, 28, TITLE_BG);
    draw_text(display, "OVERVIEW", 62, 20, &FONT_10X20, YELLOW);

    let mon = state.active_rustymon();

    // Name + level
    draw_text(display, mon.name, 10, 52, &FONT_10X20, WHITE);
    let lv = format!("Lv.{}", mon.level);
    draw_text(display, &lv, 178, 52, &FONT_10X20, YELLOW);

    // Horizontal divider
    draw_hline(display, 10, 230, 59, GRAY);

    // HP
    let hp_label = format!("HP  {}/{}", mon.hp, mon.max_hp);
    draw_text(display, &hp_label, 10, 76, &FONT_6X10, WHITE);
    draw_bar(display, 10, 80, 220, 10, mon.hp_pct(), hp_bar_color(mon.hp_pct()));

    // ATK / DEF
    let atk_s = format!("ATK: {}", mon.atk);
    let def_s = format!("DEF: {}", mon.def);
    draw_text(display, &atk_s, 10, 106, &FONT_6X10, WHITE);
    draw_text(display, &def_s, 128, 106, &FONT_6X10, WHITE);

    // EXP
    let exp_s = format!("EXP {}/{}", mon.exp, mon.exp_next);
    draw_text(display, &exp_s, 10, 124, &FONT_6X10, GRAY);
    draw_bar(display, 10, 128, 220, 6, mon.exp_pct(), ORANGE);

    // Divider
    draw_hline(display, 10, 230, 146, GRAY);

    // Simple monster icon centred between dividers
    draw_monster_icon(display, 96, 158, GRAY);

    // Buttons
    let b_sel = state.cursor == MenuCursor::Battle;
    let r_sel = state.cursor == MenuCursor::Roster;
    draw_button(display, 14, 244, 96, 30, "BATTLE", b_sel);
    draw_button(display, 130, 244, 96, 30, "ROSTER", r_sel);

    // Hint
    draw_text(display, "Tap btn | Swipe< > | SwipeUp", 6, 282, &FONT_6X10, DARK_GRAY);
}

// ─── Roster ──────────────────────────────────────────────────────────────────

fn render_roster<D: DrawTarget<Color = Rgb888>>(display: &mut D, state: &GameState) {
    fill_rect(display, 0, 0, 240, 284, BG);

    fill_rect(display, 0, 0, 240, 28, TITLE_BG);
    draw_text(display, "ROSTER", 76, 20, &FONT_10X20, YELLOW);

    for (i, mon) in state.roster.iter().enumerate() {
        let y = 32 + i as i32 * 80;
        let is_active = i == state.active;

        let card_col = if is_active { CARD_ACTIVE } else { CARD_IDLE };
        let border_col = if is_active { GREEN } else { Rgb888::new(50, 50, 90) };

        fill_rect(display, 8, y, 224, 72, card_col);
        stroke_rect(display, 8, y, 224, 72, border_col);

        // Active marker
        if is_active {
            draw_text(display, ">", 12, y + 18, &FONT_10X20, GREEN);
        }

        draw_text(display, mon.name, 26, y + 18, &FONT_10X20, WHITE);
        let lv = format!("Lv.{}", mon.level);
        draw_text(display, &lv, 180, y + 18, &FONT_10X20, YELLOW);

        let hp_s = format!("HP {}/{}", mon.hp, mon.max_hp);
        draw_text(display, &hp_s, 26, y + 38, &FONT_6X10, WHITE);
        draw_bar(display, 26, y + 42, 192, 8, mon.hp_pct(), hp_bar_color(mon.hp_pct()));

        // Fainted indicator
        if mon.is_fainted() {
            draw_text(display, "FAINTED", 150, y + 38, &FONT_6X10, RED);
        }
    }

    draw_text(display, "Tap or swipe to go back", 6, 282, &FONT_6X10, DARK_GRAY);
}

// ─── Battle ──────────────────────────────────────────────────────────────────

fn render_battle<D: DrawTarget<Color = Rgb888>>(display: &mut D, state: &GameState) {
    fill_rect(display, 0, 0, 240, 284, BG);

    fill_rect(display, 0, 0, 240, 28, BATTLE_BG);
    draw_text(display, "BATTLE", 76, 20, &FONT_10X20, RED);

    let Some(battle) = &state.battle else {
        draw_text(display, "No battle data", 30, 150, &FONT_10X20, GRAY);
        return;
    };

    let visible = state.battle_lines_shown.min(battle.log.len());

    // How many lines fit from y=32 down to y=266 with 13px line height
    const MAX_VISIBLE: usize = 18;
    let start = if visible > MAX_VISIBLE { visible - MAX_VISIBLE } else { 0 };

    for (slot, idx) in (start..visible).enumerate() {
        let y = 42 + slot as i32 * 13;
        let line = &battle.log[idx];
        let color = log_line_color(line);
        draw_text(display, line, 4, y, &FONT_6X10, color);
    }

    // Bottom status bar when finished
    if state.battle_is_done() {
        fill_rect(display, 0, 268, 240, 16, Rgb888::new(30, 30, 30));
        let (msg, col) = if battle.player_won {
            ("VICTORY!  Tap to return", GREEN)
        } else {
            ("DEFEAT...  Tap to return", RED)
        };
        draw_text(display, msg, 6, 280, &FONT_6X10, col);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn hp_bar_color(pct: u8) -> Rgb888 {
    if pct > 50 {
        GREEN
    } else if pct > 25 {
        YELLOW
    } else {
        RED
    }
}

fn log_line_color(line: &str) -> Rgb888 {
    if line.contains("WIN") || line.contains("EXP") {
        GREEN
    } else if line.contains("LOSE") || line.contains("fainted") {
        RED
    } else if line.starts_with("---") {
        YELLOW
    } else if line.starts_with("VS") || line.starts_with("You:") || line.starts_with("Foe:") {
        ORANGE
    } else {
        WHITE
    }
}

fn draw_button<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    x: i32, y: i32, w: u32, h: u32,
    label: &str,
    selected: bool,
) {
    let bg = if selected { BLUE_BTN } else { Rgb888::new(28, 28, 46) };
    let border = if selected { WHITE } else { GRAY };
    fill_rect(display, x, y, w, h, bg);
    stroke_rect(display, x, y, w, h, border);
    // Centre label: FONT_6X10 char width=6
    let text_x = x + (w as i32 - label.len() as i32 * 6) / 2;
    let text_y = y + h as i32 / 2 + 4; // approx baseline centre
    draw_text(display, label, text_x, text_y, &FONT_6X10, WHITE);
}

fn draw_bar<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    x: i32, y: i32, w: u32, h: u32,
    pct: u8,
    color: Rgb888,
) {
    fill_rect(display, x, y, w, h, DARK_GRAY);
    let filled = (w * pct as u32 / 100).max(if pct > 0 { 1 } else { 0 });
    if filled > 0 {
        fill_rect(display, x, y, filled, h, color);
    }
    stroke_rect(display, x, y, w, h, GRAY);
}

fn draw_monster_icon<D: DrawTarget<Color = Rgb888>>(display: &mut D, x: i32, y: i32, color: Rgb888) {
    let eye = YELLOW;
    // Head
    fill_rect(display, x + 12, y, 26, 20, color);
    // Eyes
    fill_rect(display, x + 16, y + 5, 6, 6, eye);
    fill_rect(display, x + 28, y + 5, 6, 6, eye);
    // Body
    fill_rect(display, x + 8, y + 22, 34, 22, color);
    // Arms
    fill_rect(display, x,      y + 24, 8, 14, color);
    fill_rect(display, x + 42, y + 24, 8, 14, color);
    // Legs
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
    display: &mut D,
    text: &str,
    x: i32, y: i32,
    font: &MonoFont<'_>,
    color: Rgb888,
) {
    let style = MonoTextStyle::new(font, color);
    let _ = Text::new(text, Point::new(x, y), style).draw(display);
}
