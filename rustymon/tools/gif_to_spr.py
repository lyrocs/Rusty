#!/usr/bin/env python3
"""
GIF → .spr converter for ESP32-C6 (Rustymon)
=============================================

Output format  (.spr)  — little-endian throughout
---------------------------------------------------
  Header (14 bytes)
    [0..3]  magic       "SPR1"
    [4..5]  width       u16
    [6..7]  height      u16
    [8..9]  frame_count u16
   [10..11] pal_size    u16   number of palette entries (1..256)
   [12..13] flags       u16   bit-0 = index-0 is transparent

  Frame table  (frame_count × 6 bytes)
    u32  byte offset from file start to frame block
    u16  display delay in ms

  Palette  (pal_size × 2 bytes)
    RGB565 LE per entry  (index 0 = transparent placeholder when flag set)

  Frame blocks  (one per frame, at offsets given in frame table)
    u8   frame_type   0 = keyframe, 1 = delta
    u32  data_size    byte count of encoded stream
    ...  encoded stream

Keyframe stream  — covers all width×height pixels in row-major order
  Control byte c:
    0x00..0x7F  LITERAL  next (c+1) bytes are palette indices   (1..128)
    0x80..0xFF  RUN      next byte repeated (c−0x7F) times      (1..128)

Delta frame stream  — same pixel order, but only changed pixels
  Control byte c:
    0x00..0x7F  SKIP     skip (c+1) pixels (keep from prev frame)  (1..128)
    0x80..0xBF  LITERAL  next (c−0x7F) bytes are new indices        (1..64)
    0xC0..0xFF  RUN      next byte repeated (c−0xBF) times          (1..64)

Usage
-----
  python gif_to_spr.py monster.gif monster.spr
  python gif_to_spr.py --batch assets/ out/
  python gif_to_spr.py --info monster.spr
"""

import argparse
import struct
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("Error: Pillow required.  pip install Pillow")
    sys.exit(1)


# ── Colour helpers ────────────────────────────────────────────────────────────

def rgb888_to_rgb565(r: int, g: int, b: int) -> int:
    return ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)

def rgb565_to_rgb888(c: int):
    r5 = (c >> 11) & 0x1F
    g6 = (c >>  5) & 0x3F
    b5 =  c        & 0x1F
    return (r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4), (b5 << 3) | (b5 >> 2)


# ── GIF frame extraction ──────────────────────────────────────────────────────

def extract_frames(gif: Image.Image) -> list[Image.Image]:
    """
    Return every GIF frame as a full-canvas RGBA composite,
    honouring GIF disposal methods.
    """
    frames = []
    canvas = Image.new("RGBA", gif.size, (0, 0, 0, 0))
    n = getattr(gif, "n_frames", 1)
    for i in range(n):
        gif.seek(i)
        frame = gif.convert("RGBA")
        disposal = getattr(gif, "disposal_method", 0)
        if disposal == 2:                       # restore to background
            canvas = Image.new("RGBA", gif.size, (0, 0, 0, 0))
        canvas = canvas.copy()
        canvas.paste(frame, (0, 0), frame)      # composite with alpha
        frames.append(canvas.copy())
        if disposal == 3:                       # restore to previous → revert
            canvas = frames[-2].copy() if len(frames) >= 2 else Image.new("RGBA", gif.size)
    gif.seek(0)
    return frames

def get_delays(gif: Image.Image) -> list[int]:
    delays = []
    n = getattr(gif, "n_frames", 1)
    for i in range(n):
        gif.seek(i)
        d = gif.info.get("duration", 100)
        delays.append(max(int(d), 10))
    gif.seek(0)
    return delays

def check_transparency(frames: list[Image.Image]) -> bool:
    for f in frames:
        raw = f.tobytes()       # RGBA → flat bytes [r,g,b,a, r,g,b,a, ...]
        for i in range(3, len(raw), 4):
            if raw[i] < 128:
                return True
    return False


# ── Palette quantisation ──────────────────────────────────────────────────────

