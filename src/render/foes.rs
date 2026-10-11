//! The enemies, drawn: lunar gothic craft in bone white, after Jason's reference
//! sheets. Each sprite is built from simple shapes tagged with materials (plate,
//! side face, recess, slot, crater, red light, the iris), then one shader paints
//! them all alike: chunky bevels lit from the top left, regolith speckle and pits.
//! Red only where it means danger: lights, a gun about to fire, the derelict's
//! weak points and, once it cracks, its burning seams.
//! Also the breather's background: drifting debris and the tiny tumbling figure.

use super::art::{Atlas, Packer, grid_image};
use super::palette::{SIGNAL, WHITE_A, tone};
use crate::sim::World;
use crate::sim::enemy::{Enemy, EnemyKind};
use crate::sim::pattern::Mount;
use crate::sim::rng::Rng;
use crate::tune::{CHARGE_FRAMES, HULL_EMERGE, MUZZLE_FRAMES};
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// An enemy sprite, its white silhouette (for flashes) and a sample of its pixels
/// relative to its center (what it shatters into).
pub struct FoeSprite {
    pub r: Rect,
    pub sil: Rect,
    pub pixels: Vec<(i8, i8, Color)>,
}

pub struct FoeArt {
    pub drone: FoeSprite,
    pub courier: FoeSprite,
    pub usher: FoeSprite,
    pub gunship: FoeSprite,
    /// The buoy's eye turning, 4 frames.
    pub buoy: [FoeSprite; 4],
    pub heavy: FoeSprite,
    pub hull: FoeSprite,
    /// The derelict's weak points: the hanging gun pods and the nose gun.
    pub pod: FoeSprite,
    pub nose: FoeSprite,
    /// Where a weak point broke off: a scorched socket.
    pub socket: Rect,
    /// The derelict's crack seams, pixels relative to its center (they burn red once it cracks).
    pub cracks: Vec<(i8, i8)>,
    /// Background scrap for the breather, and the tiny figure.
    pub debris: [Rect; 6],
    pub figure: Rect,
}

impl FoeArt {
    #[cfg(test)]
    pub fn rects(&self) -> Vec<Rect> {
        let mut v = vec![self.socket, self.figure];
        for s in [&self.drone, &self.courier, &self.usher, &self.gunship, &self.heavy, &self.hull, &self.pod, &self.nose] {
            v.extend([s.r, s.sil]);
        }
        for b in &self.buoy {
            v.extend([b.r, b.sil]);
        }
        v.extend(self.debris);
        v
    }

    pub fn sprite(&self, kind: EnemyKind, frame: u64) -> &FoeSprite {
        match kind {
            EnemyKind::Drone => &self.drone,
            EnemyKind::Courier => &self.courier,
            EnemyKind::Usher => &self.usher,
            EnemyKind::Gunship => &self.gunship,
            EnemyKind::Buoy => &self.buoy[(frame / 8 % 4) as usize],
            EnemyKind::Heavy => &self.heavy,
            EnemyKind::Hull => &self.hull,
        }
    }
}

// ---------------------------------------------------------------- the bone palette

/// Bone white, dark to light: grit, dark, shade, mid, highlight. The highlight is as
/// bright as cream, so enemy bullet cores stay the brightest pixels on screen.
pub(crate) const BONE: [Color; 5] = [rgb(0x3E3B33), rgb(0x6E6858), rgb(0x9D9682), rgb(0xC4BDA8), rgb(0xDDD6C2)];
pub(crate) const OUTLINE: Color = rgb(0x121210);
const VOIDISH: Color = rgb(0x0B0B0A);
const RECESS: Color = rgb(0x1E1D19);
const EMBER_DIM: Color = rgb(0x6A1C17);
const GLASS_DARK: Color = rgb(0x45120E);

const fn rgb(c: u32) -> Color {
    Color::new(((c >> 16) & 0xFF) as f32 / 255.0, ((c >> 8) & 0xFF) as f32 / 255.0, (c & 0xFF) as f32 / 255.0, 1.0)
}

// ---------------------------------------------------------------- shapes and materials

pub(crate) const EMPTY: u8 = 0;
/// Bone plating, facing the light.
pub(crate) const PLATE: u8 = 1;
/// A dark recess.
pub(crate) const RECESSED: u8 = 2;
/// A black slot or lens.
pub(crate) const SLOT: u8 = 3;
/// A weak point's red core (it glows when drawn).
pub(crate) const CORE: u8 = 4;
/// Plating turned away from the light.
pub(crate) const SIDE: u8 = 5;
/// Vent lines.
pub(crate) const GRILLE: u8 = 6;
/// A pit in the plating.
pub(crate) const CRATER: u8 = 7;
/// A red running light.
pub(crate) const LIGHT: u8 = 8;
/// The derelict's iris: aperture blades round its core.
pub(crate) const IRIS: u8 = 9;
/// Stained glass: dark red panes in black leading (it blazes when its owner charges).
pub(crate) const STAINED: u8 = 10;

/// One shape, in pixels from the sprite's center pixel (y down); later shapes paint
/// over earlier ones. `true` mirrors it left-right as well.
#[derive(Clone)]
pub(crate) enum P {
    Poly(Vec<(f32, f32)>, u8, bool),
    /// x0, y0, x1, y1, inclusive.
    Rect(f32, f32, f32, f32, u8, bool),
    Disc(f32, f32, f32, u8, bool),
    /// Center, inner and outer radius.
    Ring(f32, f32, f32, f32, u8, bool),
}

pub(crate) fn poly(pts: &[(f32, f32)], m: u8, mirror: bool) -> P {
    P::Poly(pts.to_vec(), m, mirror)
}

