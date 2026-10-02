"""Render the installers' artwork from the brand: the NSIS sidebar and header,
the MSI dialog background and banner.

    python scripts/gen-installer-art.py      # -> src-tauri/installer/*.bmp

The mark comes from src-tauri/icons/icon-1024.png (scripts/gen-icon.py draws
it), the colours from src/styles/tokens.css. Images carry no words but the product
name, because one installer speaks English and Russian.

Sizes are the ones the installers draw them at (96 DPI): NSIS's Welcome/Finish
sidebar 164x314 and page header 150x57, WiX's dialog 493x312 (artwork on the
left 164 px, where the dialog has no text) and banner 493x58 (title text on the
left, so the art stays right). Both tools want 24-bit BMP.

A build-time tool like gen-icon.py (Pillow + numpy, Segoe UI from Windows, or
DejaVu Sans elsewhere); its output is committed.
"""

import os

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "src-tauri", "installer")
MARK = os.path.join(ROOT, "src-tauri", "icons", "icon-1024.png")
SS = 4  # supersampling

# --- src/styles/tokens.css ---
BG_TOP = (13, 19, 28)  # --surface-1
BG_BOTTOM = (7, 10, 15)  # --bg
TEXT = (227, 233, 242)  # --text
DIM = (147, 161, 181)  # --text-dim
ACCENT = (62, 230, 176)  # --accent
ACCENT_ON_WHITE = (11, 140, 102)  # the accent dark enough for white (WCAG AA)
INK = (13, 19, 28)  # text on white
CYAN = (53, 200, 232)  # the brand gradient's middle stop
VIOLET = (139, 124, 255)  # its end


def font(bold, size):
    names = (["seguisb.ttf", "segoeuib.ttf"] if bold else ["segoeui.ttf"]) + (["DejaVuSans-Bold.ttf"] if bold else ["DejaVuSans.ttf"])
    folders = [os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "Fonts"), "/usr/share/fonts/truetype/dejavu"]
    for name in names:
        for folder in folders:
            path = os.path.join(folder, name)
            if os.path.exists(path):
                return ImageFont.truetype(path, size)
    raise SystemExit("no Segoe UI or DejaVu Sans font found")


def mark(size):
    return Image.open(MARK).convert("RGBA").resize((size, size), Image.LANCZOS)


def vertical(width, height, top, bottom):
    t = np.linspace(0, 1, height, dtype=np.float32)[:, None, None]
    rows = np.array(top, np.float32) * (1 - t) + np.array(bottom, np.float32) * t
    return np.repeat(rows, width, axis=1)


def glow(rgb, cx, cy, radius, color, strength):
    h, w = rgb.shape[:2]
    y, x = np.mgrid[0:h, 0:w].astype(np.float32)
    d = np.sqrt((x - cx) ** 2 + (y - cy) ** 2) / radius
    a = (np.clip(1 - d, 0, 1) ** 2 * strength)[:, :, None]
    return rgb * (1 - a) + np.array(color, np.float32) * a


