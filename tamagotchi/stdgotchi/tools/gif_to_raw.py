#!/usr/bin/env python3
"""
GIF to Raw RGB565 Converter for ESP32-C6

Converts animated GIF files to a simple raw frame format optimized for
streaming playback on memory-constrained devices.

File Format (.raw):
  Header (8 bytes):
    - u16: width
    - u16: height
    - u16: frame_count
    - u16: flags (bit 0: has_transparency)

  Frame Table (6 bytes per frame):
    - u32: file offset to frame data
    - u16: delay in ms

  Frame Data (for each frame):
    - width * height * 2 bytes: RGB565 pixels
    - If has_transparency: width * height / 8 bytes: transparency bitmask

Usage:
  python gif_to_raw.py input.gif output.raw
  python gif_to_raw.py --batch input_dir output_dir
"""

import argparse
import struct
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("Error: Pillow library required. Install with: pip install Pillow")
    sys.exit(1)


def rgb888_to_rgb565(r, g, b):
    """Convert RGB888 to RGB565"""
    return ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)


def convert_frame_to_rgb565(frame, has_alpha=False):
    """Convert a PIL Image frame to RGB565 bytes and optional transparency mask"""
    # Convert to RGBA to handle transparency
    if frame.mode != 'RGBA':
        frame = frame.convert('RGBA')

    width, height = frame.size
    pixels = list(frame.getdata())

    rgb565_data = bytearray()
    alpha_mask = bytearray() if has_alpha else None
    alpha_byte = 0
    bit_pos = 0

    for i, pixel in enumerate(pixels):
        r, g, b, a = pixel

        # Convert to RGB565
        rgb565 = rgb888_to_rgb565(r, g, b)
        # Little-endian for ESP32
        rgb565_data.append(rgb565 & 0xFF)
        rgb565_data.append((rgb565 >> 8) & 0xFF)

        # Build transparency bitmask
        if has_alpha:
            if a > 127:  # Pixel is opaque
                alpha_byte |= (1 << bit_pos)
            bit_pos += 1
            if bit_pos == 8:
                alpha_mask.append(alpha_byte)
                alpha_byte = 0
                bit_pos = 0

    # Flush remaining alpha bits
    if has_alpha and bit_pos > 0:
        alpha_mask.append(alpha_byte)

    return rgb565_data, alpha_mask


def has_transparency(gif):
    """Check if GIF has any transparent pixels"""
    try:
        for frame_num in range(getattr(gif, 'n_frames', 1)):
            gif.seek(frame_num)
            frame = gif.convert('RGBA')
            # Check if any pixel has alpha < 255
            pixels = list(frame.getdata())
            for pixel in pixels:
                if pixel[3] < 255:
                    return True
    except EOFError:
        pass
    gif.seek(0)
    return False


def get_frame_delay(gif):
    """Get frame delay in milliseconds"""
    try:
        delay = gif.info.get('duration', 100)
        return max(delay, 10)  # Minimum 10ms
    except:
        return 100


# Maximum number of frames to keep (for memory-constrained devices)
MAX_FRAMES = 4


def select_frames(n_frames, max_frames=MAX_FRAMES):
    """
    Select which frame indices to keep.
    Keeps first, last, and evenly distributed middle frames.

    Examples:
      n_frames=1 -> [0]
      n_frames=2 -> [0, 1]
      n_frames=3 -> [0, 1, 2]
      n_frames=4 -> [0, 1, 2, 3]
      n_frames=5 -> [0, 1, 3, 4]  (first, middle1, middle2, last)
      n_frames=8 -> [0, 2, 5, 7]  (first, 1/3, 2/3, last)
      n_frames=17 -> [0, 5, 11, 16] (first, 1/3, 2/3, last)
    """
    if n_frames <= max_frames:
        return list(range(n_frames))

    # Always include first (0) and last (n_frames-1)
    # Distribute middle frames evenly
    indices = [0]

    # Add middle frames (evenly distributed)
    middle_count = max_frames - 2
    for i in range(1, middle_count + 1):
        # Calculate position as fraction of total range
        pos = int(i * (n_frames - 1) / (max_frames - 1))
        indices.append(pos)

    # Add last frame
    indices.append(n_frames - 1)

    return indices


