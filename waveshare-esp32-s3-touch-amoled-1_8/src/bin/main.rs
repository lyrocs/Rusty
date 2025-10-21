#![no_std]
#![no_main]

extern crate alloc;

use esp_bootloader_esp_idf::esp_app_desc;
esp_app_desc!();

use alloc::boxed::Box;
use bevy_ecs::prelude::*;
use core::fmt::Write;
//ImageTransparent
use embedded_graphics::{
    Drawable,
    image::{GetPixel, Image, ImageRaw},
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics_framebuf::backends::FrameBufferBackend;
use heapless::String;

use esp_hal::Blocking;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::{dma_buffers, dma_descriptors};
use esp_hal::gpio::{Input, InputConfig, Io, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, Error as I2cError, I2c};
use esp_hal::main;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;

use log::info;

use esp_hal::rng::Rng;
use esp_println::logger::init_logger_from_env;
use esp_println::println;

use tinybmp::Bmp; // Parseur BMP
use tinygif::Gif;
// use tinytga::Tga;
// use embedded_sprites::{image::Image, include_image};
// use embedded_sprites::sprite::Sprite;
// use embedded_graphics::pixelcolor::Bgr565;

use core::cell::RefCell;
use embedded_hal_bus::i2c::RefCellDevice;
use static_cell::StaticCell;

use time::{Date, Month, PrimitiveDateTime, Time};

use embedded_hal_bus::i2c;
use embedded_hal_bus::util::AtomicCell;

use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeManager};
use embedded_hal_bus::spi::ExclusiveDevice;

use ft3x68_rs::{
    DriverError, FT3168_DEVICE_ADDRESS, Ft3x68Driver, PowerMode, ResetInterface, TouchPoint,
    TouchState,
};

// Type aliases pour simplifier
static I2C_CELL: StaticCell<AtomicCell<RefCellDevice<'static, I2c<'static, Blocking>>>> =
    StaticCell::new();

type I2cBus = RefCellDevice<'static, I2c<'static, Blocking>>;
type I2cDevice = i2c::AtomicDevice<'static, I2cBus>;
type TouchDriver = Ft3x68Driver<I2cDevice, Delay, ResetTouchDriver<I2cDevice>>;

// #[derive(Resource)]
struct TouchResource {
    touch: TouchDriver,
}

#[derive(Resource)]
struct ImageResource {
    bmp: Bmp<'static, Rgb888>,
}

// --- PCF85063 RTC Driver ---

/// Simple blocking driver for PCF85063 RTC
pub struct Pcf85063<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Pcf85063<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    const DEFAULT_ADDRESS: u8 = 0x51;

    // Register addresses
    const REG_SECONDS: u8 = 0x04;
    const REG_MINUTES: u8 = 0x05;
    const REG_HOURS: u8 = 0x06;
    const REG_DAYS: u8 = 0x07;
    const REG_MONTHS: u8 = 0x09;
    const REG_YEARS: u8 = 0x0A;

    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: Self::DEFAULT_ADDRESS,
        }
    }

    /// Read current date and time from RTC
    pub fn get_datetime(&mut self) -> Result<PrimitiveDateTime, I2C::Error> {
        let mut buf = [0u8; 7];
        self.i2c.write_read(self.address, &[Self::REG_SECONDS], &mut buf)?;

        let seconds = bcd_to_decimal(buf[0] & 0x7F);
        let minutes = bcd_to_decimal(buf[1] & 0x7F);
        let hours = bcd_to_decimal(buf[2] & 0x3F);
        let days = bcd_to_decimal(buf[3] & 0x3F);
        let months = bcd_to_decimal(buf[5] & 0x1F);
        let years = 2000 + bcd_to_decimal(buf[6]) as i32;

        let month = match months {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => Month::January,
        };

        let date = Date::from_calendar_date(years, month, days).unwrap_or_else(|_| {
            Date::from_calendar_date(2024, Month::January, 1).unwrap()
        });
        let time = Time::from_hms(hours, minutes, seconds).unwrap_or(Time::MIDNIGHT);

        Ok(PrimitiveDateTime::new(date, time))
    }

    /// Set date and time on RTC
    pub fn set_datetime(&mut self, dt: &PrimitiveDateTime) -> Result<(), I2C::Error> {
        let buf = [
            Self::REG_SECONDS,
            decimal_to_bcd(dt.time().second()),
            decimal_to_bcd(dt.time().minute()),
            decimal_to_bcd(dt.time().hour()),
            decimal_to_bcd(dt.date().day()),
            0, // weekday (not used)
            decimal_to_bcd(dt.date().month() as u8),
            decimal_to_bcd((dt.date().year() - 2000) as u8),
        ];
        self.i2c.write(self.address, &buf)
    }
}

