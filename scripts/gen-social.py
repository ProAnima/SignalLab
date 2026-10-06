"""Render the card a link to Signal Lab shows: the repository's social preview
and the documentation site's og:image.

    python scripts/gen-social.py      # -> docs/public/social.png (1280x640)

GitHub takes the image by hand (Settings -> General -> Social preview); the
published documentation names it in every page's og:image (docs/.vitepress/config.mts).
The mark comes from src-tauri/icons/icon-1024.png, the colours from
src/styles/tokens.css, as in gen-installer-art.py. One image serves every
language, so its words are English.

A build-time tool like gen-icon.py (Pillow + numpy, Segoe UI from Windows, or
DejaVu Sans elsewhere); its output is committed.
"""

import os

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "docs", "public", "social.png")
MARK = os.path.join(ROOT, "src-tauri", "icons", "icon-1024.png")
W, H = 1280, 640
SS = 2  # supersampling

# --- src/styles/tokens.css ---
BG_TOP = (13, 19, 28)  # --surface-1
BG_BOTTOM = (7, 10, 15)  # --bg
TEXT = (227, 233, 242)  # --text
DIM = (147, 161, 181)  # --text-dim
ACCENT = (62, 230, 176)  # --accent
CYAN = (53, 200, 232)  # the brand gradient's middle stop
VIOLET = (139, 124, 255)  # its end

TAGLINE = ["The lab for the protocols your show,", "installation and IoT gear speaks."]
PROTOCOLS = ["OSC", "UDP", "TCP", "HTTP", "WebSocket", "MQTT"]
FOOTER = "Windows · Linux · Server · Docker"
WHERE = "github.com/ProAnima/SignalLab"


def font(bold, size):
    names = (["seguisb.ttf", "segoeuib.ttf"] if bold else ["segoeui.ttf"]) + (["DejaVuSans-Bold.ttf"] if bold else ["DejaVuSans.ttf"])
    folders = [os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "Fonts"), "/usr/share/fonts/truetype/dejavu"]
    for name in names:
        for folder in folders:
            path = os.path.join(folder, name)
            if os.path.exists(path):
                return ImageFont.truetype(path, size)
    raise SystemExit("no Segoe UI or DejaVu Sans font found")


def background():
    t = np.linspace(0, 1, H, dtype=np.float32)[:, None, None]
    rgb = np.repeat(np.array(BG_TOP, np.float32) * (1 - t) + np.array(BG_BOTTOM, np.float32) * t, W, axis=1)
    y, x = np.mgrid[0:H, 0:W].astype(np.float32)
    for cx, cy, radius, color, strength in ((250, 250, 420, CYAN, 0.20), (1150, 560, 520, VIOLET, 0.12)):
        a = (np.clip(1 - np.sqrt((x - cx) ** 2 + (y - cy) ** 2) / radius, 0, 1) ** 2 * strength)[:, :, None]
        rgb = rgb * (1 - a) + np.array(color, np.float32) * a
    for gy in range(16, H, 32):  # the experiment canvas's dot grid, faint
        for gx in range(16, W, 32):
            rgb[gy, gx] = rgb[gy, gx] * 0.93 + np.array(TEXT, np.float32) * 0.07
    return Image.fromarray(rgb.clip(0, 255).astype(np.uint8), "RGB").convert("RGBA")


def trace(y):
    """A scope trace along the bottom: flat, the mark's spike, flat — with a soft glow."""
    layer = Image.new("RGBA", (W * SS, H * SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    shape = [(0, 0), (0.62, 0), (0.67, -0.9), (0.73, 1.1), (0.78, -0.5), (0.82, 0), (1, 0)]
    pts = [(W * fx * SS, (y + fy * 34) * SS) for fx, fy in shape]
    d.line(pts, fill=ACCENT + (255,), width=3 * SS, joint="curve")
    blurred = layer.filter(ImageFilter.GaussianBlur(8 * SS))
    halo = Image.new("RGBA", layer.size, (0, 0, 0, 0))
    halo.paste(blurred, (0, 0), blurred)
    return Image.alpha_composite(halo, layer).resize((W, H), Image.LANCZOS)


def main():
    img = background()
    img.alpha_composite(Image.open(MARK).convert("RGBA").resize((220, 220), Image.LANCZOS), (96, 128))
    big = img.resize((W * SS, H * SS), Image.LANCZOS)
    d = ImageDraw.Draw(big)
    x = 372 * SS

    name = font(True, 96 * SS)
    d.text((x, 118 * SS), "Signal", font=name, fill=TEXT)
    d.text((x + d.textlength("Signal ", font=name), 118 * SS), "Lab", font=name, fill=ACCENT)

    line = font(False, 36 * SS)
    for i, text in enumerate(TAGLINE):
        d.text((x, (252 + i * 48) * SS), text, font=line, fill=TEXT)

    chip = font(True, 24 * SS)
    tint = tuple(round(b * 0.88 + a * 0.12) for b, a in zip(BG_TOP, ACCENT))  # the accent at 12 % over the background
    cx = x
    for protocol in PROTOCOLS:
        width = d.textlength(protocol, font=chip) + 36 * SS
        d.rounded_rectangle((cx, 380 * SS, cx + width, 428 * SS), radius=24 * SS, outline=ACCENT, width=2 * SS, fill=tint)
        d.text((cx + 18 * SS, 387 * SS), protocol, font=chip, fill=TEXT)
        cx += width + 14 * SS

    small = font(False, 24 * SS)
    d.text((96 * SS, 566 * SS), FOOTER, font=small, fill=DIM)
    d.text(((W - 96) * SS - d.textlength(WHERE, font=small), 566 * SS), WHERE, font=small, fill=DIM)

    img = big.resize((W, H), Image.LANCZOS)
    img.alpha_composite(trace(500))
    img.convert("RGB").save(OUT, "PNG", optimize=True)
    print(f"wrote {os.path.relpath(OUT, ROOT)} ({W}x{H}, {os.path.getsize(OUT) // 1024} KB)")


if __name__ == "__main__":
    main()
