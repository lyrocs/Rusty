use crate::display::{Canvas, FontSize};
use crate::ui::Widget;
use embedded_graphics::pixelcolor::Rgb888;

/// Text widget for rendering strings.
pub struct TextWidget {
    pub text: String,
    pub font: FontSize,
    pub color: Rgb888,
    tag_name: Option<String>,
}

impl TextWidget {
    pub fn new(text: &str, font: FontSize, color: Rgb888) -> Self {
        Self {
            text: text.to_string(),
            font,
            color,
            tag_name: None,
        }
    }

    /// Create a small font text widget.
    pub fn small(text: &str, color: Rgb888) -> Self {
        Self::new(text, FontSize::Small, color)
    }

    /// Create a large font text widget.
    pub fn large(text: &str, color: Rgb888) -> Self {
        Self::new(text, FontSize::Large, color)
    }

    pub fn tag(mut self, name: &str) -> Self {
        self.tag_name = Some(name.to_string());
        self
    }
}

impl Widget for TextWidget {
    fn draw(&self, canvas: &mut Canvas, x: i32, y: i32) {
        canvas.draw_text(&self.text, x, y, self.font, self.color);
    }

    fn size(&self) -> (u32, u32) {
        let char_w = match self.font {
            FontSize::Small => 6,
            FontSize::Large => 10,
        };
        let char_h = match self.font {
            FontSize::Small => 10,
            FontSize::Large => 20,
        };
        (self.text.len() as u32 * char_w, char_h)
    }

    fn tag(&self) -> Option<&str> {
        self.tag_name.as_deref()
    }
}
