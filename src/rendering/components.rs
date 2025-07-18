use crate::rendering::primitives;
use crate::models::hero::Skill;
use crate::models::context::Context;
use crate::models::context::Action;
use crate::ui::generate_cta;
use epd_waveshare::{
    color::*,
    epd2in13_v2::{Display2in13},
    prelude::*,
};
use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text, TextStyleBuilder},
};

pub fn draw_character_info(
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

    primitives::draw_text(display, name, start_x, start_y);
    // draw_text(display, "Novice", START_X, START_Y + 10);
    // HP LINE
    primitives::draw_text(display, "HP:", start_x, start_y + 20);
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
    let mp_bar_width: f32 = 35.0;
    let mp = mp as f32 / max_mp as f32;
    let mp_value = (mp * mp_bar_width).round() as u32;
    primitives::draw_text(display, "SP:", start_x, start_y + 30);
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
    Rectangle::new(Point::new(start_x + 20, start_y + 33), Size::new(mp_bar_width as u32, 5))
        .into_styled(style)
        .draw(display)
        .unwrap();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::Black)
        .build();
    Rectangle::new(Point::new(start_x + 20, start_y + 33), Size::new(mp_value, 5))
        .into_styled(style)
        .draw(display)
        .unwrap();
}

pub fn draw_spell(display: &mut Display2in13, skill: &Skill, start_y: i32) {
    let icon_path = "data/".to_owned() + &skill.icon;
    let bash_data: Vec<u8> = std::fs::read(icon_path).unwrap();
    let bash_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(&bash_data).unwrap();
    Image::new(&bash_bmp, Point::new(3, start_y + 3))
        .draw(&mut display.color_converted())
        .unwrap();
    primitives::draw_bold_text(display, &skill.name, 35, start_y + 3);
    let lvl_x = 35 + 8 + 6 * skill.name.len() as i32;
    let lvl_text = format!("{}{}", "lvl", skill.level);
    primitives::draw_text(display, lvl_text.as_str(), lvl_x, start_y + 5);
    primitives::draw_text(display, &skill.description, 35, start_y + 3 + 13 + 2);

    // Push MP text to the right
    // screen Width = 122 - border
    let mp_text = format!("{}{}", skill.mp_cost, "SP");
    let mp_x = 121 - 6 * mp_text.len() as i32;
    primitives::draw_text(display, mp_text.as_str(), mp_x, start_y + 3 + 13 + 2);
}

pub fn draw_modal(display: &mut Display2in13, title: &str) {
    primitives::draw_rectangle(display, 0, 12, 122, 235);
    primitives::draw_text_center(display, title, 8);
}

pub fn draw_ctas(display: &mut Display2in13, context: &Context) {
    let cta = generate_cta(&context);
    for cta in cta.iter() {
        primitives::draw_rectangle(display, cta.x, cta.y, cta.width, cta.height);
        let text_y = (cta.y + cta.height / 2) - 5;
        if cta.action == Action::Skill {
            let skill = context.hero.skills.iter().find(|skill| skill.id == cta.id.unwrap() as u32).unwrap();
            draw_spell(display, skill, cta.y);
        } else {
            primitives::draw_text_center_by_width(display, cta.label.as_str(), text_y, cta.x, cta.width);
        }
    }
}