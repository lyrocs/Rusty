use embedded_graphics::pixelcolor::Rgb888;

/// Common color constants for convenience.
pub struct Color;

impl Color {
    pub const BLACK: Rgb888 = Rgb888::new(0, 0, 0);
    pub const WHITE: Rgb888 = Rgb888::new(255, 255, 255);
    pub const RED: Rgb888 = Rgb888::new(255, 0, 0);
    pub const GREEN: Rgb888 = Rgb888::new(0, 255, 0);
    pub const BLUE: Rgb888 = Rgb888::new(0, 0, 255);
    pub const YELLOW: Rgb888 = Rgb888::new(255, 255, 0);
    pub const CYAN: Rgb888 = Rgb888::new(0, 255, 255);
    pub const MAGENTA: Rgb888 = Rgb888::new(255, 0, 255);
    pub const ORANGE: Rgb888 = Rgb888::new(255, 165, 0);
    pub const GRAY: Rgb888 = Rgb888::new(128, 128, 128);
    pub const DARK_GRAY: Rgb888 = Rgb888::new(64, 64, 64);

    /// Create a color from RGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Rgb888 {
        Rgb888::new(r, g, b)
    }
}

/// Font sizes available for text rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSize {
    /// 6x10 pixel font
    Small,
    /// 10x20 pixel font
    Large,
}
