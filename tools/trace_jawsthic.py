"""Trace JAWSTHIC's parts from Jason's sprite atlas down to game size.

    python tools/trace_jawsthic.py        (python3 on macOS/Linux; needs Pillow)

Reads "docs/reference/stage 1 bosss/Right-facing shark boss sprite atlas.png" and
writes src/render/jaws_px.rs: each part as rows of palette letters, its pivot, and
what the game reads off it (the windows' glass). The game rotates and mirrors the
parts itself.

Each part is cut out of the atlas (its largest solid piece, holes kept), shrunk
smoothly, given back the contrast the shrink takes, its cracks kept from the
full-size art, every pixel snapped to the boss's palette with an ordered dither
between stone tones, and outlined. Then the hand pass: the eye drawn crisp, the
crosshair painted in the jaw taken out (the game draws its own), and the
pectoral's stump broken off.
"""

import os
import sys

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ATLAS = os.path.join(ROOT, "docs", "reference", "stage 1 bosss", "Right-facing shark boss sprite atlas.png")
OUT = os.path.join(ROOT, "src", "render", "jaws_px.rs")

# The boss palette (carve-era tones, kept): letter -> color.
PAL = {
    "o": (0x12, 0x12, 0x10),  # outline
    "x": (0x1E, 0x1D, 0x19),  # recess
    "0": (0x26, 0x24, 0x1F),
    "1": (0x3E, 0x3B, 0x33),
    "2": (0x6E, 0x68, 0x58),
    "3": (0x9D, 0x96, 0x82),
    "4": (0xC4, 0xBD, 0xA8),
    "5": (0xDD, 0xD6, 0xC2),
    "S": (0xF8, 0x3C, 0x30),  # signal red
    "G": (0xC9, 0x34, 0x2A),  # gum, lit
    "g": (0x8E, 0x23, 0x1B),  # gum
    "v": (0x45, 0x12, 0x0E),  # vein
    "T": (0x32, 0xDF, 0xB3),  # teal light
    "t": (0x13, 0x57, 0x46),  # teal, dim
}
RAMP = "012345"
# luma cut points between the stone tones, after the contrast is restored
EDGES = [40, 75, 115, 172, 205]
GAIN = 1.3
CRACK = 0.3
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]

# Each part: its box in the atlas, its scale to game size (the shark's parts aren't
# drawn at one scale in Jason's composed picture: the far fin is foreshortened, the
# fins and tail a little smaller than the body), and its pivot in the traced part.
PARTS = {
    "body": ((20, 48, 1004, 544), 0.165, (84, 45)),
    "jaw": ((984, 208, 1512, 636), 0.165, (7, 22)),
    "tail": ((24, 544, 552, 988), 0.165, (78, 37)),
    "dorsal": ((560, 624, 840, 928), 0.14, (22, 40)),
    "pec_near": ((768, 660, 1220, 988), 0.17, (66, 10)),
    "pec_far": ((1256, 700, 1504, 972), 0.15, (6, 6)),
}


def luma(c):
    return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]


