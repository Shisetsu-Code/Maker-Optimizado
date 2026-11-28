"""Build side-by-side approval sheets from a completed visual comparison."""
from pathlib import Path
import json
import sys
from PIL import Image, ImageDraw, ImageFont

source, output = map(Path, sys.argv[1:])
output.mkdir(exist_ok=True, parents=True)
report = json.loads((source / "comparison.json").read_text())
presets = list(dict.fromkeys(row["id"].split("-")[0] for row in report))
if len(report) != 68 or len(presets) != 17:
    raise ValueError("Se requieren las 68 comparaciones completas")
font = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
title = ImageFont.truetype(font, 42)
label = ImageFont.truetype(font, 27)
pages = []
for preset in presets:
    for view in ["lobby", "panel"]:
        page = Image.new("RGB", (2100, 1660), "white")
        draw = ImageDraw.Draw(page)
        draw.text((40, 20), f"{preset.capitalize()} · {view.capitalize()}", font=title, fill="black")
        for x, kind, heading in [(40, "original", "Original"), (1070, "maker", "Maker Optimizado")]:
            draw.text((x, 93), heading + " · escritorio", font=label, fill="black")
            draw.text((x, 794), heading + " · móvil", font=label, fill="black")
            for mode, y, maxsize in [("desktop", 140, (990, 625)), ("mobile", 846, (393, 798))]:
                with Image.open(source / f"{preset}-{view}-{mode}-{kind}.png") as im:
                    im.thumbnail(maxsize, Image.Resampling.LANCZOS)
                    page.paste(im, (x, y))
        dest = output / f"{preset}-{view}.jpg"
        page.save(dest, quality=87, optimize=True)
        pages.append(dest)
images = [Image.open(path) for path in pages]
try:
    images[0].save(output / "Maker-17-plantillas.pdf", save_all=True, append_images=images[1:], resolution=145, quality=86)
finally:
    for image in images:
        image.close()