/// Convert BCD (Binary-Coded Decimal) to normal decimal
fn bcd_to_decimal(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0F)
}

/// Convert normal decimal to BCD
fn decimal_to_bcd(decimal: u8) -> u8 {
    ((decimal / 10) << 4) | (decimal % 10)
}

// --- SD Card TimeSource Implementation ---

/// Simple TimeSource implementation for SD card file timestamps
/// Returns a fixed timestamp for now (can be enhanced with RTC later)
pub struct DummyTimeSource;

impl TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 55, // 2025
            zero_indexed_month: 9, // October (0-indexed)
            zero_indexed_day: 20, // 21st (0-indexed)
            hours: 12,
            minutes: 0,
            seconds: 0,
        }
    }
}

pub struct ResetTouchDriver<I2C> {
    i2c: I2C,
}

impl<I2C> ResetTouchDriver<I2C> {
    pub fn new(i2c: I2C) -> Self {
        ResetTouchDriver { i2c }
    }
}

impl<I2C> ResetInterface for ResetTouchDriver<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Error = I2cError;

    fn reset(&mut self) -> Result<(), Self::Error> {
        println!("Resetting touch controller via I2C GPIO expander...");
        let delay = Delay::new();
        self.i2c.write(0x20, &[0x03, 0x00]).unwrap(); // Configure all pins as output
        self.i2c.write(0x20, &[0x01, 0b0000_0000]).unwrap(); // Drive low
        delay.delay_millis(20);
        self.i2c.write(0x20, &[0x01, 0b0000_0100]).unwrap(); // Drive high
        delay.delay_millis(300);
        Ok(())
    }
}

const IMAGE_DATA: &[u8] = include_bytes!("./background.bmp");
const GIF_DATA: &[u8] = include_bytes!("./knight.gif");

// const IMAGE_DATA: &[u8] = include_bytes!("./rng.tga");
// #[include_image]
// const IMAGE: Image<Rgb888> = "./src/bin/rng.png";

// Custom panic handler for better debugging
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Print panic information
    println!("\n=== PANIC OCCURRED ===");

    // Print panic location if available
    if let Some(location) = info.location() {
        println!(
            "Panic occurred at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        println!("Panic occurred at unknown location");
    }

    // Print panic message if available
    let message = info.message();
    println!("Panic message: {}", message);

    // Print memory information
    println!("\n=== MEMORY INFO ===");
    println!("Stack pointer: unavailable (assembly removed)");

    // Print some general debug info
    println!("\n=== DEBUG INFO ===");
    println!("Target: unknown"); // The TARGET env variable is not set during runtime
    println!(
        "Profile: {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    // Force a flush to ensure all output is printed
    println!("\n=== ENTERING PANIC LOOP ===");

    // Custom panic handler loop - no automatic reset
    // The system will remain in this loop until manually reset
    loop {
        // Small delay to prevent overwhelming the output
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
    }
}

use sh8601_rs::{
    ColorMode, DMA_CHUNK_SIZE, DisplaySize, ResetDriver, Sh8601Driver, Ws18AmoledDriver,
    framebuffer_size,
};

/// A wrapper around a boxed array that implements FrameBufferBackend.
pub struct HeapBuffer<C: PixelColor, const N: usize>(Box<[C; N]>);

impl<C: PixelColor, const N: usize> HeapBuffer<C, N> {
    pub fn new(data: Box<[C; N]>) -> Self {
        Self(data)
    }
}

impl<C: PixelColor, const N: usize> core::ops::Deref for HeapBuffer<C, N> {
    type Target = [C; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: PixelColor, const N: usize> core::ops::DerefMut for HeapBuffer<C, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<C: PixelColor, const N: usize> FrameBufferBackend for HeapBuffer<C, N> {
    type Color = C;
    fn set(&mut self, index: usize, color: Self::Color) {
        self.0[index] = color;
    }
    fn get(&self, index: usize) -> Self::Color {
        self.0[index]
    }
    fn nr_elements(&self) -> usize {
        N
    }
}

// Display configuration for Waveshare ESP32-S3-Touch-AMOLED-1.8
const DISPLAY_SIZE: DisplaySize = DisplaySize::new(368, 448);
const FB_SIZE: usize = framebuffer_size(DISPLAY_SIZE, ColorMode::Rgb888);

// Type alias for the display driver
type DisplayDriver = Sh8601Driver<Ws18AmoledDriver, ResetDriver<I2c<'static, Blocking>>>;

// Conway's Game of Life grid configuration
const GRID_WIDTH: usize = 52; // 368 / 7 ≈ 52
const GRID_HEIGHT: usize = 64; // 448 / 7 ≈ 64

fn update_game_of_life(
    current: &[[u8; GRID_WIDTH]; GRID_HEIGHT],
    next: &mut [[u8; GRID_WIDTH]; GRID_HEIGHT],
) {
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let mut neighbors = 0;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && nx < GRID_WIDTH as i32
                        && ny >= 0
                        && ny < GRID_HEIGHT as i32
                        && current[ny as usize][nx as usize] > 0
                    {
                        neighbors += 1;
                    }
                }
            }
            let cell = current[y][x];
            next[y][x] = match (cell > 0, neighbors) {
                (true, 2) | (true, 3) => cell.saturating_add(1), // stay alive, age
                (false, 3) => 1,                                 // born
                _ => 0,                                          // dead
            };
        }
    }
}

