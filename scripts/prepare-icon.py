#!/usr/bin/env python3
"""Prepare the DropShrink app icon: crop, center-crop square, rounded transparent corners."""
from PIL import Image, ImageChops, ImageDraw

SRC = "assets/icon-raw.png"
DST = "assets/icon.png"
SIZE = 1024
RADIUS = 190

im = Image.open(SRC).convert("RGB")

# 1) Crop to the drawn rounded square (drop the white margin).
bg = Image.new("RGB", im.size, (255, 255, 255))
bbox = ImageChops.difference(im, bg).getbbox()
if bbox:
    im = im.crop(bbox)

# 2) Center-crop to a square, then upscale/downscale to exactly 1024.
side = min(im.size)
w, h = im.size
left, top = (w - side) // 2, (h - side) // 2
im = im.crop((left, top, left + side, top + side)).resize((SIZE, SIZE), Image.LANCZOS)

# 3) Rounded-corner alpha mask (transparent outside, like modern app icons).
mask = Image.new("L", (SIZE, SIZE), 0)
draw = ImageDraw.Draw(mask)
draw.rounded_rectangle([0, 0, SIZE - 1, SIZE - 1], radius=RADIUS, fill=255)

out = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
out.paste(im, (0, 0), mask)
out.save(DST)
print(f"saved {DST} ({out.size[0]}x{out.size[1]}, RGBA)")
