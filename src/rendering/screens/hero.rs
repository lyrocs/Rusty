
use crate::models::context::Context;
use crate::models::eink::Eink;
use crate::rendering::primitives;
use crate::rendering::components;
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
    epd2in13_v2::{Display2in13},
    prelude::*,
};

pub fn draw_hero(display: &mut Display2in13, 
    context: &Context) {
    const START_X: i32 = 65;
    const START_Y: i32 = 5;
    const SPLASH: &[u8] = include_bytes!("../../assets/novice/front.bmp");
    let splash_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(SPLASH).unwrap();
    Image::new(&splash_bmp, Point::zero())
        .draw(&mut display.color_converted())
        .unwrap();

    let hp_bar_width: f32 = 35.0;
    let hp = context.hero.hp as f32 / context.hero.max_hp as f32;
    let hp_value = (hp * hp_bar_width).round() as u32;

    primitives::draw_text(display, "Lyrocs", START_X, START_Y);
    primitives::draw_text(display, "Novice", START_X, START_Y + 10);
    // HP LINE
    primitives::draw_text(display, "HP:", START_X, START_Y + 20);
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
    primitives::draw_text(display, "SP:", START_X, START_Y + 30);
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

    let level = format!("{} / {}", context.hero.base_level, context.hero.job_level);
    primitives::draw_text(display, level.as_str(), START_X, START_Y + 40);
    

    // inventary
    let mut y = START_Y + 120;
    for item in context .hero.inventaire.iter() {
        let item_name = format!("{} x {}", item.name, item.quantity);
        primitives::draw_text_center(display, item_name.as_str(), y);
        y += 15;
    }  

    components::draw_ctas(display, context);
}
