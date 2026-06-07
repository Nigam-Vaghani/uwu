"""Generate pixel-art cat (default) and ghost (ghosty) sprite sheets for UWU Companion."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SKINS = {
    "default": ROOT / "public" / "skins" / "default",
    "ghosty": ROOT / "public" / "skins" / "ghosty",
}
LEGACY = ROOT / "public" / "sprites" / "default"

FRAME = 64


def sheet(width_frames: int) -> Image.Image:
    return Image.new("RGBA", (FRAME * width_frames, FRAME), (0, 0, 0, 0))


def px(draw: ImageDraw.ImageDraw, x: int, y: int, color: tuple[int, ...], size: int = 1) -> None:
    draw.rectangle([x, y, x + size - 1, y + size - 1], fill=color)


def draw_cat_frame(
    img: Image.Image,
    frame_idx: int,
    *,
    anim: str,
) -> None:
    ox = frame_idx * FRAME
    draw = ImageDraw.Draw(img)

    # Cat palette
    body = (255, 183, 120, 255)
    body_dark = (230, 140, 80, 255)
    ear_inner = (255, 200, 180, 255)
    eye = (20, 20, 30, 255)
    eye_shine = (255, 255, 255, 255)
    nose = (255, 140, 160, 255)
    whisker = (180, 120, 80, 255)
    stripe = (220, 130, 70, 255)

    bounce = 0
    leg_shift = 0
    blink = False
    mouth_open = 0
    tail_angle = 0

    if anim == "idle":
        bounce = [0, -1, 0, 1][frame_idx % 4]
        blink = frame_idx == 2
        tail_angle = frame_idx * 2
    elif anim == "walk":
        bounce = [0, -2, 0, -2, 0, -2][frame_idx]
        leg_shift = [-2, 0, 2, 0, -2, 0][frame_idx]
        tail_angle = frame_idx * 3
    elif anim == "sleep":
        bounce = 2
        blink = True
    elif anim == "talk":
        bounce = [0, -1, 0, -1][frame_idx]
        mouth_open = [0, 2, 4, 2][frame_idx]

    cy = 34 + bounce
    cx = 32 + (leg_shift // 2)

    # Tail
    tail_base = (cx + 14, cy + 4)
    for i in range(8):
        tx = tail_base[0] + i * 2 + tail_angle // 2
        ty = tail_base[1] - i + (tail_angle % 3) - 2
        px(draw, ox + tx, ty, body_dark if i % 2 else body, 2)

    # Body
    draw.ellipse([ox + cx - 16, cy - 14, ox + cx + 16, cy + 16], fill=body)
    draw.ellipse([ox + cx - 10, cy - 8, ox + cx + 10, cy + 10], fill=body_dark)

    # Stripes
    for sx in (-6, 0, 6):
        px(draw, ox + cx + sx, cy - 6, stripe, 2)
        px(draw, ox + cx + sx, cy + 2, stripe, 2)

    # Legs
    if anim == "walk":
        legs = [
            (cx - 10 + leg_shift, cy + 14),
            (cx - 4 - leg_shift, cy + 16),
            (cx + 4 + leg_shift, cy + 14),
            (cx + 10 - leg_shift, cy + 16),
        ]
    elif anim == "sleep":
        legs = [(cx - 8, cy + 12), (cx + 8, cy + 12)]
    else:
        legs = [(cx - 9, cy + 15), (cx - 3, cy + 17), (cx + 3, cy + 15), (cx + 9, cy + 17)]

    for lx, ly in legs:
        px(draw, ox + lx, ly, body_dark, 3)
        px(draw, ox + lx, ly + 3, (80, 50, 30, 255), 3)

    # Head
    hx, hy = cx, cy - 16
    draw.ellipse([ox + hx - 14, hy - 12, ox + hx + 14, hy + 12], fill=body)

    # Ears
    for ex, flip in [(-12, 1), (12, -1)]:
        ear_x = hx + ex
        draw.polygon(
            [
                (ox + ear_x, hy - 10),
                (ox + ear_x + flip * 6, hy - 22),
                (ox + ear_x + flip * 10, hy - 8),
            ],
            fill=body,
        )
        draw.polygon(
            [
                (ox + ear_x + flip * 2, hy - 12),
                (ox + ear_x + flip * 5, hy - 18),
                (ox + ear_x + flip * 7, hy - 10),
            ],
            fill=ear_inner,
        )

    # Eyes
    if blink or anim == "sleep":
        for ex in (-7, 7):
            draw.line([(ox + hx + ex - 3, hy - 2), (ox + hx + ex + 3, hy - 2)], fill=eye, width=2)
    else:
        for ex in (-7, 7):
            draw.ellipse([ox + hx + ex - 3, hy - 5, ox + hx + ex + 3, hy + 1], fill=eye)
            px(draw, ox + hx + ex - 1, hy - 4, eye_shine, 1)

    # Nose & mouth
    draw.polygon([(ox + hx, hy + 2), (ox + hx - 2, hy + 5), (ox + hx + 2, hy + 5)], fill=nose)
    if mouth_open > 0:
        draw.ellipse([ox + hx - mouth_open, hy + 6, ox + hx + mouth_open, hy + 10 + mouth_open], fill=(120, 60, 60, 255))
    else:
        draw.arc([ox + hx - 4, hy + 5, ox + hx + 4, hy + 11], 10, 170, fill=eye, width=1)

    # Whiskers
    for side in (-1, 1):
        for wy in (4, 6):
            draw.line([(ox + hx + side * 8, hy + wy), (ox + hx + side * 18, hy + wy - 1)], fill=whisker, width=1)

    if anim == "sleep":
        for i, (zx, zy) in enumerate([(18, 8), (24, 4), (30, 0)]):
            draw.text((ox + hx + zx, hy + zy - 10), "z" * (i + 1), fill=(120, 140, 200, 220))


def draw_ghost_frame(
    img: Image.Image,
    frame_idx: int,
    *,
    anim: str,
) -> None:
    ox = frame_idx * FRAME
    draw = ImageDraw.Draw(img)

    ghost = (210, 210, 220, 255)
    ghost_light = (235, 235, 245, 255)
    ghost_dark = (160, 160, 175, 255)
    outline = (120, 120, 135, 255)
    eye = (30, 30, 45, 255)
    blush = (255, 180, 190, 180)

    float_y = 0
    wobble = 0
    blink = False
    mouth_open = 0
    wave = 0

    if anim == "idle":
        float_y = [0, -2, 0, -1][frame_idx]
        wobble = frame_idx
        blink = frame_idx == 3
    elif anim == "walk":
        float_y = [0, -3, -1, -3, -1, -3][frame_idx]
        wobble = frame_idx * 2
    elif anim == "sleep":
        float_y = 2
        blink = True
    elif anim == "talk":
        float_y = [0, -1, 0, -2][frame_idx]
        mouth_open = [0, 3, 5, 3][frame_idx]

    cx, cy = 32, 28 + float_y

    # Ghost body (rounded top, wavy bottom)
    draw.ellipse([ox + cx - 18, cy - 16, ox + cx + 18, cy + 18], fill=ghost)
    draw.ellipse([ox + cx - 14, cy - 12, ox + cx + 14, cy + 10], fill=ghost_light)

    # Wavy skirt bottom
    base_y = cy + 16
    waves = 5
    for i in range(waves):
        wx = ox + cx - 16 + i * 8
        dip = (i + wobble + frame_idx) % 3 - 1
        draw.ellipse([wx, base_y + dip, wx + 10, base_y + 10 + dip], fill=ghost)
        draw.arc([wx, base_y + dip - 2, wx + 10, base_y + 8 + dip], 0, 180, fill=ghost_dark, width=1)

    # Outline sides
    draw.line([(ox + cx - 18, cy - 8), (ox + cx - 18, base_y + 2)], fill=outline, width=1)
    draw.line([(ox + cx + 18, cy - 8), (ox + cx + 18, base_y + 2)], fill=outline, width=1)

    # Eyes
    if blink or anim == "sleep":
        for ex in (-8, 8):
            draw.line([(ox + cx + ex - 4, cy - 2), (ox + cx + ex + 4, cy - 2)], fill=eye, width=2)
    else:
        for ex in (-8, 8):
            draw.ellipse([ox + cx + ex - 5, cy - 6, ox + cx + ex + 5, cy + 2], fill=eye)
            px(draw, ox + cx + ex - 2, cy - 4, (255, 255, 255, 255), 2)

    # Blush
    for ex in (-14, 14):
        draw.ellipse([ox + cx + ex - 3, cy + 2, ox + cx + ex + 3, cy + 8], fill=blush)

    # Mouth
    if mouth_open > 0:
        draw.ellipse([ox + cx - mouth_open, cy + 6, ox + cx + mouth_open, cy + 12 + mouth_open // 2], fill=(80, 80, 100, 255))
    elif anim == "sleep":
        draw.arc([ox + cx - 3, cy + 5, ox + cx + 3, cy + 10], 0, 180, fill=eye, width=1)
    else:
        draw.arc([ox + cx - 5, cy + 4, ox + cx + 5, cy + 12], 200, 340, fill=eye, width=2)

    if anim == "sleep":
        for i, (zx, zy) in enumerate([(14, 6), (20, 2), (26, -2)]):
            draw.text((ox + cx + zx, cy + zy - 14), "z" * (i + 1), fill=(180, 180, 210, 220))


def draw_preview(img: Image.Image, *, kind: str) -> None:
    frame = sheet(1)
    if kind == "cat":
        draw_cat_frame(frame, 0, anim="idle")
    else:
        draw_ghost_frame(frame, 0, anim="idle")
    img.paste(frame.crop((0, 0, FRAME, FRAME)), (0, 0))


def build_skin(folder: Path, kind: str) -> None:
    folder.mkdir(parents=True, exist_ok=True)
    drawer = draw_cat_frame if kind == "cat" else draw_ghost_frame

    anims = {
        "idle.png": ("idle", 4),
        "walk.png": ("walk", 6),
        "sleep.png": ("sleep", 2),
        "talk.png": ("talk", 4),
    }

    for filename, (anim, count) in anims.items():
        img = sheet(count)
        for i in range(count):
            drawer(img, i, anim=anim)
        img.save(folder / filename)

    preview = Image.new("RGBA", (FRAME, FRAME), (0, 0, 0, 0))
    draw_preview(preview, kind=kind)
    preview.save(folder / "preview.png")


def main() -> None:
    build_skin(SKINS["default"], "cat")
    build_skin(SKINS["ghosty"], "ghost")
    build_skin(LEGACY, "cat")
    print("Generated cat sprites -> default + legacy")
    print("Generated ghost sprites -> ghosty")


if __name__ == "__main__":
    main()