def cut(atlas, box):
    """The largest 8-connected solid piece in `box`, with the holes it encloses."""
    src = atlas.load()
    x0, y0, x1, y1 = box[0] - 6, box[1] - 6, box[2] + 6, box[3] + 6
    w, h = x1 - x0, y1 - y0
    solid = [[src[x0 + x, y0 + y][3] >= 200 for x in range(w)] for y in range(h)]
    seen = [[False] * w for _ in range(h)]
    best = []
    for y in range(h):
        for x in range(w):
            if solid[y][x] and not seen[y][x]:
                comp, st = [], [(x, y)]
                seen[y][x] = True
                while st:
                    cx, cy = st.pop()
                    comp.append((cx, cy))
                    for dy in (-1, 0, 1):
                        for dx in (-1, 0, 1):
                            nx, ny = cx + dx, cy + dy
                            if 0 <= nx < w and 0 <= ny < h and solid[ny][nx] and not seen[ny][nx]:
                                seen[ny][nx] = True
                                st.append((nx, ny))
                if len(comp) > len(best):
                    best = comp
    keep = set(best)
    outside = [[False] * w for _ in range(h)]
    st = [(x, y) for x in range(w) for y in (0, h - 1)] + [(x, y) for y in range(h) for x in (0, w - 1)]
    st = [(x, y) for (x, y) in st if (x, y) not in keep]
    for x, y in st:
        outside[y][x] = True
    while st:
        cx, cy = st.pop()
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = cx + dx, cy + dy
            if 0 <= nx < w and 0 <= ny < h and not outside[ny][nx] and (nx, ny) not in keep:
                outside[ny][nx] = True
                st.append((nx, ny))
    xs, ys = [p[0] for p in best], [p[1] for p in best]
    bx0, by0, bx1, by1 = min(xs), min(ys), max(xs) + 1, max(ys) + 1
    out = Image.new("RGBA", (bx1 - bx0, by1 - by0), (0, 0, 0, 0))
    op = out.load()
    for y in range(by0, by1):
        for x in range(bx0, bx1):
            if (x, y) in keep:
                op[x - bx0, y - by0] = src[x0 + x, y0 + y][:3] + (255,)
            elif not outside[y][x]:
                r, g, b, a = src[x0 + x, y0 + y]
                op[x - bx0, y - by0] = (r, g, b, 255) if a > 0 else (10, 10, 9, 255)
    return out


def trace(im, scale):
    """Rows of palette letters for `im` at `scale`."""
    W, H = im.size
    px = im.load()
    w, h = max(1, round(W * scale)), max(1, round(H * scale))
    small = im.convert("RGBa").resize((w, h), Image.LANCZOS).convert("RGBA")
    sp = small.load()
    f = 1.0 / scale
    g = [[None] * w for _ in range(h)]
    for ty in range(h):
        for tx in range(w):
            r, gg, b, a = sp[tx, ty]
            if a < 128:
                continue
            l = luma((r, gg, b))
            if r > 70 and r > gg * 1.55 and r > b * 1.4:
                g[ty][tx] = "S" if l > 85 else "G" if l > 62 else "g" if l > 42 else "v"
                continue
            if (gg + b) / 2 > r + 28 and b > 70:
                g[ty][tx] = "T" if l > 110 else "t"
                continue
            # how much of this pixel's block of the full-size art is dark ink
            x0, x1 = int(tx * f), max(int(tx * f) + 1, int((tx + 1) * f))
            y0, y1 = int(ty * f), max(int(ty * f) + 1, int((ty + 1) * f))
            n = d = 0
            for y in range(y0, min(y1, H)):
                for x in range(x0, min(x1, W)):
                    pr, pg, pb, pa = px[x, y]
                    if pa < 128:
                        continue
                    n += 1
                    if luma((pr, pg, pb)) < 48 and not (pr > pg * 1.6 and pr > 60):
                        d += 1
            ink = d / n if n else 0.0
            if ink > 0.62 or l < 28:
                g[ty][tx] = "o"
            elif ink > CRACK:
                g[ty][tx] = "x" if l < 90 else "1"
            else:
                l = 150 + (l - 150) * GAIN + ((BAYER[ty % 4][tx % 4] + 0.5) / 16.0 - 0.5) * 18
                g[ty][tx] = RAMP[sum(l > e for e in EDGES)]
    # a 1 px outline round the silhouette
    edge = [[g[y][x] is not None and any(not (0 <= x + dx < w and 0 <= y + dy < h) or g[y + dy][x + dx] is None for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))) for x in range(w)] for y in range(h)]
    for y in range(h):
        for x in range(w):
            if edge[y][x]:
                g[y][x] = "o"
    return g


def eye(g, cx, cy):
    """The eye drawn crisp: a red diamond in a black socket under a lit brow."""
    for dy in range(-4, 5):
        for dx in range(-4, 5):
            d = abs(dx) + abs(dy)
            y, x = cy + dy, cx + dx
            if g[y][x] is None:
                continue
            if d <= 1:
                g[y][x] = "S"
            elif d <= 3:
                g[y][x] = "o"
            elif d == 4 and dy < 0:
                g[y][x] = "4"