def dots(rgb, step, color, alpha):
    """The experiment canvas's dot grid, faint."""
    out = rgb.copy()
    for y in range(step // 2, rgb.shape[0], step):
        for x in range(step // 2, rgb.shape[1], step):
            out[y, x] = out[y, x] * (1 - alpha) + np.array(color, np.float32) * alpha
    return out


def pulse(width, height, y, color):
    """A scope trace: flat, the mark's spike, flat — drawn with a soft glow."""
    layer = Image.new("RGBA", (width * SS, height * SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    span = width - 28
    shape = [(0, 0), (0.30, 0), (0.38, -0.55), (0.48, 0.75), (0.56, -0.30), (0.62, 0), (1, 0)]
    pts = [((14 + span * fx) * SS, (y + fy * 22) * SS) for fx, fy in shape]
    d.line(pts, fill=color + (255,), width=2 * SS, joint="curve")
    blurred = layer.filter(ImageFilter.GaussianBlur(5 * SS))
    halo = Image.new("RGBA", layer.size, (0, 0, 0, 0))
    halo.paste(blurred, (0, 0), blurred)
    out = Image.alpha_composite(halo, layer)
    return out.resize((width, height), Image.LANCZOS)


def wordmark(draw, center_x, y, size, first, second, on_dark=True):
    face = font(True, size)
    a, b = "Signal", " Lab"
    wa = draw.textlength(a, font=face)
    wb = draw.textlength(b, font=face)
    x = center_x - (wa + wb) / 2
    draw.text((x, y), a, font=face, fill=first)
    draw.text((x + wa, y), b, font=face, fill=second)


def dark_panel(width, height):
    """The sidebar: dark, the mark lit from behind, the name, a trace, the studio."""
    rgb = vertical(width, height, BG_TOP, BG_BOTTOM)
    rgb = glow(rgb, width / 2, 74, 120, CYAN, 0.16)
    rgb = glow(rgb, width * 0.85, height * 0.80, 110, VIOLET, 0.08)
    rgb = dots(rgb, 12, TEXT, 0.05)
    img = Image.fromarray(rgb.clip(0, 255).astype(np.uint8), "RGB").convert("RGBA")
    m = mark(72)
    img.alpha_composite(m, ((width - 72) // 2, 38))
    big = img.resize((width * SS, height * SS), Image.LANCZOS)
    d = ImageDraw.Draw(big)
    wordmark(d, width * SS / 2, 124 * SS, 21 * SS, TEXT, ACCENT)
    studio = font(True, 10 * SS)
    text = "ProAnimaStudio"
    d.text(((width * SS - d.textlength(text, font=studio)) / 2, (height - 30) * SS), text, font=studio, fill=DIM)
    img = big.resize((width, height), Image.LANCZOS)
    img.alpha_composite(pulse(width, height, height * 0.70, ACCENT))
    return img.convert("RGB")


def nsis_header():
    """On the white header of every inner page, left of the page's title."""
    w, h = 150, 57
    img = Image.new("RGBA", (w * SS, h * SS), (255, 255, 255, 255))
    m = mark(34 * SS)
    img.alpha_composite(m, (10 * SS, (h - 34) // 2 * SS))
    d = ImageDraw.Draw(img)
    face = font(True, 15 * SS)
    x, y = 52 * SS, 18 * SS
    d.text((x, y), "Signal", font=face, fill=INK)
    d.text((x + d.textlength("Signal ", font=face), y), "Lab", font=face, fill=ACCENT_ON_WHITE)
    return img.resize((w, h), Image.LANCZOS).convert("RGB")


def wix_dialog():
    """Welcome and finish dialogs: the panel on the left, white where the text goes."""
    w, h, left = 493, 312, 164
    img = Image.new("RGB", (w, h), (255, 255, 255))
    img.paste(dark_panel(left, h), (0, 0))
    return img


def wix_banner():
    """Inner dialogs: the title is on the left, so the mark sits on the right over a fade of the brand."""
    w, h = 493, 58
    t = np.linspace(0, 1, w, dtype=np.float32)[None, :, None]
    tint = np.clip((t - 0.55) / 0.45, 0, 1) ** 1.6 * 0.10
    rgb = np.full((h, w, 3), 255, np.float32) * (1 - tint) + np.array(CYAN, np.float32) * tint
    img = Image.fromarray(rgb.astype(np.uint8), "RGB").convert("RGBA")
    img.alpha_composite(mark(38), (w - 38 - 12, (h - 38) // 2))
    d = ImageDraw.Draw(img)
    for x in range(w):  # the brand gradient as a hairline at the bottom
        k = x / (w - 1)
        c = ACCENT if k < 0.5 else CYAN
        mix = k * 2 if k < 0.5 else (k - 0.5) * 2
        nxt = CYAN if k < 0.5 else VIOLET
        d.point((x, h - 2), fill=tuple(int(c[i] * (1 - mix) + nxt[i] * mix) for i in range(3)))
        d.point((x, h - 1), fill=tuple(int(c[i] * (1 - mix) + nxt[i] * mix) for i in range(3)))
    return img.convert("RGB")


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, image in (
        ("nsis-sidebar.bmp", dark_panel(164, 314)),
        ("nsis-header.bmp", nsis_header()),
        ("wix-dialog.bmp", wix_dialog()),
        ("wix-banner.bmp", wix_banner()),
    ):
        path = os.path.join(OUT, name)
        image.save(path, "BMP")
        print(f"wrote {os.path.relpath(path, ROOT)} ({image.width}x{image.height})")


if __name__ == "__main__":
    main()
