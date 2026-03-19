use crate::display::Canvas;
use crate::ui::Widget;
use embedded_graphics::pixelcolor::Rgb888;

/// Circle widget.
pub struct CircleWidget {
    pub radius: u32,
    pub fill: Rgb888,
    tag_name: Option<String>,
}

impl CircleWidget {
    pub fn new(radius: u32, fill: Rgb888) -> Self {
        Self {
            radius,
            fill,
            tag_name: None,
        }
    }

    pub fn tag(mut self, name: &str) -> Self {
        self.tag_name = Some(name.to_string());
        self
    }
}

impl Widget for CircleWidget {
    fn draw(&self, canvas: &mut Canvas, x: i32, y: i32) {
        canvas.draw_circle(x + self.radius as i32, y + self.radius as i32, self.radius, self.fill);
    }

    fn size(&self) -> (u32, u32) {
        (self.radius * 2, self.radius * 2)
    }

    fn tag(&self) -> Option<&str> {
        self.tag_name.as_deref()
    }
}
