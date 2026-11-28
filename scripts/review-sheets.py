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

# Compact contact sheets for reviewing every lobby directly in a conversation.
for idx in range(0, len(presets), 6):
    group = presets[idx:idx+6]
    sheet = Image.new("RGB", (1232, ((len(group)+1)//2)*261+42), "#f8f8f8")
    draw = ImageDraw.Draw(sheet)
    draw.text((12, 8), "Lobbies · Maker Optimizado · escritorio y móvil", font=label, fill="#222")
    for pos, preset in enumerate(group):
        x, y = 12 + (pos%2)*610, 50 + (pos//2)*261
        draw.text((x, y), preset.capitalize(), font=label, fill="#222")
        for mode, dx, size in [("desktop",0,(355,222)),("mobile",365,(111,222))]:
            with Image.open(source / f"{preset}-lobby-{mode}-maker.png") as im:
                im.thumbnail(size,Image.Resampling.LANCZOS)
                sheet.paste(im,(x+dx,y+32))
    sheet.save(output / f"lobbies-{1+idx//6}.jpg", quality=79, optimize=True)

# The largest desktop/mobile differences are collected first for visual review.
for mode in ["desktop", "mobile"]:
    ranked = sorted((x for x in report if x["id"].endswith("-lobby-"+mode)), key=lambda x:-x["changed"]/x["pixels"])[:8]
    cell = (310,194) if mode=="desktop" else (134,272)
    panel=Image.new("RGB",(cell[0]*4,(cell[1]+24)*4),"white")
    draw=ImageDraw.Draw(panel)
    for i,row in enumerate(ranked):
        x=(i%2)*2*cell[0]; y=(i//2)*(cell[1]+24)
        draw.text((x+3,y+2),row["id"].split("-")[0]+"   original / Maker", fill="black")
        for col,kind in enumerate(["original","maker"]):
            with Image.open(source/(row["id"]+"-"+kind+".png")) as im:
                im.thumbnail(cell,Image.Resampling.LANCZOS)
                panel.paste(im,(x+col*cell[0],y+22))
    panel.save(output/("compare-"+mode+".jpg"),quality=76,optimize=True)
