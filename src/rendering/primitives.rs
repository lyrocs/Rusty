use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::{Baseline, Text, TextStyleBuilder},
};

use epd_waveshare::{
    color::*,
    epd2in13_v2::{Display2in13},
    prelude::*,
};

pub fn draw_line(display: &mut Display2in13, x1: i32, y1: i32, x2: i32, y2: i32) {
    let _ = Line::new(Point::new(x1, y1), Point::new(x2, y2))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
        .draw(display);
}
pub fn draw_bold_line(display: &mut Display2in13, x1: i32, y1: i32, x2: i32, y2: i32) {
    let _ = Line::new(Point::new(x1, y1), Point::new(x2, y2))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 2))
        .draw(display);
}

pub fn draw_text(display: &mut Display2in13, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}


pub fn draw_text_center(display: &mut Display2in13, text: &str, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    let text_width = (text.len() * 6) as i32;
    let _ = Text::with_text_style(text, Point::new((122 - text_width) / 2, y), style, text_style).draw(display);
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

pub fn draw_rectangle(display: &mut Display2in13, x: i32, y: i32, width: i32, height: i32) {
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();
    Rectangle::new(Point::new(x, y), Size::new(width as u32, height as u32))
        .into_styled(style)
        .draw(display)
        .unwrap();
}

pub fn draw_image(display: &mut Display2in13, image: &str, x: i32, y: i32) {
    let path = format!("data/{}.bmp", image);
    let hero_data: Vec<u8> = std::fs::read(path).unwrap();
    // const HERO: &[u8] = include_bytes!("./assets/novice/back.bmp");
    let hero_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(&hero_data).unwrap();
    Image::new(&hero_bmp, Point::new(x , y ))
        .draw(&mut display.color_converted())
        .unwrap();

}
pub fn draw_rounded_rectangle(display: &mut Display2in13, x: i32, y: i32, width: i32, height: i32) {
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(2)
        .fill_color(Color::White)
        .build();
    let _ = RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(x, y), Size::new(width as u32, height as u32)),
        Size::new(10, 10),
    ).into_styled(style)
        .draw(display)
        .unwrap();
}