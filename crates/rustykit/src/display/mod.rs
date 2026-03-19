//! Display subsystem: Canvas drawing surface wrapping the ST7789P driver.

pub mod color;
pub mod st7789p;

pub use color::{Color, FontSize};
pub use st7789p::ColorMode;

use crate::hw::pins;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
};

/// Type alias for the concrete display driver used on Waveshare ESP32-C6-Touch-LCD-1.83.
pub type DisplayDriver<'a> =
    st7789p::St7789pDriver<'a, esp_idf_svc::hal::gpio::Gpio3, esp_idf_svc::hal::gpio::Gpio4>;

/// High-level drawing surface wrapping the ST7789P framebuffer.
///
/// Implements `embedded_graphics::DrawTarget` so all eg primitives work directly.
/// Also provides convenience methods for common operations.
pub struct Canvas<'a> {
    driver: DisplayDriver<'a>,
}

impl<'a> Canvas<'a> {
    pub const WIDTH: u16 = pins::LCD_WIDTH;
    pub const HEIGHT: u16 = pins::LCD_HEIGHT;

    pub(crate) fn new(driver: DisplayDriver<'a>) -> Self {
        Self { driver }
    }

    /// Clear the entire screen with a solid color.
    pub fn clear(&mut self, color: Rgb888) {
        let _ = self.driver.clear(color);
    }

    /// Fill a rectangle with a solid color.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Rgb888) {
        let style = PrimitiveStyle::with_fill(color);
        let _ = Rectangle::new(Point::new(x, y), Size::new(w, h))
            .into_styled(style)
            .draw(&mut self.driver);
    }

    /// Draw a rectangle outline.
    pub fn stroke_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Rgb888, stroke_width: u32) {
        let style = PrimitiveStyle::with_stroke(color, stroke_width);
        let _ = Rectangle::new(Point::new(x, y), Size::new(w, h))
            .into_styled(style)
            .draw(&mut self.driver);
    }

    /// Draw text at a position.
    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, font: FontSize, color: Rgb888) {
        match font {
            FontSize::Small => {
                let style = MonoTextStyle::new(&FONT_6X10, color);
                let _ = Text::new(text, Point::new(x, y), style).draw(&mut self.driver);
            }
            FontSize::Large => {
                let style = MonoTextStyle::new(&FONT_10X20, color);
                let _ = Text::new(text, Point::new(x, y), style).draw(&mut self.driver);
            }
        }
    }

    /// Draw a filled circle.
    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: u32, fill: Rgb888) {
        let style = PrimitiveStyle::with_fill(fill);
        let _ = Circle::new(
            Point::new(cx - radius as i32, cy - radius as i32),
            radius * 2,
        )
        .into_styled(style)
        .draw(&mut self.driver);
    }

    /// Draw a progress/HP bar.
    pub fn draw_bar(
        &mut self,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        percent: u8,
        fill_color: Rgb888,
        bg_color: Rgb888,
    ) {
        // Background
        self.fill_rect(x, y, w, h, bg_color);
        // Fill
        let fill_w = (w as u32 * percent.min(100) as u32) / 100;
        if fill_w > 0 {
            self.fill_rect(x, y, fill_w, h, fill_color);
        }
    }

    /// Set a single pixel.
    #[inline]
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Rgb888) {
        self.driver.set_pixel(x, y, color);
    }

    /// Set a single pixel from RGB565.
    #[inline]
    pub fn set_pixel_rgb565(&mut self, x: u16, y: u16, rgb565: u16) {
        self.driver.set_pixel_rgb565(x, y, rgb565);
    }

    /// Blit raw RGB565 data.
    pub fn blit_rgb565(
        &mut self,
        x: i32,
        y: i32,
        w: u16,
        h: u16,
        data: &[u8],
        alpha_mask: Option<&[u8]>,
    ) -> usize {
        self.driver.blit_rgb565(x, y, w, h, data, false, alpha_mask)
    }

    /// Flush framebuffer to display.
    pub fn flush(&mut self) -> crate::error::Result<()> {
        self.driver
            .flush()
            .map_err(|e| crate::error::RustyError::Display(e.to_string()))
    }

    /// Turn display on (exit sleep + backlight).
    pub fn display_on(&mut self) -> crate::error::Result<()> {
        self.driver
            .display_on()
            .map_err(|e| crate::error::RustyError::Display(e.to_string()))
    }

    /// Turn display off (backlight + sleep).
    pub fn display_off(&mut self) -> crate::error::Result<()> {
        self.driver
            .display_off()
            .map_err(|e| crate::error::RustyError::Display(e.to_string()))
    }

    /// Access the underlying driver for advanced operations.
    pub fn driver_mut(&mut self) -> &mut DisplayDriver<'a> {
        &mut self.driver
    }
}

impl DrawTarget for Canvas<'_> {
    type Color = Rgb888;
    type Error = Box<dyn std::error::Error>;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.driver.draw_iter(pixels)
    }
}

impl OriginDimensions for Canvas<'_> {
    fn size(&self) -> Size {
        Size::new(Self::WIDTH as u32, Self::HEIGHT as u32)
    }
}
