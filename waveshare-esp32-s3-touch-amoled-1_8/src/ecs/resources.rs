use bevy_ecs::prelude::*;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics_framebuf::FrameBuf;
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;
use esp_hal::gpio::Input;
use esp_hal::rng::Rng;
use time::PrimitiveDateTime;
use tinybmp::Bmp;
use axp2101::core::Axp2101;
use embedded_hal_bus::i2c;
use embedded_sdmmc::{SdCard, VolumeManager};

use crate::display::{HeapBuffer, LCD_H_RES, LCD_V_RES, LCD_BUFFER_SIZE};
use crate::drivers::{Tca9554Driver, Pcf85063};
use crate::game::{GRID_WIDTH, GRID_HEIGHT};

use ft3x68_rs::Ft3x68Driver;
use sh8601_rs::{Sh8601Driver, Ws18AmoledDriver, ResetDriver};

// Type aliases for I2C devices
pub type I2cBus = RefCellDevice<'static, I2c<'static, Blocking>>;
pub type I2cDevice = i2c::AtomicDevice<'static, I2cBus>;
pub type TouchDriver = Ft3x68Driver<I2cDevice, esp_hal::delay::Delay, crate::drivers::ResetTouchDriver<I2cDevice>>;

// Type aliases for display
pub type FbBuffer = HeapBuffer<Rgb888, LCD_BUFFER_SIZE>;
pub type MyFrameBuf = FrameBuf<Rgb888, FbBuffer>;

/// Framebuffer resource for double buffering
#[derive(Resource)]
pub struct FrameBufferResource {
    pub frame_buf: MyFrameBuf,
}

impl FrameBufferResource {
    pub fn new() -> Self {
        let fb_data: alloc::boxed::Box<[Rgb888; LCD_BUFFER_SIZE]> = alloc::boxed::Box::new([Rgb888::BLACK; LCD_BUFFER_SIZE]);
        let heap_buffer = HeapBuffer::new(fb_data);
        let frame_buf = MyFrameBuf::new(heap_buffer, LCD_H_RES, LCD_V_RES);
        Self { frame_buf }
    }
}

/// Game of Life state resource
#[derive(Resource)]
pub struct GameOfLifeResource {
    pub grid: [[u8; GRID_WIDTH]; GRID_HEIGHT],
    pub next_grid: [[u8; GRID_WIDTH]; GRID_HEIGHT],
    pub generation: usize,
    pub fps: usize,
    pub background_drawn: bool, // Track if background has been drawn
    pub display_on: bool,       // Track display on/off state
}

impl Default for GameOfLifeResource {
    fn default() -> Self {
        Self {
            grid: [[0; GRID_WIDTH]; GRID_HEIGHT],
            next_grid: [[0; GRID_WIDTH]; GRID_HEIGHT],
            generation: 0,
            fps: 0,
            background_drawn: false,
            display_on: true, // Display starts ON
        }
    }
}

/// Random number generator resource
#[derive(Resource)]
pub struct RngResource(pub Rng);

/// Image resource (BMP background)
#[derive(Resource)]
pub struct ImageResource {
    pub bmp: Bmp<'static, Rgb888>,
}

/// Battery resource for tracking battery voltage and percentage
#[derive(Resource)]
pub struct BatteryResource {
    pub voltage_mv: u16,
    pub percent: u8,
    pub last_update_generation: usize,
}

/// Touch controller resource
pub struct TouchResource {
    pub touch: TouchDriver,
}

/// Display resource - NonSend because it contains non-thread-safe components
pub struct DisplayResource {
    pub display: Sh8601Driver<Ws18AmoledDriver, ResetDriver<RefCellDevice<'static, I2c<'static, Blocking>>>>,
}

/// RTC resource - NonSend because it contains non-thread-safe I2C device
/// Combines RTC (for absolute timestamps) with cycle counting (for precise frame timing)
pub struct RtcResource {
    pub rtc: Pcf85063<I2cDevice>,
    pub last_timestamp: Option<PrimitiveDateTime>, // Absolute time from RTC
    pub last_cycles: u32,                          // CPU cycles at last frame
    pub cpu_freq_mhz: u64,                         // CPU frequency for cycle->time conversion
}

/// AXP2101 PMIC resource for battery management
/// I2C address: 0x34
pub struct Axp2101Resource {
    pub pmic: Axp2101<i2c::AtomicDevice<'static, RefCellDevice<'static, I2c<'static, Blocking>>>>,
}

/// Button resource for tracking button states and debouncing
/// GPIO0 is the boot button with pull-up (active low)
/// EXIO4 is the PWR button via TCA9554PWR GPIO expander
pub struct ButtonResource {
    pub boot_button: Input<'static>,             // GPIO0 - BOOT button
    pub gpio_expander: Tca9554Driver<I2cDevice>, // TCA9554PWR for EXIO4 (PWR button)
    pub boot_last_state: bool,                   // Last debounced state of BOOT (true = pressed)
    pub pwr_last_state: bool,                    // Last debounced state of PWR (true = pressed)
    pub boot_debounce_counter: u8,               // Counter for debouncing BOOT
    pub pwr_debounce_counter: u8,                // Counter for debouncing PWR
}

/// SD Card resource for save/load functionality
/// Uses the VolumeManager from embedded-sdmmc with proper types
pub struct SdCardResource {
    pub volume_mgr: VolumeManager<
        SdCard<
            embedded_hal_bus::spi::ExclusiveDevice<
                esp_hal::spi::master::Spi<'static, esp_hal::Blocking>,
                crate::drivers::ExioPin<embedded_hal_bus::i2c::AtomicDevice<'static, RefCellDevice<'static, esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>>>>,
                esp_hal::delay::Delay
            >,
            esp_hal::delay::Delay
        >,
        crate::utils::DummyTimeSource,
        4,
        4,
        1
    >,
}
