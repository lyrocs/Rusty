use crate::display::{Canvas, Color, FontSize};
use crate::ui::Widget;
use embedded_graphics::pixelcolor::Rgb888;

/// Tappable button widget with label and selection state.
pub struct ButtonWidget {
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub selected: bool,
    pub color: Rgb888,
    pub selected_color: Rgb888,
    pub text_color: Rgb888,
    tag_name: String,
}

impl ButtonWidget {
    pub fn new(label: &str, width: u32, height: u32, tag: &str) -> Self {
        Self {
            label: label.to_string(),
            width,
            height,
            selected: false,
            color: Color::DARK_GRAY,
            selected_color: Color::BLUE,
            text_color: Color::WHITE,
            tag_name: tag.to_string(),
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn color(mut self, normal: Rgb888, selected: Rgb888) -> Self {
        self.color = normal;
        self.selected_color = selected;
        self
    }

    pub fn text_color(mut self, color: Rgb888) -> Self {
        self.text_color = color;
        self
    }
}

impl Widget for ButtonWidget {
    fn draw(&self, canvas: &mut Canvas, x: i32, y: i32) {
        let bg = if self.selected { self.selected_color } else { self.color };
        canvas.fill_rect(x, y, self.width, self.height, bg);
        canvas.stroke_rect(x, y, self.width, self.height, Color::WHITE, 1);

        // Center the text
        let text_w = self.label.len() as u32 * 6;
        let text_x = x + (self.width as i32 - text_w as i32) / 2;
        let text_y = y + (self.height as i32 + 10) / 2; // baseline offset
        canvas.draw_text(&self.label, text_x, text_y, FontSize::Small, self.text_color);
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn tag(&self) -> Option<&str> {
        Some(&self.tag_name)
    }
}
