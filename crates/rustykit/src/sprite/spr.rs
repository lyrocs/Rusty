//! SPR sprite format decoder.
//!
//! Extracted from rustymon/src/sprite.rs. Decodes .spr files produced by
//! tools/gif_to_spr.py.

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

const MAGIC: &[u8; 4] = b"SPR1";
const FLAG_TRANSPARENT: u16 = 0x0001;
const FRAME_KEYFRAME: u8 = 0;
const FRAME_DELTA: u8 = 1;

#[derive(Debug)]
pub enum SprError {
    TooShort,
    BadMagic,
    BadFrameType(u8),
    InvalidData(&'static str),
}

impl core::fmt::Display for SprError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SprError {}

/// A fully-loaded .spr sprite, kept in RAM for fast frame access.
pub struct Sprite {
    pub width: u16,
    pub height: u16,
    pub frame_count: u16,
    pub flags: u16,
    palette: Vec<u16>,
    pub delays: Vec<u16>,
    frame_data: Vec<Vec<u8>>,
    frame_types: Vec<u8>,
    pixels: Vec<u16>,
    current_frame: usize,
}

impl Sprite {
    /// Parse a .spr file from a byte slice.
    pub fn from_bytes(data: &[u8]) -> Result<Self, SprError> {
        let mut c = Cursor::new(data);

        if c.read_bytes(4)? != MAGIC {
            return Err(SprError::BadMagic);
        }

        let width = c.read_u16()?;
        let height = c.read_u16()?;
        let frame_count = c.read_u16()?;
        let pal_size = c.read_u16()?;
        let flags = c.read_u16()?;

        let mut offsets: Vec<u32> = Vec::with_capacity(frame_count as usize);
        let mut delays: Vec<u16> = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            offsets.push(c.read_u32()?);
            delays.push(c.read_u16()?);
        }

        let mut palette: Vec<u16> = Vec::with_capacity(pal_size as usize);
        for _ in 0..pal_size {
            palette.push(c.read_u16()?);
        }

        let n_pixels = width as usize * height as usize;
        let mut frame_data: Vec<Vec<u8>> = Vec::with_capacity(frame_count as usize);
        let mut frame_types: Vec<u8> = Vec::with_capacity(frame_count as usize);

        for i in 0..frame_count as usize {
            let off = offsets[i] as usize;
            if off >= data.len() {
                return Err(SprError::TooShort);
            }
            let mut fc = Cursor::new(&data[off..]);
            let ftype = fc.read_u8()?;
            if ftype != FRAME_KEYFRAME && ftype != FRAME_DELTA {
                return Err(SprError::BadFrameType(ftype));
            }
            let dsz = fc.read_u32()? as usize;
            let enc = fc.read_slice(dsz)?;
            frame_types.push(ftype);
            frame_data.push(enc.to_vec());
        }

        let mut pixels = vec![0u16; n_pixels];
        if frame_count > 0 {
            decode_keyframe(&frame_data[0], &palette, &mut pixels)?;
        }

        Ok(Sprite {
            width,
            height,
            frame_count,
            flags,
            palette,
            delays,
            frame_data,
            frame_types,
            pixels,
            current_frame: 0,
        })
    }

    pub fn is_transparent(&self) -> bool {
        self.flags & FLAG_TRANSPARENT != 0
    }

    pub fn current_delay_ms(&self) -> u16 {
        self.delays.get(self.current_frame).copied().unwrap_or(100)
    }

    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// Advance to the next frame (wraps around).
    pub fn next_frame(&mut self) -> Result<(), SprError> {
        let next = (self.current_frame + 1) % self.frame_count as usize;
        self.seek_frame(next)
    }

    /// Decode a specific frame.
    pub fn seek_frame(&mut self, target: usize) -> Result<(), SprError> {
        if target == self.current_frame {
            return Ok(());
        }

        let start = if self.frame_types[target] == FRAME_KEYFRAME
            || target == self.current_frame + 1
        {
            target
        } else {
            let mut k = target;
            while k > 0 && self.frame_types[k] != FRAME_KEYFRAME {
                k -= 1;
            }
            k
        };

        if start != self.current_frame {
            let kf = if self.frame_types[start] == FRAME_KEYFRAME {
                start
            } else {
                0
            };
            decode_keyframe(&self.frame_data[kf], &self.palette, &mut self.pixels)?;
            self.current_frame = kf;
        }

        for f in (self.current_frame + 1)..=target {
            match self.frame_types[f] {
                FRAME_KEYFRAME => {
                    decode_keyframe(&self.frame_data[f], &self.palette, &mut self.pixels)?;
                }
                FRAME_DELTA => {
                    decode_delta(&self.frame_data[f], &self.palette, &mut self.pixels)?;
                }
                t => return Err(SprError::BadFrameType(t)),
            }
        }
        self.current_frame = target;
        Ok(())
    }