fn randomize_grid(rng: &mut Rng, grid: &mut [[u8; GRID_WIDTH]; GRID_HEIGHT]) {
    for row in grid.iter_mut() {
        for cell in row.iter_mut() {
            let mut buf = [0u8; 1];
            rng.read(&mut buf);
            // Randomly set cell to 1 (alive) or 0 (dead)
            *cell = if buf[0] & 1 != 0 { 1 } else { 0 };
        }
    }
}

fn age_to_color(age: u8) -> Rgb888 {
    if age == 0 {
        Rgb888::BLACK
    } else {
        let max_age = 10;
        let a = age.min(max_age) as u32;
        let r = ((255 * a) + 5) / max_age as u32;
        let g = ((255 * a) + 5) / max_age as u32;
        let b = 255; // Keep blue channel constant
        Rgb888::new(r as u8, g as u8, b as u8)
    }
}

fn draw_grid<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    grid: &[[u8; GRID_WIDTH]; GRID_HEIGHT],
) -> Result<(), D::Error> {
    let border_color = Rgb888::new(230, 230, 230);
    for (y, row) in grid.iter().enumerate() {
        for (x, &age) in row.iter().enumerate() {
            let point = Point::new(x as i32 * 7, y as i32 * 7);
            if age > 0 {
                // Draw a border then fill with color based on age.
                Rectangle::new(point, Size::new(7, 7))
                    .into_styled(PrimitiveStyle::with_fill(border_color))
                    .draw(display)?;
                // Draw an inner cell with color according to age.
                Rectangle::new(point + Point::new(1, 1), Size::new(5, 5))
                    .into_styled(PrimitiveStyle::with_fill(age_to_color(age)))
                    .draw(display)?;
            } else {
                // Draw a dead cell as black.
                Rectangle::new(point, Size::new(7, 7))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLACK))
                    .draw(display)?;
            }
        }
    }
    Ok(())
}

fn write_generation<D: DrawTarget<Color = Rgb888>>(
    display: &mut D,
    generation: usize,
) -> Result<(), D::Error> {
    let mut num_str = String::<20>::new();
    write!(num_str, "Gen: {generation}").unwrap();
    Text::new(
        num_str.as_str(),
        Point::new(8, 400),
        MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE),
    )
    .draw(display)?;
    Ok(())
}

fn write_fps<D: DrawTarget<Color = Rgb888>>(display: &mut D, fps: usize) -> Result<(), D::Error> {
    let mut num_str = String::<20>::new();
    write!(num_str, "FPS: {fps}").unwrap();
    Text::new(
        num_str.as_str(),
        Point::new(250, 400),
        MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE),
    )
    .draw(display)?;
    Ok(())
}