# The tail's thick part, evened out: from just ahead of the tail fin's collar to
# where it meets the body, its outline runs straight from its own height there to
# the body's height at the joint (the traced tail narrows to a little end cap just
# where it meets the body, so the thick part pinched and bulged). Columns in the
# traced tail; its root joint sits on this pixel of the body.
EVEN_FROM, EVEN_TO = 45, 79
ROOT_IN_BODY, ROOT_IN_TAIL = (8, 36), (80, 46)


def even_tail(tail, body, body_cut):
    """Straighten the thick part's top and bottom edges into an even taper ending at
    the body's own height where the two meet, keeping the traced rings: gaps along
    the edge take the stone next to them, what sticks out goes, a clean outline."""
    h, w = len(tail), len(tail[0])
    def span(g, x):
        rows = [y for y in range(len(g)) if g[y][x] is not None]
        return rows[0], rows[-1]
    t0, b0 = span(tail, EVEN_FROM)
    bt, bb = span(body, body_cut)
    dy = ROOT_IN_TAIL[1] - ROOT_IN_BODY[1]
    t1, b1 = bt + dy, bb + dy
    # the traced outline (an outline pixel next to the empty), as it was
    edge = {(x, y) for y in range(h) for x in range(w) if tail[y][x] == "o" and any(not (0 <= x + dx < w and 0 <= y + dy < h) or tail[y + dy][x + dx] is None for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))}
    for x in range(EVEN_FROM, min(EVEN_TO, w - 1) + 1):
        k = (x - EVEN_FROM) / (EVEN_TO - EVEN_FROM)
        top, bot = round(t0 + (t1 - t0) * k), round(b0 + (b1 - b0) * k)
        rows = [y for y in range(h) if tail[y][x] is not None]
        if not rows:
            continue
        inside = [tail[y][x] for y in range(rows[0] + 1, rows[-1]) if tail[y][x] is not None]
        # a column on a seam between two rings carries the seam out to the new edge
        seam = sum(c in "ox01" for c in inside) > 0.6 * max(1, len(inside))
        for y in range(h):
            if y < top or y > bot:
                tail[y][x] = None
            elif tail[y][x] is None or (x, y) in edge:
                if seam:
                    tail[y][x] = "x"
                else:
                    # the nearest stone in this column (never a light), toward the middle
                    step = 1 if y < (top + bot) / 2 else -1
                    yy = y + step
                    while 0 <= yy < h and not (tail[yy][x] is not None and tail[yy][x] in "2345" and (x, yy) not in edge):
                        yy += step
                    tail[y][x] = tail[yy][x] if 0 <= yy < h else ("4" if step > 0 else "2")
    # the edge: outline where the stone meets the empty, the top lit
    for x in range(EVEN_FROM, min(EVEN_TO, w - 1) + 1):
        rows = [y for y in range(h) if tail[y][x] is not None]
        if rows:
            tail[rows[0]][x] = "o"
            tail[rows[-1]][x] = "o"
            if rows[0] + 1 < rows[-1]:
                tail[rows[0] + 1][x] = "4"


def trim_cap(g, x0):
    """The body's narrow rear cap goes (the tail's first ring takes its place, so the
    two don't pinch together), the cut seamed dark."""
    for row in g:
        for x in range(x0):
            row[x] = None
        if row[x0] is not None:
            row[x0] = "o"


def unpaint_crosshair(g, x0, y0, x1, y1):
    """The jaw's painted crosshair goes back to the dark of the mouth."""
    for y in range(y0, y1):
        for x in range(x0, x1):
            if g[y][x] in ("S", "G", "g"):
                g[y][x] = "x"