    /// Draw the current frame. Transparent pixels are skipped.
    pub fn draw<D: DrawTarget<Color = Rgb888>>(&self, display: &mut D, x: i32, y: i32) {
        let w = self.width as i32;
        let h = self.height as i32;
        let transp = self.is_transparent();

        for row in 0..h {
            for col in 0..w {
                let idx = (row * w + col) as usize;
                let c = self.pixels[idx];
                if transp && c == self.palette[0] {
                    continue;
                }
                let rgb = rgb565_to_rgb888(c);
                let _ = Pixel(Point::new(x + col, y + row), rgb).draw(display);
            }
        }
    }

    /// Draw with transparent pixels filled with a background color.
    pub fn draw_with_bg<D: DrawTarget<Color = Rgb888>>(
        &self,
        display: &mut D,
        x: i32,
        y: i32,
        bg: Rgb888,
    ) {
        let w = self.width as i32;
        let h = self.height as i32;
        let transp = self.is_transparent();

        for row in 0..h {
            for col in 0..w {
                let idx = (row * w + col) as usize;
                let c = self.pixels[idx];
                let rgb = if transp && c == self.palette[0] {
                    bg
                } else {
                    rgb565_to_rgb888(c)
                };
                let _ = Pixel(Point::new(x + col, y + row), rgb).draw(display);
            }
        }
    }
}

fn decode_keyframe(data: &[u8], palette: &[u16], out: &mut Vec<u16>) -> Result<(), SprError> {
    let mut c = Cursor::new(data);
    let mut dst = 0usize;
    let cap = out.len();

    while c.remaining() > 0 && dst < cap {
        let ctrl = c.read_u8()?;
        if ctrl <= 0x7F {
            let count = ctrl as usize + 1;
            for _ in 0..count {
                if dst >= cap { break; }
                let idx = c.read_u8()? as usize;
                out[dst] = *palette.get(idx).ok_or(SprError::InvalidData("pal idx"))?;
                dst += 1;
            }
        } else {
            let count = (ctrl - 0x7F) as usize;
            let idx = c.read_u8()? as usize;
            let color = *palette.get(idx).ok_or(SprError::InvalidData("pal idx"))?;
            for _ in 0..count {
                if dst >= cap { break; }
                out[dst] = color;
                dst += 1;
            }
        }
    }
    Ok(())
}

fn decode_delta(data: &[u8], palette: &[u16], pixels: &mut Vec<u16>) -> Result<(), SprError> {
    let mut c = Cursor::new(data);
    let mut dst = 0usize;
    let cap = pixels.len();

    while c.remaining() > 0 && dst < cap {
        let ctrl = c.read_u8()?;
        if ctrl <= 0x7F {
            dst = (dst + ctrl as usize + 1).min(cap);
        } else if ctrl <= 0xBF {
            let count = (ctrl - 0x7F) as usize;
            for _ in 0..count {
                if dst >= cap { break; }
                let idx = c.read_u8()? as usize;
                pixels[dst] = *palette.get(idx).ok_or(SprError::InvalidData("pal idx"))?;
                dst += 1;
            }
        } else {
            let count = (ctrl - 0xBF) as usize;
            let idx = c.read_u8()? as usize;
            let color = *palette.get(idx).ok_or(SprError::InvalidData("pal idx"))?;
            for _ in 0..count {
                if dst >= cap { break; }
                pixels[dst] = color;
                dst += 1;
            }
        }
    }
    Ok(())
}

#[inline(always)]
fn rgb565_to_rgb888(c: u16) -> Rgb888 {
    let r5 = ((c >> 11) & 0x1F) as u8;
    let g6 = ((c >> 5) & 0x3F) as u8;
    let b5 = (c & 0x1F) as u8;
    Rgb888::new(
        (r5 << 3) | (r5 >> 2),
        (g6 << 2) | (g6 >> 4),
        (b5 << 3) | (b5 >> 2),
    )
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self { Self { data, pos: 0 } }
    fn remaining(&self) -> usize { self.data.len().saturating_sub(self.pos) }

    fn read_u8(&mut self) -> Result<u8, SprError> {
        if self.pos >= self.data.len() { return Err(SprError::TooShort); }
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    fn read_u16(&mut self) -> Result<u16, SprError> {
        let lo = self.read_u8()? as u16;
        let hi = self.read_u8()? as u16;
        Ok(lo | (hi << 8))
    }

    fn read_u32(&mut self) -> Result<u32, SprError> {
        let lo = self.read_u16()? as u32;
        let hi = self.read_u16()? as u32;
        Ok(lo | (hi << 16))
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], SprError> {
        if self.pos + n > self.data.len() { return Err(SprError::TooShort); }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn read_slice(&mut self, n: usize) -> Result<&'a [u8], SprError> {
        self.read_bytes(n)
    }
}
