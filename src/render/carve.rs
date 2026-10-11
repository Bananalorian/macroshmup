//! The boss painter: shapes carved in bone, shaded like stone under one light.
//!
//! Stage 1's painter draws each sprite once, flat. The boss moves: its tail swings,
//! its jaw drops, its fins paddle, and it turns to face either way. So here every
//! part is drawn at any angle and either facing. The shapes stay in the part's own
//! coordinates and each pixel looks back into them, so edges stay crisp at every
//! angle (no rotated-pixel mush) and the stone's grain stays put on the stone.
//!
//! The light always comes from the top left of the screen, whatever the angle. Each
//! part has a form (the body is a long barrel, the fins are flat plates) that rounds
//! its shading; each plate between the seams gets its own bevel, a bright lip along
//! its top and left, shade along its bottom and right; and the tones step through a
//! short ramp with ordered dithering only where two tones meet (the same engraved
//! stipple as the black hole), so faces stay clean.

use macroquad::prelude::{Color, Image};

pub type Pt = (f32, f32);

/// What a pixel is made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum M {
    Empty,
    /// Bone plating, rounded by the part's form and beveled plate by plate.
    Hull,
    /// The groove between two plates: an inner line, lighter than the outline.
    Seam,
    /// A dark recess, in shadow under its top edge.
    Recess,
    /// Black: a slot, a lens, the void inside a window.
    Slot,
    /// A red light.
    Red,
    /// A teal light.
    Teal,
    /// Ivory teeth, lit down their left side.
    Tooth,
    /// Red gums.
    Gum,
    /// The dark of the mouth.
    Mouth,
    /// A dark red vein in the mouth.
    Vein,
    /// A pit in the stone: shadowed at its top left, catching light at its bottom right.
    Pit,
    /// A raised molding (window frames, ribs): a hard bevel.
    Frame,
    /// A turned pipe joint, banded across its axis.
    Pipe,
    /// Plating in the shadow of something above it.
    Shade,
}

impl M {
    /// Lit like stone (everything else is a fixed dark or a light).
    fn stone(self) -> bool {
        matches!(self, M::Hull | M::Frame | M::Pipe | M::Shade | M::Pit)
    }
}

#[derive(Clone, Debug)]
pub enum Sh {
    Poly(Vec<Pt>),
    Disc(Pt, f32),
    /// Center, inner and outer radius.
    Ring(Pt, f32, f32),
    /// A segment from `a` to `b`, `w` px thick.
    Line(Pt, Pt, f32),
}

/// One shape and what it's made of. Later shapes cover earlier ones.
#[derive(Clone, Debug)]
pub struct Shape {
    pub sh: Sh,
    pub m: M,
    bb: [f32; 4],
    /// Turned about a hinge (point, angle): its stone is lit and grained as if it hadn't been.
    turn: Option<(Pt, f32)>,
}

impl Shape {
    pub fn new(sh: Sh, m: M) -> Shape {
        let bb = match &sh {
            Sh::Poly(p) => {
                let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
                for &(x, y) in p {
                    b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
                }
                b
            }
            Sh::Disc((x, y), r) | Sh::Ring((x, y), _, r) => [x - r, y - r, x + r, y + r],
            Sh::Line((ax, ay), (bx, by), w) => [ax.min(*bx) - w, ay.min(*by) - w, ax.max(*bx) + w, ay.max(*by) + w],
        };
        Shape { sh, m, bb: [bb[0] - 0.5, bb[1] - 0.5, bb[2] + 0.5, bb[3] + 0.5], turn: None }
    }

