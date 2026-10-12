"""Trace JAWSTHIC's parts from Jason's sprite atlas down to game size, and draw its tail.

    python tools/trace_jawsthic.py        (python3 on macOS/Linux; needs Pillow)

Reads "docs/reference/stage 1 bosss/Right-facing shark boss sprite atlas.png" and
writes src/render/jaws_px.rs: each part as rows of palette letters, its pivot, and
what the game reads off it (the windows' glass). The game rotates and mirrors the
parts itself.

Each part is cut out of the atlas (its largest solid piece, holes kept), shrunk
smoothly, given back the contrast the shrink takes, its cracks kept from the
full-size art, every pixel snapped to the boss's palette with an ordered dither
between stone tones, and outlined, then tidied to sit in the game like the enemies
(no lone pixels or specks, a clean silhouette, lit from the top left). Then the hand
pass: the eye drawn crisp, the crosshair painted in the jaw taken out (the game draws
its own), and the pectoral's stump broken off.

The tail isn't traced: at game size its traced rings were noise. It's drawn in code
here, one piece from under the body to the tail fin, after Jason's design.
"""

import math
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
                l = 150 + (l - 150) * GAIN + ((BAYER[ty % 4][tx % 4] + 0.5) / 16.0 - 0.5) * 10
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


def trim_cap(g, x0):
    """The body's rear goes from column `x0` back (the drawn tail runs under the cut
    instead), the cut seamed dark like the seams between its plates."""
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


# ---------------------------------------------------------------- drawn, not traced

def hsh(x, y):
    h = ((x & 0xFFFFFFFF) * 0x9E3779B1 ^ (y & 0xFFFFFFFF) * 0x85EBCA77 ^ 0x5EED0007) & 0xFFFFFFFF
    h ^= h >> 15
    h = (h * 0x2C1B3C6D) & 0xFFFFFFFF
    h ^= h >> 12
    return (h & 0xFFFF) / 65536.0


# The tail, drawn (at game size the traced one is noise): its joints in the body's
# own pixels (the traced body, pivot (84, 45)), root to the tail fin, along its
# centerline; its half-height at the root and at the fin's joint; its ring plates'
# lengths from the root; where its finlets stand (distance along it).
TAIL_JOINTS = [(52, 47), (40, 44), (28, 40), (17, 35), (7, 30), (-3, 24)]
TAIL_HALF = (20.0, 6.0)
RINGS = [8.0, 8.0, 7.5, 7.0, 6.5, 6.0, 5.5, 5.0]
FINLETS = (41.0, 44.0)
# the body is cut here; the tail's root runs under it
BODY_CUT = 53


def polyline_frame(joints):
    """Segments of the centerline: (start, unit along, unit normal below, length, s0)."""
    segs, s = [], 0.0
    for a, b in zip(joints, joints[1:]):
        dx, dy = b[0] - a[0], b[1] - a[1]
        ln = (dx * dx + dy * dy) ** 0.5
        u = (dx / ln, dy / ln)
        segs.append((a, u, (-u[1], u[0]) if u[0] > 0 else (u[1], -u[0]), ln, s))
        s += ln
    return segs, s


def locate(segs, total, p):
    """Arc length along the tail and signed distance below it for point `p`."""
    best = None
    for a, u, n, ln, s0 in segs:
        rx, ry = p[0] - a[0], p[1] - a[1]
        t = max(0.0, min(ln, rx * u[0] + ry * u[1]))
        qx, qy = a[0] + u[0] * t, a[1] + u[1] * t
        d2 = (p[0] - qx) ** 2 + (p[1] - qy) ** 2
        if best is None or d2 < best[0]:
            best = (d2, s0 + t, (p[0] - qx) * n[0] + (p[1] - qy) * n[1], rx * u[0] + ry * u[1], s0, ln)
    _, s, d, along, s0, ln = best
    # past either end, carry on straight out
    if s0 == 0.0 and along < 0:
        s = along
    if s0 + ln >= total - 1e-6 and along > ln:
        s = s0 + along
    return s, d


