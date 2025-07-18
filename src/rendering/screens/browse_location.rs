use crate::models::context::Context;
use crate::models::context::Action;
use crate::rendering::components;
use crate::rendering::primitives;
use crate::ui::generate_cta;
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

pub fn draw_browse_location(display: &mut Display2in13, context: &Context) {
    components::draw_modal(display, "Browse Location");
    primitives::draw_text(display, "Fight", 15, 230);
    primitives::draw_text(display, "Menu", 75, 230);
    primitives::draw_line(display, 60, 220, 60, 246);
    primitives::draw_text_center(display, context.location.name.as_str(), 35);
    if context.location.enemies.is_empty() {
        primitives::draw_text_center(display, "No enemies", 75);
    } else {
        let mut y = 75;
        for enemy in context.location.enemies.iter() {
            primitives::draw_text_center(display, enemy.name.as_str(), y);
            y += 30;
        }
    }
    let mut y = 200;
    for connection in context.location.connections.iter() {
        primitives::draw_text_center(display, connection.label.as_str(), y);
        y -= 30;
    }

    components::draw_ctas(display, context);
}