fn inside(pts: &[(f32, f32)], x: f32, y: f32) -> bool {
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

impl P {
    fn hits(&self, x: f32, y: f32) -> Option<u8> {
        let test = |x: f32| match self {
            P::Poly(pts, ..) => inside(pts, x, y),
            P::Rect(x0, y0, x1, y1, ..) => x >= *x0 && x <= *x1 && y >= *y0 && y <= *y1,
            P::Disc(cx, cy, r, ..) => (x - cx).powi(2) + (y - cy).powi(2) <= r * r + 0.25,
            P::Ring(cx, cy, r0, r1, ..) => {
                let d = (x - cx).powi(2) + (y - cy).powi(2);
                d > r0 * r0 + 0.25 && d <= r1 * r1 + 0.25
            }
        };
        let (m, mirror) = match self {
            P::Poly(_, m, mi) | P::Rect(.., m, mi) | P::Disc(.., m, mi) | P::Ring(.., m, mi) => (*m, *mi),
        };
        (test(x) || (mirror && test(-x))).then_some(m)
    }
}

/// The material of every pixel of a `w` x `h` sprite (odd sizes, so it has a center pixel).
pub(crate) fn raster(w: i32, h: i32, shapes: &[P]) -> Vec<u8> {
    let (cx, cy) = (w / 2, h / 2);
    let mut m = vec![EMPTY; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let (lx, ly) = ((x - cx) as f32, (y - cy) as f32);
            for s in shapes {
                if let Some(k) = s.hits(lx, ly) {
                    m[(y * w + x) as usize] = k;
                }
            }
        }
    }
    m
}

/// A fixed per-pixel noise in 0..1 (the regolith's speckle).
fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65536.0
}

