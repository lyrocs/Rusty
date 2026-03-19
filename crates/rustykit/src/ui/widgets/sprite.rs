use crate::display::Canvas;
use crate::sprite::Sprite;
use crate::ui::Widget;
use embedded_graphics::pixelcolor::Rgb888;

/// Widget that renders a sprite's current frame.
pub struct SpriteWidget<'a> {
    pub sprite: &'a Sprite,
    pub bg_color: Option<Rgb888>,
    tag_name: Option<String>,
}

impl<'a> SpriteWidget<'a> {
    /// Create a sprite widget (transparent pixels skipped).
    pub fn new(sprite: &'a Sprite) -> Self {
        Self {
            sprite,
            bg_color: None,
            tag_name: None,
        }
    }

    /// Create a sprite widget with a background color for transparent pixels.
    pub fn with_bg(sprite: &'a Sprite, bg: Rgb888) -> Self {
        Self {
            sprite,
            bg_color: Some(bg),
            tag_name: None,
        }
    }

    pub fn tag(mut self, name: &str) -> Self {
        self.tag_name = Some(name.to_string());
        self
    }
}

impl Widget for SpriteWidget<'_> {
    fn draw(&self, canvas: &mut Canvas, x: i32, y: i32) {
        if let Some(bg) = self.bg_color {
            self.sprite.draw_with_bg(canvas, x, y, bg);
        } else {
            self.sprite.draw(canvas, x, y);
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.sprite.width as u32, self.sprite.height as u32)
    }

    fn tag(&self) -> Option<&str> {
        self.tag_name.as_deref()
    }
}
