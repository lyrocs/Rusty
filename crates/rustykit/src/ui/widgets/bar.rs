use crate::display::{Canvas, Color};
use crate::ui::Widget;
use embedded_graphics::pixelcolor::Rgb888;

/// Progress/HP bar widget.
pub struct BarWidget {
    pub width: u32,
    pub height: u32,
    pub percent: u8,
    pub fill_color: Rgb888,
    pub bg_color: Rgb888,
    pub border_color: Option<Rgb888>,
    tag_name: Option<String>,
}

impl BarWidget {
    pub fn new(width: u32, height: u32, percent: u8, fill_color: Rgb888) -> Self {
        Self {
            width,
            height,
            percent: percent.min(100),
            fill_color,
            bg_color: Color::DARK_GRAY,
            border_color: None,
            tag_name: None,
        }
    }

    pub fn bg_color(mut self, color: Rgb888) -> Self {
        self.bg_color = color;
        self
    }

    pub fn border(mut self, color: Rgb888) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn tag(mut self, name: &str) -> Self {
        self.tag_name = Some(name.to_string());
        self
    }
}

impl Widget for BarWidget {
    fn draw(&self, canvas: &mut Canvas, x: i32, y: i32) {
        if let Some(border) = self.border_color {
            canvas.stroke_rect(x, y, self.width, self.height, border, 1);
            canvas.draw_bar(
                x + 1,
                y + 1,
                self.width.saturating_sub(2),
                self.height.saturating_sub(2),
                self.percent,
                self.fill_color,
                self.bg_color,
            );
        } else {
            canvas.draw_bar(x, y, self.width, self.height, self.percent, self.fill_color, self.bg_color);
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn tag(&self) -> Option<&str> {
        self.tag_name.as_deref()
    }
}