def at_s(segs, total, s, off):
    """The point `s` along the centerline, `off` below it."""
    for a, u, n, ln, s0 in segs:
        if s <= s0 + ln or s0 + ln >= total - 1e-6:
            t = s - s0
            return (a[0] + u[0] * t + n[0] * off, a[1] + u[1] * t + n[1] * off)


def inside(poly, x, y):
    c = False
    for (x0, y0), (x1, y1) in zip(poly, poly[1:] + poly[:1]):
        if (y0 > y) != (y1 > y) and x < x0 + (y - y0) * (x1 - x0) / (y1 - y0):
            c = not c
    return c


def outline(g):
    h, w = len(g), len(g[0])
    edge = [[g[y][x] is not None and any(not (0 <= x + dx < w and 0 <= y + dy < h) or g[y + dy][x + dx] is None for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))) for x in range(w)] for y in range(h)]
    for y in range(h):
        for x in range(w):
            if edge[y][x]:
                g[y][x] = "o"


def lit_edges(g, stone="12345"):
    """Light from the top left: the stone just inside the outline along its top and
    left goes a tone lighter, along its bottom and right a tone darker."""
    h, w = len(g), len(g[0])
    out = [row[:] for row in g]
    up = lambda c: "012345"[min(5, "012345".index(c) + 1)]
    down = lambda c: "012345"[max(1, "012345".index(c) - 1)]
    for y in range(h):
        for x in range(w):
            c = g[y][x]
            if c is None or c not in stone:
                continue
            n = lambda dx, dy: g[y + dy][x + dx] if 0 <= x + dx < w and 0 <= y + dy < h else None
            if n(0, -1) in ("o", None) or n(-1, 0) in ("o", None):
                out[y][x] = up(c)
            elif n(0, 1) in ("o", None) or n(1, 0) in ("o", None):
                out[y][x] = down(c)
    return out