def convert_gif_to_raw(input_path, output_path, verbose=True):
    """Convert a single GIF file to raw format"""
    if verbose:
        print(f"Converting: {input_path}")

    try:
        gif = Image.open(input_path)
    except Exception as e:
        print(f"  Error opening file: {e}")
        return False

    width, height = gif.size
    n_frames = getattr(gif, 'n_frames', 1)

    # Select which frames to keep (max 4 frames for memory efficiency)
    selected_indices = select_frames(n_frames, MAX_FRAMES)

    if verbose:
        print(f"  Size: {width}x{height}, Original frames: {n_frames}")
        if n_frames > MAX_FRAMES:
            print(f"  Reducing to {len(selected_indices)} frames: {selected_indices}")

    # Check for transparency
    check_transparency = has_transparency(gif)
    flags = 1 if check_transparency else 0

    if verbose and check_transparency:
        print(f"  Has transparency: yes")

    # Collect selected frames only
    frames = []
    delays = []

    try:
        for frame_num in range(n_frames):
            gif.seek(frame_num)

            # Skip frames not in our selection
            if frame_num not in selected_indices:
                continue

            # Make a copy and convert
            frame = gif.copy()
            delay = get_frame_delay(gif)

            rgb565_data, alpha_mask = convert_frame_to_rgb565(frame, check_transparency)
            frames.append((rgb565_data, alpha_mask))
            delays.append(delay)

            if verbose:
                print(f"  Frame {frame_num}: {len(rgb565_data)} bytes, delay={delay}ms")
    except EOFError:
        pass

    # Calculate offsets
    header_size = 8
    frame_table_size = 6 * len(frames)
    data_offset = header_size + frame_table_size

    frame_offsets = []
    current_offset = data_offset

    for rgb565_data, alpha_mask in frames:
        frame_offsets.append(current_offset)
        current_offset += len(rgb565_data)
        if alpha_mask:
            current_offset += len(alpha_mask)

    # Write output file
    with open(output_path, 'wb') as f:
        # Header
        f.write(struct.pack('<HHHH', width, height, len(frames), flags))

        # Frame table
        for i, offset in enumerate(frame_offsets):
            f.write(struct.pack('<IH', offset, delays[i]))

        # Frame data
        for rgb565_data, alpha_mask in frames:
            f.write(rgb565_data)
            if alpha_mask:
                f.write(alpha_mask)

    output_size = Path(output_path).stat().st_size
    input_size = Path(input_path).stat().st_size

    if verbose:
        ratio = output_size / input_size if input_size > 0 else 0
        print(f"  Output: {output_size} bytes ({ratio:.1f}x original)")

    return True


def batch_convert(input_dir, output_dir, verbose=True):
    """Convert all GIF files in a directory structure"""
    input_path = Path(input_dir)
    output_path = Path(output_dir)

    if not input_path.exists():
        print(f"Error: Input directory does not exist: {input_dir}")
        return False

    # Find all GIF files
    gif_files = list(input_path.rglob("*.gif")) + list(input_path.rglob("*.GIF"))

    if not gif_files:
        print("No GIF files found")
        return False

    print(f"Found {len(gif_files)} GIF files")

    success_count = 0
    fail_count = 0

    for gif_file in gif_files:
        # Maintain directory structure
        relative_path = gif_file.relative_to(input_path)
        raw_file = output_path / relative_path.with_suffix('.raw')

        # Create output directory if needed
        raw_file.parent.mkdir(parents=True, exist_ok=True)

        if convert_gif_to_raw(gif_file, raw_file, verbose):
            success_count += 1
        else:
            fail_count += 1

    print(f"\nConversion complete: {success_count} succeeded, {fail_count} failed")
    return fail_count == 0


def main():
    parser = argparse.ArgumentParser(
        description='Convert GIF files to raw RGB565 format for ESP32'
    )
    parser.add_argument('input', help='Input GIF file or directory (with --batch)')
    parser.add_argument('output', help='Output raw file or directory (with --batch)')
    parser.add_argument('--batch', '-b', action='store_true',
                        help='Batch convert all GIFs in directory')
    parser.add_argument('--quiet', '-q', action='store_true',
                        help='Suppress verbose output')

    args = parser.parse_args()
    verbose = not args.quiet

    if args.batch:
        success = batch_convert(args.input, args.output, verbose)
    else:
        success = convert_gif_to_raw(args.input, args.output, verbose)

    sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()