/// Renders a GIF animation with optimized background restoration
///
/// # Parameters
/// - `display`: The display target to draw on
/// - `background`: The background image to restore when clearing frames
/// - `gif_data`: The GIF data bytes
/// - `gif_res`: Resource tracking GIF position and frame state
/// - `target_frame_index`: The frame to display (typically generation % total_frames)
/// - `gif_width`: Width of the GIF in pixels
/// - `gif_height`: Height of the GIF in pixels
///
/// # Returns
/// - `true` if rendering occurred, `false` if no rendering was needed
fn render_gif_optimized<D, I>(
    display: &mut D,
    background: &I,
    gif_data: &[u8],
    gif_res: &mut GifResource,
    target_frame_index: usize,
    gif_width: u32,
    gif_height: u32,
) -> bool
where
    D: DrawTarget<Color = Rgb888>,
    I: GetPixel<Color = Rgb888>,
{
    // Check if position or frame changed or if it's the first render
    let position_changed = gif_res.position != gif_res.previous_position;
    let frame_changed = gif_res.frame_index != target_frame_index;
    let needs_render = position_changed || frame_changed || gif_res.first_render;

    if !needs_render {
        return false;
    }

    // Step 1: Clear the old GIF position by restoring background (only if position changed)
    if position_changed {
        let old_gif_area =
            Rectangle::new(gif_res.previous_position, Size::new(gif_width, gif_height));

        for pixel in old_gif_area.points() {
            if let Some(color) = background.pixel(pixel) {
                embedded_graphics::Pixel(pixel, color).draw(display).ok();
            }
        }

        // Also restore background at new position when moving
        let new_gif_area = Rectangle::new(gif_res.position, Size::new(gif_width, gif_height));

        for pixel in new_gif_area.points() {
            if let Some(color) = background.pixel(pixel) {
                embedded_graphics::Pixel(pixel, color).draw(display).ok();
            }
        }
    } else if frame_changed {
        // Step 2: For frame changes only, restore background to clear previous frame
        let gif_area = Rectangle::new(gif_res.position, Size::new(gif_width, gif_height));

        for pixel in gif_area.points() {
            if let Some(color) = background.pixel(pixel) {
                embedded_graphics::Pixel(pixel, color).draw(display).ok();
            }
        }
    }

    // Step 3: Draw the target GIF frame at the current position
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");
    let mut current_index = 0;
    for frame in gif.frames() {
        if current_index == target_frame_index {
            Image::new(&frame, gif_res.position).draw(display).ok();
            break;
        }
        current_index += 1;
    }

    // Update the GIF state
    gif_res.frame_index = target_frame_index;
    gif_res.first_render = false;

    true
}

// --- Bevy ECS Resources ---

// Framebuffer resource for double buffering
const LCD_H_RES: usize = 368;
const LCD_V_RES: usize = 448;
const LCD_BUFFER_SIZE: usize = LCD_H_RES * LCD_V_RES;

type FbBuffer = HeapBuffer<Rgb888, LCD_BUFFER_SIZE>;
type MyFrameBuf = FrameBuf<Rgb888, FbBuffer>;

#[derive(Resource)]
struct FrameBufferResource {
    frame_buf: MyFrameBuf,
}

impl FrameBufferResource {
    fn new() -> Self {
        let fb_data: Box<[Rgb888; LCD_BUFFER_SIZE]> = Box::new([Rgb888::BLACK; LCD_BUFFER_SIZE]);
        let heap_buffer = HeapBuffer::new(fb_data);
        let frame_buf = MyFrameBuf::new(heap_buffer, LCD_H_RES, LCD_V_RES);
        Self { frame_buf }
    }
}

#[derive(Resource)]
struct GameOfLifeResource {
    grid: [[u8; GRID_WIDTH]; GRID_HEIGHT],
    next_grid: [[u8; GRID_WIDTH]; GRID_HEIGHT],
    generation: usize,
    fps: usize,
    background_drawn: bool, // Track if background has been drawn
}