def draw_tail():
    """The tail drawn as one piece, ring plates tapering from under the body to the
    tail fin's collar: the seams between them dark with a teal or red light in each,
    each ring's edge facing the light a tone lighter, a finlet above and below near
    the fin, all lit from the top left like Stage 1's enemies. Returns the plates
    (pivot on the root joint), the joints in the plates' pixels, and the tail fin
    (pivot on its joint)."""
    segs, total = polyline_frame(TAIL_JOINTS)
    half = lambda s: TAIL_HALF[1] + (TAIL_HALF[0] - TAIL_HALF[1]) * max(0.0, 1.0 - max(0.0, s) / total) ** 1.25
    gaps, s = [], 0.0
    for r in RINGS:
        s += r
        if s < total - 2:
            gaps.append(s)
    def bulge(s):
        prev = max([0.0] + [g for g in gaps if g <= s])
        nxt = min([total] + [g for g in gaps if g > s])
        k = (s - prev) / max(1e-3, nxt - prev)
        return 0.9 * math.sin(math.pi * k) - 0.7
    # the finlets: little triangles off its back and belly near the fin
    def finlet(s0, side, length, height):
        base0 = at_s(segs, total, s0, side * half(s0))
        base1 = at_s(segs, total, s0 + length, side * half(s0 + length))
        apex = at_s(segs, total, s0 + length + 3, side * (half(s0 + length) + height))
        return [base0, base1, apex]
    fins = [finlet(FINLETS[0], -1, 9, 10), finlet(FINLETS[1], 1, 8, 8)]

    x0, x1 = TAIL_JOINTS[-1][0] - 4, TAIL_JOINTS[0][0] + 5
    y0, y1 = min(j[1] for j in TAIL_JOINTS) - 26, max(j[1] for j in TAIL_JOINTS) + 26
    W, H = x1 - x0, y1 - y0
    g = [[None] * W for _ in range(H)]
    for yy in range(H):
        for xx in range(W):
            x, y = x0 + xx + 0.5, y0 + yy + 0.5
            s, d = locate(segs, total, (x, y))
            if s < -6 or s > total + 1:
                continue
            he = half(s) + bulge(s)
            if abs(d) <= he:
                v = d / he
                # a thin lit rim on top, mid stone down its flank like the body's
                # shadowed arcade, darker underneath
                t = 5 if v < -0.84 else 4 if v < -0.38 else 3 if v < 0.22 else 2 if v < 0.62 else 1
                # hairline cracks across the stone, like the body's flagstones
                if t >= 3 and hsh(xx * 3 + 7, yy) < 0.035:
                    t -= 2
                seam = min((abs(s - q), q) for q in gaps) if gaps else (99, 0)
                if seam[0] < 0.55 and abs(v) < 0.92:
                    i = gaps.index(seam[1])
                    g[yy][xx] = ("t" if i % 2 == 0 else "g") if abs(v) < 0.18 else "x"
                    continue
                # the edge of each ring facing the light, and the shade just behind a seam
                if any(0.55 <= q - s < 1.6 for q in gaps):
                    t = min(5, t + 1)
                elif any(0.55 <= s - q < 1.4 for q in gaps):
                    t = max(1, t - 1)
                if t >= 3 and hsh(xx, yy) < 0.11:
                    t -= 1
                g[yy][xx] = str(t)
            elif any(inside(f, x, y) for f in fins):
                g[yy][xx] = "4" if hsh(xx, yy) > 0.08 else "3"
    outline(g)
    g = lit_edges(g)
    pivot = (TAIL_JOINTS[0][0] - x0, TAIL_JOINTS[0][1] - y0)
    joints = [(jx - x0, jy - y0) for jx, jy in TAIL_JOINTS]

    # the tail fin: a stone crescent off the last joint, in the frame of the tail's
    # last plate (along it, then below it)
    a, u, n, ln, s0 = segs[-1]
    J = TAIL_JOINTS[-1]
    crescent = [(-3, -5), (3, -9), (10, -16), (17, -23), (24, -29), (31, -34), (27, -27), (22, -18), (18, -9), (16, 0), (18, 9), (21, 17), (25, 24), (30, 31), (22, 26), (14, 19), (8, 13), (3, 8), (-3, 5)]
    poly = [(J[0] + u[0] * p + n[0] * q, J[1] + u[1] * p + n[1] * q) for p, q in crescent]
    fx0, fx1 = int(min(p[0] for p in poly)) - 2, int(max(p[0] for p in poly)) + 3
    fy0, fy1 = int(min(p[1] for p in poly)) - 2, int(max(p[1] for p in poly)) + 3
    fw, fh = fx1 - fx0, fy1 - fy0
    f = [[None] * fw for _ in range(fh)]
    for yy in range(fh):
        for xx in range(fw):
            x, y = fx0 + xx + 0.5, fy0 + yy + 0.5
            if not inside(poly, x, y):
                continue
            # along the fin (from its joint) and across it
            rx, ry = x - J[0], y - J[1]
            along, across = rx * u[0] + ry * u[1], rx * n[0] + ry * n[1]
            # how far out along its lobe, and how far back from the lobe's leading edge
            reach = math.hypot(along, across)
            back = along - abs(across) * 0.55
            if along < 1.0:
                c = "3"  # its collar
            elif along < 2.2:
                c = "x"  # the crease where it meets the tail
            elif reach < 11 and abs(across) < 7 and back > 3:
                # the dark web where the lobes meet, a light in it
                c = ("t" if across < 0 else "g") if abs(abs(across) - 3) < 0.8 and abs(along - 7) < 1.2 else "x"
            elif reach > 9 and (reach % 6.5) < 0.9:
                c = "2"  # the seam between two of its plates
            else:
                # stone, lit along the leading edge, shaded toward the trailing one
                c = "5" if back < 2.5 else "4" if back < 9 else "3"
                if c != "3" and hsh(xx, yy) < 0.08:
                    c = str(int(c) - 1)
            f[yy][xx] = c
    outline(f)
    f = lit_edges(f)
    fin_pivot = (J[0] - fx0, J[1] - fy0)

    print(f"tail: {W} x {H}, root {pivot}; fin: {fw} x {fh}, joint {fin_pivot}", file=sys.stderr)
    # what the sim needs (tune.rs): the root from the body's pivot, each plate's
    # length and drawn slope
    print("JAWS_TAIL_ROOT", (TAIL_JOINTS[0][0] - 84, TAIL_JOINTS[0][1] - 45), file=sys.stderr)
    for (ax, ay), (bx, by) in zip(TAIL_JOINTS, TAIL_JOINTS[1:]):
        print("  plate", round(math.hypot(ax - bx, ay - by), 2), round(math.atan2(ay - by, ax - bx), 3), file=sys.stderr)
    return (g, pivot), joints, (f, fin_pivot)


