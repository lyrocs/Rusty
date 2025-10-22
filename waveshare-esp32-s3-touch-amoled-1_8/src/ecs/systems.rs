use bevy_ecs::prelude::*;
use embedded_graphics::{prelude::*, pixelcolor::Rgb888, image::Image, primitives::Rectangle};
use ft3x68_rs::{TouchState, TouchPoint};
use tinygif::Gif;
use tinybmp::Bmp;
use sh8601_rs::ColorMode;

use crate::ecs::resources::*;
use crate::game::{update_game_of_life, RESET_AFTER_GENERATIONS};
use crate::ui::{render_gif_optimized, write_generation, write_fps, write_battery, write_pwr_button, voltage_to_battery_percent, GifResource};

pub const DEBOUNCE_THRESHOLD: u8 = 3; // Number of consecutive readings needed to confirm button press

/// Button system to handle display on/off toggle
/// Supports both BOOT (GPIO0) and PWR (GPIO10) buttons
pub fn button_system(
    mut button_res: NonSendMut<ButtonResource>,
    mut display_res: NonSendMut<DisplayResource>,
    mut game: ResMut<GameOfLifeResource>,
) {
    // --- BOOT Button (GPIO0) - Active Low ---
    let boot_pressed = button_res.boot_button.is_low();

    // Debouncing logic for BOOT
    if boot_pressed {
        if button_res.boot_debounce_counter < DEBOUNCE_THRESHOLD {
            button_res.boot_debounce_counter += 1;
        }
    } else {
        button_res.boot_debounce_counter = 0;
    }

    // Detect rising edge (button release after being pressed)
    if button_res.boot_last_state && !boot_pressed && button_res.boot_debounce_counter == 0 {
        toggle_display(&mut display_res, &mut game, "BOOT");
    }

    // Update BOOT last state
    button_res.boot_last_state = button_res.boot_debounce_counter >= DEBOUNCE_THRESHOLD;

    // --- PWR Button (EXIO4 via TCA9554PWR) - Active Low (button pressed = LOW) ---
    let pwr_pin_state = button_res.gpio_expander.read_pin(4).unwrap_or(false); // EXIO4 = pin 4
    let pwr_low = !pwr_pin_state; // Active low: pressed = false (LOW), released = true (HIGH)

    // PWR button is Active Low: pressed = LOW, released = HIGH
    let pwr_pressed = pwr_low;

    // Debouncing logic for PWR
    if pwr_pressed {
        if button_res.pwr_debounce_counter < DEBOUNCE_THRESHOLD {
            button_res.pwr_debounce_counter += 1;
        }
    } else {
        button_res.pwr_debounce_counter = 0;
    }

    // Detect rising edge for PWR button
    if button_res.pwr_last_state && !pwr_pressed && button_res.pwr_debounce_counter == 0 {
        toggle_display(&mut display_res, &mut game, "PWR");
    }

    // Update PWR last state
    button_res.pwr_last_state = button_res.pwr_debounce_counter >= DEBOUNCE_THRESHOLD;

    // Debug: Print PWR button state every 10 frames for more frequent monitoring
    if game.generation % 10 == 0 {
        esp_println::println!("=== BUTTON DEBUG ===");
        esp_println::println!(
            "PWR Button (GPIO10): {} (LOW: {}, HIGH: {})",
            if pwr_pressed { "PRESSED" } else { "RELEASED" },
            pwr_low,
            pwr_pin_state
        );
        esp_println::println!(
            "BOOT Button (GPIO0): {} (LOW: {}, HIGH: {})",
            if boot_pressed { "PRESSED" } else { "RELEASED" },
            button_res.boot_button.is_low(),
            button_res.boot_button.is_high()
        );
        esp_println::println!("===================");
    }
}

/// Helper function to toggle display state
fn toggle_display(
    display_res: &mut NonSendMut<DisplayResource>,
    game: &mut ResMut<GameOfLifeResource>,
    button_name: &str,
) {
    // Toggle display state
    game.display_on = !game.display_on;

    // Apply the change to the display
    if game.display_on {
        esp_println::println!("{} Button: Turning display ON", button_name);
        display_res.display.display_on().ok();
    } else {
        esp_println::println!("{} Button: Turning display OFF", button_name);
        display_res.display.display_off().ok();
    }
}

