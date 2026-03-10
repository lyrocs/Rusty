// .spr sprite decoder for ESP32-C6 / Rustymon
//
// File format produced by tools/gif_to_spr.py — see that file for the full
// spec.  Brief recap:
//
//   Header (14 bytes): magic "SPR1", w, h, frame_count, pal_size, flags
//   Frame table (frame_count × 6 bytes): (offset: u32, delay_ms: u16)
//   Palette (pal_size × 2 bytes): RGB565 LE; index 0 = transparent when flagged
//   Frame blocks (at offsets from frame table):
//     frame_type: u8   (0 = keyframe, 1 = delta)
//     data_size:  u32
//     data:       N bytes  (encoded stream)
//
// Keyframe stream
//   0x00..0x7F  LITERAL  next (c+1) palette indices    (1..128 pixels)
//   0x80..0xFF  RUN      next byte repeated (c−0x7F)×   (1..128 pixels)
//
// Delta frame stream
//   0x00..0x7F  SKIP     skip (c+1) pixels (keep prev)  (1..128)
//   0x80..0xBF  LITERAL  next (c−0x7F) new indices       (1..64)
//   0xC0..0xFF  RUN      next byte × (c−0xBF)            (1..64)

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};

// ─── Constants ────────────────────────────────────────────────────────────────

const MAGIC: &[u8; 4] = b"SPR1";
const FLAG_TRANSPARENT: u16 = 0x0001;

const FRAME_KEYFRAME: u8 = 0;
const FRAME_DELTA:    u8 = 1;

// ─── Public types ─────────────────────────────────────────────────────────────

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
    pub width:       u16,
    pub height:      u16,
    pub frame_count: u16,
    pub flags:       u16,
    /// RGB565 palette (index 0 = transparent when FLAG_TRANSPARENT is set)
    palette:         Vec<u16>,
    /// Display delay per frame in ms
    pub delays:      Vec<u16>,
    /// Encoded byte stream per frame
    frame_data:      Vec<Vec<u8>>,
    frame_types:     Vec<u8>,
    /// Decoded pixels of the currently shown frame (RGB565)
    pixels:          Vec<u16>,
    /// Which frame is currently decoded into `pixels`
    current_frame:   usize,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

impl Sprite {
    /// Parse a .spr file from a byte slice (the full file content).
    pub fn from_bytes(data: &[u8]) -> Result<Self, SprError> {
        let mut c = Cursor::new(data);

        // Magic
        if c.read_bytes(4)? != MAGIC {
            return Err(SprError::BadMagic);
        }

        // Header
        let width       = c.read_u16()?;
        let height      = c.read_u16()?;
        let frame_count = c.read_u16()?;
        let pal_size    = c.read_u16()?;
        let flags       = c.read_u16()?;

        // Frame table
        let mut offsets: Vec<u32> = Vec::with_capacity(frame_count as usize);
        let mut delays:  Vec<u16> = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            offsets.push(c.read_u32()?);
            delays.push(c.read_u16()?);
        }

        // Palette
        let mut palette: Vec<u16> = Vec::with_capacity(pal_size as usize);
        for _ in 0..pal_size {
            palette.push(c.read_u16()?);
        }

        // Frame blocks (seek by offset from file start)
        let n_pixels = width as usize * height as usize;
        let mut frame_data:  Vec<Vec<u8>> = Vec::with_capacity(frame_count as usize);
        let mut frame_types: Vec<u8>      = Vec::with_capacity(frame_count as usize);

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

        // Allocate pixel buffer and decode frame 0
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

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn is_transparent(&self) -> bool {
        self.flags & FLAG_TRANSPARENT != 0
    }

    pub fn current_delay_ms(&self) -> u16 {
        self.delays.get(self.current_frame).copied().unwrap_or(100)
    }

    // ── Frame navigation ──────────────────────────────────────────────────────

    /// Advance to the next frame, decoding it in place.  Wraps around.
    pub fn next_frame(&mut self) -> Result<(), SprError> {
        let next = (self.current_frame + 1) % self.frame_count as usize;
        self.seek_frame(next)
    }