def stump(g, pivot, reach):
    """What's left of a pectoral fin: its joint and a ragged root of the blade."""
    px, py = pivot
    h, w = len(g), len(g[0])
    out = [[None] * w for _ in range(h)]
    for y in range(h):
        for x in range(w):
            if g[y][x] is None:
                continue
            # a jagged break across the blade
            d = ((x - px) ** 2 + (y - py) ** 2) ** 0.5
            jag = reach + (3 if (x + 2 * y) % 7 < 3 else 0)
            if d <= jag:
                out[y][x] = g[y][x]
    # the break: burnt dark along its edge
    for y in range(h):
        for x in range(w):
            if out[y][x] is not None and out[y][x] not in ("o", "x"):
                near = any(0 <= x + dx < w and 0 <= y + dy < h and out[y + dy][x + dx] is None and g[y + dy][x + dx] is not None for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))
                if near:
                    out[y][x] = "x"
    return out


def windows(g, pivot):
    """Each lancet window's glass, pixels from the body's pivot: the dark inside it."""
    out = []
    for cx in (60, 71, 82, 92):
        glass = []
        for y in range(33, 59):
            for x in range(cx - 2, cx + 3):
                if g[y][x] in ("o", "x", "0", "1", "t", "T", "g", "G", "S", "v"):
                    glass.append((x - pivot[0], y - pivot[1]))
        out.append(glass)
    return out


def rows(g):
    return ["".join(c if c else "." for c in r) for r in g]


def emit(parts, glass):
    lines = [
        "//! JAWSTHIC's parts, traced from Jason's sprite atlas",
        "//! (docs/reference/stage 1 bosss/Right-facing shark boss sprite atlas.png) by",
        "//! tools/trace_jawsthic.py. Generated: change the tool, not this file.",
        "//!",
        "//! Letters: `o` outline, `x` recess, `0`-`5` stone dark to light, `S` red light,",
        "//! `G` `g` gums, `v` a vein, `T` `t` teal light, `.` empty. Every part faces right.",
        "",
        "/// A traced part: its rows, and its pivot (the pixel it turns about and hangs from).",
        "pub struct Grid {",
        "    pub pivot: (i32, i32),",
        "    pub rows: &'static [&'static str],",
        "}",
        "",
    ]
    names = {"body": "BODY", "jaw": "JAW", "tail": "TAIL", "dorsal": "DORSAL", "pec_near": "PECTORAL", "pec_far": "FAR_FIN", "stump": "STUMP"}
    for key, (g, pivot) in parts.items():
        lines.append(f"pub const {names[key]}: Grid = Grid {{")
        lines.append(f"    pivot: ({pivot[0]}, {pivot[1]}),")
        lines.append("    rows: &[")
        for r in rows(g):
            lines.append(f'        "{r}",')
        lines.append("    ],")
        lines.append("};")
        lines.append("")
    lines.append("/// Each lancet window's glass, pixels from the body's pivot (facing right).")
    lines.append("pub const GLASS: [&[(i8, i8)]; 4] = [")
    for gl in glass:
        lines.append("    &[" + ", ".join(f"({x}, {y})" for x, y in gl) + "],")
    lines.append("];")
    open(OUT, "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")


def main():
    atlas = Image.open(ATLAS).convert("RGBA")
    parts = {}
    for name, (box, scale, pivot) in PARTS.items():
        g = trace(cut(atlas, box), scale)
        print(f"{name}: {len(g[0])} x {len(g)}, pivot {pivot}", file=sys.stderr)
        parts[name] = (g, pivot)
    eye(parts["body"][0], 110, 23)
    trim_cap(parts["body"][0], 7)
    even_tail(parts["tail"][0], parts["body"][0], 7)
    unpaint_crosshair(parts["jaw"][0], 46, 16, 70, 38)
    parts["stump"] = (stump(parts["pec_near"][0], PARTS["pec_near"][2], 17), PARTS["pec_near"][2])
    emit(parts, windows(parts["body"][0], PARTS["body"][2]))
    print(f"wrote {OUT}", file=sys.stderr)


if __name__ == "__main__":
    main()
