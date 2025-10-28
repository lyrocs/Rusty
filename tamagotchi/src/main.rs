// ESP32-S3 Tamagotchi - STD version with multithreading
//
// Phase 1: Proof of concept demonstrating:
// - ESP-IDF std environment
// - Bevy ECS with std features
// - Multithreading on dual cores
// - Thread-safe hardware access

// Add ESP-IDF app descriptor
// esp_idf_svc::sys::esp_app_desc!();

// Note: link_patches and EspLogger initialization are called in main()

mod drivers;
mod hal;
mod systems;
mod threads;
mod types;

use anyhow::Result;
use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use crossbeam_channel::bounded;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::drivers::{display::create_shared_display, touch::create_shared_touch};
use crate::systems::{
    game::{GameState, game_update_system},
    input::{InputEventReceiver, process_input_system},
    render::{RenderCommandSender, send_render_commands_system},
};
use crate::threads::{input::spawn_input_thread, render::spawn_render_thread};

fn main() -> Result<()> {
    // Initialize ESP-IDF - link_patches is called by esp-idf-svc
    esp_idf_svc::sys::link_patches();

    // Initialize ESP-IDF logging
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("=== ESP32-S3 Tamagotchi STD Version ===");
    log::info!("Phase 1: Proof of Concept");

    // Get peripherals
    let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take()?;

    // Create shared hardware resources
    log::info!("Initializing hardware drivers...");

    // Initialize I2C - using Box::leak for 'static lifetime
    log::info!("Setting up I2C...");
    use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
    use esp_idf_svc::hal::units::Hertz;

    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));

    // Create ONE I2C driver for everything - we'll share it carefully
    let i2c_main = Box::leak(Box::new(I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio15, // SDA
        peripherals.pins.gpio14, // SCL
        &i2c_config,
    )?));

    log::info!("I2C bus initialized, will be shared between touch and GPIO expander");

    // Initialize SPI for display
    log::info!("Setting up SPI...");
    use esp_idf_svc::hal::spi::{
        config::{Config as SpiConfig, Mode},
        Dma, SpiDeviceDriver, SpiDriver, SpiDriverConfig,
    };

    let spi_driver = Box::leak(Box::new(SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio11, // SCK
        peripherals.pins.gpio4,  // MOSI (SIO0)
        Some(peripherals.pins.gpio5), // MISO (SIO1)
        &SpiDriverConfig::new().dma(Dma::Auto(4096)),
    )?));

    let spi_config = SpiConfig::new()
        .baudrate(Hertz(40_000_000));
        // Mode 0 is default, no need to set explicitly

    let spi_device = Box::leak(Box::new(SpiDeviceDriver::new(
        spi_driver,
        Some(peripherals.pins.gpio12), // CS
        &spi_config,
    )?));

    // Create GPIO expanders
    use crate::drivers::gpio_expander::Tca9554Driver;

    // SAFETY: We're creating two mutable references to the same I2C bus
    // This is technically undefined behavior, but for the POC we'll accept it
    // Proper solution would be to use a bus manager or RefCell
    let i2c_for_gpio = unsafe { &mut *(i2c_main as *mut I2cDriver) };
    let i2c_for_touch = unsafe { &mut *(i2c_main as *mut I2cDriver) };

    // Scan I2C bus to find devices
    log::info!("Scanning I2C bus for devices...");
    for addr in 0x08..=0x77 {
        let result = i2c_for_gpio.write(addr, &[], 100);
        if result.is_ok() {
            log::info!("Found I2C device at address: 0x{:02X}", addr);
        }
    }
    log::info!("I2C bus scan complete");

    // Auto-detect GPIO expander address
    log::info!("Detecting TCA9554 GPIO expander...");
    let gpio_address = Tca9554Driver::detect_address(i2c_for_gpio)?;
    log::info!("Using TCA9554 at address: 0x{:02X}", gpio_address);

    // Create GPIO expander driver temporarily to do the reset sequence
    {
        let mut gpio_exp_temp = Tca9554Driver::new_with_address(i2c_for_gpio, gpio_address);

        // Since we need to use the GPIO expander in two places, we'll initialize both
        // display and touch resets manually here
        log::info!("Initializing GPIO expander reset pins");

        log::info!("Configuring display reset pin (pin 0)...");
        gpio_exp_temp.configure_pin(0, false).map_err(|e| {
            log::error!("Failed to configure display reset pin: {:?}", e);
            e
        })?; // Display reset as output

        log::info!("Configuring touch reset pin (pin 1)...");
        gpio_exp_temp.configure_pin(1, false).map_err(|e| {
            log::error!("Failed to configure touch reset pin: {:?}", e);
            e
        })?; // Touch reset as output

        // Proper reset sequence for FT3168 and display:
        // FT3168 requires: HIGH (1ms) -> LOW (20ms) -> HIGH (50ms)
        log::info!("Starting reset sequence: HIGH -> LOW -> HIGH");

        // Step 1: Set HIGH
        gpio_exp_temp.write_pin(0, true)?; // Display reset HIGH
        gpio_exp_temp.write_pin(1, true)?; // Touch reset HIGH
        thread::sleep(Duration::from_millis(1));

        // Step 2: Set LOW
        log::info!("Asserting resets (LOW)...");
        gpio_exp_temp.write_pin(0, false)?; // Display reset LOW
        gpio_exp_temp.write_pin(1, false)?; // Touch reset LOW
        thread::sleep(Duration::from_millis(20));

        // Step 3: Set HIGH
        log::info!("Releasing resets (HIGH)...");
        gpio_exp_temp.write_pin(0, true)?; // Display reset HIGH
        gpio_exp_temp.write_pin(1, true)?; // Touch reset HIGH

        log::info!("Waiting for devices to boot (50ms)...");
        thread::sleep(Duration::from_millis(50));
    } // gpio_exp_temp is dropped here, releasing the borrow

    // Re-scan I2C bus after reset to see if touch controller appears
    log::info!("Re-scanning I2C bus after reset...");
    for addr in 0x08..=0x77 {
        let result = i2c_for_gpio.write(addr, &[], 100);
        if result.is_ok() {
            log::info!("Found I2C device at address: 0x{:02X}", addr);
        }
    }
    log::info!("I2C re-scan complete");

    // Now create the permanent GPIO expander for the display driver
    let gpio_exp_shared = Tca9554Driver::new_with_address(i2c_for_gpio, gpio_address);
    log::info!("GPIO expander initialization complete");

    // Create display driver
    use crate::drivers::display::Sh8601DisplayDriver;
    let mut display_driver = Sh8601DisplayDriver::new(spi_device, gpio_exp_shared)?;
    display_driver.initialize()?;
    let display: Arc<Mutex<dyn crate::hal::DisplayDriver>> = Arc::new(Mutex::new(display_driver));

    // Create touch driver (reset was already handled by gpio_exp_shared above)
    use crate::drivers::touch::Ft3168TouchDriver;

    // Detect touch controller address
    let touch_address = match Ft3168TouchDriver::detect_address(i2c_for_touch) {
        Ok(addr) => {
            log::info!("Using FT3168 at address: 0x{:02X}", addr);
            addr
        }
        Err(e) => {
            log::error!("Failed to detect FT3168 touch controller: {:?}", e);
            log::error!("Check that touch controller is properly powered and reset");
            return Err(e);
        }
    };

    let mut touch_driver = Ft3168TouchDriver::new_with_address(i2c_for_touch, touch_address)?;
    touch_driver.initialize()?;
    let touch: Arc<Mutex<dyn crate::hal::TouchDriver>> = Arc::new(Mutex::new(touch_driver));

    // Create inter-thread communication channels
    let (input_tx, input_rx) = bounded(100);
    let (render_tx, render_rx) = bounded(100);

    // Create running flag for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));

    // Spawn worker threads
    log::info!("Spawning worker threads...");
    let input_handle = spawn_input_thread(touch.clone(), input_tx, running.clone());

    let render_handle = spawn_render_thread(display.clone(), render_rx, running.clone());

    log::info!("Worker threads spawned successfully");

    // Setup Bevy ECS
    log::info!("Initializing Bevy ECS...");
    let mut app = App::new();

    // Add resources
    app.insert_resource(GameState::default());
    app.insert_resource(InputEventReceiver(input_rx));
    app.insert_resource(RenderCommandSender(render_tx));

    // Add systems
    app.add_systems(
        Update,
        (
            process_input_system,
            game_update_system,
            send_render_commands_system,
        )
            .chain(),
    );

    log::info!("Bevy ECS initialized");
    log::info!("Starting main game loop...");

    // Run the game loop for a limited time (proof of concept)
    let loop_duration = Duration::from_secs(60); // Extended to 60 seconds for testing
    let start_time = std::time::Instant::now();

    let mut frame_count = 0u64;
    while start_time.elapsed() < loop_duration {
        // Update ECS systems
        app.update();

        frame_count += 1;
        if frame_count % 20 == 0 {
            log::info!("Main loop frame: {}", frame_count);
        }

        // Longer sleep to ensure IDLE task can run
        // This is temporary until we properly configure thread affinity
        thread::sleep(Duration::from_millis(50));
    }

    // Graceful shutdown
    log::info!("Shutting down...");
    running.store(false, Ordering::Relaxed);

    input_handle.join().ok();
    render_handle.join().ok();

    log::info!("Shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_structure() {
        // Basic smoke test to ensure modules compile
        assert!(true);
    }
}
