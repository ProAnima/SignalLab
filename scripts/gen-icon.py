"""Render the app icon from the in-app brand mark.

The mark itself lives in src/components/Brand.tsx as inline SVG (32-unit box);
this redraws the same geometry at 1024px so `npx tauri icon` can cut the .ico /
.icns / PNG set from it. Keep the two in sync — the numbers below are lifted
straight from the SVG, except the pulse stroke, which is thickened a little so
it still reads at 16x16 in the taskbar.

    python scripts/gen-icon.py            # -> src-tauri/icons/icon-1024.png
    npx tauri icon src-tauri/icons/icon-1024.png

Needs Pillow + numpy, and is a build-time tool only: nothing in the app or the
Rust engine depends on it.
"""

import os

import numpy as np
from PIL import Image, ImageDraw

SIZE = 1024  # canvas
SS = 4  # supersampling factor for the masks
MARGIN = 0.04  # keep the tile off the icon bounds
BOX = 31.0  # the SVG's tile is 0.5 .. 31.5

# --- brand values, straight from BrandMark ---
STOPS = ((0.00, (0x5D, 0xF0, 0xBD)), (0.55, (0x35, 0xC8, 0xE8)), (1.00, (0x8B, 0x7C, 0xFF)))
TILE_RADIUS = 9.5
EDGE_WIDTH = 0.55  # 1.0 in the SVG, where the tile is 32px wide and this is a hairline
EDGE_ALPHA = 0.28
PULSE = ((5, 16.5), (9.2, 16.5), (11.2, 10.1), (14.3, 22.3), (16.8, 14.2), (18.4, 16.5), (27, 16.5))
PULSE_WIDTH = 2.4  # 2.1 in the SVG; bolder survives the 16px downsample
PULSE_COLOR = (0x08, 0x13, 0x0F)
PULSE_ALPHA = 0.92


def diagonal_gradient(n):
    """The SVG's 0,0 -> 32,32 linear gradient, evaluated per pixel."""
    axis = np.arange(n, dtype=np.float32)
    t = (axis[None, :] + axis[:, None]) / (2 * (n - 1))
    out = np.zeros((n, n, 3), dtype=np.float32)
    for (t0, c0), (t1, c1) in zip(STOPS, STOPS[1:]):
        seg = (t >= t0) & (t <= t1)
        k = ((t - t0) / (t1 - t0))[seg][:, None]
        out[seg] = np.array(c0, dtype=np.float32) * (1 - k) + np.array(c1, dtype=np.float32) * k
    return out


def main():
    inset = SIZE * MARGIN
    span = SIZE - 2 * inset
    scale = span / BOX

    def px(v):  # 32-unit brand space -> canvas pixels
        return inset + (v - 0.5) * scale

    def mask(draw_on):
        """Draw at SS scale, then band-limit down to the output size."""
        img = Image.new("L", (SIZE * SS, SIZE * SS), 0)
        draw_on(ImageDraw.Draw(img), SS)
        return np.asarray(img.resize((SIZE, SIZE), Image.LANCZOS), dtype=np.float32) / 255.0

    def tile_box(s):
        return [inset * s, inset * s, (SIZE - inset) * s - 1, (SIZE - inset) * s - 1]

    tile = mask(lambda d, s: d.rounded_rectangle(tile_box(s), radius=TILE_RADIUS * scale * s, fill=255))
    edge = mask(
        lambda d, s: d.rounded_rectangle(
            tile_box(s), radius=TILE_RADIUS * scale * s, outline=255, width=max(1, round(EDGE_WIDTH * scale * s))
        )
    )

    def draw_pulse(d, s):
        pts = [(px(x) * s, px(y) * s) for x, y in PULSE]
        w = PULSE_WIDTH * scale * s
        d.line(pts, fill=255, width=round(w), joint="curve")
        for x, y in (pts[0], pts[-1]):  # round caps
            d.ellipse([x - w / 2, y - w / 2, x + w / 2, y + w / 2], fill=255)

    pulse = mask(draw_pulse)

    rgb = diagonal_gradient(SIZE)
    for layer, alpha in ((np.full(3, 255.0), edge * EDGE_ALPHA), (np.array(PULSE_COLOR, np.float32), pulse * PULSE_ALPHA)):
        a = (alpha * tile)[:, :, None]  # never paint outside the tile
        rgb = rgb * (1 - a) + layer * a

    out = np.concatenate([rgb, (tile * 255)[:, :, None]], axis=2).clip(0, 255).astype(np.uint8)
    path = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "src-tauri", "icons", "icon-1024.png")
    Image.fromarray(out, "RGBA").save(path)
    print(f"wrote {path} ({SIZE}x{SIZE})")


if __name__ == "__main__":
    main()
