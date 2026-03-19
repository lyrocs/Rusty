use crate::display::Canvas;
use crate::ui::Widget;
use embedded_graphics::pixelcolor::Rgb888;

/// Rectangle widget (filled and/or stroked).
pub struct RectWidget {
    pub width: u32,
    pub height: u32,
    pub fill: Option<Rgb888>,
    pub stroke: Option<(Rgb888, u32)>,
    tag_name: Option<String>,
}

impl RectWidget {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            fill: None,
            stroke: None,
            tag_name: None,
        }
    }

    /// Create a filled rectangle.
    pub fn filled(width: u32, height: u32, color: Rgb888) -> Self {
        Self {
            width,
            height,
            fill: Some(color),
            stroke: None,
            tag_name: None,
        }
    }

    /// Create a stroked (outline) rectangle.
    pub fn stroked(width: u32, height: u32, color: Rgb888, stroke_width: u32) -> Self {
        Self {
            width,
            height,
            fill: None,
            stroke: Some((color, stroke_width)),
            tag_name: None,
        }
    }

    pub fn tag(mut self, name: &str) -> Self {
        self.tag_name = Some(name.to_string());
        self
    }
}

impl Widget for RectWidget {
    fn draw(&self, canvas: &mut Canvas, x: i32, y: i32) {
        if let Some(fill) = self.fill {
            canvas.fill_rect(x, y, self.width, self.height, fill);
        }
        if let Some((color, w)) = self.stroke {
            canvas.stroke_rect(x, y, self.width, self.height, color, w);
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn tag(&self) -> Option<&str> {
        self.tag_name.as_deref()
    }
}