/// Paint a material map in bone: an ink outline round the silhouette, a bright lip
/// along each plate's top and left, shade along its bottom and right, faces lighter
/// near the top, speckled and pitted like moon rock. `iris` is the iris's center.
pub(crate) fn bone(w: i32, h: i32, m: &[u8], seed: u32, iris: (f32, f32)) -> Image {
    let mut img = Image::gen_image_color(w as u16, h as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    let at = |x: i32, y: i32| if x < 0 || y < 0 || x >= w || y >= h { EMPTY } else { m[(y * w + x) as usize] };
    let plated = |k: u8| matches!(k, PLATE | SIDE | CRATER);
    let run = |x: i32, y: i32, dx: i32, dy: i32| (1..48).take_while(|&k| plated(at(x + dx * k, y + dy * k))).count() as f32;
    let (cx, cy) = (w / 2, h / 2);
    for y in 0..h {
        for x in 0..w {
            let k = at(x, y);
            if k == EMPTY {
                continue;
            }
            let edge = at(x - 1, y) == EMPTY || at(x + 1, y) == EMPTY || at(x, y - 1) == EMPTY || at(x, y + 1) == EMPTY;
            let speck = hash(x, y, seed);
            let shade = |t: i32, dense: f32| {
                let t = t - (speck < dense) as i32 - (speck < dense * 0.25) as i32;
                BONE[t.clamp(0, 4) as usize]
            };
            let c = if edge && k != LIGHT && k != CORE {
                OUTLINE
            } else {
                match k {
                    PLATE => {
                        let (up, down, left, right) = (run(x, y, 0, -1), run(x, y, 0, 1), run(x, y, -1, 0), run(x, y, 1, 0));
                        let t = if up < 1.5 || left < 1.5 {
                            4
                        } else if down < 1.5 {
                            1
                        } else if down < 3.5 || right < 1.5 {
                            2
                        } else if up / (up + down) < 0.4 {
                            4
                        } else {
                            3
                        };
                        shade(t, 0.11)
                    }
                    SIDE => {
                        let down = run(x, y, 0, 1);
                        shade(if down < 1.5 { 1 } else { 2 }, 0.2)
                    }
                    CRATER => {
                        // a pit: its upper-left wall in shadow, its lower-right wall catching the light
                        if at(x - 1, y - 1) != CRATER {
                            BONE[1]
                        } else if at(x + 1, y + 1) != CRATER {
                            BONE[4]
                        } else {
                            BONE[2]
                        }
                    }
                    RECESSED => {
                        if at(x, y - 1) != RECESSED { OUTLINE } else if speck < 0.3 { RECESS } else { VOIDISH }
                    }
                    SLOT => VOIDISH,
                    GRILLE => if y % 2 == 0 { BONE[1] } else { OUTLINE },
                    LIGHT => SIGNAL,
                    CORE => EMBER_DIM,
                    STAINED => {
                        // leading wherever the pane meets a different pane of the grid
                        if (x + y) % 5 == 0 || (x - y).rem_euclid(5) == 0 { OUTLINE } else if speck < 0.4 { GLASS_DARK } else { EMBER_DIM }
                    }
                    IRIS => {
                        // aperture blades, curving round the core
                        let (dx, dy) = ((x - cx) as f32 - iris.0, (y - cy) as f32 - iris.1);
                        let r = dx.hypot(dy);
                        let a = (dy.atan2(dx) + r * 0.09).rem_euclid(TAU) / (TAU / 8.0);
                        if r < 3.6 {
                            VOIDISH
                        } else if a.fract() < 0.14 {
                            OUTLINE
                        } else if (a as i32) % 2 == 0 {
                            shade(3, 0.08)
                        } else {
                            shade(2, 0.08)
                        }
                    }
                    _ => OUTLINE,
                }
            };
            img.set_pixel(x as u32, y as u32, c);
        }
    }
    img
}

fn paint(w: i32, h: i32, shapes: &[P], seed: u32) -> Image {
    bone(w, h, &raster(w, h, shapes), seed, (0.0, 0.0))
}

// ---------------------------------------------------------------- the roster

/// The popcorn drone (after CLERK): a swept arrowhead, a black slot, a red eye.
fn clerk() -> Image {
    paint(17, 17, &[
        poly(&[(0.0, -8.4), (1.8, -8.4), (3.0, -5.4), (6.0, -3.6), (8.4, -4.6), (8.4, 1.0), (6.4, 2.4), (3.2, 4.6), (1.0, 8.4), (0.0, 8.4)], PLATE, true),
        poly(&[(3.0, 1.6), (8.4, 0.0), (8.4, 1.0), (6.4, 2.4), (3.2, 4.6)], SIDE, true),
        P::Rect(-1.0, -5.0, 1.0, 4.0, SLOT, false),
        P::Rect(0.0, -3.0, 0.0, -2.0, LIGHT, false),
    ], 11)
}

/// The courier: two tall blades either side of a needle body.
fn courier() -> Image {
    paint(17, 25, &[
        poly(&[(5.4, -12.4), (8.4, -5.0), (8.4, 3.0), (6.6, 7.4), (4.0, 4.0), (4.0, -5.0)], PLATE, true),
        poly(&[(4.0, -5.0), (5.0, -3.0), (5.0, 4.6), (4.0, 4.0)], SIDE, true),
        P::Rect(2.0, 2.0, 4.0, 3.0, SIDE, true),
        poly(&[(0.0, -9.0), (1.0, -8.0), (2.4, -4.0), (2.4, 6.0), (1.2, 10.0), (0.0, 12.4)], PLATE, true),
        P::Rect(0.0, -12.0, 0.0, -9.0, PLATE, false),
        P::Rect(-1.0, -4.0, 1.0, 5.0, SLOT, false),
        P::Rect(0.0, -2.0, 0.0, -1.0, LIGHT, false),
    ], 12)
}

/// The usher: a thick horseshoe arch, open below, a light in each prong.
fn usher() -> Image {
    paint(25, 21, &[
        poly(&[(0.0, -10.4), (6.0, -10.4), (10.0, -8.0), (12.4, -4.0), (12.4, 5.0), (11.0, 9.0), (10.4, 10.4), (8.6, 10.4), (8.6, -2.0), (6.0, -5.4), (0.0, -5.4)], PLATE, true),
        poly(&[(0.0, -5.4), (6.0, -5.4), (8.6, -2.0), (8.6, 0.6), (6.0, -3.6), (0.0, -3.6)], SIDE, true),
        P::Rect(-4.0, -9.0, 4.0, -7.0, GRILLE, false),
        P::Rect(10.0, -1.0, 10.0, 6.0, SLOT, true),
        P::Rect(10.0, 1.0, 10.0, 2.0, LIGHT, true),
    ], 13)
}

/// The turret buoy (after INSPECTOR): a diamond with a knob at each corner and a
/// great black eye; a notch in the eye turns, frame by frame.
fn inspector(frame: u32) -> Image {
    let a = frame as f32 * TAU / 4.0 + TAU / 8.0;
    paint(25, 25, &[
        poly(&[(0.0, -10.6), (10.6, 0.0), (0.0, 10.6), (-10.6, 0.0)], PLATE, false),
        poly(&[(0.0, 5.0), (5.6, 5.0), (0.0, 10.6)], SIDE, true),
        P::Rect(0.0, -12.0, 0.0, -11.0, PLATE, false),
        P::Rect(11.0, 0.0, 12.0, 0.0, PLATE, true),
        P::Rect(0.0, 11.0, 0.0, 12.0, PLATE, false),
        P::Rect(-1.0, -12.0, 1.0, -12.0, PLATE, false),
        P::Rect(12.0, -1.0, 12.0, 1.0, PLATE, true),
        P::Rect(-1.0, 12.0, 1.0, 12.0, PLATE, false),
        P::Rect(-1.0, -7.0, 1.0, -6.0, SLOT, false),
        P::Rect(-1.0, 6.0, 1.0, 7.0, SLOT, false),
        P::Rect(6.0, -1.0, 7.0, 1.0, SLOT, true),
        P::Ring(0.0, 0.0, 2.8, 3.9, SIDE, false),
        P::Disc(0.0, 0.0, 2.8, SLOT, false),
        P::Disc(a.cos() * 3.4, a.sin() * 3.4, 0.5, SLOT, false),
        P::Disc(0.0, 0.0, 1.0, LIGHT, false),
    ], 14 + frame)
}

/// The gunship (after BAILIFF): a bulky pointed hull with a dark lens, a pod on each
/// side with a red light.
fn bailiff() -> Image {
    paint(31, 27, &[
        poly(&[(0.0, -13.4), (1.6, -13.4), (2.6, -9.0), (6.0, -8.0), (9.4, -4.0), (10.4, 1.0), (9.6, 6.0), (6.0, 10.6), (2.6, 12.0), (0.0, 12.4)], PLATE, true),
        poly(&[(6.0, 4.0), (9.6, 6.0), (6.0, 10.6), (2.6, 12.0), (3.0, 9.0)], SIDE, true),
        P::Rect(10.0, 0.0, 11.0, 2.0, SIDE, true),
        poly(&[(11.6, -4.0), (14.4, -3.0), (14.4, 7.0), (12.6, 8.4), (11.6, 7.0)], PLATE, true),
        P::Rect(12.0, 5.0, 14.0, 7.0, SIDE, true),
        P::Rect(12.0, 0.0, 13.0, 1.0, LIGHT, true),
        poly(&[(0.0, -7.0), (1.8, -5.0), (2.4, 1.0), (1.6, 6.0), (0.0, 8.0)], SLOT, true),
        P::Rect(0.0, -4.0, 0.0, -3.0, LIGHT, false),
        P::Disc(0.0, 12.0, 1.6, PLATE, false),
        P::Disc(-6.0, -3.0, 1.0, CRATER, false),
    ], 15)
}

/// The heavy (after NOTARY): two tall towers, windows dark, guns beneath, bridged by
/// a block with a red light over its ring launcher.
fn notary() -> Image {
    paint(49, 35, &[
        poly(&[(10.4, -13.0), (12.6, -15.4), (21.4, -15.4), (23.6, -13.0), (23.6, 9.0), (21.0, 12.6), (13.0, 12.6), (10.4, 9.0)], PLATE, true),
        poly(&[(10.4, 6.0), (23.6, 6.0), (23.6, 9.0), (21.0, 12.6), (13.0, 12.6), (10.4, 9.0)], SIDE, true),
        P::Rect(15.0, -10.0, 17.0, 3.0, SLOT, true),
        P::Rect(13.0, 8.0, 20.0, 9.0, GRILLE, true),
        P::Rect(15.0, 13.0, 17.0, 16.0, PLATE, true),
        P::Rect(16.0, 16.0, 16.0, 16.0, SLOT, true),
        P::Rect(9.0, -3.0, 10.0, 3.0, SIDE, true),
        P::Rect(-9.0, -6.0, 9.0, 5.0, PLATE, false),
        P::Rect(-6.0, -5.0, 6.0, -3.0, GRILLE, false),
        P::Rect(-5.0, -1.0, 5.0, 3.0, RECESSED, false),
        P::Disc(0.0, 1.0, 1.0, LIGHT, false),
        P::Disc(0.0, 7.0, 2.2, PLATE, false),
        P::Rect(-1.0, 8.0, 1.0, 8.0, SLOT, false),
        P::Disc(19.0, -8.0, 1.6, CRATER, false),
        P::Disc(13.0, 1.0, 1.2, CRATER, false),
        P::Disc(-14.0, -11.0, 1.3, CRATER, false),
        P::Disc(-20.0, 2.0, 1.0, CRATER, false),
    ], 16)
}

/// The derelict's sprite size and the center of its iris (from the sprite's center).
const HULL_SIZE: (i32, i32) = (121, 105);
const IRIS_AT: (f32, f32) = (0.0, -2.0);

/// The derelict (after Jason's mid-boss sheet): a gothic spire with an arched window
/// and a cross, a great iris eye with a red core, cratered shoulders, recesses
/// beneath them, and the mounts its gun pods hang from.
fn derelict_shapes() -> Vec<P> {
    let (ix, iy) = IRIS_AT;
    vec![
        poly(&[(12.0, -22.0), (20.0, -36.0), (30.0, -44.0), (46.0, -47.0), (56.0, -40.0), (60.0, -26.0), (57.0, -6.0), (50.0, 3.0), (40.0, -1.0), (30.0, -10.0), (20.0, -6.0), (13.0, -2.0)], PLATE, true),
        poly(&[(30.0, -10.0), (40.0, -1.0), (50.0, 3.0), (57.0, -6.0), (58.0, -12.0), (50.0, -4.0), (40.0, -8.0), (32.0, -16.0)], SIDE, true),
        poly(&[(20.0, -36.0), (30.0, -44.0), (33.0, -40.0), (23.0, -32.0)], SIDE, true),
        P::Disc(46.0, -36.0, 3.3, CRATER, false),
        P::Disc(53.0, -24.0, 2.2, CRATER, false),
        P::Disc(38.0, -38.0, 1.6, CRATER, false),
        P::Disc(49.0, -12.0, 2.4, CRATER, false),
        P::Disc(-44.0, -34.0, 2.6, CRATER, false),
        P::Disc(-52.0, -18.0, 3.0, CRATER, false),
        P::Disc(-36.0, -40.0, 1.4, CRATER, false),
        P::Disc(-47.0, -8.0, 1.8, CRATER, false),
        // the gothic recesses under the shoulders, with a rib in each
        poly(&[(14.0, -2.0), (20.0, -6.0), (28.0, -10.0), (32.0, -4.0), (32.0, 12.0), (14.0, 12.0)], RECESSED, true),
        P::Rect(22.0, -4.0, 23.0, 11.0, SIDE, true),
        // the gun pods' mounts
        P::Disc(44.0, 4.0, 4.6, PLATE, true),
        P::Ring(44.0, 4.0, 2.4, 3.4, SIDE, true),
        P::Disc(44.0, 4.0, 2.4, SLOT, true),
        // the spire
        poly(&[(0.0, -51.0), (3.0, -47.0), (6.0, -41.0), (9.0, -33.0), (9.0, -25.0), (15.0, -21.0), (15.0, 16.0), (10.0, 22.0), (8.0, 30.0), (4.0, 34.0), (0.0, 35.0)], PLATE, true),
        poly(&[(12.0, -18.0), (15.0, -21.0), (15.0, 16.0), (12.0, 19.0)], SIDE, true),
        // the arched window and its cross
        poly(&[(0.0, -44.0), (2.5, -40.0), (4.0, -35.0), (4.0, -25.0), (0.0, -25.0)], SLOT, true),
        P::Rect(0.0, -41.0, 0.0, -27.0, PLATE, false),
        P::Rect(-2.0, -36.0, 2.0, -36.0, PLATE, false),
        P::Rect(-7.0, -24.0, 7.0, -22.0, SIDE, false),
        P::Rect(11.0, -16.0, 12.0, -6.0, SLOT, true),
        P::Rect(11.0, 4.0, 12.0, 12.0, SLOT, true),
        // the iris
        P::Disc(ix, iy, 12.4, SIDE, false),
        P::Ring(ix, iy, 10.4, 12.4, PLATE, false),
        P::Disc(ix, iy, 10.4, IRIS, false),
        P::Disc(ix, iy, 2.2, CORE, false),
        // under it, vents and a light
        P::Rect(-3.0, 14.0, 3.0, 25.0, GRILLE, false),
        P::Rect(0.0, 27.0, 0.0, 28.0, LIGHT, false),
        // an antenna on one shoulder
        P::Rect(52.0, -51.0, 52.0, -47.0, PLATE, false),
        P::Rect(54.0, -49.0, 54.0, -47.0, PLATE, false),
        P::Rect(52.0, -52.0, 52.0, -52.0, LIGHT, false),
    ]
}

fn derelict() -> Image {
    let (w, h) = HULL_SIZE;
    bone(w, h, &raster(w, h, &derelict_shapes()), 17, IRIS_AT)
}

/// A gun pod hanging from the derelict's shoulder: a long housing, a slot of red
/// lights, twin barrels below.
fn pod() -> Image {
    paint(19, 37, &[
        poly(&[(0.0, -17.4), (5.0, -16.0), (7.4, -12.0), (8.4, -6.0), (8.4, 10.0), (6.4, 14.4), (0.0, 14.4)], PLATE, true),
        poly(&[(0.0, 11.0), (8.4, 10.0), (6.4, 14.4), (0.0, 14.4)], SIDE, true),
        P::Disc(0.0, -13.0, 3.2, SIDE, false),
        P::Disc(0.0, -13.0, 1.6, SLOT, false),
        P::Rect(-1.0, -7.0, 1.0, 9.0, SLOT, false),
        P::Rect(0.0, -3.0, 0.0, -2.0, LIGHT, false),
        P::Rect(0.0, 1.0, 0.0, 2.0, CORE, false),
        P::Rect(2.0, 14.0, 4.0, 18.0, PLATE, true),
        P::Rect(3.0, 18.0, 3.0, 18.0, SLOT, true),
        P::Disc(4.0, 3.0, 1.0, CRATER, false),
    ], 18)
}

/// The derelict's nose gun, at the tip of its spire.
fn nose() -> Image {
    paint(13, 17, &[
        poly(&[(0.0, -8.4), (4.0, -7.0), (6.4, -2.0), (5.0, 4.0), (2.4, 7.0), (1.4, 8.4), (0.0, 8.4)], PLATE, true),
        poly(&[(3.0, 2.0), (5.0, 4.0), (2.4, 7.0), (1.4, 8.4), (0.0, 8.4), (0.0, 5.0)], SIDE, true),
        P::Rect(-1.0, -5.0, 1.0, 2.0, SLOT, false),
        P::Rect(0.0, -2.0, 0.0, -1.0, LIGHT, false),
        P::Rect(0.0, 7.0, 0.0, 8.0, SLOT, false),
    ], 19)
}

/// A sample of an image's lit pixels relative to its center, every `every`-th one.
fn pixels(img: &Image, every: usize) -> Vec<(i8, i8, Color)> {
    let (w, h) = (img.width as i32, img.height as i32);
    let mut v = vec![];
    let mut k = 0;
    for y in 0..h {
        for x in 0..w {
            let c = img.get_pixel(x as u32, y as u32);
            if c.a > 0.0 {
                if k % every == 0 {
                    v.push(((x - w / 2) as i8, (y - h / 2) as i8, Color { a: 1.0, ..c }));
                }
                k += 1;
            }
        }
    }
    v
}

fn foe(p: &mut Packer, img: &Image, every: usize) -> FoeSprite {
    let (r, sil) = p.add_with_silhouette(img);
    FoeSprite { r, sil, pixels: pixels(img, every) }
}

/// The derelict's crack seams: jagged lines from the iris out through the plating.
fn cracks() -> Vec<(i8, i8)> {
    let (w, h) = HULL_SIZE;
    let m = raster(w, h, &derelict_shapes());
    let (cx, cy) = (w / 2, h / 2);
    let plate = |x: i32, y: i32| x + cx >= 0 && y + cy >= 0 && x + cx < w && y + cy < h && matches!(m[((y + cy) * w + x + cx) as usize], PLATE | SIDE | CRATER);
    let mut rng = Rng::new(0xC4AC);
    let mut v = vec![];
    let starts = [(14.0f32, -10.0f32, 1.0f32, -0.3f32), (-14.0, -6.0, -1.0, -0.2), (16.0, 6.0, 1.0, 0.3), (-15.0, 10.0, -1.0, 0.2), (0.0, -24.0, 0.2, -1.0), (30.0, -30.0, 1.0, 0.4), (-30.0, -28.0, -1.0, 0.5)];
    for (sx, sy, dx, dy) in starts {
        let (mut x, mut y) = (sx, sy);
        for _ in 0..rng.range(10.0, 22.0) as i32 {
            x += dx + rng.range(-0.5, 0.5);
            y += dy + rng.range(-0.6, 0.6);
            let (px, py) = (x.round() as i32, y.round() as i32);
            if plate(px, py) {
                v.push((px as i8, py as i8));
            }
        }
    }
    v
}

/// Scrap and rock for the breather's debris field: dim, inside the background's budget.
fn debris(i: usize) -> Image {
    let mut rng = Rng::new(0xDEB0 + i as u64);
    let s = [5u16, 7, 9, 6, 11, 8][i];
    let mut img = Image::gen_image_color(s, s, Color::new(0.0, 0.0, 0.0, 0.0));
    let c = s as f32 * 0.5;
    let bumps: Vec<f32> = (0..6).map(|_| rng.range(0.55, 1.0)).collect();
    for y in 0..s as i32 {
        for x in 0..s as i32 {
            let (dx, dy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
            let a = dy.atan2(dx).rem_euclid(TAU);
            let r = bumps[((a / TAU * 6.0) as usize).min(5)] * c;
            let d = dx.hypot(dy);
            if d > r {
                continue;
            }
            let v = if dx + dy < -c * 0.3 { 0.24 } else if d > r - 1.0 { 0.1 } else { 0.16 };
            img.set_pixel(x as u32, y as u32, tone(v));
        }
    }
    img
}

/// The tiny figure from Jason's black hole image, arms up, sneakers first.
const FIGURE: [&str; 11] = [
    "a.....a",
    ".a...a.",
    "..hh...",
    "..hh...",
    ".bbbb..",
    "..bb...",
    "..bbb..",
    ".b..b..",
    ".b...b.",
    "s....s.",
    "ss...ss",
];

pub fn build(p: &mut Packer) -> FoeArt {
    // a broken weak point's socket: a scorched ring
    let mut socket = Image::gen_image_color(13, 9, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..9i32 {
        for x in 0..13i32 {
            let (dx, dy) = ((x - 6) as f32 / 6.4, (y - 4) as f32 / 4.4);
            let d = dx * dx + dy * dy;
            if d <= 1.0 {
                socket.set_pixel(x as u32, y as u32, if d > 0.55 { OUTLINE } else if (x + y) % 2 == 0 { RECESS } else { BONE[0] });
            }
        }
    }
    let figure_pal = |c: char| match c {
        'h' => Some(tone(0.26)),
        'a' => Some(tone(0.2)),
        'b' => Some(tone(0.15)),
        's' => Some(tone(0.3)),
        _ => None,
    };
    let figure = grid_image(&FIGURE.iter().map(|r| r.chars().collect()).collect::<Vec<_>>(), figure_pal);
    FoeArt {
        drone: foe(p, &clerk(), 1),
        courier: foe(p, &courier(), 1),
        usher: foe(p, &usher(), 1),
        gunship: foe(p, &bailiff(), 2),
        buoy: [0, 1, 2, 3].map(|f| foe(p, &inspector(f), 2)),
        heavy: foe(p, &notary(), 3),
        hull: foe(p, &derelict(), 6),
        pod: foe(p, &pod(), 2),
        nose: foe(p, &nose(), 2),
        socket: p.add(&socket),
        cracks: cracks(),
        debris: [0, 1, 2, 3, 4, 5].map(|i| p.add(&debris(i))),
        figure: p.add(&figure),
    }
}

fn snap(v: f32) -> i32 {
    v.round() as i32
}

/// How far the derelict has risen out of the disk: 0 (a black shape) to 1.
pub fn emerged(e: &Enemy) -> f32 {
    if e.kind == EnemyKind::Hull { (e.t as f32 / HULL_EMERGE as f32).min(1.0) } else { 1.0 }
}

/// The hit flash: on bone white a white flash wouldn't show, so they flush red (the
/// big ones only faintly, or the screen's middle would keep turning red).
fn flash(a: &Atlas, sil: Rect, x: i32, y: i32) {
    let area = sil.w * sil.h;
    let k = if area > 2000.0 { 0.18 } else if area > 600.0 { 0.35 } else { 0.55 };
    a.blit(sil, x, y, 1, Color { a: k, ..SIGNAL });
}

/// Every enemy, the big ones underneath, with hit flashes, broken sockets, burning
/// seams and the red charge-up on every gun about to fire.
pub fn draw(a: &Atlas, w: &World, ox: i32, oy: i32) {
    let art = &a.r.foes;
    let f = w.frame;
    for kind in [EnemyKind::Hull, EnemyKind::Heavy, EnemyKind::Buoy, EnemyKind::Gunship, EnemyKind::Usher, EnemyKind::Drone, EnemyKind::Courier] {
        for e in w.foes.list.iter().filter(|e| e.kind == kind && e.alive()) {
            let s = art.sprite(kind, f);
            let (x, y) = (ox + snap(e.x - s.r.w * 0.5), oy + snap(e.y - s.r.h * 0.5));
            let k = emerged(e);
            let tint = Color::new(k, k, k, 1.0);
            a.blit(s.r, x, y, 1, tint);
            // popcorn that's going to shoot: its eye pulses until it does
            let eye = match kind {
                EnemyKind::Drone => Some(-3),
                EnemyKind::Courier => Some(-2),
                _ => None,
            };
            if let Some(dy) = eye.filter(|_| e.variant == 1 && e.guns.first().is_some_and(|g| g.e.since == u32::MAX)) {
                let k = 0.5 + 0.5 * ((f as f32) * 0.35).sin();
                a.fill(ox + snap(e.x) - 1, oy + snap(e.y) + dy, 3, 2, Color { a: 0.35 + 0.5 * k, ..SIGNAL });
            }
            if kind == EnemyKind::Hull {
                draw_hull_parts(a, e, ox, oy, tint, f);
            }
            if e.parts[0].flash > 0 {
                flash(a, s.sil, x, y);
            }
            draw_telegraphs(a, e, ox, oy, f);
        }
    }
}

fn draw_hull_parts(a: &Atlas, e: &Enemy, ox: i32, oy: i32, tint: Color, f: u64) {
    let art = &a.r.foes;
    let (hx, hy) = (ox + snap(e.x), oy + snap(e.y));
    // once it's cracked the seams burn and the iris opens on a swelling red core
    if e.phase == 2 {
        let pulse = 0.55 + 0.35 * ((f as f32) * 0.15).sin();
        for &(dx, dy) in &art.cracks {
            a.fill(hx + dx as i32, hy + dy as i32, 1, 1, Color { a: pulse, ..SIGNAL });
        }
        let r = (4.0 + 1.5 * ((f as f32) * 0.2).sin()).round() as usize;
        let disc = a.r.discs[r - 1];
        let (cx, cy) = (hx + IRIS_AT.0 as i32, hy + IRIS_AT.1 as i32);
        a.blit(disc, cx - r as i32 - 1, cy - r as i32 - 1, 1, Color { a: 0.85, ..SIGNAL });
    }
    for (i, q) in e.parts[1..].iter().enumerate() {
        let (cx, cy) = (ox + snap(e.x + q.dx), oy + snap(e.y + q.dy));
        if !q.alive() {
            a.blit(art.socket, cx - 6, cy - (q.h * 0.5) as i32 - 2, 1, tint);
            continue;
        }
        let s = if i == 1 { &art.nose } else { &art.pod };
        let (x, y) = (cx - (s.r.w * 0.5) as i32, cy - (s.r.h * 0.5) as i32);
        a.blit(s.r, x, y, 1, tint);
        // the weak point's red core, brighter as it wears down
        let hurt = 1.0 - q.hp / q.max_hp;
        a.fill(cx - 1, cy, 3, 3, Color { a: (0.3 + 0.6 * hurt) * tint.r, ..SIGNAL });
        if q.flash > 0 {
            flash(a, s.sil, x, y);
        }
    }
}

/// A gun about to fire glows red at its muzzle, swelling until the shot (the
/// telegraph); the moment it fires, a red flash.
fn draw_telegraphs(a: &Atlas, e: &Enemy, ox: i32, oy: i32, f: u64) {
    for g in &e.guns {
        if !e.parts[g.part as usize].alive() || (g.phase != 0 && g.phase != e.phase) {
            continue;
        }
        let Mount::Owner { dx, dy } = g.e.gun.mount else { continue };
        let (x, y) = (ox + snap(e.x + dx), oy + snap(e.y + dy));
        let left = g.e.until_next();
        if e.armed && left <= CHARGE_FRAMES {
            let k = 1.0 - left as f32 / CHARGE_FRAMES as f32;
            let flicker = if f % 4 < 2 { 1.0 } else { 0.8 };
            let r = (1.0 + k * 2.4).round() as usize;
            let disc = a.r.discs[r.clamp(1, 16) - 1];
            // red only: a white center would read as a bullet's core
            a.blit(disc, x - r as i32 - 1, y - r as i32 - 1, 1, Color { a: (0.2 + 0.5 * k) * flicker, ..SIGNAL });
        }
        if g.e.since < MUZZLE_FRAMES {
            let k = 1.0 - g.e.since as f32 / MUZZLE_FRAMES as f32;
            a.fill(x - 1, y - 1, 3, 3, Color { a: k, ..SIGNAL });
            a.fill(x - 3, y, 7, 1, Color { a: k * 0.6, ..SIGNAL });
            a.fill(x, y - 3, 1, 7, Color { a: k * 0.6, ..SIGNAL });
        }
    }
}

/// The derelict after its death, `k` of the way (0 to 1) to the final blast: the wreck
/// shudders, its seams blaze and it darkens as the blasts eat through it.
pub fn draw_wreck(a: &Atlas, ox: i32, oy: i32, x: f32, y: f32, k: f32, f: u64) {
    let art = &a.r.foes;
    let s = &art.hull;
    let jx = [0, 1, 0, -1][(f % 4) as usize];
    let jy = [0, 0, 1, 0][(f / 2 % 4) as usize];
    let (cx, cy) = (ox + snap(x) + jx, oy + snap(y) + jy);
    let dark = 1.0 - 0.6 * k;
    a.blit(s.r, cx - (s.r.w * 0.5) as i32, cy - (s.r.h * 0.5) as i32, 1, Color::new(dark, dark, dark, 1.0));
    let burn = 0.6 + 0.4 * ((f as f32) * 0.6).sin().abs();
    for &(dx, dy) in &art.cracks {
        a.fill(cx + dx as i32, cy + dy as i32, 1, 1, Color { a: burn, ..SIGNAL });
    }
    // the blasts light it red now and then
    if f % 6 < 2 {
        a.blit(s.sil, cx - (s.r.w * 0.5) as i32, cy - (s.r.h * 0.5) as i32, 1, Color { a: 0.35 * (1.0 - k), ..SIGNAL });
    }
}

/// Engine exhausts (offsets from the center): enemies fly nose down, so their engines face up.
pub fn engines(kind: EnemyKind) -> &'static [(f32, f32)] {
    match kind {
        EnemyKind::Drone => &[(0.0, -8.0)],
        EnemyKind::Courier => &[(0.0, -11.0)],
        EnemyKind::Usher => &[(-6.0, -10.0), (6.0, -10.0)],
        EnemyKind::Gunship => &[(0.0, -13.0), (-13.0, -4.0), (13.0, -4.0)],
        EnemyKind::Buoy => &[],
        EnemyKind::Heavy => &[(-16.0, -15.0), (16.0, -15.0)],
        EnemyKind::Hull => &[],
    }
}

/// The breather's background: scrap drifting down through the far distance, `t`
/// frames into it (pure function of the time, like the stars).
pub fn draw_debris(a: &Atlas, ox: i32, oy: i32, t: u32, fade: f32) {
    let mut rng = Rng::new(0x5C4A9);
    for i in 0..26 {
        let x = rng.range(10.0, 260.0);
        let speed = rng.range(0.25, 0.9);
        let start = rng.range(0.0, 400.0);
        let spin = rng.range(-0.04, 0.04);
        let src = a.r.foes.debris[i % 6];
        let y = (start + t as f32 * speed) % 420.0 - 40.0;
        let drift = (t as f32 * 0.01 + i as f32).sin() * 4.0;
        let ang = t as f32 * spin + i as f32;
        a.blit_rot(src, ox as f32 + x + drift, oy as f32 + y, ang, Color { a: fade, ..WHITE_A });
    }
}

/// The tiny figure tumbling toward the hole, trailing red, green and blue streaks.
/// `k` runs 0 to 1 across its fall; it fades as it nears the hole.
pub fn draw_figure(a: &Atlas, ox: i32, oy: i32, k: f32, hole: (f32, f32)) {
    let from = (214.0, 330.0);
    let ease = k * k * (3.0 - 2.0 * k);
    let (x, y) = (from.0 + (hole.0 - from.0) * ease, from.1 + (hole.1 + 14.0 - from.1) * ease);
    let fade = (1.0 - ((k - 0.75) / 0.25).clamp(0.0, 1.0)) * (k / 0.08).min(1.0);
    if fade <= 0.0 {
        return;
    }
    // streaks behind it, along the way it came
    let (dx, dy) = (from.0 - hole.0, from.1 - hole.1);
    let d = dx.hypot(dy);
    let (ux, uy) = (dx / d, dy / d);
    for (i, (c, a0)) in [(SIGNAL, 0.5), (Color::new(0.2, 0.8, 0.4, 1.0), 0.22), (Color::new(0.3, 0.45, 1.0, 1.0), 0.22)].iter().enumerate() {
        let side = i as f32 - 0.6;
        for s in 0..14 {
            if (s + i) % 3 == 2 {
                continue;
            }
            let along = 4.0 + s as f32 * 1.3;
            let px = x + ux * along - uy * side * 1.5;
            let py = y + uy * along + ux * side * 1.5;
            let fall = 1.0 - s as f32 / 14.0;
            a.fill(ox + snap(px), oy + snap(py), 1, 1, Color { a: a0 * fall * fade, ..*c });
        }
    }
    a.blit_rot(a.r.foes.figure, ox as f32 + x, oy as f32 + y, k * 9.0, Color { a: fade, ..WHITE_A });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::{CORE as BULLET_CORE, luma};

    fn brightest(img: &Image) -> f32 {
        img.get_image_data().iter().filter(|p| p[3] > 0).map(|p| luma(Color::from_rgba(p[0], p[1], p[2], 255))).fold(0.0, f32::max)
    }

    fn roster() -> Vec<(&'static str, Image)> {
        vec![("clerk", clerk()), ("courier", courier()), ("usher", usher()), ("inspector", inspector(0)), ("bailiff", bailiff()), ("notary", notary()), ("derelict", derelict()), ("pod", pod()), ("nose", nose())]
    }

    /// Bone white (Jason's call): bright enough to stand off the black, never as bright
    /// as an enemy bullet's core.
    #[test]
    fn enemies_are_bone_white_but_never_outshine_a_bullet_core() {
        for (name, im) in roster() {
            let b = brightest(&im);
            assert!((0.8..=0.86).contains(&b), "{name}: {b}");
            assert!(b < luma(BULLET_CORE) - 0.08, "{name}");
            // and most of it is bone, not shadow
            let lit = im.get_image_data().iter().filter(|p| p[3] > 0).map(|p| luma(Color::from_rgba(p[0], p[1], p[2], 255))).collect::<Vec<_>>();
            let bright = lit.iter().filter(|&&l| l > 0.55).count() as f32 / lit.len() as f32;
            assert!(bright > 0.3, "{name}: only {:.0}% of it is bright", bright * 100.0);
        }
    }

    #[test]
    fn the_background_extras_stay_in_the_background_budget() {
        for i in 0..6 {
            assert!(brightest(&debris(i)) <= 0.3);
        }
        let fig = grid_image(&FIGURE.iter().map(|r| r.chars().collect()).collect::<Vec<_>>(), |c| match c {
            '.' => None,
            _ => Some(tone(0.3)),
        });
        assert!(brightest(&fig) <= 0.31);
    }

    #[test]
    fn sprites_are_as_big_as_their_hurtboxes() {
        use crate::tune::{BUOY, COURIER, DRONE, GUNSHIP, HEAVY, HULL, HULL_TURRETS, USHER};
        let sizes = [(clerk(), DRONE), (courier(), COURIER), (usher(), USHER), (inspector(0), BUOY), (bailiff(), GUNSHIP), (notary(), HEAVY)];
        for (im, (_, w, h)) in sizes {
            assert!((im.width as f32 - w).abs() <= 3.0 && (im.height as f32 - h).abs() <= 3.0, "{}x{} vs {w}x{h}", im.width, im.height);
        }
        // the derelict is a composite: its hurtbox sits inside the sprite, and each weak
        // point's sprite covers its hurtbox
        let (w, h) = HULL_SIZE;
        assert!(HULL.1 <= w as f32 && HULL.2 <= h as f32);
        let (p, n) = (pod(), nose());
        for (i, &(_, _, tw, th)) in HULL_TURRETS.iter().enumerate() {
            let s = if i == 1 { &n } else { &p };
            assert!(s.width as f32 >= tw && s.height as f32 >= th && s.height as f32 - th <= 4.0);
        }
    }

    #[test]
    fn the_cracks_run_through_the_plating_from_the_iris() {
        let c = cracks();
        assert!(c.len() > 40, "{}", c.len());
        assert!(c.iter().any(|&(x, y)| (x as f32 - IRIS_AT.0).hypot(y as f32 - IRIS_AT.1) < 20.0));
    }
}
