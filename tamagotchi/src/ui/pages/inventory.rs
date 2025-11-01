use core::fmt::Write;
use embedded_graphics::{
    image::Image,
    mono_font::ascii::{FONT_9X15, FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;
use tinygif::Gif;

use crate::core::GameState;
use super::super::helpers::*;
use super::super::colors::*;

/// Draw inventory page showing all collected items with icons
pub fn draw_inventory<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear display
    display.clear(COLOR_BG)?;

    // Draw farming header if active
    use crate::ui::farming_header::draw_farming_header;
    let has_farming_header = draw_farming_header(display, game_state)?;
    let title_y = if has_farming_header { 40 } else { 20 };

    // Header
    draw_text(
        display,
        "=== INVENTORY ===",
        Point::new(85, title_y),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Draw item list
    let inventory = &game_state.hero.inventory;

    if inventory.is_empty() {
        draw_text(
            display,
            "No items yet!",
            Point::new(110, 200),
            &FONT_10X20,
            COLOR_TEXT_DIM,
        )?;
        draw_text(
            display,
            "Farm monsters to collect items",
            Point::new(35, 230),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    } else {
        // Draw items in a 2-column list with icon + name + quantity
        const COLS: usize = 2;
        const ICON_SIZE: i32 = 32;
        const ROW_HEIGHT: i32 = 40;
        const START_Y: i32 = 60;
        const COL_WIDTH: i32 = 180;
        const LEFT_COL_X: i32 = 10;
        const RIGHT_COL_X: i32 = 190;

        let items_per_page = 20; // 10 rows × 2 columns
        let offset = game_state.inventory_scroll_offset;

        // Get items for current page
        let page_items: heapless::Vec<_, 20> = inventory
            .iter()
            .skip(offset)
            .take(items_per_page)
            .collect();

        for (i, item) in page_items.iter().enumerate() {
            let col = i % COLS;
            let row = i / COLS;

            let x = if col == 0 { LEFT_COL_X } else { RIGHT_COL_X };
            let y = START_Y + (row as i32 * ROW_HEIGHT);

            // Try to load and draw item icon (first frame only)
            let icon_x = x;
            let icon_y = y;

            let icon_loaded = (|| -> Result<(), ()> {
                use crate::data::items::get_item_icon;

                // Get compiled-in icon data
                let gif_data = get_item_icon(item.id).ok_or(())?;
                let gif: Gif<Rgb888> = Gif::from_slice(gif_data).map_err(|_| ())?;

                // Draw the first frame of the GIF
                if let Some(frame) = gif.frames().next() {
                    Image::new(&frame, Point::new(icon_x, icon_y)).draw(display).map_err(|_| ())?;
                }
                Ok(())
            })().is_ok();

            // Draw item text (name + quantity)
            let text_x = x + ICON_SIZE + 5;
            let text_y = y + 22; // Vertically centered with icon

            if !icon_loaded {
                // If no icon, show a placeholder bullet
                draw_text(
                    display,
                    "•",
                    Point::new(x, text_y),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
            }

            // Build item text: "Name x123"
            let mut item_text = String::<48>::new();
            let name_len = item.name.len().min(12); // Limit name length
            let name_slice = &item.name[..name_len];
            write!(item_text, "{} x{}", name_slice, item.quantity).ok();

            draw_text(
                display,
                &item_text,
                Point::new(text_x, text_y),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
        }

        // Navigation buttons and page info
        let total_items = inventory.len();
        let total_pages = (total_items + items_per_page - 1) / items_per_page;
        let current_page = (offset / items_per_page) + 1;

        // Page info
        if total_pages > 1 {
            let mut page_text = String::<32>::new();
            write!(page_text, "Page {} of {}", current_page, total_pages).ok();
            draw_text(
                display,
                &page_text,
                Point::new(130, 450),
                &FONT_9X15,
                COLOR_TEXT,
            )?;

            // UP button (previous page)
            if current_page > 1 {
                Rectangle::new(Point::new(30, 460), Size::new(80, 40))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 120)))
                    .draw(display)?;
                Rectangle::new(Point::new(30, 460), Size::new(80, 40))
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 120, 160), 2))
                    .draw(display)?;
                draw_text(
                    display,
                    "UP",
                    Point::new(50, 485),
                    &FONT_9X15,
                    Rgb888::WHITE,
                )?;
            }

            // DOWN button (next page)
            if current_page < total_pages {
                Rectangle::new(Point::new(250, 460), Size::new(80, 40))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 120)))
                    .draw(display)?;
                Rectangle::new(Point::new(250, 460), Size::new(80, 40))
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 120, 160), 2))
                    .draw(display)?;
                draw_text(
                    display,
                    "DOWN",
                    Point::new(260, 485),
                    &FONT_9X15,
                    Rgb888::WHITE,
                )?;
            }
        }
    }

    // Footer
    draw_text(
        display,
        "Tap item for details | Back",
        Point::new(60, 510),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