/// System to update Game of Life logic
pub fn update_game_of_life_system(
    mut game: ResMut<GameOfLifeResource>,
    mut rng_res: ResMut<RngResource>,
) {
    // Create a temporary copy of the grid to avoid borrowing issues
    let temp_grid = game.grid;
    update_game_of_life(&temp_grid, &mut game.next_grid);

    // Swap the grids by copying instead of using mem::swap to avoid borrowing issues
    let temp = game.grid;
    game.grid = game.next_grid;
    game.next_grid = temp;

    game.generation += 1;

    if game.generation >= RESET_AFTER_GENERATIONS {
        crate::game::randomize_grid(&mut rng_res.0, &mut game.grid);
        game.generation = 0;
    }
}

/// Handles touch input and updates GIF position
fn handle_touch_input(touch_res: &mut TouchResource, gif_res: &mut GifResource) {
    let touching = touch_res
        .touch
        .touch1()
        .unwrap_or_else(|_e| TouchState::Released);

    if let TouchState::Pressed(TouchPoint { x, y }) = touching {
        gif_res.previous_position = gif_res.position;
        gif_res.position = Point::new(x as i32, y as i32);
    }
}

/// Renders the initial background with BMP image and first GIF frame
fn render_initial_background<D>(
    display: &mut D,
    background: &Bmp<Rgb888>,
    gif_res: &mut GifResource,
    gif_data: &[u8],
    generation: usize,
    fps: usize,
    battery_mv: u16,
    battery_pct: u8,
    pwr_pressed: bool,
) where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(Rgb888::BLACK).ok();
    Image::new(background, Point::new(0, 0)).draw(display).ok();

    // Draw initial GIF frame
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");
    if let Some(first_frame) = gif.frames().next() {
        Image::new(&first_frame, gif_res.position)
            .draw(display)
            .ok();
    }
    gif_res.first_render = false;
    gif_res.frame_index = 0;

    // Draw initial text overlays
    write_generation(display, generation).ok();
    write_fps(display, fps).ok();
    write_battery(display, battery_mv, battery_pct).ok();
    // For initial render, we don't have the raw values yet, so use defaults
    write_pwr_button(display, pwr_pressed, false, false).ok();
}

/// Restores background and renders updated text (generation + FPS + battery + PWR button)
fn render_text_overlay<D, I>(
    display: &mut D,
    background: &I,
    generation: usize,
    fps: usize,
    battery_mv: u16,
    battery_pct: u8,
    pwr_pressed: bool,
    pwr_low: bool,
    pwr_high: bool,
) where
    D: DrawTarget<Color = Rgb888>,
    I: embedded_graphics::image::GetPixel<Color = Rgb888>,
{
    // Restore background in text area before drawing new text
    // Expanded area to include battery display at y=420
    let text_area = Rectangle::new(Point::new(0, 380), embedded_graphics::prelude::Size::new(380, 60));

    for pixel in text_area.points() {
        if let Some(color) = background.pixel(pixel) {
            embedded_graphics::Pixel(pixel, color).draw(display).ok();
        }
    }

    // Draw updated text
    write_generation(display, generation).ok();
    write_fps(display, fps).ok();
    write_battery(display, battery_mv, battery_pct).ok();
    write_pwr_button(display, pwr_pressed, pwr_low, pwr_high).ok();
}

/// Renders the animated GIF at current generation frame
fn render_gif_animation<D, I>(
    display: &mut D,
    background: &I,
    gif_res: &mut GifResource,
    gif_data: &[u8],
    generation: usize,
) -> bool
where
    D: DrawTarget<Color = Rgb888>,
    I: embedded_graphics::image::GetPixel<Color = Rgb888>,
{
    const GIF_WIDTH: u32 = 153;
    const GIF_HEIGHT: u32 = 141;

    // Calculate target frame based on generation
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");
    let total_frames = gif.frames().count();
    let target_frame_index = generation % total_frames;

    // Render using optimized function
    render_gif_optimized(
        display,
        background,
        gif_data,
        gif_res,
        target_frame_index,
        GIF_WIDTH,
        GIF_HEIGHT,
    )
}

/// Flushes updated display regions (text area and GIF area if needed)
fn flush_display_regions(
    display: &mut crate::ecs::resources::DisplayResource,
    gif_needs_render: bool,
    gif_position: Point,
) {
    const GIF_WIDTH: u32 = 153;
    const GIF_HEIGHT: u32 = 141;

    // Flush text area
    display
        .display
        .partial_flush(0, 350, 380, 420, ColorMode::Rgb888)
        .ok();

    // Flush GIF area if it was rendered
    if gif_needs_render {
        let flush_x_start = gif_position.x.max(0) as u16;
        let flush_y_start = gif_position.y.max(0) as u16;
        let flush_x_end = (flush_x_start + GIF_WIDTH as u16).min(368);
        let flush_y_end = (flush_y_start + GIF_HEIGHT as u16).min(448);

        display
            .display
            .partial_flush(
                flush_x_start,
                flush_x_end,
                flush_y_start,
                flush_y_end,
                ColorMode::Rgb888,
            )
            .ok();
    }
}