    /// This shape swung `angle` radians about `hinge` (a jaw dropping), still lit as itself.
    pub fn turned(&self, hinge: Pt, angle: f32) -> Shape {
        let (s, c) = angle.sin_cos();
        let t = |(x, y): Pt| {
            let (dx, dy) = (x - hinge.0, y - hinge.1);
            (hinge.0 + c * dx - s * dy, hinge.1 + s * dx + c * dy)
        };
        let sh = match &self.sh {
            Sh::Poly(p) => Sh::Poly(p.iter().map(|&q| t(q)).collect()),
            Sh::Disc(q, r) => Sh::Disc(t(*q), *r),
            Sh::Ring(q, r0, r1) => Sh::Ring(t(*q), *r0, *r1),
            Sh::Line(a, b, w) => Sh::Line(t(*a), t(*b), *w),
        };
        let before = self.turn.map_or(0.0, |(_, a)| a);
        Shape { turn: Some((hinge, before + angle)), ..Shape::new(sh, self.m) }
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        if x < self.bb[0] || x > self.bb[2] || y < self.bb[1] || y > self.bb[3] {
            return false;
        }
        match &self.sh {
            Sh::Poly(p) => inside(p, x, y),
            Sh::Disc((cx, cy), r) => (x - cx).powi(2) + (y - cy).powi(2) <= r * r + 0.25,
            Sh::Ring((cx, cy), r0, r1) => {
                let d = (x - cx).powi(2) + (y - cy).powi(2);
                d > r0 * r0 + 0.25 && d <= r1 * r1 + 0.25
            }
            Sh::Line(a, b, w) => seg_dist(x, y, *a, *b) <= w * 0.5,
        }
    }
}

pub fn poly(pts: &[Pt], m: M) -> Shape {
    Shape::new(Sh::Poly(pts.to_vec()), m)
}

pub fn disc(c: Pt, r: f32, m: M) -> Shape {
    Shape::new(Sh::Disc(c, r), m)
}

pub fn ring(c: Pt, r0: f32, r1: f32, m: M) -> Shape {
    Shape::new(Sh::Ring(c, r0, r1), m)
}

pub fn line(a: Pt, b: Pt, w: f32, m: M) -> Shape {
    Shape::new(Sh::Line(a, b, w), m)
}

/// A seam along a chain of points.
pub fn seams(pts: &[Pt]) -> Vec<Shape> {
    pts.windows(2).map(|w| line(w[0], w[1], 1.0, M::Seam)).collect()
}

fn inside(pts: &[Pt], x: f32, y: f32) -> bool {
    let mut c = false;
    let n = pts.len();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + n - 1) % n]);
        if (a.1 > y) != (b.1 > y) && x < (b.0 - a.0) * (y - a.1) / (b.1 - a.1) + a.0 {
            c = !c;
        }
    }
    c
}

fn seg_dist(x: f32, y: f32, a: Pt, b: Pt) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = (dx * dx + dy * dy).max(1e-6);
    let t = (((x - a.0) * dx + (y - a.1) * dy) / len2).clamp(0.0, 1.0);
    (x - a.0 - dx * t).hypot(y - a.1 - dy * t)
}

/// How a part's surface curves, which rounds its light.
#[derive(Clone, Debug)]
pub enum Form {
    /// A long barrel along x (the body, the tail): its top and bottom edge at each x,
    /// sampled every half pixel from `x0`, shifted by `at` (so a part cut from a bigger
    /// barrel, like the jaw from the head, keeps the head's curve).
    Barrel { x0: f32, rows: Vec<(f32, f32)>, at: Pt },
    /// A flat plate facing out, its normal tipped by `(nx, ny)`, and a little lighter
    /// toward `lean` (a gentle gradient across the plate, per px).
    Flat { tip: Pt, lean: Pt },
}

impl Form {
    /// A barrel whose edges are the outline of `shape` (a polygon), in its own coordinates.
    pub fn barrel(shape: &Shape) -> Form {
        let bb = shape.bb;
        let x0 = bb[0];
        let n = ((bb[2] - bb[0]) * 2.0).ceil() as usize + 1;
        let mut rows = Vec::with_capacity(n);
        for k in 0..n {
            let x = x0 + k as f32 * 0.5;
            let (mut top, mut bot) = (f32::MAX, f32::MIN);
            let mut y = bb[1];
            while y <= bb[3] {
                if shape.hit(x, y) {
                    top = top.min(y);
                    bot = bot.max(y);
                }
                y += 0.5;
            }
            rows.push(if top <= bot { (top, bot) } else { (0.0, 0.0) });
        }
        Form::Barrel { x0, rows, at: (0.0, 0.0) }
    }

