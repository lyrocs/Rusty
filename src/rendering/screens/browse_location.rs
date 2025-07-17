use crate::models::context::Context;
use crate::models::context::Action;
use crate::rendering::components;
use crate::rendering::primitives;
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
    components::draw_modal(display, "Browse Location", "Fight");
    primitives::draw_text(display, context.location.name.as_str(), 65, 5);
}
