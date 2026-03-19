use crate::display::Canvas;
use crate::ui::Widget;

/// Image format for raw pixel data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Rgb565,
}

/// Widget that renders raw image data.
pub struct ImageWidget<'a> {
    pub data: &'a [u8],
    pub width: u16,
    pub height: u16,
    pub format: ImageFormat,
    tag_name: Option<String>,
}

impl<'a> ImageWidget<'a> {
    pub fn rgb565(data: &'a [u8], width: u16, height: u16) -> Self {
        Self {
            data,
            width,
            height,
            format: ImageFormat::Rgb565,
            tag_name: None,
        }
    }

    pub fn tag(mut self, name: &str) -> Self {
        self.tag_name = Some(name.to_string());
        self
    }
}

impl Widget for ImageWidget<'_> {
    fn draw(&self, canvas: &mut Canvas, x: i32, y: i32) {
        match self.format {
            ImageFormat::Rgb565 => {
                canvas.blit_rgb565(x, y, self.width, self.height, self.data, None);
            }
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }

    fn tag(&self) -> Option<&str> {
        self.tag_name.as_deref()
    }
}
