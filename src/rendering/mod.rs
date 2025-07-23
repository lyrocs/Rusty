mod primitives;
mod screens;
mod components;
use crate::models::context::{AutoCombatState, ManualCombatState, Context};
use crate::models::eink::Eink;
use crate::models::context::Activity;
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

pub fn render(
    eink: &mut Eink,
    context: &mut Context,
) {
    if !context.needs_redraw {
        return;
    }
    // RefreshLut::Quick
    // RefreshLut::Quick
    context.needs_redraw = false;
    eink.epd2in13
    .set_refresh(&mut eink.spi, &mut eink.delay, RefreshLut::Quick)
    .expect("set refresh");
    eink.display.clear(Color::White).ok();

    match &context.activity {
        Activity::HeroOverview => {
            screens::hero::draw_hero(&mut eink.display, &context);
        }
        Activity::AutoCombat(AutoCombatState::Searching { end_time }) => {
            screens::auto_battle::draw_searching(&mut eink.display, &context);
        }
        Activity::AutoCombat(AutoCombatState::Fighting) => {
            screens::auto_battle::draw_fighting(&mut eink.display, &context);
        }
        Activity::ManualCombat(ManualCombatState::Overview) => {
            screens::manual_battle::draw_fighting(&mut eink.display, &context);
        }
        Activity::ManualCombat(ManualCombatState::SelectingSkill) => {
            screens::manual_battle::draw_selecting_skill(&mut eink.display, &context);
        }
        Activity::ManualCombat(ManualCombatState::Result { rewards }) => {
            screens::manual_battle::draw_result(&mut eink.display, &context, rewards);
        }
        Activity::BrowseLocation => {
            screens::browse_location::draw_browse_location(&mut eink.display, &context);
        }
        Activity::AutoCombat(AutoCombatState::Dead { end_time }) => {
            screens::auto_battle::draw_dead(&mut eink.display, &context);
        }
        Activity::Inventory => {
            screens::inventory::draw_inventory(&mut eink.display, &context);
        }
        _ => {
            println!("Activity no found");
        }
    }
    eink.epd2in13
    .update_and_display_frame(&mut eink.spi, eink.display.buffer(), &mut eink.delay)
    .expect("display frame new graphics");
}