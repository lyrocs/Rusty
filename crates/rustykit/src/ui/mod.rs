//! UI subsystem: Widget trait, View composition, and built-in widgets.

pub mod widgets;

use crate::display::Canvas;

/// Trait for renderable UI elements.
pub trait Widget {
    /// Render this widget at the given origin.
    fn draw(&self, canvas: &mut Canvas, x: i32, y: i32);

    /// Bounding box (width, height) for hit testing.
    fn size(&self) -> (u32, u32);

    /// Optional tag for hit testing. If set, `View::hit_test` returns this.
    fn tag(&self) -> Option<&str> {
        None
    }
}

struct PositionedWidget {
    x: i32,
    y: i32,
    widget: Box<dyn Widget>,
}

/// A positioned collection of widgets. Build it, draw it, done.
///
/// Immediate-mode: you create a new `View` each frame and render it.
///
/// # Example
/// ```ignore
/// let view = View::new()
///     .with_background(Color::BLACK)
///     .add(20, 20, TextWidget::large("Hello!", Color::YELLOW))
///     .add(20, 50, BarWidget::new(200, 10, 75, Color::GREEN));
/// view.draw(canvas);
/// ```
pub struct View {
    widgets: Vec<PositionedWidget>,
    background: Option<embedded_graphics::pixelcolor::Rgb888>,
}

impl View {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            background: None,
        }
    }

    /// Set a background color (fills the entire screen before drawing widgets).
    pub fn with_background(mut self, color: embedded_graphics::pixelcolor::Rgb888) -> Self {
        self.background = Some(color);
        self
    }

    /// Add a widget at an absolute position.
    pub fn add<W: Widget + 'static>(mut self, x: i32, y: i32, widget: W) -> Self {
        self.widgets.push(PositionedWidget {
            x,
            y,
            widget: Box::new(widget),
        });
        self
    }

    /// Draw all widgets to the canvas.
    pub fn draw(&self, canvas: &mut Canvas) {
        if let Some(bg) = self.background {
            canvas.clear(bg);
        }
        for pw in &self.widgets {
            pw.widget.draw(canvas, pw.x, pw.y);
        }
    }

    /// Hit-test: which widget (if any) contains the point?
    /// Returns the widget's tag if set.
    pub fn hit_test(&self, x: i32, y: i32) -> Option<&str> {
        // Iterate in reverse (last drawn = on top).
        for pw in self.widgets.iter().rev() {
            let (w, h) = pw.widget.size();
            if x >= pw.x
                && x < pw.x + w as i32
                && y >= pw.y
                && y < pw.y + h as i32
            {
                if let Some(tag) = pw.widget.tag() {
                    return Some(tag);
                }
            }
        }
        None
    }
}
