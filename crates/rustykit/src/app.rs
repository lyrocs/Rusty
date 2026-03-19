//! App entry point and main loop.

use crate::display::{Canvas, ColorMode, DisplayDriver};
use crate::error::{Result, RustyError};
use crate::hw;
use crate::input::InputEvent;
use crate::net::Network;
use crate::storage::Storage;
use crate::ui::View;

use embedded_graphics::prelude::*;

use crossbeam_channel::Receiver;
use embedded_graphics::pixelcolor::Rgb888;
use std::time::{Duration, Instant};

/// Application configuration.
pub struct AppConfig {
    /// Display color mode (default: Rgb565, saves 66KB RAM).
    pub color_mode: ColorMode,
    /// Target frames per second (default: 20).
    pub target_fps: u8,
    /// Swipe detection threshold in pixels (default: 50).
    pub swipe_threshold: i32,
    /// Enable WiFi (default: false).
    pub enable_wifi: bool,
    /// WiFi config file path on SD card (default: "WIFI.JSN").
    pub wifi_config_path: &'static str,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            color_mode: ColorMode::Rgb565,
            target_fps: 20,
            swipe_threshold: 50,
            enable_wifi: false,
            wifi_config_path: "WIFI.JSN",
        }
    }
}

/// Context passed to the user's update closure each frame.
pub struct AppContext<'a> {
    /// The drawing surface.
    pub canvas: &'a mut Canvas<'static>,
    /// Input events since last frame.
    pub input: Vec<InputEvent>,
    /// SD card storage (if mounted).
    pub storage: Option<&'a mut Storage>,
    /// Network handle (if WiFi enabled and connected).
    pub network: Option<&'a mut Network>,
    /// Time elapsed since the previous frame.
    pub dt: Duration,
    /// Frame counter (starts at 0).
    pub frame: u64,
}

impl<'a> AppContext<'a> {
    /// Draw a View to the canvas.
    pub fn draw_view(&mut self, view: &View) {
        view.draw(self.canvas);
    }
}

/// The main application handle.
///
/// Created via `App::init()`, then run via `App::run()`.
///
/// # Example
/// ```ignore
/// let app = App::init(AppConfig::default()).unwrap();
/// app.run(|ctx| {
///     ctx.canvas.clear(Color::BLACK);
///     ctx.canvas.draw_text("Hello!", 20, 30, FontSize::Large, Color::YELLOW);
/// });
/// ```
pub struct App {
    config: AppConfig,
    canvas: Canvas<'static>,
    input_rx: Receiver<InputEvent>,
    storage: Option<Storage>,
    network: Option<Network>,
    screen_on: bool,
}

impl App {
    /// Initialize all hardware and create the application.
    ///
    /// This is the single entry point. Users never touch SPI, I2C, or GPIO directly.
    pub fn init(config: AppConfig) -> Result<Self> {
        log::info!("Initializing rustykit...");

        let hw = hw::init_hardware().map_err(|e| RustyError::Io(e.to_string()))?;

        // Display
        let mut driver = DisplayDriver::new(
            hw.display_spi,
            hw.dc,
            hw.rst,
            hw::pins::LCD_WIDTH,
            hw::pins::LCD_HEIGHT,
            config.color_mode,
        )
        .map_err(|e| RustyError::Display(e.to_string()))?;
        driver.set_backlight_pin(hw.backlight);
        driver
            .initialize(config.color_mode)
            .map_err(|e| RustyError::Display(e.to_string()))?;
        driver
            .backlight_on()
            .map_err(|e| RustyError::Display(e.to_string()))?;
        let _ = driver.clear(Rgb888::BLACK);
        driver
            .flush()
            .map_err(|e| RustyError::Display(e.to_string()))?;

        let canvas = Canvas::new(driver);

        // SD card (optional)
        let storage = match Storage::new(hw.sd_spi) {
            Ok(mut s) => {
                s.ls_root();
                Some(s)
            }
            Err(e) => {
                log::warn!("SD card unavailable: {:?}", e);
                None
            }
        };

        // Touch + input thread
        let (sender, receiver) = crossbeam_channel::unbounded();

        // Initialize touch via shared I2C
        let i2c_static: &'static mut _ = Box::leak(Box::new(hw.i2c));

        let mut touch =
            crate::input::touch::Cst816dDriver::new(crate::input::touch::CST816D_DEVICE_ADDRESS);
        let touch_ok = touch
            .initialize(i2c_static)
            .map_err(|e| log::warn!("Touch init failed: {:?}", e))
            .is_ok();

        if touch_ok {
            unsafe {
                crate::input::thread::init_touch_i2c(i2c_static);
            }
        }

        // Spawn input thread
        let _input_handle = crate::input::thread::spawn_input_thread(
            hw.boot_pin,
            hw.pwr_pin,
            sender,
            config.swipe_threshold,
        );

        // WiFi (optional)
        let network = None; // WiFi requires modem peripheral, handled separately

        log::info!("rustykit initialized (touch={})", touch_ok);

        Ok(Self {
            config,
            canvas,
            input_rx: receiver,
            storage,
            network,
            screen_on: true,
        })
    }

    /// Access the canvas for setup drawing before entering the main loop.
    pub fn canvas(&mut self) -> &mut Canvas<'static> {
        &mut self.canvas
    }

    /// Access storage for setup operations before entering the main loop.
    pub fn storage(&mut self) -> Option<&mut Storage> {
        self.storage.as_mut()
    }

    /// Access network for setup operations before entering the main loop.
    pub fn network(&mut self) -> Option<&mut Network> {
        self.network.as_mut()
    }

    /// Run the application loop.
    ///
    /// Calls `update` each frame with an `AppContext` containing canvas, input,
    /// storage, and network. Handles frame timing and display flushing.
    pub fn run<F>(mut self, mut update: F) -> !
    where
        F: FnMut(&mut AppContext),
    {
        let frame_duration =
            Duration::from_millis(1000 / self.config.target_fps.max(1) as u64);
        let mut last_frame = Instant::now();
        let mut frame_count: u64 = 0;

        loop {
            let now = Instant::now();
            let dt = now.duration_since(last_frame);
            last_frame = now;

            // Drain input events
            let mut events: Vec<InputEvent> = Vec::new();
            while let Ok(event) = self.input_rx.try_recv() {
                // Handle power button internally
                match &event {
                    InputEvent::PowerReleased => {
                        self.screen_on = !self.screen_on;
                        if self.screen_on {
                            let _ = self.canvas.display_on();
                        } else {
                            let _ = self.canvas.display_off();
                        }
                    }
                    _ => {}
                }
                events.push(event);
            }

            // Skip rendering if screen is off (but still process events)
            if self.screen_on {
                let mut ctx = AppContext {
                    canvas: &mut self.canvas,
                    input: events,
                    storage: self.storage.as_mut(),
                    network: self.network.as_mut(),
                    dt,
                    frame: frame_count,
                };

                update(&mut ctx);

                let _ = self.canvas.flush();
            }

            frame_count += 1;

            // Frame timing
            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }
}