impl Default for GameOfLifeResource {
    fn default() -> Self {
        Self {
            grid: [[0; GRID_WIDTH]; GRID_HEIGHT],
            next_grid: [[0; GRID_WIDTH]; GRID_HEIGHT],
            generation: 0,
            fps: 0,
            background_drawn: false,
        }
    }
}

#[derive(Resource)]
struct RngResource(Rng);

#[derive(Resource)]
struct GifResource {
    position: Point,          // Current GIF position
    previous_position: Point, // Previous position for cleanup
    frame_index: usize,       // Current frame index
    first_render: bool,       // Track if GIF has been rendered at least once
}

impl Default for GifResource {
    fn default() -> Self {
        Self {
            position: Point::new(160, 200), // Center of screen roughly
            previous_position: Point::new(160, 200),
            frame_index: 0,
            first_render: true, // Force initial render
        }
    }
}

// Display resource - NonSend because it contains non-thread-safe components
// #[derive(Resource)]
struct DisplayResource {
    display:
        Sh8601Driver<Ws18AmoledDriver, ResetDriver<RefCellDevice<'static, I2c<'static, Blocking>>>>,
}

// RTC resource - NonSend because it contains non-thread-safe I2C device
// Combines RTC (for absolute timestamps) with cycle counting (for precise frame timing)
struct RtcResource {
    rtc: Pcf85063<I2cDevice>,
    last_timestamp: Option<PrimitiveDateTime>, // Absolute time from RTC
    last_cycles: u32,                           // CPU cycles at last frame
    cpu_freq_mhz: u64,                          // CPU frequency for cycle->time conversion
}

// --- Bevy ECS Systems ---

const RESET_AFTER_GENERATIONS: usize = 300;

fn update_game_of_life_system(
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
        randomize_grid(&mut rng_res.0, &mut game.grid);
        game.generation = 0;
    }
}

// --- Render System Helper Functions ---

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
    generation: usize,
    fps: usize,
) where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(Rgb888::BLACK).ok();
    Image::new(background, Point::new(0, 0)).draw(display).ok();

    // Draw initial GIF frame
    let gif = Gif::<Rgb888>::from_slice(GIF_DATA).expect("Failed to parse GIF");
    if let Some(first_frame) = gif.frames().next() {
        Image::new(&first_frame, gif_res.position).draw(display).ok();
    }
    gif_res.first_render = false;
    gif_res.frame_index = 0;

    // Draw initial text overlays
    write_generation(display, generation).ok();
    write_fps(display, fps).ok();
}

/// Restores background and renders updated text (generation + FPS)
fn render_text_overlay<D, I>(
    display: &mut D,
    background: &I,
    generation: usize,
    fps: usize,
) where
    D: DrawTarget<Color = Rgb888>,
    I: GetPixel<Color = Rgb888>,
{
    // Restore background in text area before drawing new text
    let text_area = Rectangle::new(Point::new(0, 380), Size::new(380, 40));

    for pixel in text_area.points() {
        if let Some(color) = background.pixel(pixel) {
            embedded_graphics::Pixel(pixel, color).draw(display).ok();
        }
    }

    // Draw updated text
    write_generation(display, generation).ok();
    write_fps(display, fps).ok();
}

/// Renders the animated GIF at current generation frame
fn render_gif_animation<D, I>(
    display: &mut D,
    background: &I,
    gif_res: &mut GifResource,
    generation: usize,
) -> bool
where
    D: DrawTarget<Color = Rgb888>,
    I: GetPixel<Color = Rgb888>,
{
    const GIF_WIDTH: u32 = 153;
    const GIF_HEIGHT: u32 = 141;

    // Calculate target frame based on generation
    let gif = Gif::<Rgb888>::from_slice(GIF_DATA).expect("Failed to parse GIF");
    let total_frames = gif.frames().count();
    let target_frame_index = generation % total_frames;

    // Render using optimized function
    render_gif_optimized(
        display,
        background,
        GIF_DATA,
        gif_res,
        target_frame_index,
        GIF_WIDTH,
        GIF_HEIGHT,
    )
}

