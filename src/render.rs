use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text, TextStyleBuilder},
};

use epd_waveshare::{
    color::*,
    epd2in13_v2::{Display2in13, Epd2in13},
    prelude::*,
};

use linux_embedded_hal::{Delay, SpidevDevice, SysfsPin};

use crate::models::hero::Personnage;
use crate::models::hero::Skill;
use crate::models::battle::Battle;
use crate::models::context::Context;
use crate::models::enemy::Enemy;

use image::{GenericImageView, DynamicImage};


pub fn draw_text(display: &mut Display2in13, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}

pub fn draw_bold_text(display: &mut Display2in13, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X13_BOLD)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}

pub fn draw_line(display: &mut Display2in13, x1: i32, y1: i32, x2: i32, y2: i32) {
    let _ = Line::new(Point::new(x1, y1), Point::new(x2, y2))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
        .draw(display);
}

pub fn render(
    epd2in13: &mut Epd2in13<SpidevDevice, SysfsPin, SysfsPin, SysfsPin, Delay>,
    display: &mut Display2in13,
    spi: &mut SpidevDevice,
    delay: &mut Delay,
    context: &Context,
) {
    // RefreshLut::Full
    // RefreshLut::Quick
    epd2in13
        .set_refresh(spi, delay, RefreshLut::Quick)
        .expect("set refresh");
    display.clear(Color::White).ok();
    draw_body(display, &context);
    draw_footer(display, &context);
    epd2in13
        .update_and_display_frame(spi, display.buffer(), delay)
        .expect("display frame new graphics");
}

fn draw_body(
    display: &mut Display2in13,
    context: &Context,
) {
    if context.action == "battle" || context.action == "battle_spell" {
        draw_battle(display, context);
    } else if context.action == "overview" {
        draw_hero(display, &context.hero);
    }
}

fn draw_battle(display: &mut Display2in13, context: &Context) {

    let battle = match &context.battle {
        Some(b) => b,
        None => return,
    };
    let enemy = match &context.enemy {
        Some(e) => e,
        None => return,
    };
    draw_character_info(
        display,
        &enemy.name,
        enemy.hp,
        enemy.max_hp,
        enemy.mp,
        enemy.max_mp,
        5,
        5,
    );
    let monster_data: Vec<u8> = std::fs::read("data/poring.bmp").unwrap();
    // const MONSTER: &[u8] = include_bytes!("./assets/poring/front.bmp");
    let monster_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(&monster_data).unwrap();
    Image::new(&monster_bmp, Point::new(120 - 40, 0))
        .draw(&mut display.color_converted())
        .unwrap();

    draw_text(display, &battle.message, 5, 75);

    if context.action != "battle_spell" {
        let hero_data: Vec<u8> = std::fs::read("data/back.bmp").unwrap();
        // const HERO: &[u8] = include_bytes!("./assets/novice/back.bmp");
        let hero_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(&hero_data).unwrap();
        Image::new(&hero_bmp, Point::new(0, 100))
            .draw(&mut display.color_converted())
            .unwrap();

        draw_character_info(
            display,
            &context.hero.nom,
            context.hero.hp,
            context.hero.max_hp,
            context.hero.mp,
            context.hero.max_mp,
            65,
            100,
        );
    }
}

fn draw_hero(display: &mut Display2in13, hero: &Personnage) {
    const START_X: i32 = 65;
    const START_Y: i32 = 5;
    const SPLASH: &[u8] = include_bytes!("./assets/novice/front.bmp");
    let splash_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(SPLASH).unwrap();
    Image::new(&splash_bmp, Point::zero())
        .draw(&mut display.color_converted())
        .unwrap();

    let hp_bar_width: f32 = 35.0;
    let hp = hero.hp as f32 / hero.max_hp as f32;
    let hp_value = (hp * hp_bar_width).round() as u32;

    draw_text(display, "Lyrocs", START_X, START_Y);
    draw_text(display, "Novice", START_X, START_Y + 10);
    // HP LINE
    draw_text(display, "HP:", START_X, START_Y + 20);
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
    Rectangle::new(Point::new(START_X + 20, START_Y + 23), Size::new(35, 5))
        .into_styled(style)
        .draw(display)
        .unwrap();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::Black)
        .build();
    Rectangle::new(
        Point::new(START_X + 20, START_Y + 23),
        Size::new(hp_value, 5),
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // SP LINE
    draw_text(display, "SP:", START_X, START_Y + 30);
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
    Rectangle::new(Point::new(START_X + 20, START_Y + 33), Size::new(35, 5))
        .into_styled(style)
        .draw(display)
        .unwrap();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::Black)
        .build();
    Rectangle::new(Point::new(START_X + 20, START_Y + 33), Size::new(30, 5))
        .into_styled(style)
        .draw(display)
        .unwrap();
}