def build_palette(frames_rgba: list[Image.Image], has_transparency: bool):
    """
    Quantise all frames to a shared palette of at most 256 entries.

    Returns
    -------
    palette_rgb565 : list[int]   RGB565 values (len = pal_size)
    frames_idx     : list[list[int]]  palette indices per pixel per frame
    """
    max_colors = 255 if has_transparency else 256

    # ── Step 1: build a representative image from all opaque pixels ───────────
    all_rgb = []
    for frame in frames_rgba:
        raw = frame.tobytes()   # flat [r,g,b,a, ...]
        for i in range(0, len(raw), 4):
            if raw[i + 3] >= 128:
                all_rgb.append((raw[i], raw[i + 1], raw[i + 2]))

    if not all_rgb:
        # fully transparent sprite — one dummy entry
        pal_rgb565 = [0x0000] * (2 if has_transparency else 1)
        empty = [[0] * (frames_rgba[0].size[0] * frames_rgba[0].size[1])
                 for _ in frames_rgba]
        return pal_rgb565, empty

    # Create a small image from all opaque pixels for quantisation
    side = min(len(all_rgb), 1 << 16)   # cap at 65 536 pixels
    step = max(1, len(all_rgb) // side)
    sample = all_rgb[::step][:side]
    w = min(len(sample), 256)
    h = (len(sample) + w - 1) // w
    quant_src = Image.new("RGB", (w, h), (0, 0, 0))
    # pad sample to fill image
    sample += [sample[-1]] * (w * h - len(sample))
    quant_src.putdata(sample)

    # ── Step 2: quantise to max_colors ───────────────────────────────────────
    q = quant_src.quantize(colors=max_colors, method=Image.Quantize.MEDIANCUT, dither=0)
    raw_pal = q.getpalette()  # [r, g, b, r, g, b, ...]

    # Build palette image for per-frame nearest-colour lookup
    pal_img = Image.new("P", (1, 1))
    pal_img.putpalette(raw_pal)

    # ── Step 3: build RGB565 palette list ─────────────────────────────────────
    # Index 0 is reserved as transparent when has_transparency=True
    palette_rgb565: list[int] = []
    if has_transparency:
        palette_rgb565.append(0x0000)       # index 0 = transparent

    # getpalette() returns only the entries actually stored (not always 256×3)
    actual_colors = len(raw_pal) // 3
    for i in range(actual_colors):
        r = raw_pal[i * 3]
        g = raw_pal[i * 3 + 1]
        b = raw_pal[i * 3 + 2]
        palette_rgb565.append(rgb888_to_rgb565(r, g, b))

    # ── Step 4: quantise each frame using the shared palette ──────────────────
    offset = 1 if has_transparency else 0
    frames_idx: list[list[int]] = []

    for frame_rgba in frames_rgba:
        frame_rgb = frame_rgba.convert("RGB")
        # Map to nearest colour in our palette
        frame_p = frame_rgb.quantize(palette=pal_img, dither=0)
        q_data   = frame_p.tobytes()            # flat bytes, one index per pixel
        raw_rgba = frame_rgba.tobytes()         # flat [r,g,b,a, ...]

        indices: list[int] = []
        for j, q_idx in enumerate(q_data):
            alpha = raw_rgba[j * 4 + 3]
            if alpha < 128 and has_transparency:
                indices.append(0)               # transparent
            else:
                indices.append(q_idx + offset)  # shift past transparent slot
        frames_idx.append(indices)

    return palette_rgb565, frames_idx


# ── RLE encoders ──────────────────────────────────────────────────────────────

def encode_keyframe(data: list[int]) -> bytes:
    """
    PackBits-style RLE for keyframes (full frame).

    0x00..0x7F  LITERAL  next (c+1) palette indices  (1..128)
    0x80..0xFF  RUN      next byte × (c−0x7F)        (1..128)
    """
    out = bytearray()
    i = 0
    n = len(data)
    while i < n:
        # try to start a run
        j = i + 1
        while j < n and j - i < 128 and data[j] == data[i]:
            j += 1
        run_len = j - i
        if run_len >= 2:
            out.append(0x7F + run_len)   # 0x80 → 1 rep, 0xFF → 128 reps
            out.append(data[i])
            i = j
        else:
            # collect literals until a profitable run begins
            lits: list[int] = []
            while i < n and len(lits) < 128:
                # peek for run of ≥ 2
                k = i + 1
                while k < n and k - i < 2 and data[k] == data[i]:
                    k += 1
                if k - i >= 2:
                    break
                lits.append(data[i])
                i += 1
            out.append(len(lits) - 1)   # 0x00 → 1 literal, 0x7F → 128 literals
            out.extend(lits)
    return bytes(out)


def encode_delta(prev: list[int], curr: list[int]) -> bytes:
    """
    Delta + RLE for subsequent frames.

    0x00..0x7F  SKIP     skip (c+1) unchanged pixels    (1..128)
    0x80..0xBF  LITERAL  next (c−0x7F) new indices       (1..64)
    0xC0..0xFF  RUN      next byte × (c−0xBF)            (1..64)
    """
    assert len(prev) == len(curr)
    n = len(curr)
    changed = [prev[i] != curr[i] for i in range(n)]

    out = bytearray()
    i = 0
    while i < n:
        if not changed[i]:
            # ── SKIP run ──────────────────────────────────────────────────────
            j = i + 1
            while j < n and not changed[j] and j - i < 128:
                j += 1
            out.append(j - i - 1)       # 0x00 → skip 1, 0x7F → skip 128
            i = j
        else:
            # ── changed pixels: run or literals ───────────────────────────────
            j = i + 1
            while j < n and changed[j] and j - i < 64 and curr[j] == curr[i]:
                j += 1
            run_len = j - i
            if run_len >= 2:
                # RUN  0xC0..0xFF → (c−0xBF) repetitions
                out.append(0xBF + run_len)   # 0xC0 → 1 rep, 0xFF → 64 reps
                out.append(curr[i])
                i = j
            else:
                # LITERAL  0x80..0xBF → (c−0x7F) bytes follow
                lits: list[int] = []
                while i < n and changed[i] and len(lits) < 64:
                    # peek for run ≥ 2
                    k = i + 1
                    while k < n and changed[k] and k - i < 2 and curr[k] == curr[i]:
                        k += 1
                    if k - i >= 2:
                        break
                    lits.append(curr[i])
                    i += 1
                out.append(0x7F + len(lits))   # 0x80 → 1 literal, 0xBF → 64
                out.extend(lits)
    return bytes(out)


# ── Main conversion ───────────────────────────────────────────────────────────

HEADER_MAGIC   = b"SPR1"
FLAG_TRANSP    = 0x0001

FRAME_KEYFRAME = 0
FRAME_DELTA    = 1


def convert(input_path: Path, output_path: Path, verbose: bool = True,
            max_frames: int = 0) -> bool:
    """Convert one GIF to .spr.  max_frames=0 means keep all frames."""
    if verbose:
        print(f"  {input_path.name}")

    try:
        gif = Image.open(input_path)
    except Exception as e:
        print(f"    Error opening: {e}")
        return False

    width, height = gif.size
    all_frames = extract_frames(gif)
    all_delays = get_delays(gif)

    # Optional frame subsampling
    if max_frames and len(all_frames) > max_frames:
        indices = [round(i * (len(all_frames) - 1) / (max_frames - 1))
                   for i in range(max_frames)]
        all_frames = [all_frames[i] for i in indices]
        all_delays = [all_delays[i] for i in indices]

    n_frames = len(all_frames)
    has_transp = check_transparency(all_frames)
    flags = FLAG_TRANSP if has_transp else 0

    if verbose:
        print(f"    {width}×{height}  {n_frames} frames  "
              f"{'transparent' if has_transp else 'opaque'}")

    # ── Quantise ──────────────────────────────────────────────────────────────
    palette_rgb565, frames_idx = build_palette(all_frames, has_transp)
    pal_size = len(palette_rgb565)

    if verbose:
        print(f"    Palette: {pal_size} colours")

    # ── Encode frames ─────────────────────────────────────────────────────────
    frame_types:  list[int]   = []
    frame_encoded: list[bytes] = []

    for fi in range(n_frames):
        curr = frames_idx[fi]
        if fi == 0:
            data = encode_keyframe(curr)
            ftype = FRAME_KEYFRAME
        else:
            prev = frames_idx[fi - 1]
            # Use delta only when it's smaller than a new keyframe
            delta = encode_delta(prev, curr)
            key   = encode_keyframe(curr)
            if len(delta) <= len(key):
                data  = delta
                ftype = FRAME_DELTA
            else:
                data  = key
                ftype = FRAME_KEYFRAME
        frame_types.append(ftype)
        frame_encoded.append(data)
        if verbose:
            tag = "key" if ftype == FRAME_KEYFRAME else "dlt"
            print(f"    Frame {fi:2d} [{tag}]  {len(data):5d} B  delay={all_delays[fi]}ms")

    # ── Calculate file offsets ────────────────────────────────────────────────
    HEADER_SIZE     = 14
    FRAME_TABLE_SZ  = n_frames * 6
    PALETTE_SZ      = pal_size * 2
    DATA_START      = HEADER_SIZE + FRAME_TABLE_SZ + PALETTE_SZ

    offsets: list[int] = []
    pos = DATA_START
    for enc in frame_encoded:
        offsets.append(pos)
        pos += 1 + 4 + len(enc)   # frame_type(1) + data_size(4) + data

    # ── Write file ────────────────────────────────────────────────────────────
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "wb") as f:
        # Header
        f.write(HEADER_MAGIC)
        f.write(struct.pack("<HHHHH", width, height, n_frames, pal_size, flags))

        # Frame table
        for i in range(n_frames):
            f.write(struct.pack("<IH", offsets[i], all_delays[i]))

        # Palette
        for c in palette_rgb565:
            f.write(struct.pack("<H", c))

        # Frame blocks
        for ftype, enc in zip(frame_types, frame_encoded):
            f.write(struct.pack("<BI", ftype, len(enc)))
            f.write(enc)

    in_size  = input_path.stat().st_size
    out_size = output_path.stat().st_size
    ratio    = out_size / in_size if in_size else 0
    if verbose:
        print(f"    → {out_size} B  ({ratio:.2f}× original)")
    return True


