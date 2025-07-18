use crate::models::context::Context;
use crate::models::context::Action;
use crate::models::context::LootItem;
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
    // Footer
    primitives::draw_rectangle(display, 0, 200, 122, 50);
    primitives::draw_line(display, 60, 200, 60, 250);
    primitives::draw_text(display, "Attack", 5, 225);
    primitives::draw_text(display, "Spell", 65, 225);
}

pub fn draw_selecting_skill(display: &mut Display2in13, context: &Context) {
    primitives::draw_rectangle(display, 0, 100, 122, 150);
    primitives::draw_line(display, 0, 130, 122, 130);
    primitives::draw_line(display, 0, 160, 122, 160);
    primitives::draw_line(display, 0, 190, 122, 190);
    primitives::draw_line(display, 0, 220, 122, 220);

    let mut y = 100;
    for skill in context.hero.skills.iter() {
        components::draw_spell(display, skill, y);
        y += 30;
    }
    primitives::draw_text(display, "Back", 5, 225);
}

pub fn draw_result(display: &mut Display2in13, context: &Context, rewards: &Vec<LootItem>) {
    components::draw_modal(display, "Fight result", "Confirm");
    let mut y = 75;
    for reward in rewards.iter() {
        let item_format = format!("{} x {}", reward.name, reward.quantity);
        primitives::draw_text(display, &item_format, 5, y);
        y += 30;
    }
}