    /// Decode a specific frame.  If it's a delta from a non-adjacent frame,
    /// we replay from the last keyframe.
    pub fn seek_frame(&mut self, target: usize) -> Result<(), SprError> {
        if target == self.current_frame {
            return Ok(());
        }

        // If the target frame is a delta and we're not at the right predecessor,
        // find the preceding keyframe and replay forward.
        let start = if self.frame_types[target] == FRAME_KEYFRAME
                       || target == self.current_frame + 1
        {
            target
        } else {
            // Walk backwards to the nearest keyframe
            let mut k = target;
            while k > 0 && self.frame_types[k] != FRAME_KEYFRAME {
                k -= 1;
            }
            k
        };

        // If start != current_frame we need to decode from `start`
        if start != self.current_frame {
            // Must restart from a keyframe
            let kf = if self.frame_types[start] == FRAME_KEYFRAME {
                start
            } else {
                0
            };
            decode_keyframe(&self.frame_data[kf], &self.palette, &mut self.pixels)?;
            self.current_frame = kf;
        }

        // Apply frames from current_frame+1 up to target
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

    // ── Rendering ─────────────────────────────────────────────────────────────

    /// Draw the current frame at (x, y) on any embedded-graphics DrawTarget.
    /// Transparent pixels (index 0 when FLAG_TRANSPARENT) are skipped.
    pub fn draw<D: DrawTarget<Color = Rgb888>>(
        &self,
        display: &mut D,
        x: i32,
        y: i32,
    ) {
        let w = self.width  as i32;
        let h = self.height as i32;
        let transp = self.is_transparent();

        for row in 0..h {
            for col in 0..w {
                let idx = (row * w + col) as usize;
                let color_rgb565 = self.pixels[idx];

                // Skip transparent pixels
                if transp && color_rgb565 == self.palette[0] {
                    continue;
                }

                let rgb = rgb565_to_rgb888(color_rgb565);
                let pt = Point::new(x + col, y + row);
                let _ = Pixel(pt, rgb).draw(display);
            }
        }
    }

    /// Draw the current frame into a filled rectangle, painting transparent
    /// pixels with `bg` instead of skipping them.
    pub fn draw_with_bg<D: DrawTarget<Color = Rgb888>>(
        &self,
        display: &mut D,
        x: i32,
        y: i32,
        bg: Rgb888,
    ) {
        let w = self.width  as i32;
        let h = self.height as i32;
        let transp = self.is_transparent();

        for row in 0..h {
            for col in 0..w {
                let idx = (row * w + col) as usize;
                let color_rgb565 = self.pixels[idx];
                let rgb = if transp && color_rgb565 == self.palette[0] {
                    bg
                } else {
                    rgb565_to_rgb888(color_rgb565)
                };
                let _ = Pixel(Point::new(x + col, y + row), rgb).draw(display);
            }
        }
    }
}

// ─── Frame decoders ───────────────────────────────────────────────────────────

fn decode_keyframe(data: &[u8], palette: &[u16], out: &mut Vec<u16>)
    -> Result<(), SprError>
{
    let mut c   = Cursor::new(data);
    let mut dst = 0usize;
    let cap     = out.len();

    while c.remaining() > 0 && dst < cap {
        let ctrl = c.read_u8()?;
        if ctrl <= 0x7F {
            // LITERAL: next (ctrl+1) palette indices
            let count = ctrl as usize + 1;
            for _ in 0..count {
                if dst >= cap { break; }
                let idx = c.read_u8()? as usize;
                out[dst] = *palette.get(idx).ok_or(SprError::InvalidData("pal idx"))?;
                dst += 1;
            }
        } else {
            // RUN: next byte repeated (ctrl−0x7F) times
            let count = (ctrl - 0x7F) as usize;
            let idx   = c.read_u8()? as usize;
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

fn decode_delta(data: &[u8], palette: &[u16], pixels: &mut Vec<u16>)
    -> Result<(), SprError>
{
    let mut c   = Cursor::new(data);
    let mut dst = 0usize;
    let cap     = pixels.len();

    while c.remaining() > 0 && dst < cap {
        let ctrl = c.read_u8()?;
        if ctrl <= 0x7F {
            // SKIP: advance dst by (ctrl+1)
            dst = (dst + ctrl as usize + 1).min(cap);
        } else if ctrl <= 0xBF {
            // LITERAL: next (ctrl−0x7F) new palette indices
            let count = (ctrl - 0x7F) as usize;
            for _ in 0..count {
                if dst >= cap { break; }
                let idx = c.read_u8()? as usize;
                pixels[dst] = *palette.get(idx).ok_or(SprError::InvalidData("pal idx"))?;
                dst += 1;
            }
        } else {
            // RUN: next byte repeated (ctrl−0xBF) times
            let count = (ctrl - 0xBF) as usize;
            let idx   = c.read_u8()? as usize;
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

// ─── Colour conversion ────────────────────────────────────────────────────────

#[inline(always)]
fn rgb565_to_rgb888(c: u16) -> Rgb888 {
    let r5 = ((c >> 11) & 0x1F) as u8;
    let g6 = ((c >>  5) & 0x3F) as u8;
    let b5 = ( c        & 0x1F) as u8;
    Rgb888::new(
        (r5 << 3) | (r5 >> 2),
        (g6 << 2) | (g6 >> 4),
        (b5 << 3) | (b5 >> 2),
    )
}

// ─── Minimal byte-slice cursor ────────────────────────────────────────────────

struct Cursor<'a> {
    data: &'a [u8],
    pos:  usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self { Self { data, pos: 0 } }
    fn remaining(&self) -> usize  { self.data.len().saturating_sub(self.pos) }

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

// ─── SD card integration ──────────────────────────────────────────────────────

/// Load a .spr file from the SD card root directory.
///
/// ```rust
/// let mut sd = SdCardResource::new(spi_sd)?;
/// let mut sprite = load_sprite(&mut sd, "MONSTER.SPR")?;
/// sprite.draw(&mut display, 88, 100);   // centre on 240-wide screen
/// ```
pub fn load_sprite<DEV>(
    sd: &mut crate::sdcard::SdCardResource<DEV>,
    filename: &str,
) -> Result<Sprite, Box<dyn std::error::Error>>
where
    DEV: embedded_hal::spi::SpiDevice<Error: core::fmt::Debug>,
{
    log::info!("Loading sprite: {}", filename);
    let bytes = sd.read_file(filename)?;
    let sprite = Sprite::from_bytes(&bytes)
        .map_err(|e| format!("spr parse error: {e}"))?;
    log::info!(
        "  {}×{}  {} frames  {} palette entries",
        sprite.width, sprite.height,
        sprite.frame_count, sprite.palette.len()
    );
    Ok(sprite)
}
