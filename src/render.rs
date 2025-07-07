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

use crate::hero::Personnage;

use crate::context::Context;

pub fn draw_text(display: &mut Display2in13, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
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
    hero: &Personnage,
) {
    epd2in13
        .set_refresh(spi, delay, RefreshLut::Quick)
        .expect("set refresh");
    display.clear(Color::White).ok();
    draw_body(display, &context, &hero);
    draw_footer(display);
    epd2in13
        .update_and_display_frame(spi, display.buffer(), delay)
        .expect("display frame new graphics");
}

fn draw_body(display: &mut Display2in13, context: &Context, hero: &Personnage) {
    if context.action == "battle" {
        draw_battle(display);
    } else if context.action == "overview" {
        draw_hero(display, hero);
    }
}

fn draw_battle(display: &mut Display2in13) {
    const MONSTER: &[u8] = include_bytes!("./assets/poring/front.bmp");
    let monster_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(MONSTER).unwrap();
    Image::new(&monster_bmp, Point::new(120 - 40, 0))
        .draw(&mut display.color_converted())
        .unwrap();

    const HERO: &[u8] = include_bytes!("./assets/novice/back.bmp");
    let hero_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(HERO).unwrap();
    Image::new(&hero_bmp, Point::new(0, 100))
        .draw(&mut display.color_converted())
        .unwrap();
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

fn draw_footer(display: &mut Display2in13) {
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
}