/// Updates the generation counter with wraparound
fn update_generation(game: &mut GameOfLifeResource) {
    game.generation += 1;
    if game.generation >= RESET_AFTER_GENERATIONS {
        game.generation = 0;
    }
}

/// Main render system
pub fn render_system(
    mut display_res: NonSendMut<DisplayResource>,
    mut touch_res: NonSendMut<TouchResource>,
    mut rtc_res: NonSendMut<RtcResource>,
    mut axp_res: NonSendMut<Axp2101Resource>,
    mut button_res: NonSendMut<ButtonResource>,
    image_res: Res<ImageResource>,
    mut game: ResMut<GameOfLifeResource>,
    mut gif_res: ResMut<GifResource>,
    mut battery_res: ResMut<BatteryResource>,
    mut _fb_res: ResMut<FrameBufferResource>,
) {
    const GIF_DATA: &[u8] = include_bytes!("../bin/knight.gif");

    // 0. Measure frame timing with hybrid approach
    let current_cycles = esp_hal::xtensa_lx::timer::get_cycle_count();

    // Calculate frame time using CPU cycles (precise for short intervals)
    let elapsed_cycles = current_cycles.wrapping_sub(rtc_res.last_cycles);
    let frame_time_us = (elapsed_cycles as u64 * 1_000_000) / (rtc_res.cpu_freq_mhz * 1_000_000);

    // Update last cycle count
    rtc_res.last_cycles = current_cycles;

    // Read RTC timestamp and battery every 100 frames
    if game.generation % 100 == 0 {
        // Update battery reading from AXP2101 PMIC
        if let Ok(battery_voltage_mv) = axp_res.pmic.battery_voltage() {
            let battery_percent = voltage_to_battery_percent(battery_voltage_mv);

            battery_res.voltage_mv = battery_voltage_mv;
            battery_res.percent = battery_percent;
            battery_res.last_update_generation = game.generation;

            esp_println::println!(
                "Gen {}: Frame={}us, Battery={}mV ({}%)",
                game.generation, frame_time_us, battery_voltage_mv, battery_percent
            );
        }

        // Read RTC timestamp
        if let Ok(current_time) = rtc_res.rtc.get_datetime() {
            esp_println::println!(
                "RTC timestamp: {:02}:{:02}:{:02}",
                current_time.time().hour(),
                current_time.time().minute(),
                current_time.time().second()
            );
            rtc_res.last_timestamp = Some(current_time);
        }
    }

    // 1. Handle touch input
    handle_touch_input(&mut touch_res, &mut gif_res);

    // 2. Render initial background (one-time setup)
    if !game.background_drawn {
        let pwr_pin_state = button_res.gpio_expander.read_pin(4).unwrap_or(false); // EXIO4 = pin 4
        let pwr_pressed = !pwr_pin_state; // Active low: pressed = false (LOW), released = true (HIGH)
        render_initial_background(
            &mut display_res.display,
            &image_res.bmp,
            &mut gif_res,
            GIF_DATA,
            game.generation,
            game.fps,
            battery_res.voltage_mv,
            battery_res.percent,
            pwr_pressed,
        );
        game.background_drawn = true;
        display_res.display.flush().ok();
        return;
    }

    // 3. Render text overlay (generation + FPS + Battery + PWR button)
    let pwr_pin_state = button_res.gpio_expander.read_pin(4).unwrap_or(false); // EXIO4 = pin 4
    let pwr_low = !pwr_pin_state; // Active low: pressed = false (LOW), released = true (HIGH)
    let pwr_high = pwr_pin_state; // Inverted logic for display
    let pwr_pressed = pwr_low; // PWR button is Active Low

    render_text_overlay(
        &mut display_res.display,
        &image_res.bmp,
        game.generation,
        game.fps,
        battery_res.voltage_mv,
        battery_res.percent,
        pwr_pressed,
        pwr_low,
        pwr_high,
    );

    // 4. Render GIF animation
    let gif_needs_render = render_gif_animation(
        &mut display_res.display,
        &image_res.bmp,
        &mut gif_res,
        GIF_DATA,
        game.generation,
    );

    // 5. Flush updated display regions
    flush_display_regions(&mut display_res, gif_needs_render, gif_res.position);

    // 6. Update generation counter
    update_generation(&mut game);
}