    /// The surface normal at local `(x, y)`: (nx, ny, nz), nz toward the viewer.
    fn normal(&self, x: f32, y: f32) -> (f32, f32, f32) {
        match self {
            Form::Barrel { x0, rows, at } => {
                let (x, y) = (x + at.0, y + at.1);
                let row = |x: f32| {
                    let k = ((x - x0) * 2.0).round().clamp(0.0, (rows.len() - 1) as f32) as usize;
                    rows[k]
                };
                let (top, bot) = row(x);
                let half = ((bot - top) * 0.5).max(0.5);
                let ny = ((y - (top + bot) * 0.5) / half).clamp(-1.0, 1.0);
                // where the barrel narrows (the snout, the tail), its surface turns that way
                let (a, b) = (row(x - 1.5), row(x + 1.5));
                let slope = ((b.1 - b.0) - (a.1 - a.0)) / 6.0;
                let nz = (1.0 - ny * ny).max(0.0).sqrt();
                let nx = -slope * nz * 1.6;
                let l = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-4);
                (nx / l, ny / l, nz / l)
            }
            Form::Flat { tip, .. } => {
                let nz = (1.0 - tip.0 * tip.0 - tip.1 * tip.1).max(0.1).sqrt();
                (tip.0, tip.1, nz)
            }
        }
    }

    fn lean(&self, x: f32, y: f32) -> f32 {
        match self {
            Form::Flat { lean, .. } => lean.0 * x + lean.1 * y,
            _ => 0.0,
        }
    }
}

/// A part to carve: its shapes (in px from its pivot, y down, facing right) and form.
#[derive(Clone, Debug)]
pub struct Part {
    pub shapes: Vec<Shape>,
    pub form: Form,
    /// Seeds its grain, so each part's stone differs.
    pub seed: u32,
}

/// How a part is turned: `angle` radians (positive turns clockwise on screen, so a
/// tail swinging down), and whether it's mirrored to face left (mirrored after
/// turning, about the pivot).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    pub angle: f32,
    pub flip: bool,
}

/// A carved image and where its pivot is (the pixel whose center is the pivot).
pub struct Frame {
    pub img: Image,
    pub pivot: (i32, i32),
}

// ---------------------------------------------------------------- the palette

const fn rgb(c: u32) -> Color {
    Color::new(((c >> 16) & 0xFF) as f32 / 255.0, ((c >> 8) & 0xFF) as f32 / 255.0, (c & 0xFF) as f32 / 255.0, 1.0)
}

/// Bone, dark to light. The top is Stage 1's bone highlight (about 84% bright), so
/// enemy bullet cores stay the brightest pixels on screen.
pub const RAMP: [Color; 6] = [rgb(0x26241F), rgb(0x3E3B33), rgb(0x6E6858), rgb(0x9D9682), rgb(0xC4BDA8), rgb(0xDDD6C2)];
pub const OUTLINE: Color = rgb(0x121210);
pub const VOIDISH: Color = rgb(0x0B0B0A);
pub const RECESS: Color = rgb(0x1E1D19);
pub const SIGNAL: Color = rgb(0xF83C30);
pub const GUM: Color = rgb(0x8E231B);
pub const GUM_LIT: Color = rgb(0xC9342A);
pub const VEIN: Color = rgb(0x45120E);
pub const TEAL: Color = rgb(0x32DFB3);

/// The light, from the top left of the screen and a little in front.
const LIGHT: (f32, f32, f32) = (-0.47, -0.71, 0.52);
const AMBIENT: f32 = 0.2;

/// A fixed per-cell noise in 0..1.
pub fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65536.0
}

