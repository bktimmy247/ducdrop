from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
ICONS = ROOT / "src-tauri" / "icons"
ICONS.mkdir(parents=True, exist_ok=True)
STATIC = ROOT / "static"
STATIC.mkdir(parents=True, exist_ok=True)


def make(size: int) -> Image.Image:
    s = size / 256
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    def sc(v): return int(round(v * s))
    # rounded square gradient-ish (two layers)
    d.rounded_rectangle([sc(8), sc(8), sc(248), sc(248)], radius=sc(56), fill=(38, 112, 224, 255))
    d.rounded_rectangle([sc(8), sc(8), sc(248), sc(160)], radius=sc(56), fill=(75, 147, 255, 255))
    # down arrow (white)
    cx = sc(128)
    # shaft
    d.rounded_rectangle([cx - sc(20), sc(70), cx + sc(20), sc(150)], radius=sc(12), fill=(255, 255, 255, 255))
    # head
    d.polygon([(cx - sc(52), sc(140)), (cx + sc(52), sc(140)), (cx, sc(196))], fill=(255, 255, 255, 255))
    # tray line
    d.rounded_rectangle([sc(74), sc(200), sc(182), sc(214)], radius=sc(7), fill=(255, 255, 255, 235))
    return img


for size, name in [(32, "32x32.png"), (128, "128x128.png"), (256, "128x128@2x.png")]:
    make(size).save(ICONS / name)

make(256).save(ICONS / "icon.png")
make(256).save(
    ICONS / "icon.ico",
    sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (128, 128), (256, 256)],
)
make(64).save(STATIC / "favicon.png")

print("DucDrop icons generated ->", ICONS)
