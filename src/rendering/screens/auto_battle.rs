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

pub fn draw_searching(display: &mut Display2in13, context: &Context) {
    primitives::draw_text(display, "Searching for enemy", 65, 5);
}

pub fn draw_fighting(display: &mut Display2in13, context: &Context) {
    components::draw_character_info(
        display,
        &context.enemy.name,
        context.enemy.hp,
        context.enemy.max_hp,
        0,
        0,
        5,
        5,
    );
    primitives::draw_image(display, "poring", 120 - 40, 0);
    primitives::draw_text(display, &context.battle.message, 5, 75);
    primitives::draw_image(display, "back", 0, 100);
    components::draw_character_info(
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