/// The 4x4 Bayer threshold at a pixel, in 0..1.
fn bayer(x: i32, y: i32) -> f32 {
    const B: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    (B[(y & 3) as usize][(x & 3) as usize] as f32 + 0.5) / 16.0
}

/// Light levels mapped onto the ramp: this dark is its darkest tone, this bright its brightest.
const DARK: f32 = 0.16;
const BRIGHT: f32 = 0.96;

/// A light level to a ramp tone: mostly solid bands, dithered only in a narrow strip
/// where two tones meet.
fn quantize(v: f32, x: i32, y: i32) -> usize {
    let t = ((v - DARK) / (BRIGHT - DARK)).clamp(0.0, 1.0) * (RAMP.len() - 1) as f32;
    let (lo, f) = (t.floor(), t.fract());
    const BAND: f32 = 0.34;
    let up = if f < 0.5 - BAND * 0.5 {
        false
    } else if f > 0.5 + BAND * 0.5 {
        true
    } else {
        (f - (0.5 - BAND * 0.5)) / BAND > bayer(x, y)
    };
    ((lo as usize) + up as usize).min(RAMP.len() - 1)
}

// ---------------------------------------------------------------- carving

/// Where each pixel landed: its material and its point in the part's coordinates.
struct Cut {
    w: i32,
    h: i32,
    m: Vec<M>,
    at: Vec<Pt>,
    /// How far the shape under each pixel was turned (its light turns with it).
    tilt: Vec<f32>,
}

impl Cut {
    fn m(&self, x: i32, y: i32) -> M {
        if x < 0 || y < 0 || x >= self.w || y >= self.h { M::Empty } else { self.m[(y * self.w + x) as usize] }
    }
}

/// Screen offset from the pivot to local coordinates.
fn to_local(sx: f32, sy: f32, pose: Pose) -> Pt {
    let sx = if pose.flip { -sx } else { sx };
    let (s, c) = pose.angle.sin_cos();
    (c * sx + s * sy, -s * sx + c * sy)
}

/// Local to screen offset from the pivot.
pub fn to_screen(lx: f32, ly: f32, pose: Pose) -> Pt {
    let (s, c) = pose.angle.sin_cos();
    let (x, y) = (c * lx - s * ly, s * lx + c * ly);
    (if pose.flip { -x } else { x }, y)
}