fn draw_character_info(
    display: &mut Display2in13,
    name: &str,
    hp: u32,
    max_hp: u32,
    mp: u32,
    max_mp: u32,
    start_x: i32,
    start_y: i32,
) {
    let hp_bar_width: f32 = 35.0;
    let hp = hp as f32 / max_hp as f32;
    let hp_value = (hp * hp_bar_width).round() as u32;

    draw_text(display, name, start_x, start_y);
    // draw_text(display, "Novice", START_X, START_Y + 10);
    // HP LINE
    draw_text(display, "HP:", start_x, start_y + 20);
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
    Rectangle::new(Point::new(start_x + 20, start_y + 23), Size::new(35, 5))
        .into_styled(style)
        .draw(display)
        .unwrap();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::Black)
        .build();
    Rectangle::new(
        Point::new(start_x + 20, start_y + 23),
        Size::new(hp_value, 5),
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // SP LINE
    draw_text(display, "SP:", start_x, start_y + 30);
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
    Rectangle::new(Point::new(start_x + 20, start_y + 33), Size::new(35, 5))
        .into_styled(style)
        .draw(display)
        .unwrap();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::Black)
        .build();
    Rectangle::new(Point::new(start_x + 20, start_y + 33), Size::new(30, 5))
        .into_styled(style)
        .draw(display)
        .unwrap();
}

fn draw_footer(display: &mut Display2in13, context: &Context) {


    if context.action == "overview" {
        let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
        Rectangle::new(Point::new(0, 200), Size::new(122, 50))
            .into_styled(style)
            .draw(display)
            .unwrap();
        draw_line(display, 60, 200, 60, 250);
        draw_text(display, "Battle", 5, 225);
        draw_text(display, "Nothing", 65, 225);
        return;
    }
    if context.action == "battle" {
        let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
        Rectangle::new(Point::new(0, 200), Size::new(122, 50))
            .into_styled(style)
            .draw(display)
            .unwrap();
        draw_line(display, 60, 200, 60, 250);
        draw_text(display, "Attack", 5, 225);
        draw_text(display, "Spell", 65, 225);
        return;
    }

    let battle = match &context.battle {
        Some(b) => b,
        None => return,
    };
    if (context.action == "battle" || context.action == "battle_spell") && battle.turn != "hero" {
        return;
    }

    if context.action == "battle_spell" {
        let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
        Rectangle::new(Point::new(0, 100), Size::new(122, 150))
            .into_styled(style)
            .draw(display)
            .unwrap();
        draw_line(display, 0, 130, 122, 130);
        draw_line(display, 0, 160, 122, 160);
        draw_line(display, 0, 190, 122, 190);
        draw_line(display, 0, 220, 122, 220);

        let mut y = 100;
        for skill in context.hero.skills.iter() {
            draw_spell(display, skill, y);
            y += 30;
        }

        draw_text(display, "Back", 5, 225);
    }
}



fn draw_spell(display: &mut Display2in13, skill: &Skill, start_y: i32) {
    let icon_path = "data/".to_owned() + &skill.icon;
    let bash_data: Vec<u8> = std::fs::read(icon_path).unwrap();
    let bash_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(&bash_data).unwrap();
    Image::new(&bash_bmp, Point::new(3, start_y + 3))
        .draw(&mut display.color_converted())
        .unwrap();
    draw_bold_text(display, &skill.name, 35, start_y + 3);
    let lvl_x = 35 + 8 + 6 * skill.name.len() as i32;
    let lvl_text = format!("{}{}", "lvl", skill.level);
    draw_text(display, lvl_text.as_str(), lvl_x, start_y + 5);
    draw_text(display, &skill.description, 35, start_y + 3 + 13 + 2);

    // Push MP text to the right
    // screen Width = 122 - border
    let mp_text = format!("{}{}", skill.mp_cost, "SP");
    let mp_x = 121 - 6 * mp_text.len() as i32;
    draw_text(display, mp_text.as_str(), mp_x, start_y + 3 + 13 + 2);
}
