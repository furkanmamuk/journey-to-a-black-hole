"""Dependency-free PNG checks for Bevy verification captures (8-bit RGB/RGBA)."""
import json
import struct
import sys
import zlib
from pathlib import Path


def read_png(path):
    data = Path(path).read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    cursor, compressed = 8, bytearray()
    while cursor < len(data):
        length = struct.unpack_from(">I", data, cursor)[0]
        kind = data[cursor + 4:cursor + 8]
        payload = data[cursor + 8:cursor + 8 + length]
        if kind == b"IHDR":
            width, height, depth, color, _, _, interlace = struct.unpack(">IIBBBBB", payload)
            assert depth == 8 and color in (2, 6) and interlace == 0
            channels = 3 if color == 2 else 4
        elif kind == b"IDAT":
            compressed.extend(payload)
        cursor += length + 12
    raw = zlib.decompress(compressed)
    stride = width * channels
    previous = bytearray(stride)
    pixels = bytearray()
    for y in range(height):
        start = y * (stride + 1)
        mode = raw[start]
        row = bytearray(raw[start + 1:start + 1 + stride])
        for x in range(stride):
            a = row[x - channels] if x >= channels else 0
            b = previous[x]
            c = previous[x - channels] if x >= channels else 0
            if mode == 1:
                predictor = a
            elif mode == 2:
                predictor = b
            elif mode == 3:
                predictor = (a + b) // 2
            elif mode == 4:
                p = a + b - c
                da, db, dc = abs(p - a), abs(p - b), abs(p - c)
                predictor = a if da <= db and da <= dc else b if db <= dc else c
            else:
                assert mode == 0
                predictor = 0
            row[x] = (row[x] + predictor) & 255
        pixels.extend(row)
        previous = row
    return width, height, channels, pixels


def inspect(path):
    w, h, channels, pixels = read_png(path)
    black, magenta = 0, 0
    for i in range(0, len(pixels), channels):
        r, g, b = pixels[i:i + 3]
        black += r == g == b == 0
        # ACES desaturates the HDR diagnostic; detect its magenta hue, not zero G.
        magenta += r > 180 and b > 120 and g < 0.8 * min(r, b)
    # Quantized 1920x1080 output from 480x270: every 4x4 cell must be constant.
    nonconstant = 0
    for y in range(0, h - 3, 4):
        for x in range(0, w - 3, 4):
            offset = (y * w + x) * channels
            reference = pixels[offset:offset + 3]
            if any(pixels[((y + dy) * w + x + dx) * channels:
                          ((y + dy) * w + x + dx) * channels + 3] != reference
                   for dy in range(4) for dx in range(4)):
                nonconstant += 1
    return {"file": str(path), "size": [w, h], "exact_black_percent": 100 * black / (w * h),
            "magenta_pixels": magenta, "nonconstant_4x4_cells": nonconstant}


if __name__ == "__main__":
    print(json.dumps([inspect(path) for path in sys.argv[1:]], indent=2))
