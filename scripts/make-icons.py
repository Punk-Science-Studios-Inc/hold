"""Draws the tray icons. Run from the repo root: python scripts/make-icons.py"""

from PIL import Image, ImageDraw

SIZE = 256
DARK = (32, 32, 32, 255)
LIGHT = (250, 250, 250, 255)
SIZES = [(16, 16), (20, 20), (24, 24), (32, 32), (48, 48)]


def curve(p0, c, p1, steps=64):
    """Points along a quadratic Bezier from p0 to p1 with control point c."""
    pts = []
    for i in range(steps + 1):
        t = i / steps
        x = (1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * c[0] + t**2 * p1[0]
        y = (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * c[1] + t**2 * p1[1]
        pts.append((x, y))
    return pts


def open_eye():
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    left, right = (12, 128), (244, 128)
    top = curve(left, (128, 8), right)
    bottom = curve(right, (128, 248), left)
    almond = top + bottom
    d.polygon(almond, fill=LIGHT)
    d.line(almond + [almond[0]], fill=DARK, width=22, joint="curve")
    d.ellipse((76, 76, 180, 180), fill=DARK)
    d.ellipse((104, 92, 132, 120), fill=LIGHT)
    return img


def closed_eye():
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    lid = curve((12, 104), (128, 224), (244, 104))
    lashes = [((72, 150), (52, 196)), ((128, 164), (128, 214)), ((184, 150), (204, 196))]
    for colour, width in ((LIGHT, 46), (DARK, 22)):
        d.line(lid, fill=colour, width=width, joint="curve")
        for a, b in lashes:
            d.line([a, b], fill=colour, width=width)
    return img


open_eye().save("res/eye_open.ico", sizes=SIZES)
closed_eye().save("res/eye_closed.ico", sizes=SIZES)