/// Flushes updated display regions (text area and GIF area if needed)
fn flush_display_regions(
    display: &mut Sh8601Driver<
        Ws18AmoledDriver,
        ResetDriver<RefCellDevice<'static, I2c<'static, Blocking>>>,
    >,
    gif_needs_render: bool,
    gif_position: Point,
) {
    const GIF_WIDTH: u32 = 153;
    const GIF_HEIGHT: u32 = 141;

    // Flush text area
    display
        .partial_flush(0, 350, 380, 420, ColorMode::Rgb888)
        .ok();

    // Flush GIF area if it was rendered
    if gif_needs_render {
        let flush_x_start = gif_position.x.max(0) as u16;
        let flush_y_start = gif_position.y.max(0) as u16;
        let flush_x_end = (flush_x_start + GIF_WIDTH as u16).min(368);
        let flush_y_end = (flush_y_start + GIF_HEIGHT as u16).min(448);

        display
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

// --- Main Render System ---

fn render_system(
    mut display_res: NonSendMut<DisplayResource>,
    mut touch_res: NonSendMut<TouchResource>,
    mut rtc_res: NonSendMut<RtcResource>,
    image_res: Res<ImageResource>,
    mut game: ResMut<GameOfLifeResource>,
    mut gif_res: ResMut<GifResource>,
    mut fb_res: ResMut<FrameBufferResource>,
) {
    // 0. Measure frame timing with hybrid approach
    let current_cycles = esp_hal::xtensa_lx::timer::get_cycle_count();

    // Calculate frame time using CPU cycles (precise for short intervals)
    let elapsed_cycles = current_cycles.wrapping_sub(rtc_res.last_cycles);
    let frame_time_us = (elapsed_cycles as u64 * 1_000_000) / (rtc_res.cpu_freq_mhz * 1_000_000);

    // Update last cycle count
    rtc_res.last_cycles = current_cycles;

    // Read RTC timestamp every 100 frames for absolute time tracking
    if game.generation % 100 == 0 {
        if let Ok(current_time) = rtc_res.rtc.get_datetime() {
            println!(
                "Gen {}: Frame={}us, RTC timestamp: {:02}:{:02}:{:02}",
                game.generation,
                frame_time_us,
                current_time.time().hour(),
                current_time.time().minute(),
                current_time.time().second()
            );
            rtc_res.last_timestamp = Some(current_time);
        } else {
            println!("Gen {}: Frame={}us", game.generation, frame_time_us);
        }
    }

    // 1. Handle touch input
    handle_touch_input(&mut touch_res, &mut gif_res);

    // 2. Render initial background (one-time setup)
    if !game.background_drawn {
        render_initial_background(
            &mut display_res.display,
            &image_res.bmp,
            &mut gif_res,
            game.generation,
            game.fps,
        );
        game.background_drawn = true;
        display_res.display.flush().ok();
        return;
    }

    // 3. Render text overlay (generation + FPS)
    render_text_overlay(
        &mut display_res.display,
        &image_res.bmp,
        game.generation,
        game.fps,
    );

    // 4. Render GIF animation
    let gif_needs_render = render_gif_animation(
        &mut display_res.display,
        &image_res.bmp,
        &mut gif_res,
        game.generation,
    );

    // 5. Flush updated display regions
    flush_display_regions(
        &mut display_res.display,
        gif_needs_render,
        gif_res.position,
    );

    // 6. Update generation counter
    update_generation(&mut game);
}

static I2C_BUS: StaticCell<RefCell<I2c<'static, Blocking>>> = StaticCell::new();

#[main]
fn main() -> ! {
    println!("[MAIN] Starting main function");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    println!("[MAIN] Config created");

    let peripherals = esp_hal::init(config);
    println!("[MAIN] Peripherals initialized");

    println!("[MAIN] Starting up...");
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);
    println!("[MAIN] PSRAM allocator initialized");

    init_logger_from_env();
    println!("[MAIN] Logger initialized");

    let delay = Delay::new();

    info!("Initializing display...");

    // --- DMA Buffers for SPI ---
    #[allow(clippy::manual_div_ceil)]
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(DMA_CHUNK_SIZE);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // SPI Configuration for Waveshare ESP32-S3 1.8inch AMOLED Touch Display
    let lcd_spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40_u32))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sio0(peripherals.GPIO4)
    .with_sio1(peripherals.GPIO5)
    .with_sio2(peripherals.GPIO6)
    .with_sio3(peripherals.GPIO7)
    .with_cs(peripherals.GPIO12)
    .with_sck(peripherals.GPIO11)
    .with_dma(peripherals.DMA_CH0)
    .with_buffers(dma_rx_buf, dma_tx_buf);

    // I2C Configuration for Waveshare ESP32-S3 1.8inch AMOLED Touch Display
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO15)
    .with_scl(peripherals.GPIO14);

    let i2c_bus = I2C_BUS.init(RefCell::new(i2c));
    let i2c_device = RefCellDevice::new(i2c_bus);
    let ws_driver = Ws18AmoledDriver::new(lcd_spi);
    let reset = ResetDriver::new(i2c_device);

    let i2c_touch = RefCellDevice::new(i2c_bus);
    let i2c_cell = I2C_CELL.init(AtomicCell::new(i2c_touch));

    let reset_touch = ResetTouchDriver::new(i2c::AtomicDevice::new(i2c_cell));
    let mut touch = Ft3x68Driver::new(
        i2c::AtomicDevice::new(i2c_cell),
        FT3168_DEVICE_ADDRESS,
        reset_touch,
        delay,
    );

    touch
        .initialize()
        .expect("Failed to initialize touch driver");

    // // Activate Gesture Mode to detect gestures
    touch
        .set_gesture_mode(true)
        .expect("Failed to set gesture mode");

    // Initialize RTC (PCF85063)
    println!("Initializing PCF85063 RTC...");
    let i2c_rtc = i2c::AtomicDevice::new(i2c_cell);
    let mut rtc = Pcf85063::new(i2c_rtc);

    // Optional: Set initial time if needed (commented out for now)
    // let initial_time = PrimitiveDateTime::new(
    //     Date::from_calendar_date(2025, Month::October, 21).unwrap(),
    //     Time::from_hms(12, 0, 0).unwrap(),
    // );
    // rtc.set_datetime(&initial_time).ok();

    // Read current time from RTC
    match rtc.get_datetime() {
        Ok(dt) => println!("RTC initialized. Current time: {:?}", dt),
        Err(_) => println!("Warning: Could not read RTC time"),
    }

    // Initialize SD Card (TF Card) via SPI3
    println!("Initializing SD Card...");

    // SD Card uses its own SPI bus with these pins:
    // SCLK: GPIO2, MOSI: GPIO1, MISO: GPIO3, CS: EXIO7 (via GPIO expander)
    // For now, we'll use a simpler approach with a dedicated GPIO for CS

    // Create DMA buffers for SD card SPI
    let (sd_rx_buffer, sd_rx_descriptors, sd_tx_buffer, sd_tx_descriptors) = dma_buffers!(8192);
    let sd_dma_rx_buf = DmaRxBuf::new(sd_rx_descriptors, sd_rx_buffer).unwrap();
    let sd_dma_tx_buf = DmaTxBuf::new(sd_tx_descriptors, sd_tx_buffer).unwrap();

    let sd_spi = Spi::new(
        peripherals.SPI3,
        SpiConfig::default()
            .with_frequency(Rate::from_khz(400)) // Start slow for initialization
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO2)
    .with_mosi(peripherals.GPIO1)
    .with_miso(peripherals.GPIO3)
    .with_dma(peripherals.DMA_CH1)
    .with_buffers(sd_dma_rx_buf, sd_dma_tx_buf);

    // For CS, we need to control GPIO via I2C expander (EXIO7)
    // This is complex, so for POC we'll use GPIO10 as temporary CS for testing
    let sd_cs = peripherals.GPIO10;
    let sd_cs_output = Output::new(sd_cs, Level::High, OutputConfig::default());

    let sd_device = ExclusiveDevice::new(sd_spi, sd_cs_output, Delay::new()).unwrap();
    let sdcard = SdCard::new(sd_device, Delay::new());

    match sdcard.num_bytes() {
        Ok(size) => {
            let size_mb = size / (1024 * 1024);
            println!("SD Card detected! Size: {} MB", size_mb);
        }
        Err(e) => {
            println!("SD Card initialization failed or not inserted: {:?}", e);
            println!("Continuing without SD card support...");
        }
    }

    // Instantiate and Initialize Display
    println!("Initializing SH8601 Display...");
    let display_res = Sh8601Driver::new_heap::<_, FB_SIZE>(
        ws_driver,
        reset,
        ColorMode::Rgb888,
        DISPLAY_SIZE,
        delay,
    );

    let display = match display_res {
        Ok(d) => {
            println!("Display initialized successfully.");
            d
        }
        Err(e) => {
            println!("Error initializing display: {:?}", e);
            panic!("Failed to initialize display");
        }
    };

    let bmp = Bmp::<Rgb888>::from_slice(IMAGE_DATA).expect("Failed to parse BMP image");

    // Initialize RNG
    let mut rng = Rng::new(peripherals.RNG);

    // Initialize game resources
    let mut game = GameOfLifeResource::default();
    // randomize_grid(&mut rng, &mut game.grid);

    // Create framebuffer resource
    let fb_res = FrameBufferResource::new();

    // Initialize Bevy ECS World
    let mut world = World::default();
    world.insert_resource(game);
    // world.insert_resource(RngResource(rng));
    world.insert_resource(fb_res);
    world.insert_resource(ImageResource { bmp });
    world.insert_resource(GifResource::default());

    // Insert display as NonSend resource
    world.insert_non_send_resource(DisplayResource { display });
    world.insert_non_send_resource(TouchResource { touch });
    // Get initial cycle count and CPU frequency
    let initial_cycles = esp_hal::xtensa_lx::timer::get_cycle_count();
    let cpu_freq_mhz = 240; // ESP32-S3 at max frequency

    world.insert_non_send_resource(RtcResource {
        rtc,
        last_timestamp: None,
        last_cycles: initial_cycles,
        cpu_freq_mhz,
    });
    // Create schedule and add systems
    let mut schedule = Schedule::default();
    // schedule.add_systems(update_game_of_life_system);
    schedule.add_systems(render_system);

    let loop_delay = Delay::new();

    const BOOT_BUTTON_PIN: u8 = 0; // GPIO0
    const POWER_BUTTON_PIN: u8 = 14;

    let mut io = Io::new(peripherals.IO_MUX);
    let button = peripherals.GPIO0;
    let config = InputConfig::default().with_pull(Pull::Up);
    let mut button = Input::new(button, config);

    info!("Entering Bevy ECS main loop...");

    // Variables for timing statistics
    let mut total_cycles: u64 = 0;
    let mut frame_count: u64 = 0;
    let mut max_cycles: u32 = 0;
    let mut min_cycles: u32 = u32::MAX;

    // Get CPU frequency for time calculations
    let cpu_freq_mhz = 240; // ESP32-S3 running at 240 MHz

    loop {
        if button.is_low() {
            println!("Bouton APPUYÉ !");
        }

        // Measure CPU cycles before schedule.run()
        let start = esp_hal::xtensa_lx::timer::get_cycle_count();
        schedule.run(&mut world);
        let end = esp_hal::xtensa_lx::timer::get_cycle_count();

        // Calculate elapsed cycles (handle wraparound)
        let elapsed_cycles = end.wrapping_sub(start);

        // Update statistics
        total_cycles += elapsed_cycles as u64;
        frame_count += 1;
        if elapsed_cycles > max_cycles {
            max_cycles = elapsed_cycles;
        }
        if elapsed_cycles < min_cycles {
            min_cycles = elapsed_cycles;
        }

        // Print timing every 100 frames
        if frame_count % 100 == 0 {
            let avg_cycles = total_cycles / frame_count;
            let avg_time_us = avg_cycles / cpu_freq_mhz;
            let fps = 1_000_000 / avg_time_us;
            let last_time_us = elapsed_cycles as u64 / cpu_freq_mhz;
            let min_time_us = min_cycles as u64 / cpu_freq_mhz;
            let max_time_us = max_cycles as u64 / cpu_freq_mhz;

            // Access game resource through the world
            if let Some(mut game) = world.get_resource_mut::<GameOfLifeResource>() {
                game.fps = fps as usize;
            }
            println!(
                "Frame {}: Avg={}us ({}fps), Min={}us, Max={}us, Last={}us",
                frame_count, avg_time_us, fps, min_time_us, max_time_us, last_time_us
            );
        }
    }
}