fn cut(part: &Part, pose: Pose) -> (Cut, (i32, i32)) {
    // the turned bounding box of every shape
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for s in &part.shapes {
        for (cx, cy) in [(s.bb[0], s.bb[1]), (s.bb[2], s.bb[1]), (s.bb[0], s.bb[3]), (s.bb[2], s.bb[3])] {
            let (x, y) = to_screen(cx, cy, pose);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
    }
    let (px, py) = ((-x0).ceil() as i32 + 1, (-y0).ceil() as i32 + 1);
    let (w, h) = (px + x1.ceil() as i32 + 2, py + y1.ceil() as i32 + 2);
    let mut c = Cut { w, h, m: vec![M::Empty; (w * h) as usize], at: vec![(0.0, 0.0); (w * h) as usize], tilt: vec![0.0; (w * h) as usize] };
    for y in 0..h {
        for x in 0..w {
            let (lx, ly) = to_local((x - px) as f32, (y - py) as f32, pose);
            if let Some(s) = part.shapes.iter().rev().find(|s| s.hit(lx, ly)) {
                let i = (y * w + x) as usize;
                c.m[i] = s.m;
                c.at[i] = (lx, ly);
                if let Some(((hx, hy), a)) = s.turn {
                    // back where it was before it turned
                    let (sn, cs) = a.sin_cos();
                    let (dx, dy) = (lx - hx, ly - hy);
                    c.at[i] = (hx + cs * dx + sn * dy, hy - sn * dx + cs * dy);
                    c.tilt[i] = a;
                }
            }
        }
    }
    (c, (px, py))
}

/// Carve `part` turned to `pose`.
pub fn carve(part: &Part, pose: Pose) -> Frame {
    carve_with(part, pose).0
}

/// Carve `part`, and say what each of its pixels is made of (row by row).
pub fn carve_with(part: &Part, pose: Pose) -> (Frame, Vec<M>) {
    let (c, pivot) = cut(part, pose);
    let (w, h) = (c.w, c.h);
    let mut img = Image::gen_image_color(w as u16, h as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    let (lx, ly, lz) = LIGHT;
    let ll = (lx * lx + ly * ly + lz * lz).sqrt();
    for y in 0..h {
        for x in 0..w {
            let k = c.m(x, y);
            if k == M::Empty {
                continue;
            }
            let (ax, ay) = c.at[(y * w + x) as usize];
            let edge = [c.m(x - 1, y), c.m(x + 1, y), c.m(x, y - 1), c.m(x, y + 1)].contains(&M::Empty);
            let grain = hash(ax.floor() as i32, ay.floor() as i32, part.seed);
            let col = if edge && !matches!(k, M::Red | M::Teal) {
                OUTLINE
            } else if k.stone() {
                // the form's light, turned with the part (the light stays put)
                let (nx, ny, nz) = part.form.normal(ax, ay);
                let (s, co) = (pose.angle + c.tilt[(y * w + x) as usize]).sin_cos();
                let (mut sx, sy) = (co * nx - s * ny, s * nx + co * ny);
                if pose.flip {
                    sx = -sx;
                }
                let d = (sx * lx + sy * ly + nz * lz) / ll;
                let mut v = AMBIENT + (1.0 - AMBIENT) * d.max(0.0) + part.form.lean(ax, ay);
                // light bounced up from the disk below, along the underside, and a rim
                // of it right along the bottom edge, so a dark belly still stands off
                // the black
                if sy > 0.6 {
                    v += (sy - 0.6) * 0.5;
                }
                if c.m(x, y + 2) == M::Empty && c.m(x, y + 1) != M::Empty {
                    v = v.max(0.5);
                }
                // each plate's bevel: a lip on its top and left, shade on its bottom and right
                let part_of = |m: M| m.stone();
                let (up, left, down, right) = (c.m(x, y - 1), c.m(x - 1, y), c.m(x, y + 1), c.m(x + 1, y));
                let lip = if k == M::Frame { 0.28 } else { 0.18 };
                let drop = if k == M::Frame { 0.26 } else { 0.16 };
                if !part_of(up) || !part_of(left) {
                    v += lip;
                } else if !part_of(down) || !part_of(right) {
                    v -= drop;
                } else if !part_of(c.m(x, y + 2)) {
                    v -= drop * 0.45;
                }
                match k {
                    M::Shade => v -= 0.22,
                    M::Pit => {
                        // a pit: its top-left wall in shadow, its bottom-right wall lit
                        v += if c.m(x - 1, y - 1) != M::Pit { -0.3 } else if c.m(x + 1, y + 1) != M::Pit { 0.12 } else { -0.12 };
                    }
                    M::Pipe => {
                        // turned rings: bands across the pipe
                        v += if (ax.floor() as i32).rem_euclid(3) == 0 { -0.14 } else { 0.03 };
                    }
                    _ => {}
                }
                // the regolith: sparse speckle, denser where it's dark
                let dense = 0.025 + 0.09 * (1.0 - v).clamp(0.0, 1.0);
                if grain < dense {
                    v -= 0.15;
                } else if grain > 0.99 && v > 0.45 {
                    v += 0.1;
                }
                RAMP[quantize(v, x, y)]
            } else {
                match k {
                    M::Seam => {
                        // lighter than the outline: an inner line
                        if grain < 0.5 { RAMP[1] } else { RAMP[0] }
                    }
                    M::Recess => {
                        if !matches!(c.m(x, y - 1), M::Recess) { OUTLINE } else if grain < 0.25 { RECESS } else { VOIDISH }
                    }
                    M::Slot => VOIDISH,
                    M::Mouth => {
                        if grain < 0.12 { RECESS } else { VOIDISH }
                    }
                    M::Vein => VEIN,
                    M::Red => SIGNAL,
                    M::Teal => TEAL,
                    M::Gum => {
                        if matches!(c.m(x, y - 1), M::Tooth) || matches!(c.m(x, y + 1), M::Tooth) { GUM_LIT } else { GUM }
                    }
                    M::Tooth => {
                        // a cone: lit down its left, shaded down its right, bright at the tip
                        let (l, r) = (c.m(x - 1, y), c.m(x + 1, y));
                        if l != M::Tooth { RAMP[5] } else if r != M::Tooth { RAMP[3] } else { RAMP[4] }
                    }
                    _ => OUTLINE,
                }
            };
            img.set_pixel(x as u32, y as u32, col);
        }
    }
    (Frame { img, pivot }, c.m)
}

/// Copy `src` onto `dst` with its pivot at `(x, y)` (alpha 0 pixels skipped): the
/// review sheet's compositing.
#[cfg(test)]
pub fn stamp(dst: &mut Image, f: &Frame, x: i32, y: i32) {
    let (dw, dh) = (dst.width as i32, dst.height as i32);
    for sy in 0..f.img.height as i32 {
        for sx in 0..f.img.width as i32 {
            let c = f.img.get_pixel(sx as u32, sy as u32);
            let (tx, ty) = (x - f.pivot.0 + sx, y - f.pivot.1 + sy);
            if c.a > 0.0 && tx >= 0 && ty >= 0 && tx < dw && ty < dh {
                dst.set_pixel(tx as u32, ty as u32, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Part {
        let body = poly(&[(-10.0, -6.0), (10.0, -6.0), (10.0, 6.0), (-10.0, 6.0)], M::Hull);
        let form = Form::barrel(&body);
        Part { shapes: vec![body], form, seed: 1 }
    }

    #[test]
    fn a_turned_point_comes_back() {
        for pose in [Pose { angle: 0.4, flip: false }, Pose { angle: -1.1, flip: true }] {
            let (x, y) = to_screen(7.0, -3.0, pose);
            let (lx, ly) = to_local(x, y, pose);
            assert!((lx - 7.0).abs() < 1e-4 && (ly + 3.0).abs() < 1e-4);
        }
    }

    #[test]
    fn carving_keeps_the_pivot_and_the_outline() {
        let f = carve(&square(), Pose::default());
        let (px, py) = f.pivot;
        assert!(f.img.get_pixel(px as u32, py as u32).a > 0.0, "the pivot is inside the part");
        // its silhouette is ringed by the outline
        let left = (0..f.img.width as u32).find(|&x| f.img.get_pixel(x, py as u32).a > 0.0).unwrap();
        assert_eq!(f.img.get_pixel(left, py as u32), OUTLINE);
        // turned a quarter, it's as tall as it was wide
        let t = carve(&square(), Pose { angle: std::f32::consts::FRAC_PI_2, flip: false });
        let lit = |f: &Frame| (0..f.img.height as u32).filter(|&y| (0..f.img.width as u32).any(|x| f.img.get_pixel(x, y).a > 0.0)).count();
        assert_eq!(lit(&t) as i32, (0..f.img.width as u32).filter(|&x| (0..f.img.height as u32).any(|y| f.img.get_pixel(x, y).a > 0.0)).count() as i32);
    }

    #[test]
    fn a_barrel_is_lit_on_top_and_dark_underneath() {
        let f = carve(&square(), Pose::default());
        let (px, py) = f.pivot;
        let luma = |c: Color| 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
        let at = |dy: i32| luma(f.img.get_pixel(px as u32 + 3, (py + dy) as u32));
        assert!(at(-4) > at(4) + 0.15, "top {} vs bottom {}", at(-4), at(4));
        // flipped, still lit from the top left of the screen
        let g = carve(&square(), Pose { angle: 0.0, flip: true });
        let gat = |dy: i32| luma(g.img.get_pixel(px as u32 + 3, (py + dy) as u32));
        assert!(gat(-4) > gat(4) + 0.15);
    }
}