def cleanup(g, keep_light=True):
    """Tidy a traced part so it sits in the game like the enemies: lone pixels take
    their neighbors' color, lone dark specks in the stone go, the silhouette loses its
    one-pixel spikes and notches and gets a clean outline, and the stone just inside it
    is lit from the top left."""
    h, w = len(g), len(g[0])
    lights = set("SGgvTt")
    def nb(y, x, ds):
        return [g[y + dy][x + dx] for dx, dy in ds if 0 <= x + dx < w and 0 <= y + dy < h]
    four = ((1, 0), (-1, 0), (0, 1), (0, -1))
    eight = four + ((1, 1), (-1, -1), (1, -1), (-1, 1))
    for _ in range(2):
        out = [row[:] for row in g]
        for y in range(h):
            for x in range(w):
                c = g[y][x]
                if c is None or c in lights:
                    continue
                ns = [n for n in nb(y, x, four) if n is not None]
                stone = [n for n in nb(y, x, eight) if n is not None and n in "12345"]
                if len(ns) == 4 and c not in ns and stone:
                    # a lone pixel: its neighbors' most common stone
                    out[y][x] = max(set(stone), key=stone.count)
                elif c in "ox0" and len(ns) == 4 and sum(1 for n in nb(y, x, eight) if n in ("o", "x", "0")) <= 1 and stone:
                    out[y][x] = max(set(stone), key=stone.count)
        g = out
    # the silhouette: spikes go, notches fill
    for _ in range(2):
        out = [row[:] for row in g]
        for y in range(h):
            for x in range(w):
                filled = sum(1 for n in nb(y, x, four) if n is not None)
                if g[y][x] is not None and filled <= 1:
                    out[y][x] = None
                elif g[y][x] is None and filled >= 3:
                    out[y][x] = "o"
        g = out
    outline(g)
    return lit_edges(g)


def emit(parts, glass, joints):
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
    names = {"body": "BODY", "jaw": "JAW", "plates": "PLATES", "fin": "FIN", "dorsal": "DORSAL", "pec_near": "PECTORAL", "pec_far": "FAR_FIN", "stump": "STUMP"}
    for key, (g, pivot) in parts.items():
        lines.append(f"pub const {names[key]}: Grid = Grid {{")
        lines.append(f"    pivot: ({pivot[0]}, {pivot[1]}),")
        lines.append("    rows: &[")
        for r in rows(g):
            lines.append(f'        "{r}",')
        lines.append("    ],")
        lines.append("};")
        lines.append("")
    lines.append("/// The tail's joints in `PLATES`, root to the tail fin, on its centerline.")
    lines.append("pub const TAIL_JOINTS: [(i32, i32); 6] = [" + ", ".join(f"({x}, {y})" for x, y in joints) + "];")
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
        g = cleanup(trace(cut(atlas, box), scale))
        print(f"{name}: {len(g[0])} x {len(g)}, pivot {pivot}", file=sys.stderr)
        parts[name] = (g, pivot)
    eye(parts["body"][0], 110, 23)
    trim_cap(parts["body"][0], BODY_CUT)
    unpaint_crosshair(parts["jaw"][0], 46, 16, 70, 38)
    parts["stump"] = (stump(parts["pec_near"][0], PARTS["pec_near"][2], 17), PARTS["pec_near"][2])
    plates, joints, fin = draw_tail()
    parts["plates"], parts["fin"] = plates, fin
    emit(parts, windows(parts["body"][0], PARTS["body"][2]), joints)
    print(f"wrote {OUT}", file=sys.stderr)


if __name__ == "__main__":
    main()
