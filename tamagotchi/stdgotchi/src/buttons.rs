//! Button Input Module for ESP32-S3
//!
//! Handles physical button inputs with debouncing:
//! - BOOT button (GPIO0) - Built-in boot button with pull-up (active low)
//! - PWR button (EXIO4) - Power button via TCA9554 GPIO expander (active low)
//!
//! # Features
//! - Hardware debouncing (3 consecutive reads required)
//! - Both press and release events
//! - Active-low button logic
//!
//! # Example
//! ```no_run
//! let mut buttons = Buttons::new(&mut i2c, boot_pin)?;
//!
//! loop {
//!     if let Some(event) = buttons.poll(&mut i2c)? {
//!         match event {
//!             ButtonEvent::BootPress => println!("Boot pressed!"),
//!             ButtonEvent::BootRelease => println!("Boot released!"),
//!             ButtonEvent::PowerPress => println!("Power pressed!"),
//!             ButtonEvent::PowerRelease => println!("Power released!"),
//!         }
//!     }
//!     thread::sleep(Duration::from_millis(10));
//! }
//! ```

use esp_idf_svc::hal::gpio::{Input, PinDriver};
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::peripheral::Peripheral;
use std::error::Error;

/// TCA9554 GPIO expander I2C address
const TCA9554_ADDRESS: u8 = 0x20;
/// TCA9554 input register (read pin states)
const REG_INPUT: u8 = 0x00;
/// TCA9554 configuration register (set pin direction)
const REG_CONFIG: u8 = 0x03;

/// Debounce threshold - number of consecutive identical reads required
const DEBOUNCE_THRESHOLD: u8 = 3;

/// Button events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent {
    /// BOOT button pressed
    BootPress,
    /// BOOT button released
    BootRelease,
    /// PWR button pressed
    PowerPress,
    /// PWR button released
    PowerRelease,
}

/// Button state tracker with debouncing
pub struct Buttons<'d, T>
where
    T: esp_idf_svc::hal::gpio::Pin + esp_idf_svc::hal::gpio::InputPin,
{
    /// GPIO0 boot button (active low with pull-up)
    boot_pin: PinDriver<'d, T, Input>,

    /// Boot button state
    boot_last_state: bool,
    boot_debounce: u8,

    /// Power button state
    pwr_last_state: bool,
    pwr_debounce: u8,
}

impl<'d, T> Buttons<'d, T>
where
    T: esp_idf_svc::hal::gpio::Pin + esp_idf_svc::hal::gpio::InputPin,
{
    /// Create a new button handler
    ///
    /// # Arguments
    /// * `i2c` - I2C driver for accessing GPIO expander
    /// * `boot_pin` - GPIO0 pin for boot button
    pub fn new(
        i2c: &mut I2cDriver,
        boot_pin: impl Peripheral<P = T> + 'd,
    ) -> Result<Self, Box<dyn Error>> {
        // Configure GPIO0 as input with pull-up (active low)
        let boot_pin = PinDriver::input(boot_pin)?;

        // Configure EXIO4 (PWR button) as input on GPIO expander
        Self::configure_gpio_expander(i2c)?;

        Ok(Self {
            boot_pin,
            boot_last_state: false,
            boot_debounce: 0,
            pwr_last_state: false,
            pwr_debounce: 0,
        })
    }

    /// Configure GPIO expander to set EXIO4 as input
    fn configure_gpio_expander(i2c: &mut I2cDriver) -> Result<(), Box<dyn Error>> {
        // Read current configuration
        let mut config = [0u8; 1];
        i2c.write_read(TCA9554_ADDRESS, &[REG_CONFIG], &mut config, 1000)?;

        // Set bit 4 to 1 (input mode for EXIO4)
        let new_config = config[0] | 0b0001_0000;
        i2c.write(TCA9554_ADDRESS, &[REG_CONFIG, new_config], 1000)?;

        Ok(())
    }

    /// Read PWR button state from GPIO expander (EXIO4)
    fn read_pwr_button(i2c: &mut I2cDriver) -> Result<bool, Box<dyn Error>> {
        let mut input_state = [0u8; 1];
        i2c.write_read(TCA9554_ADDRESS, &[REG_INPUT], &mut input_state, 1000)?;

        // EXIO4 is bit 4, active low
        let pin_high = (input_state[0] & 0b0001_0000) != 0;
        Ok(!pin_high) // Invert for active-low
    }

    /// Poll both buttons and return any state changes (with debouncing)
    ///
    /// Call this regularly (e.g., every 10ms) to detect button presses
    pub fn poll(&mut self, i2c: &mut I2cDriver) -> Result<Option<ButtonEvent>, Box<dyn Error>> {
        // Poll BOOT button (GPIO0, active low)
        let boot_pressed = self.boot_pin.is_low();

        if boot_pressed != self.boot_last_state {
            self.boot_debounce = self.boot_debounce.saturating_add(1);

            if self.boot_debounce >= DEBOUNCE_THRESHOLD {
                self.boot_last_state = boot_pressed;
                self.boot_debounce = 0;

                return Ok(Some(if boot_pressed {
                    ButtonEvent::BootPress
                } else {
                    ButtonEvent::BootRelease
                }));
            }
        } else {
            self.boot_debounce = 0;
        }

        // Poll PWR button (EXIO4 via GPIO expander, active low)
        if let Ok(pwr_pressed) = Self::read_pwr_button(i2c) {
            if pwr_pressed != self.pwr_last_state {
                self.pwr_debounce = self.pwr_debounce.saturating_add(1);

                if self.pwr_debounce >= DEBOUNCE_THRESHOLD {
                    self.pwr_last_state = pwr_pressed;
                    self.pwr_debounce = 0;

                    return Ok(Some(if pwr_pressed {
                        ButtonEvent::PowerPress
                    } else {
                        ButtonEvent::PowerRelease
                    }));
                }
            } else {
                self.pwr_debounce = 0;
            }
        }

        Ok(None)
    }

    /// Get current boot button state (true = pressed)
    #[allow(dead_code)]
    pub fn is_boot_pressed(&self) -> bool {
        self.boot_last_state
    }

    /// Get current power button state (true = pressed)
    #[allow(dead_code)]
    pub fn is_power_pressed(&self) -> bool {
        self.pwr_last_state
    }
}
