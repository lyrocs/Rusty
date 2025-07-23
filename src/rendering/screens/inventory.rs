use crate::models::context::Context;
use crate::models::hero::InventoryItem;
use crate::rendering::primitives;
use crate::rendering::components;
use epd_waveshare::epd2in13_v2::Display2in13;
pub fn draw_inventory(display: &mut Display2in13, context: &Context) {
    components::draw_ctas(display, context);
}