# ── Info dump ─────────────────────────────────────────────────────────────────

def info(path: Path) -> None:
    with open(path, "rb") as f:
        magic = f.read(4)
        if magic != HEADER_MAGIC:
            print("Not a .spr file")
            return
        w, h, nf, ps, flags = struct.unpack("<HHHHH", f.read(10))
        print(f"SPR1  {w}×{h}  {nf} frames  palette={ps}  "
              f"{'transparent' if flags & FLAG_TRANSP else 'opaque'}")
        for i in range(nf):
            off, delay = struct.unpack("<IH", f.read(6))
            print(f"  Frame {i:2d}  offset={off}  delay={delay}ms")
        for i in range(ps):
            c, = struct.unpack("<H", f.read(2))
            r, g, b = rgb565_to_rgb888(c)
            print(f"  Pal[{i:3d}]  #{r:02x}{g:02x}{b:02x}")
        for i in range(nf):
            ftype, dsz = struct.unpack("<BI", f.read(5))
            tag = "keyframe" if ftype == FRAME_KEYFRAME else "delta"
            print(f"  Frame {i:2d} [{tag}]  {dsz} encoded bytes")
            f.seek(dsz, 1)


# ── Batch conversion ──────────────────────────────────────────────────────────

def batch(input_dir: Path, output_dir: Path, max_frames: int,
          verbose: bool) -> bool:
    gifs = sorted(input_dir.rglob("*.gif")) + sorted(input_dir.rglob("*.GIF"))
    if not gifs:
        print("No GIF files found.")
        return False
    print(f"Found {len(gifs)} GIF(s)")
    ok = err = 0
    for g in gifs:
        rel  = g.relative_to(input_dir)
        out  = output_dir / rel.with_suffix(".spr")
        if convert(g, out, verbose, max_frames):
            ok += 1
        else:
            err += 1
    print(f"\nDone: {ok} OK, {err} failed")
    return err == 0


# ── CLI ───────────────────────────────────────────────────────────────────────

def main() -> None:
    p = argparse.ArgumentParser(
        description="Convert GIF → .spr (RLE + delta, 8-bit palette) for ESP32-C6"
    )
    p.add_argument("input",  help="Input GIF (or directory with --batch)")
    p.add_argument("output", nargs="?", help="Output .spr (or directory with --batch)")
    p.add_argument("--batch", "-b", action="store_true",
                   help="Batch-convert all GIFs in input directory")
    p.add_argument("--info", "-i", action="store_true",
                   help="Dump header info from a .spr file (input only)")
    p.add_argument("--max-frames", "-m", type=int, default=0,
                   help="Limit frames per sprite (0 = keep all)")
    p.add_argument("--quiet", "-q", action="store_true")
    args = p.parse_args()

    if args.info:
        info(Path(args.input))
        return

    if not args.output:
        p.error("output path required (unless --info)")

    if args.batch:
        ok = batch(Path(args.input), Path(args.output),
                   args.max_frames, not args.quiet)
    else:
        ok = convert(Path(args.input), Path(args.output),
                     not args.quiet, args.max_frames)

    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
